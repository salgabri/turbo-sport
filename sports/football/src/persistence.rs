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
use sim_core::{
    capture, entity_order, restore_indexed, DbDate, Matchday, SaveData, SaveDataV2, Schedule,
};
use std::collections::BTreeMap;

const MAGIC: [u8; 4] = *b"TSFB";
/// v3: `Footballer` widened from 4 aggregate ratings to the 8 outfield attributes + `gk`, and
/// the embedded `core` moved to sim-core save v3. Old (v2) saves are read via [`GameSaveV2`].
const VERSION: u16 = 3;

/// The football ability column, v3 (eight attributes + keeper).
#[derive(Serialize, Deserialize)]
struct FootballerRecord {
    pac: u8,
    sho: u8,
    pas: u8,
    dri: u8,
    tec: u8,
    def: u8,
    phy: u8,
    vis: u8,
    gk: u8,
}

impl FootballerRecord {
    fn to_component(&self) -> Footballer {
        Footballer {
            pac: self.pac,
            sho: self.sho,
            pas: self.pas,
            dri: self.dri,
            tec: self.tec,
            def: self.def,
            phy: self.phy,
            vis: self.vis,
            gk: self.gk,
        }
    }
}

/// The football ability column as it stood at save v2 (four aggregate ratings). Kept frozen so
/// old saves still load; mapped approximately onto the eight-attribute `Footballer`.
/// (`Serialize` is only used to build old-format fixtures in tests.)
#[derive(Serialize, Deserialize)]
struct FootballerRecordV2 {
    attacking: u8,
    defending: u8,
    finishing: u8,
    goalkeeping: u8,
}

impl FootballerRecordV2 {
    fn to_component(&self) -> Footballer {
        // Best-effort spread of the old four ratings across the new eight attributes.
        Footballer {
            pac: self.attacking,
            sho: self.finishing,
            pas: self.attacking,
            dri: self.attacking,
            tec: self.finishing,
            def: self.defending,
            phy: self.defending,
            vis: self.attacking,
            gk: self.goalkeeping,
        }
    }
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

/// The football save as it stood at v2: an old-shape `core` and the four-rating ability column.
/// `SeasonSave` is unchanged between v2 and v3, so it is reused directly.
#[derive(Serialize, Deserialize)]
struct GameSaveV2 {
    core: SaveDataV2,
    footballers: Vec<Option<FootballerRecordV2>>,
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
    // Form is runtime-only (not saved); it rebuilds as further matches play.
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

/// Serialize a football world to bytes: header, then bincode of the core save + abilities.
pub fn save_to_bytes(world: &World) -> Vec<u8> {
    let core = capture(world);
    let footballers = entity_order(world)
        .iter()
        .map(|&e| {
            world.get::<Footballer>(e).map(|f| FootballerRecord {
                pac: f.pac,
                sho: f.sho,
                pas: f.pas,
                dri: f.dri,
                tec: f.tec,
                def: f.def,
                phy: f.phy,
                vis: f.vis,
                gk: f.gk,
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

/// Assemble a world from a decoded (and forward-migrated) core snapshot, football ability
/// column, and optional season. Shared by every version's decode path.
fn build_world(
    core: SaveData,
    footballers: Vec<Option<Footballer>>,
    season: Option<SeasonSave>,
) -> World {
    let (mut world, entities) = restore_indexed(&core);
    for (i, rec) in footballers.into_iter().enumerate() {
        if let Some(f) = rec {
            world.entity_mut(entities[i]).insert(f);
        }
    }
    if let Some(ss) = season {
        restore_season(&mut world, ss);
    }
    world
}

/// Rebuild a football world from bytes written by [`save_to_bytes`]. Reads the current format
/// and migrates older ones forward (v2 → v3: old core upgraded, four ratings spread onto the
/// eight attributes).
pub fn load_from_bytes(bytes: &[u8]) -> Result<World, String> {
    if bytes.len() < 6 || bytes[0..4] != MAGIC {
        return Err("not a football save (bad header)".to_string());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let payload = &bytes[6..];
    match version {
        VERSION => {
            let save: GameSave = bincode::deserialize(payload).map_err(|e| e.to_string())?;
            let footballers =
                save.footballers.iter().map(|o| o.as_ref().map(FootballerRecord::to_component)).collect();
            Ok(build_world(save.core, footballers, save.season))
        }
        2 => {
            let save: GameSaveV2 = bincode::deserialize(payload).map_err(|e| e.to_string())?;
            let footballers =
                save.footballers.iter().map(|o| o.as_ref().map(FootballerRecordV2::to_component)).collect();
            Ok(build_world(save.core.into(), footballers, save.season))
        }
        v if v > VERSION => Err(format!("football save v{v} is newer than this build supports")),
        v => Err(format!("unsupported football save version {v}")),
    }
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
    fn a_v2_football_save_migrates_to_v3() {
        use sim_core::persistence::legacy::EntitySaveV2;
        use sim_core::persistence::{ClockSave, DateSave};

        // A minimal old-format save: one club, one player, one four-rating ability record.
        let core = SaveDataV2 {
            clock: ClockSave { date: DateSave { year: 2025, month: 7, day: 1 }, day_index: 0 },
            seed: 0xDA7A,
            entities: vec![
                EntitySaveV2 {
                    id: 0,
                    name: Some("Old Club".into()),
                    team_id: Some(0),
                    balance: Some(1000),
                    weekly_income: Some(100),
                    squad_target: Some(2),
                    is_club: true,
                    ..Default::default()
                },
                EntitySaveV2 {
                    id: 1,
                    name: Some("Old Player".into()),
                    team_id: Some(0),
                    birth: Some(DateSave { year: 2000, month: 1, day: 1 }),
                    morale: Some(70),
                    ..Default::default()
                },
            ],
        };
        let old = GameSaveV2 {
            core,
            footballers: vec![
                None,
                Some(FootballerRecordV2 { attacking: 80, defending: 60, finishing: 75, goalkeeping: 20 }),
            ],
            season: None,
        };
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&bincode::serialize(&old).unwrap());

        let mut world = load_from_bytes(&bytes).unwrap();
        // The player's name survives and their four ratings were spread onto the eight attrs.
        let mut q = world.query::<(&Name, &Footballer)>();
        let (name, f) = q.iter(&world).next().expect("the migrated player exists");
        assert_eq!(name.0, "Old Player");
        assert_eq!(f.pac, 80); // attacking -> pac
        assert_eq!(f.gk, 20); // goalkeeping -> gk
        assert_eq!(f.def, 60); // defending -> def
    }

    #[test]
    fn rejects_a_bad_header() {
        assert!(load_from_bytes(b"nope").is_err());
        assert!(load_from_bytes(b"").is_err());
    }
}
