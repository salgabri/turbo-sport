//! Saving and loading a basketball game — the same sport-extension pattern as football
//! (`sim-core` core save + an aligned `Baller` ability column + the in-progress season).

use crate::attributes::Baller;
use crate::season::{Season, TeamRecord};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use sim_core::{capture, entity_order, restore_indexed, DbDate, Matchday, SaveData, Schedule};
use std::collections::BTreeMap;

const MAGIC: [u8; 4] = *b"TSBB";
const VERSION: u16 = 1;

#[derive(Serialize, Deserialize)]
struct BallerRecord {
    offense: u8,
    defense: u8,
    three_point: u8,
    rebounding: u8,
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
                offense: b.offense,
                defense: b.defense,
                three_point: b.three_point,
                rebounding: b.rebounding,
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

/// Rebuild a basketball world from bytes written by [`save_to_bytes`].
pub fn load_from_bytes(bytes: &[u8]) -> Result<World, String> {
    if bytes.len() < 6 || bytes[0..4] != MAGIC {
        return Err("not a basketball save (bad header)".to_string());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != VERSION {
        return Err(format!("unsupported basketball save version {version}"));
    }
    let save: GameSave = bincode::deserialize(&bytes[6..]).map_err(|e| e.to_string())?;
    let (mut world, entities) = restore_indexed(&save.core);
    for (i, rec) in save.ballers.iter().enumerate() {
        if let Some(r) = rec {
            world.entity_mut(entities[i]).insert(Baller {
                offense: r.offense,
                defense: r.defense,
                three_point: r.three_point,
                rebounding: r.rebounding,
            });
        }
    }
    if let Some(ss) = save.season {
        let matchdays = ss
            .matchdays
            .into_iter()
            .map(|m| Matchday { date: m.date.to_date(), fixtures: m.fixtures, played: m.played })
            .collect();
        let table: BTreeMap<u32, TeamRecord> = ss.table.into_iter().collect();
        world.insert_resource(Season {
            teams: ss.teams,
            schedule: Schedule::from_parts(matchdays, ss.next),
            table,
            world_seed: ss.world_seed,
            season_id: ss.season_id,
        });
    }
    Ok(world)
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
}
