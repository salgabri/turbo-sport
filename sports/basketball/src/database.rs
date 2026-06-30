//! Basketball's starting database — the **second** sport on the shared
//! [`sim_core::Database`] core, which is what justifies the generalization. Only the ability
//! record ([`BasketballAbility`]) and how to spawn the `Baller` component are basketball's;
//! the clubs/divisions/contracts structure, JSON I/O, validation, and the world loader are all
//! shared, unchanged, with football.

use crate::attributes::Baller;
use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};
use sim_core::database::{ClubRecord, DbDate, DivisionRecord, PersonRecord};

/// Basketball's per-player ability fields (flattened into each player in the JSON).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BasketballAbility {
    pub offense: u8,
    pub defense: u8,
    pub three_point: u8,
    pub rebounding: u8,
}

/// A basketball starting database.
pub type Database = sim_core::Database<BasketballAbility>;

/// Build a ready-to-play basketball world from a database.
pub fn load_world(db: &Database) -> World {
    sim_core::database::load_world(db, |e, a: &BasketballAbility| {
        e.insert(Baller {
            offense: a.offense,
            defense: a.defense,
            three_point: a.three_point,
            rebounding: a.rebounding,
        });
    })
}

/// Write a basketball database to a JSON file.
pub fn save(db: &Database, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    sim_core::database::save(db, path)
}

/// Read a basketball database from a JSON file.
pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Database> {
    sim_core::database::load::<BasketballAbility>(path)
}

/// A small sample league: one division of eight clubs with named rosters.
pub fn sample() -> Database {
    const FIRST: [&str; 8] = ["Jay", "Marcus", "Tyler", "Andre", "Kobe", "Luka", "Nikola", "Trae"];
    const LAST: [&str; 8] = ["Johnson", "Williams", "Brooks", "Carter", "Davis", "Young", "Reed", "Foster"];
    const CITIES: [&str; 8] =
        ["Capital", "Harbor", "Summit", "Valley", "Metro", "Coastal", "Granite", "Delta"];
    let start = DbDate { year: 2025, month: 10, day: 1 };

    let mut clubs = Vec::new();
    let mut players = Vec::new();
    let mut club_ids = Vec::new();

    for id in 0..8u32 {
        let strength = 48 + ((id * 5) % 24) as i32;
        clubs.push(ClubRecord {
            id,
            name: format!("{} {}", CITIES[id as usize % 8], "Hoops"),
            balance: 30_000_000,
            weekly_income: 400_000,
            squad_target: 10,
        });
        for p in 0..10u32 {
            let base = (strength + ((p * 7 + id * 3) % 18) as i32 - 9).clamp(35, 95) as u8;
            let age = 19 + ((p * 2 + id) % 16) as i32;
            players.push(PersonRecord {
                name: format!(
                    "{} {}",
                    FIRST[(p as usize + id as usize) % 8],
                    LAST[(p as usize * 3 + id as usize) % 8]
                ),
                club_id: Some(id),
                birth: DbDate { year: 2025 - age, month: 1 + ((p + id) % 12) as u8, day: 1 + ((p * 2) % 28) as u8 },
                wage: 5_000 + u32::from(base) * 100,
                contract_until: DbDate { year: 2027 + (p % 3) as i32, month: 6, day: 30 },
                ability: BasketballAbility {
                    offense: base,
                    defense: base.saturating_sub(2),
                    three_point: base.saturating_sub(6),
                    rebounding: base,
                },
            });
        }
        club_ids.push(id);
    }

    let divisions = vec![DivisionRecord { name: "Conference".into(), tier: 1, club_ids }];
    Database { name: "Sample Basketball".into(), start_date: start, seed: 0xB00B, divisions, clubs, players }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gather_rosters;
    use bevy_ecs::prelude::*;
    use sim_core::Club;

    #[test]
    fn sample_is_valid_and_round_trips() {
        let db = sample();
        db.validate().unwrap();
        assert_eq!(Database::from_json(&db.to_json()).unwrap(), db);
    }

    #[test]
    fn load_world_builds_a_playable_world() {
        let db = sample();
        let mut world = load_world(&db);
        assert_eq!(world.query_filtered::<(), With<Club>>().iter(&world).count(), db.clubs.len());
        // Every club fields a roster -> a season could start.
        assert_eq!(gather_rosters(&mut world).len(), db.clubs.len());
    }
}
