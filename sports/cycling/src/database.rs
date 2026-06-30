//! Cycling's starting database — an **individual** sport on the shared
//! [`sim_core::Database`] core.
//!
//! The contrast that proves the core generalizes beyond team sports: cycling has no clubs and
//! no divisions. The database is a flat roster of riders — `clubs`/`divisions` are empty and
//! every rider is unattached (`club_id: None`). Only [`CyclingAbility`] and how to spawn the
//! `Rider` component are cycling's; everything else is the shared core.

use crate::attributes::Rider;
use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};
use sim_core::database::{DbDate, PersonRecord};

/// A rider's ability fields (flattened into each rider in the JSON).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CyclingAbility {
    pub climbing: u8,
    pub sprinting: u8,
    pub time_trial: u8,
    pub endurance: u8,
}

/// A cycling starting database (a flat roster — no clubs/divisions).
pub type Database = sim_core::Database<CyclingAbility>;

/// Build a world of riders from a database.
pub fn load_world(db: &Database) -> World {
    sim_core::database::load_world(db, |e, a: &CyclingAbility| {
        e.insert(Rider {
            climbing: a.climbing,
            sprinting: a.sprinting,
            time_trial: a.time_trial,
            endurance: a.endurance,
        });
    })
}

/// Write a cycling database to a JSON file.
pub fn save(db: &Database, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    sim_core::database::save(db, path)
}

/// Read a cycling database from a JSON file.
pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Database> {
    sim_core::database::load::<CyclingAbility>(path)
}

/// A sample peloton: a flat roster of riders across the usual archetypes.
pub fn sample() -> Database {
    const FIRST: [&str; 8] = ["Tadej", "Jonas", "Remco", "Primoz", "Wout", "Mathieu", "Egan", "Geraint"];
    const LAST: [&str; 8] = ["Pogacar", "Vingegaard", "Evenepoel", "Roglic", "Lampaert", "Poel", "Bernal", "Thomas"];
    let start = DbDate { year: 2025, month: 1, day: 1 };

    let players = (0..60u32)
        .map(|i| {
            // Four archetypes: climber, sprinter, time-trialist, all-rounder.
            let (clb, spr, tt, end) = match i % 4 {
                0 => (88, 45, 70, 80),
                1 => (45, 90, 55, 72),
                2 => (60, 60, 88, 78),
                _ => (76, 68, 76, 84),
            };
            let jitter = (i % 7) as i32 - 3;
            let adj = |x: i32| (x + jitter).clamp(1, 99) as u8;
            PersonRecord {
                name: format!("{} {}", FIRST[(i as usize) % 8], LAST[(i as usize / 2) % 8]),
                club_id: None,
                birth: DbDate { year: 2025 - (20 + (i % 16) as i32), month: 1 + (i % 12) as u8, day: 1 + (i % 28) as u8 },
                wage: 0,
                contract_until: start,
                ability: CyclingAbility {
                    climbing: adj(clb),
                    sprinting: adj(spr),
                    time_trial: adj(tt),
                    endurance: adj(end),
                },
            }
        })
        .collect();

    Database {
        name: "Sample Peloton".into(),
        start_date: start,
        seed: 0xC1C0,
        divisions: vec![],
        clubs: vec![],
        players,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gather_riders;

    #[test]
    fn sample_is_valid_and_round_trips() {
        let db = sample();
        db.validate().unwrap();
        assert_eq!(Database::from_json(&db.to_json()).unwrap(), db);
    }

    #[test]
    fn load_world_builds_a_peloton() {
        let db = sample();
        let mut world = load_world(&db);
        let riders = gather_riders(&mut world);
        assert_eq!(riders.len(), db.players.len());
    }
}
