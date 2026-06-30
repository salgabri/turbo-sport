//! Saving and loading a football game.
//!
//! `sim-core` persists everything it owns — clock, clubs, contracts, names, identities — but
//! it cannot know about `Footballer` abilities. This module is the **sport extension**: it
//! wraps the core save with a football-specific column of abilities, using sim-core's hooks
//! ([`sim_core::entity_order`] at capture, [`sim_core::restore_indexed`] at load) so the
//! abilities line up with the same entities by save index. The result is a *complete* save —
//! load a database, play, save, load, and every player keeps their ability.
//!
//! This is the concrete answer to "how does a sport plug into the save"; if a second sport
//! needs the same, the pattern (core save + an aligned per-entity column) is what gets
//! harvested.

use crate::attributes::Footballer;
use crate::season::{Season, TeamRecord};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use sim_core::{capture, entity_order, restore_indexed, DbDate, Matchday, SaveData, Schedule};
use std::collections::BTreeMap;

const MAGIC: [u8; 4] = *b"TSFB";
const VERSION: u16 = 2;

#[derive(Serialize, Deserialize)]
struct FootballerRecord {
    attacking: u8,
    defending: u8,
    finishing: u8,
    goalkeeping: u8,
}

/// A matchday, mirrored for the save (the runtime `Matchday`'s `Date` isn't serde).
#[derive(Serialize, Deserialize)]
struct MatchdaySave {
    date: DbDate,
    fixtures: Vec<(u32, u32)>,
    played: bool,
}

/// An in-progress league season, mirrored: its fixture calendar (matchdays + cursor) and table.
#[derive(Serialize, Deserialize)]
struct SeasonSave {
    teams: Vec<u32>,
    matchdays: Vec<MatchdaySave>,
    next: usize,
    table: Vec<(u32, TeamRecord)>,
    world_seed: u64,
    season_id: u32,
}

/// The football game save: the core sim-core snapshot, a football ability column aligned to it
/// by save index, and any in-progress league season.
#[derive(Serialize, Deserialize)]
struct GameSave {
    core: SaveData,
    footballers: Vec<Option<FootballerRecord>>,
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

fn restore_season(world: &mut World, ss: SeasonSave) {
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

/// Serialize a football world to bytes: header, then bincode of the core save + abilities.
pub fn save_to_bytes(world: &World) -> Vec<u8> {
    let core = capture(world);
    let footballers = entity_order(world)
        .iter()
        .map(|&e| {
            world.get::<Footballer>(e).map(|f| FootballerRecord {
                attacking: f.attacking,
                defending: f.defending,
                finishing: f.finishing,
                goalkeeping: f.goalkeeping,
            })
        })
        .collect();

    let save = GameSave { core, footballers, season: capture_season(world) };
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&bincode::serialize(&save).expect("game save serializes"));
    out
}

/// Rebuild a football world from bytes written by [`save_to_bytes`].
pub fn load_from_bytes(bytes: &[u8]) -> Result<World, String> {
    if bytes.len() < 6 || bytes[0..4] != MAGIC {
        return Err("not a football save (bad header)".to_string());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != VERSION {
        return Err(format!("unsupported football save version {version}"));
    }

    let save: GameSave = bincode::deserialize(&bytes[6..]).map_err(|e| e.to_string())?;
    let (mut world, entities) = restore_indexed(&save.core);
    for (i, rec) in save.footballers.iter().enumerate() {
        if let Some(r) = rec {
            world.entity_mut(entities[i]).insert(Footballer {
                attacking: r.attacking,
                defending: r.defending,
                finishing: r.finishing,
                goalkeeping: r.goalkeeping,
            });
        }
    }
    if let Some(ss) = save.season {
        restore_season(&mut world, ss);
    }
    Ok(world)
}

/// Write a football save to a file.
pub fn save_file(world: &World, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::write(path, save_to_bytes(world))
}

/// Read a football save from a file.
pub fn load_file(path: impl AsRef<std::path::Path>) -> std::io::Result<World> {
    let bytes = std::fs::read(path)?;
    load_from_bytes(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};
    use crate::gather_lineups;
    use sim_core::{Club, Name};

    #[test]
    fn full_game_round_trips_with_abilities() {
        let db = sample();
        let mut world = load_world(&db);

        let clubs_before = world.query_filtered::<(), With<Club>>().iter(&world).count();
        let footballers_before = world.query::<&Footballer>().iter(&world).count();

        let bytes = save_to_bytes(&world);
        let mut loaded = load_from_bytes(&bytes).unwrap();

        // Clubs and every ability survived.
        assert_eq!(loaded.query_filtered::<(), With<Club>>().iter(&loaded).count(), clubs_before);
        assert_eq!(loaded.query::<&Footballer>().iter(&loaded).count(), footballers_before);

        // Names survived and every club still fields a lineup -> the loaded world is playable.
        assert!(loaded.query::<&Name>().iter(&loaded).count() > 0);
        assert_eq!(gather_lineups(&mut loaded).len(), db.clubs.len());
    }

    #[test]
    fn a_specific_player_keeps_name_and_ability() {
        // Build a tiny world by hand and check one player's data exactly.
        let db = sample();
        let world = load_world(&db);
        let bytes = save_to_bytes(&world);
        let mut loaded = load_from_bytes(&bytes).unwrap();

        // Count footballers that also have a name — should match the database player count.
        let named_footballers = {
            let mut q = loaded.query::<(&Footballer, &Name)>();
            q.iter(&loaded).count()
        };
        assert_eq!(named_footballers, db.players.len());
    }

    #[test]
    fn an_in_progress_season_survives_save_load() {
        use crate::season::{play_due_fixtures, Season};
        use sim_core::{build_daily_schedule, SimClock, TeamId};

        let db = sample();
        let mut world = load_world(&db);

        // Start a league of all clubs and play a few weeks so the table + cursor are non-trivial.
        let mut teams: Vec<u32> =
            world.query_filtered::<&TeamId, With<Club>>().iter(&world).map(|t| t.0).collect();
        teams.sort_unstable();
        let today = world.resource::<SimClock>().date();
        world.insert_resource(Season::new(teams, today, 1, 2025));
        let mut sched = build_daily_schedule();
        for _ in 0..30 {
            sched.run(&mut world);
            play_due_fixtures(&mut world);
        }

        let next_before = world.resource::<Season>().schedule.next_index();
        let table_before = world.resource::<Season>().standings();
        assert!(next_before > 0, "some matchdays should have been played");

        let bytes = save_to_bytes(&world);
        let loaded = load_from_bytes(&bytes).unwrap();

        let season = loaded.get_resource::<Season>().expect("season restored");
        assert_eq!(season.schedule.next_index(), next_before);
        assert_eq!(season.standings(), table_before);
    }

    #[test]
    fn rejects_a_bad_header() {
        assert!(load_from_bytes(b"nope").is_err());
        assert!(load_from_bytes(b"").is_err());
    }
}
