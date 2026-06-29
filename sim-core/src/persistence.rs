//! Versioned save/load. Build step 6.
//!
//! Design (see `docs/ARCHITECTURE.md`, "Persistence"):
//! - The on-disk bytes are `[MAGIC][FORMAT_VERSION][codec payload]`. The version header
//!   is present from the very first save, because migrating multi-season saves across
//!   patches is a hard requirement, not a later concern.
//! - The payload is a [`SaveData`] snapshot encoded through a [`SaveCodec`]. The default
//!   codec is `bincode` (serde-based → ordinary migration).
//! - [`SaveData`] is a set of **mirror structs**, deliberately separate from the runtime
//!   ECS components. The format is therefore decoupled from the live types: a runtime
//!   component can change shape while a `migrate` step still reads the old save. This is
//!   exactly the property `rkyv`'s zero-copy layout would cost us.
//!
//! Scope: this captures `sim-core`'s own components (entity lifecycle + economy). It does
//! **not** know about sport-specific components (a footballer's attributes) — how a sport
//! plugs its own data into the save is a generalization to harvest later (constraint #4),
//! not to design speculatively now.

use crate::economy::{Balance, WeeklyIncome};
use crate::entity::{BirthDate, Condition, Contract, FreeAgent, Morale, Retired};
use crate::rng::SimSeed;
use crate::time::{Date, SimClock};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Identifies the file as one of ours.
pub const MAGIC: [u8; 4] = *b"TSPS";

/// The save format version. Bump on any breaking change to [`SaveData`] and add a
/// migration arm in [`migrate`].
pub const FORMAT_VERSION: u16 = 1;

/// A calendar date, mirrored for serialization independently of the runtime [`Date`].
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateSave {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl From<Date> for DateSave {
    fn from(d: Date) -> Self {
        Self { year: d.year(), month: d.month(), day: d.day() }
    }
}

impl From<DateSave> for Date {
    fn from(d: DateSave) -> Self {
        Date::new(d.year, d.month, d.day)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConditionSave {
    pub fitness: u8,
    pub injury_days: u16,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContractSave {
    /// The employing club, by its save index (not a live `Entity`, which is remapped).
    pub club: u32,
    pub until: DateSave,
    pub wage: u32,
}

/// The world clock, mirrored.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockSave {
    pub date: DateSave,
    pub day_index: u64,
}

/// One entity's `sim-core` components. Every field is optional so the same record serves
/// people (birth/contract/condition) and clubs (balance/income).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct EntitySave {
    /// Stable index assigned at capture; references (e.g. a contract's club) use it.
    pub id: u32,
    pub birth: Option<DateSave>,
    pub morale: Option<u8>,
    pub condition: Option<ConditionSave>,
    pub contract: Option<ContractSave>,
    pub balance: Option<i64>,
    pub weekly_income: Option<i64>,
    pub free_agent: bool,
    pub retired: bool,
}

/// A full snapshot of the simulation's `sim-core` state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SaveData {
    pub clock: ClockSave,
    pub seed: u64,
    pub entities: Vec<EntitySave>,
}

/// Errors from reading a save.
#[derive(Debug)]
pub enum SaveError {
    /// The bytes are too short or don't start with [`MAGIC`].
    BadMagic,
    /// The save was written by a newer game version than this one understands.
    FutureVersion(u16),
    /// The version is older than any this build can migrate (none exist yet).
    UnsupportedVersion(u16),
    /// The codec failed to encode or decode the payload.
    Codec(bincode::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::BadMagic => write!(f, "not a recognized save file (bad magic/header)"),
            SaveError::FutureVersion(v) => {
                write!(f, "save format v{v} is newer than this build supports (v{FORMAT_VERSION})")
            }
            SaveError::UnsupportedVersion(v) => write!(f, "save format v{v} is no longer supported"),
            SaveError::Codec(e) => write!(f, "save codec error: {e}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaveError::Codec(e) => Some(e),
            _ => None,
        }
    }
}

/// Encodes/decodes a [`SaveData`] payload. Swappable so the byte format can change
/// (e.g. to `rkyv`) without touching capture/restore.
pub trait SaveCodec {
    fn encode(&self, data: &SaveData) -> Result<Vec<u8>, SaveError>;
    fn decode(&self, bytes: &[u8]) -> Result<SaveData, SaveError>;
}

/// The default codec: `bincode`.
pub struct BincodeCodec;

impl SaveCodec for BincodeCodec {
    fn encode(&self, data: &SaveData) -> Result<Vec<u8>, SaveError> {
        bincode::serialize(data).map_err(SaveError::Codec)
    }
    fn decode(&self, bytes: &[u8]) -> Result<SaveData, SaveError> {
        bincode::deserialize(bytes).map_err(SaveError::Codec)
    }
}

/// Snapshot the world's `sim-core` state into a [`SaveData`].
///
/// Two conceptual passes: entities are enumerated and assigned stable save indices, then
/// each entity's components are read, resolving a contract's club `Entity` to that club's
/// save index so the relation survives a round-trip.
pub fn capture(world: &World) -> SaveData {
    let clock = world.resource::<SimClock>();
    let seed = world.get_resource::<SimSeed>().map_or(0, |s| s.0);

    let ids: Vec<Entity> = world.iter_entities().map(|e| e.id()).collect();
    let index: HashMap<Entity, u32> =
        ids.iter().enumerate().map(|(i, &e)| (e, i as u32)).collect();

    let entities = ids
        .iter()
        .enumerate()
        .map(|(i, &e)| {
            let er = world.entity(e);
            EntitySave {
                id: i as u32,
                birth: er.get::<BirthDate>().map(|b| b.0.into()),
                morale: er.get::<Morale>().map(|m| m.0),
                condition: er
                    .get::<Condition>()
                    .map(|c| ConditionSave { fitness: c.fitness, injury_days: c.injury_days }),
                contract: er.get::<Contract>().map(|c| ContractSave {
                    club: index[&c.club],
                    until: c.until.into(),
                    wage: c.wage,
                }),
                balance: er.get::<Balance>().map(|b| b.0),
                weekly_income: er.get::<WeeklyIncome>().map(|w| w.0),
                free_agent: er.contains::<FreeAgent>(),
                retired: er.contains::<Retired>(),
            }
        })
        .collect();

    SaveData {
        clock: ClockSave { date: clock.date().into(), day_index: clock.day_index() },
        seed,
        entities,
    }
}

/// Rebuild a fresh world from a [`SaveData`]. Entities are spawned first to fix the
/// save-index → live-`Entity` mapping, then components are inserted, resolving a
/// contract's club index back to the live club entity.
pub fn restore(data: &SaveData) -> World {
    let mut world = World::new();
    world.insert_resource(SimClock::from_parts(data.clock.date.into(), data.clock.day_index));
    world.insert_resource(SimSeed(data.seed));

    let map: HashMap<u32, Entity> =
        data.entities.iter().map(|rec| (rec.id, world.spawn_empty().id())).collect();

    for rec in &data.entities {
        let mut em = world.entity_mut(map[&rec.id]);
        if let Some(d) = rec.birth {
            em.insert(BirthDate(d.into()));
        }
        if let Some(m) = rec.morale {
            em.insert(Morale(m));
        }
        if let Some(c) = rec.condition {
            em.insert(Condition { fitness: c.fitness, injury_days: c.injury_days });
        }
        if let Some(c) = rec.contract {
            em.insert(Contract { club: map[&c.club], until: c.until.into(), wage: c.wage });
        }
        if let Some(b) = rec.balance {
            em.insert(Balance(b));
        }
        if let Some(w) = rec.weekly_income {
            em.insert(WeeklyIncome(w));
        }
        if rec.free_agent {
            em.insert(FreeAgent);
        }
        if rec.retired {
            em.insert(Retired);
        }
    }

    world
}

/// Bring an older [`SaveData`] up to the current [`FORMAT_VERSION`]. The migration seam:
/// future versions add arms here that transform the decoded data forward.
fn migrate(version: u16, data: SaveData) -> Result<SaveData, SaveError> {
    match version {
        FORMAT_VERSION => Ok(data),
        v if v > FORMAT_VERSION => Err(SaveError::FutureVersion(v)),
        // No older formats exist yet; once they do, each gets an arm that upgrades it.
        v => Err(SaveError::UnsupportedVersion(v)),
    }
}

/// Serialize the world to bytes: magic + version header, then the codec payload.
pub fn write_save(world: &World, codec: &impl SaveCodec) -> Result<Vec<u8>, SaveError> {
    let data = capture(world);
    let mut out = Vec::with_capacity(6);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&codec.encode(&data)?);
    Ok(out)
}

/// Read bytes back into a fresh world: validate the header, migrate the payload to the
/// current format, then restore.
pub fn read_save(bytes: &[u8], codec: &impl SaveCodec) -> Result<World, SaveError> {
    if bytes.len() < 6 || bytes[0..4] != MAGIC {
        return Err(SaveError::BadMagic);
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let data = migrate(version, codec.decode(&bytes[6..])?)?;
    Ok(restore(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_daily_schedule;

    /// A small world: a club with finances and three people in varied states.
    fn sample_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
        world.insert_resource(SimSeed(0xABCDEF));

        let club = world.spawn((Balance(5_000), WeeklyIncome(200))).id();
        world.spawn((
            BirthDate(Date::new(2000, 4, 1)),
            Morale(70),
            Condition { fitness: 80, injury_days: 5 },
            Contract { club, until: Date::new(2028, 6, 30), wage: 150 },
        ));
        world.spawn((BirthDate(Date::new(1990, 1, 1)), Morale(60), Condition::fit(), Retired));
        world
    }

    #[test]
    fn header_is_present_and_correct() {
        let world = sample_world();
        let bytes = write_save(&world, &BincodeCodec).unwrap();
        assert_eq!(&bytes[0..4], &MAGIC);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), FORMAT_VERSION);
    }

    #[test]
    fn round_trip_preserves_state_and_relations() {
        let mut world = sample_world();
        // Advance a few days so the clock's date and day_index are non-trivial.
        let mut sched = build_daily_schedule();
        for _ in 0..10 {
            sched.run(&mut world);
        }

        let before = capture(&world);
        let bytes = write_save(&world, &BincodeCodec).unwrap();
        let loaded = read_save(&bytes, &BincodeCodec).unwrap();
        let after = capture(&loaded);

        // Clock and seed survive exactly.
        assert_eq!(after.clock, before.clock);
        assert_eq!(after.seed, before.seed);
        assert_eq!(after.entities.len(), before.entities.len());

        // The contract→club relation still resolves to a club entity (one with finances),
        // proving the Entity reference was remapped correctly across the round-trip.
        let mut checked = false;
        for rec in &after.entities {
            if let Some(c) = rec.contract {
                let club = after.entities.iter().find(|e| e.id == c.club).unwrap();
                assert!(club.balance.is_some(), "contract must point at a club with a balance");
                checked = true;
            }
        }
        assert!(checked, "expected at least one contracted person");
    }

    #[test]
    fn loaded_world_continues_simulating() {
        let world = sample_world();
        let bytes = write_save(&world, &BincodeCodec).unwrap();
        let mut loaded = read_save(&bytes, &BincodeCodec).unwrap();

        // The injured player (5 days) should heal after 5 ticks on the loaded world.
        let mut sched = build_daily_schedule();
        for _ in 0..5 {
            sched.run(&mut loaded);
        }
        let any_injured = loaded
            .iter_entities()
            .filter_map(|e| e.get::<Condition>())
            .any(|c| c.is_injured());
        assert!(!any_injured, "injuries should have healed on the restored world");
    }

    #[test]
    fn rejects_bad_magic_and_future_version() {
        assert!(matches!(read_save(b"xx", &BincodeCodec), Err(SaveError::BadMagic)));
        assert!(matches!(read_save(b"NOPExx", &BincodeCodec), Err(SaveError::BadMagic)));

        let mut bytes = write_save(&sample_world(), &BincodeCodec).unwrap();
        bytes[4] = 0xFF; // bump version far past FORMAT_VERSION
        bytes[5] = 0xFF;
        assert!(matches!(read_save(&bytes, &BincodeCodec), Err(SaveError::FutureVersion(_))));
    }
}
