//! Football's starting database: the sport-agnostic [`sim_core::Database`] core specialized to
//! football's ability record.
//!
//! The structure — clubs, divisions, contracts, the sample generator's shape — is shared
//! (sim-core). The only football parts are [`FootballAbility`] (the per-player ability fields,
//! flattened into the JSON so the file stays flat) and how to turn it into the `Footballer`
//! component on load. A second sport reuses the same core with its own ability type — see
//! `basketball::database`.

use crate::attributes::Footballer;
use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};
use sim_core::database::{ClubRecord, DbDate, DivisionRecord, PersonRecord};

/// Football's per-player ability fields. Flattened into each player in the database JSON.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FootballAbility {
    pub attacking: u8,
    pub defending: u8,
    pub finishing: u8,
    pub goalkeeping: u8,
}

/// A football starting database.
pub type Database = sim_core::Database<FootballAbility>;
/// A football player record.
pub type PlayerRecord = PersonRecord<FootballAbility>;

/// Build a ready-to-play football world from a database (clubs, contracted/free players with
/// `Footballer` abilities, squad membership synced).
pub fn load_world(db: &Database) -> World {
    sim_core::database::load_world(db, |e, a: &FootballAbility| {
        e.insert(Footballer {
            attacking: a.attacking,
            defending: a.defending,
            finishing: a.finishing,
            goalkeeping: a.goalkeeping,
        });
    })
}

/// Write a football database to a JSON file.
pub fn save(db: &Database, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    sim_core::database::save(db, path)
}

/// Read a football database from a JSON file.
pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Database> {
    sim_core::database::load::<FootballAbility>(path)
}

/// A small, valid sample database: two divisions of six clubs, squads of named players, plus a
/// handful of free agents.
pub fn sample() -> Database {
    const FIRST: [&str; 10] =
        ["Alex", "Sam", "Jordan", "Chris", "Luca", "Marco", "Diego", "Liam", "Noah", "Ethan"];
    const LAST: [&str; 12] = [
        "Smith", "Garcia", "Rossi", "Muller", "Silva", "Dubois", "Kovac", "Nowak", "Bauer",
        "Lopez", "Costa", "Haas",
    ];
    const TOWNS: [&str; 12] = [
        "Northgate", "Riverside", "Kingsford", "Ashton", "Bellmont", "Hartwell", "Westbrook",
        "Oakdale", "Stonebridge", "Fairview", "Lakeport", "Highcliff",
    ];
    let start = DbDate { year: 2025, month: 7, day: 1 };

    let mut clubs = Vec::new();
    let mut players = Vec::new();
    let mut divisions = Vec::new();

    let mut id = 0u32;
    for div in 0..2u32 {
        let mut club_ids = Vec::new();
        for _ in 0..6 {
            let strength = 50 + ((id * 3) % 25) as i32 - (div as i32) * 12;
            clubs.push(ClubRecord {
                id,
                name: format!("{} {}", TOWNS[id as usize % 12], if div == 0 { "FC" } else { "Town" }),
                balance: 20_000_000,
                weekly_income: 300_000,
                squad_target: 16,
            });
            for p in 0..16u32 {
                let base = (strength + ((p * 5 + id * 7) % 20) as i32 - 8).clamp(30, 92) as u8;
                let age = 17 + ((p * 3 + id) % 18) as i32;
                players.push(PersonRecord {
                    name: format!(
                        "{} {}",
                        FIRST[(p as usize + id as usize) % 10],
                        LAST[(p as usize * 2 + id as usize) % 12]
                    ),
                    club_id: Some(id),
                    birth: DbDate {
                        year: 2025 - age,
                        month: 1 + ((p + id) % 12) as u8,
                        day: 1 + ((p * 2) % 28) as u8,
                    },
                    wage: 1_000 + u32::from(base) * 50,
                    contract_until: DbDate { year: 2027 + (p % 3) as i32, month: 6, day: 30 },
                    ability: FootballAbility {
                        attacking: base,
                        defending: base.saturating_sub(3),
                        finishing: base.saturating_add(2).min(99),
                        goalkeeping: base.saturating_sub(5),
                    },
                });
            }
            club_ids.push(id);
            id += 1;
        }
        divisions.push(DivisionRecord { name: format!("Division {}", div + 1), tier: div + 1, club_ids });
    }

    for k in 0..8u32 {
        let base = (40 + (k * 4) % 25) as u8;
        players.push(PersonRecord {
            name: format!("{} {}", FIRST[(k as usize + 3) % 10], LAST[(k as usize + 5) % 12]),
            club_id: None,
            birth: DbDate { year: 2025 - (18 + (k % 15) as i32), month: 3, day: 12 },
            wage: 800 + u32::from(base) * 40,
            contract_until: start,
            ability: FootballAbility {
                attacking: base,
                defending: base,
                finishing: base,
                goalkeeping: base,
            },
        });
    }

    Database { name: "Sample League".into(), start_date: start, seed: 0xDA7A, divisions, clubs, players }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gather_lineups;
    use bevy_ecs::prelude::*;
    use sim_core::{Club, FreeAgent};

    #[test]
    fn sample_is_valid() {
        sample().validate().unwrap();
    }

    #[test]
    fn json_round_trips() {
        let db = sample();
        assert_eq!(Database::from_json(&db.to_json()).unwrap(), db);
    }

    #[test]
    fn load_world_builds_a_playable_world() {
        let db = sample();
        let mut world = load_world(&db);

        let clubs = world.query_filtered::<(), With<Club>>().iter(&world).count();
        assert_eq!(clubs, db.clubs.len());

        let lineups = gather_lineups(&mut world);
        assert_eq!(lineups.len(), db.clubs.len());

        let free = world.query_filtered::<(), With<FreeAgent>>().iter(&world).count();
        assert_eq!(free, db.players.iter().filter(|p| p.club_id.is_none()).count());
    }

    #[test]
    fn validate_catches_a_dangling_player_club() {
        let mut db = sample();
        db.players[0].club_id = Some(9999);
        assert!(db.validate().is_err());
    }
}
