//! Saving and loading a basketball game — the same sport-extension pattern as football
//! (`sim-core` core save + an aligned `Baller` ability column + the in-progress season).

use crate::attributes::Baller;
use crate::season::{Season, TeamRecord};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use sim_core::{
    capture, entity_order, restore_indexed, DbDate, Matchday, SaveData, SaveDataV2, Schedule,
};
use std::collections::BTreeMap;

const MAGIC: [u8; 4] = *b"TSBB";
/// v2: `Baller` widened from four aggregate ratings to the six design attributes, and the
/// embedded core moved to sim-core save v3. Old (v1) saves load via [`GameSaveV1`].
const VERSION: u16 = 2;

/// The basketball ability column, v2 (six attributes).
#[derive(Serialize, Deserialize)]
struct BallerRecord {
    ins: u8,
    out: u8,
    pm: u8,
    reb: u8,
    def: u8,
    ath: u8,
}

impl BallerRecord {
    fn to_component(&self) -> Baller {
        Baller { ins: self.ins, out: self.out, pm: self.pm, reb: self.reb, def: self.def, ath: self.ath }
    }
}

/// The basketball ability column as it stood at save v1 (four aggregate ratings). Kept frozen so
/// old saves still load; mapped approximately onto the six attributes. (`Serialize` only used to
/// build fixtures in tests.)
#[derive(Serialize, Deserialize)]
struct BallerRecordV1 {
    offense: u8,
    defense: u8,
    three_point: u8,
    rebounding: u8,
}

impl BallerRecordV1 {
    fn to_component(&self) -> Baller {
        Baller {
            ins: self.offense,
            out: self.three_point,
            pm: self.offense,
            reb: self.rebounding,
            def: self.defense,
            ath: self.defense,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct MatchdaySave {
    date: DbDate,
    fixtures: Vec<(u32, u32)>,
    played: bool,
}

#[derive(Serialize, Deserialize)]
struct SeasonSave {
    teams: Vec<u32>,
    matchdays: Vec<MatchdaySave>,
    next: usize,
    table: Vec<(u32, TeamRecord)>,
    world_seed: u64,
    season_id: u32,
}

#[derive(Serialize, Deserialize)]
struct GameSave {
    core: SaveData,
    ballers: Vec<Option<BallerRecord>>,
    season: Option<SeasonSave>,
}

/// The basketball save as it stood at v1: an old-shape `core` and the four-rating ability
/// column. `SeasonSave` is unchanged, so it is reused directly.
#[derive(Serialize, Deserialize)]
struct GameSaveV1 {
    core: SaveDataV2,
    ballers: Vec<Option<BallerRecordV1>>,
    season: Option<SeasonSave>,
}

fn capture_season(world: &World) -> Option<SeasonSave> {
    let s = world.get_resource::<Season>()?;
    let matchdays = (0..s.schedule.len())
        .map(|i| {
            let m = s.schedule.matchday(i);
            MatchdaySave { date: m.date.into(), fixtures: m.fixtures.clone(), played: m.played }
        })
        .collect();
    Some(SeasonSave {
        teams: s.teams.clone(),
        matchdays,
        next: s.schedule.next_index(),
        table: s.table.iter().map(|(&k, &v)| (k, v)).collect(),
        world_seed: s.world_seed,
        season_id: s.season_id,
    })
}

/// Serialize a basketball world to bytes.
pub fn save_to_bytes(world: &World) -> Vec<u8> {
    let core = capture(world);
    let ballers = entity_order(world)
        .iter()
        .map(|&e| {
            world.get::<Baller>(e).map(|b| BallerRecord {
                ins: b.ins,
                out: b.out,
                pm: b.pm,
                reb: b.reb,
                def: b.def,
                ath: b.ath,
            })
        })
        .collect();
    let save = GameSave { core, ballers, season: capture_season(world) };
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&bincode::serialize(&save).expect("game save serializes"));
    out
}

fn restore_season(world: &mut World, ss: SeasonSave) {
    let matchdays = ss
        .matchdays
        .into_iter()
        .map(|m| Matchday { date: m.date.to_date(), fixtures: m.fixtures, played: m.played })
        .collect();
    let table: BTreeMap<u32, TeamRecord> = ss.table.into_iter().collect();
    let form = ss.teams.iter().map(|&t| (t, Vec::new())).collect();
    world.insert_resource(Season {
        teams: ss.teams,
        schedule: Schedule::from_parts(matchdays, ss.next),
        table,
        world_seed: ss.world_seed,
        season_id: ss.season_id,
        form,
    });
}

fn build_world(core: SaveData, ballers: Vec<Option<Baller>>, season: Option<SeasonSave>) -> World {
    let (mut world, entities) = restore_indexed(&core);
    for (i, rec) in ballers.into_iter().enumerate() {
        if let Some(b) = rec {
            world.entity_mut(entities[i]).insert(b);
        }
    }
    if let Some(ss) = season {
        restore_season(&mut world, ss);
    }
    world
}

/// Rebuild a basketball world from bytes written by [`save_to_bytes`]. Reads the current format
/// and migrates v1 forward (old core upgraded, four ratings spread onto the six attributes).
pub fn load_from_bytes(bytes: &[u8]) -> Result<World, String> {
    if bytes.len() < 6 || bytes[0..4] != MAGIC {
        return Err("not a basketball save (bad header)".to_string());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let payload = &bytes[6..];
    match version {
        VERSION => {
            let save: GameSave = bincode::deserialize(payload).map_err(|e| e.to_string())?;
            let ballers = save.ballers.iter().map(|o| o.as_ref().map(BallerRecord::to_component)).collect();
            Ok(build_world(save.core, ballers, save.season))
        }
        1 => {
            let save: GameSaveV1 = bincode::deserialize(payload).map_err(|e| e.to_string())?;
            let ballers = save.ballers.iter().map(|o| o.as_ref().map(BallerRecordV1::to_component)).collect();
            Ok(build_world(save.core.into(), ballers, save.season))
        }
        v if v > VERSION => Err(format!("basketball save v{v} is newer than this build supports")),
        v => Err(format!("unsupported basketball save version {v}")),
    }
}

/// Write a basketball save to a file.
pub fn save_file(world: &World, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::write(path, save_to_bytes(world))
}

/// Read a basketball save from a file.
pub fn load_file(path: impl AsRef<std::path::Path>) -> std::io::Result<World> {
    let bytes = std::fs::read(path)?;
    load_from_bytes(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};
    use sim_core::Club;

    #[test]
    fn round_trips_with_abilities() {
        let world = load_world(&sample());
        let before = world.iter_entities().filter(|e| e.contains::<Baller>()).count();
        let bytes = save_to_bytes(&world);
        let mut loaded = load_from_bytes(&bytes).unwrap();
        let after = loaded.query::<&Baller>().iter(&loaded).count();
        assert_eq!(after, before);
        assert!(loaded.query_filtered::<(), With<Club>>().iter(&loaded).count() > 0);
    }

    #[test]
    fn a_v1_basketball_save_migrates_to_v2() {
        use sim_core::persistence::legacy::EntitySaveV2;
        use sim_core::persistence::{ClockSave, DateSave};
        use sim_core::Name;

        let core = SaveDataV2 {
            clock: ClockSave { date: DateSave { year: 2025, month: 10, day: 1 }, day_index: 0 },
            seed: 0xB00B,
            entities: vec![
                EntitySaveV2 { id: 0, name: Some("Capital Hoops".into()), team_id: Some(0), balance: Some(1000), is_club: true, ..Default::default() },
                EntitySaveV2 { id: 1, name: Some("Old Baller".into()), team_id: Some(0), morale: Some(70), ..Default::default() },
            ],
        };
        let old = GameSaveV1 {
            core,
            ballers: vec![None, Some(BallerRecordV1 { offense: 82, defense: 70, three_point: 60, rebounding: 55 })],
            season: None,
        };
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&bincode::serialize(&old).unwrap());

        let mut world = load_from_bytes(&bytes).unwrap();
        let mut q = world.query::<(&Name, &Baller)>();
        let (name, b) = q.iter(&world).next().expect("migrated player exists");
        assert_eq!(name.0, "Old Baller");
        assert_eq!(b.ins, 82); // offense -> ins
        assert_eq!(b.out, 60); // three_point -> out
        assert_eq!(b.reb, 55); // rebounding -> reb
    }
}
