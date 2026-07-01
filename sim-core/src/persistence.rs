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

use crate::club::Club;
use crate::economy::{Balance, MarketValue, WeeklyIncome};
use crate::entity::{
    BirthDate, Condition, Contract, FreeAgent, Morale, Name, Nationality, PositionGroup, Rating,
    Retired, SquadTarget, TeamId, WageDemand,
};
use crate::rng::SimSeed;
use crate::time::{Date, SimClock};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Identifies the file as one of ours.
pub const MAGIC: [u8; 4] = *b"TSPS";

/// The save format version. Bump on any breaking change to [`SaveData`] and add a decode arm
/// in [`read_save`]. History:
/// - v2 added name / team_id / squad_target / wage_demand / is_club to [`EntitySave`].
/// - v3 added nationality / position_group / rating / market_value to [`EntitySave`] and the
///   rating/value roll-ups to the DTOs. `bincode` is positional (not self-describing), so an
///   old save is read by decoding it into the frozen [`legacy`] mirror and converting forward.
pub const FORMAT_VERSION: u16 = 3;

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

/// A player's rating, mirrored.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RatingSave {
    pub overall: u8,
    pub potential: u8,
}

/// One entity's `sim-core` components. Every field is optional so the same record serves
/// people (birth/contract/condition) and clubs (balance/income).
///
/// **Adding a field is a breaking change for `bincode`** (positional format): bump
/// [`FORMAT_VERSION`], snapshot the previous shape into [`legacy`], and add a decode arm to
/// [`read_save`]. Do NOT rely on `#[serde(default)]` for back-compat — that only helps
/// self-describing formats (JSON), not bincode.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct EntitySave {
    /// Stable index assigned at capture; references (e.g. a contract's club) use it.
    pub id: u32,
    pub name: Option<String>,
    pub team_id: Option<u32>,
    pub birth: Option<DateSave>,
    pub morale: Option<u8>,
    pub condition: Option<ConditionSave>,
    pub contract: Option<ContractSave>,
    pub balance: Option<i64>,
    pub weekly_income: Option<i64>,
    pub squad_target: Option<u32>,
    pub wage_demand: Option<u32>,
    pub free_agent: bool,
    pub retired: bool,
    /// True if this entity is a club (carries the `Club` marker).
    pub is_club: bool,
    // ---- v3 additions ----
    pub nationality: Option<String>,
    pub position_group: Option<u8>,
    pub rating: Option<RatingSave>,
    pub market_value: Option<i64>,
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
                name: er.get::<Name>().map(|n| n.0.clone()),
                team_id: er.get::<TeamId>().map(|t| t.0),
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
                squad_target: er.get::<SquadTarget>().map(|s| s.0),
                wage_demand: er.get::<WageDemand>().map(|w| w.0),
                free_agent: er.contains::<FreeAgent>(),
                retired: er.contains::<Retired>(),
                is_club: er.contains::<Club>(),
                nationality: er.get::<Nationality>().map(|n| n.0.clone()),
                position_group: er.get::<PositionGroup>().map(|p| p.0),
                rating: er
                    .get::<Rating>()
                    .map(|r| RatingSave { overall: r.overall, potential: r.potential }),
                market_value: er.get::<MarketValue>().map(|v| v.0),
            }
        })
        .collect();

    SaveData {
        clock: ClockSave { date: clock.date().into(), day_index: clock.day_index() },
        seed,
        entities,
    }
}

/// Rebuild a world from a [`SaveData`], also returning a `Vec` mapping each save index to its
/// live `Entity`. A sport uses this (with [`entity_order`] at capture time) to re-attach its
/// own per-entity components after the core state is restored.
///
/// Entities are spawned first to fix the save-index → live-`Entity` mapping, then components
/// are inserted, resolving a contract's club index back to the live club entity.
pub fn restore_indexed(data: &SaveData) -> (World, Vec<Entity>) {
    let mut world = World::new();
    world.insert_resource(SimClock::from_parts(data.clock.date.into(), data.clock.day_index));
    world.insert_resource(SimSeed(data.seed));

    let map: HashMap<u32, Entity> =
        data.entities.iter().map(|rec| (rec.id, world.spawn_empty().id())).collect();

    for rec in &data.entities {
        let mut em = world.entity_mut(map[&rec.id]);
        if let Some(n) = &rec.name {
            em.insert(Name(n.clone()));
        }
        if let Some(t) = rec.team_id {
            em.insert(TeamId(t));
        }
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
        if let Some(s) = rec.squad_target {
            em.insert(SquadTarget(s));
        }
        if let Some(w) = rec.wage_demand {
            em.insert(WageDemand(w));
        }
        if rec.free_agent {
            em.insert(FreeAgent);
        }
        if rec.retired {
            em.insert(Retired);
        }
        if rec.is_club {
            em.insert(Club);
        }
        if let Some(n) = &rec.nationality {
            em.insert(Nationality(n.clone()));
        }
        if let Some(p) = rec.position_group {
            em.insert(PositionGroup(p));
        }
        if let Some(r) = rec.rating {
            em.insert(Rating { overall: r.overall, potential: r.potential });
        }
        if let Some(v) = rec.market_value {
            em.insert(MarketValue(v));
        }
    }

    // Save ids are 0..n in capture order, so a positional vec maps index -> entity.
    let mut ordered = vec![Entity::PLACEHOLDER; data.entities.len()];
    for rec in &data.entities {
        ordered[rec.id as usize] = map[&rec.id];
    }
    (world, ordered)
}

/// Rebuild a fresh world from a [`SaveData`].
pub fn restore(data: &SaveData) -> World {
    restore_indexed(data).0
}

/// The order `capture` assigns save indices in: `capture(world).entities[i]` describes
/// `entity_order(world)[i]`. A sport captures its own per-entity components in this same order
/// so they line up with the indices for [`restore_indexed`].
pub fn entity_order(world: &World) -> Vec<Entity> {
    world.iter_entities().map(|e| e.id()).collect()
}

/// Frozen mirrors of older save shapes, kept so old bytes can still be decoded.
///
/// `bincode` reads a struct positionally, so once [`EntitySave`] grows a field, old payloads no
/// longer match it — they must be decoded into the *old* struct and converted forward. Each
/// past [`FORMAT_VERSION`] keeps its exact shape here. These types are `pub` so a sport's own
/// save (which embeds [`SaveData`], e.g. `football::persistence`) can compose the same
/// migration for its bytes.
pub mod legacy {
    use super::{ClockSave, ConditionSave, ContractSave, DateSave};
    use serde::{Deserialize, Serialize};

    /// [`super::EntitySave`] as it stood at `FORMAT_VERSION` 2 (before the v3 rating fields).
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
    pub struct EntitySaveV2 {
        pub id: u32,
        pub name: Option<String>,
        pub team_id: Option<u32>,
        pub birth: Option<DateSave>,
        pub morale: Option<u8>,
        pub condition: Option<ConditionSave>,
        pub contract: Option<ContractSave>,
        pub balance: Option<i64>,
        pub weekly_income: Option<i64>,
        pub squad_target: Option<u32>,
        pub wage_demand: Option<u32>,
        pub free_agent: bool,
        pub retired: bool,
        pub is_club: bool,
    }

    /// [`super::SaveData`] as it stood at `FORMAT_VERSION` 2.
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct SaveDataV2 {
        pub clock: ClockSave,
        pub seed: u64,
        pub entities: Vec<EntitySaveV2>,
    }
}

/// Public alias for the v2 [`SaveData`] shape, for sports composing their own migration.
pub use legacy::SaveDataV2;

impl From<legacy::EntitySaveV2> for EntitySave {
    fn from(o: legacy::EntitySaveV2) -> Self {
        EntitySave {
            id: o.id,
            name: o.name,
            team_id: o.team_id,
            birth: o.birth,
            morale: o.morale,
            condition: o.condition,
            contract: o.contract,
            balance: o.balance,
            weekly_income: o.weekly_income,
            squad_target: o.squad_target,
            wage_demand: o.wage_demand,
            free_agent: o.free_agent,
            retired: o.retired,
            is_club: o.is_club,
            // v3 additions: unknown in a v2 save.
            nationality: None,
            position_group: None,
            rating: None,
            market_value: None,
        }
    }
}

impl From<legacy::SaveDataV2> for SaveData {
    fn from(o: legacy::SaveDataV2) -> Self {
        SaveData {
            clock: o.clock,
            seed: o.seed,
            entities: o.entities.into_iter().map(EntitySave::from).collect(),
        }
    }
}

/// Decode a payload of a given save version into the current [`SaveData`], migrating forward.
///
/// The current version goes through the pluggable [`SaveCodec`]; older versions are decoded
/// with `bincode` into their frozen [`legacy`] shape (historical saves were always bincode)
/// and converted. A future codec swap only affects the current-version path.
fn decode_versioned(
    version: u16,
    payload: &[u8],
    codec: &impl SaveCodec,
) -> Result<SaveData, SaveError> {
    match version {
        FORMAT_VERSION => codec.decode(payload),
        2 => {
            let old: legacy::SaveDataV2 =
                bincode::deserialize(payload).map_err(SaveError::Codec)?;
            Ok(old.into())
        }
        v if v > FORMAT_VERSION => Err(SaveError::FutureVersion(v)),
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
    let data = decode_versioned(version, &bytes[6..], codec)?;
    Ok(restore(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_daily_schedule;

    /// A small world: a named club with identity + finances, and three people in varied states.
    fn sample_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
        world.insert_resource(SimSeed(0xABCDEF));

        let club = world
            .spawn((Club, TeamId(0), Name("Rovers".into()), Balance(5_000), WeeklyIncome(200), SquadTarget(20)))
            .id();
        world.spawn((
            Name("A Player".into()),
            TeamId(0),
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
    fn round_trip_preserves_names_team_ids_and_club_marker() {
        let world = sample_world();
        let bytes = write_save(&world, &BincodeCodec).unwrap();
        let loaded = read_save(&bytes, &BincodeCodec).unwrap();
        let after = capture(&loaded);

        let club = after.entities.iter().find(|e| e.is_club).expect("a club survives");
        assert_eq!(club.name.as_deref(), Some("Rovers"));
        assert_eq!(club.team_id, Some(0));
        assert_eq!(club.squad_target, Some(20));

        let player = after
            .entities
            .iter()
            .find(|e| e.name.as_deref() == Some("A Player"))
            .expect("named player survives");
        assert_eq!(player.team_id, Some(0));
        assert!(player.contract.is_some());
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
    fn v3_round_trips_rating_nationality_position_and_value() {
        use crate::economy::MarketValue;
        use crate::entity::{Nationality, PositionGroup, Rating};

        let mut world = sample_world();
        // A player carrying every v3 field.
        world.spawn((
            Name("Ada".into()),
            TeamId(0),
            BirthDate(Date::new(2001, 2, 3)),
            Nationality("BRA".into()),
            PositionGroup(2),
            Rating { overall: 84, potential: 89 },
            MarketValue(12_500_000),
        ));

        let bytes = write_save(&world, &BincodeCodec).unwrap();
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 3);
        let mut loaded = read_save(&bytes, &BincodeCodec).unwrap();

        let mut q = loaded.query::<(&Name, &Rating, &Nationality, &PositionGroup, &MarketValue)>();
        let hit = q
            .iter(&loaded)
            .find(|(n, ..)| n.0 == "Ada")
            .expect("the v3 player round-trips");
        assert_eq!(hit.1.overall, 84);
        assert_eq!(hit.1.potential, 89);
        assert_eq!(hit.2 .0, "BRA");
        assert_eq!(hit.3 .0, 2);
        assert_eq!(hit.4 .0, 12_500_000);
    }

    #[test]
    fn a_v2_save_migrates_forward_to_v3() {
        // Hand-build a v2 payload with the frozen v2 shapes, wrap it in a v2 header, and prove
        // the current build reads it — the v3 fields defaulting to absent.
        let v2 = legacy::SaveDataV2 {
            clock: ClockSave { date: DateSave { year: 2025, month: 7, day: 1 }, day_index: 3 },
            seed: 0xABCDEF,
            entities: vec![
                legacy::EntitySaveV2 {
                    id: 0,
                    name: Some("Rovers".into()),
                    team_id: Some(0),
                    balance: Some(5_000),
                    weekly_income: Some(200),
                    squad_target: Some(20),
                    is_club: true,
                    ..Default::default()
                },
                legacy::EntitySaveV2 {
                    id: 1,
                    name: Some("A Player".into()),
                    team_id: Some(0),
                    birth: Some(DateSave { year: 2000, month: 4, day: 1 }),
                    morale: Some(70),
                    contract: Some(ContractSave {
                        club: 0,
                        until: DateSave { year: 2028, month: 6, day: 30 },
                        wage: 150,
                    }),
                    ..Default::default()
                },
            ],
        };
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&2u16.to_le_bytes()); // v2 header
        bytes.extend_from_slice(&bincode::serialize(&v2).unwrap());

        let loaded = read_save(&bytes, &BincodeCodec).unwrap();
        let after = capture(&loaded);
        // Core state survives...
        let club = after.entities.iter().find(|e| e.is_club).expect("club survived");
        assert_eq!(club.name.as_deref(), Some("Rovers"));
        assert_eq!(club.squad_target, Some(20));
        let player = after
            .entities
            .iter()
            .find(|e| e.name.as_deref() == Some("A Player"))
            .expect("player survived");
        assert!(player.contract.is_some());
        // ...and the v3 fields are simply absent on a migrated v2 save.
        assert!(after.entities.iter().all(|e| e.rating.is_none() && e.nationality.is_none()));
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
