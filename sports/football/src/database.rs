//! The starting database — editable pre-game data that seeds a new game.
//!
//! This is **not** a save. A save (`sim_core::persistence`) is a binary snapshot of an
//! in-play world; the database is the human-authored *source* a new game is built from: named
//! clubs, named players with abilities, the division structure. It is stored as JSON so it can
//! be hand-edited, diffed, and edited by the standalone editor app. [`load_world`] turns a
//! database into a ready-to-play ECS world.
//!
//! Football-specific because player records carry football abilities; the genuinely shared
//! parts (names, clubs, contracts) come from `sim-core`. A sport-agnostic database format is a
//! later generalization, harvested once a second sport needs one.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::attributes::Footballer;
use sim_core::{
    sync_squad_membership, BirthDate, ClubBundle, Condition, Contract, Date, FreeAgent, Morale,
    Name, SimClock, SimSeed, WageDemand,
};

/// A calendar date in the database — a serializable mirror of `sim_core::Date`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl DbDate {
    fn to_date(self) -> Date {
        Date::new(self.year, self.month, self.day)
    }
}

/// A club in the starting database.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClubRecord {
    pub id: u32,
    pub name: String,
    pub balance: i64,
    pub weekly_income: i64,
    pub squad_target: u32,
}

/// A player in the starting database. `club_id` `None` means a free agent.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PlayerRecord {
    pub name: String,
    pub club_id: Option<u32>,
    pub birth: DbDate,
    pub attacking: u8,
    pub defending: u8,
    pub finishing: u8,
    pub goalkeeping: u8,
    pub wage: u32,
    pub contract_until: DbDate,
}

/// A division (one rung of the league pyramid) and its member clubs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DivisionRecord {
    pub name: String,
    pub tier: u32,
    pub club_ids: Vec<u32>,
}

/// A complete starting database.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Database {
    pub name: String,
    pub start_date: DbDate,
    pub seed: u64,
    pub divisions: Vec<DivisionRecord>,
    pub clubs: Vec<ClubRecord>,
    pub players: Vec<PlayerRecord>,
}

impl Database {
    /// Serialize to pretty JSON (what the editor reads/writes).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a Database always serializes")
    }

    /// Parse from JSON.
    pub fn from_json(s: &str) -> serde_json::Result<Database> {
        serde_json::from_str(s)
    }

    /// Referential-integrity check: unique club ids, and every player/division reference
    /// points at a club that exists.
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = std::collections::HashSet::new();
        for c in &self.clubs {
            if !ids.insert(c.id) {
                return Err(format!("duplicate club id {}", c.id));
            }
        }
        for p in &self.players {
            if let Some(cid) = p.club_id {
                if !ids.contains(&cid) {
                    return Err(format!("player '{}' references missing club {cid}", p.name));
                }
            }
        }
        for d in &self.divisions {
            for &cid in &d.club_ids {
                if !ids.contains(&cid) {
                    return Err(format!("division '{}' references missing club {cid}", d.name));
                }
            }
        }
        Ok(())
    }

    /// A small, valid sample database to start from or open in the editor: two divisions of
    /// six clubs, squads of named players, plus a handful of free agents.
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
                // Top division is stronger on average.
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
                    players.push(PlayerRecord {
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
                        attacking: base,
                        defending: base.saturating_sub(3),
                        finishing: base.saturating_add(2).min(99),
                        goalkeeping: base.saturating_sub(5),
                        wage: 1_000 + u32::from(base) * 50,
                        contract_until: DbDate { year: 2027 + (p % 3) as i32, month: 6, day: 30 },
                    });
                }
                club_ids.push(id);
                id += 1;
            }
            divisions.push(DivisionRecord { name: format!("Division {}", div + 1), tier: div + 1, club_ids });
        }

        for k in 0..8u32 {
            let base = (40 + (k * 4) % 25) as u8;
            players.push(PlayerRecord {
                name: format!("{} {}", FIRST[(k as usize + 3) % 10], LAST[(k as usize + 5) % 12]),
                club_id: None,
                birth: DbDate { year: 2025 - (18 + (k % 15) as i32), month: 3, day: 12 },
                attacking: base,
                defending: base,
                finishing: base,
                goalkeeping: base,
                wage: 800 + u32::from(base) * 40,
                contract_until: start,
            });
        }

        Database { name: "Sample League".into(), start_date: start, seed: 0xDA7A, divisions, clubs, players }
    }
}

/// Build a ready-to-play ECS world from a database: spawn named clubs and players (contracted
/// or free), then derive squad membership from the contracts.
pub fn load_world(db: &Database) -> World {
    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(db.start_date.to_date()));
    world.insert_resource(SimSeed(db.seed));

    let mut club_entity: HashMap<u32, Entity> = HashMap::new();
    for c in &db.clubs {
        let e = world
            .spawn((
                ClubBundle::new(c.id, c.balance, c.weekly_income, c.squad_target),
                Name(c.name.clone()),
            ))
            .id();
        club_entity.insert(c.id, e);
    }

    for p in &db.players {
        let mut e = world.spawn((
            Footballer {
                attacking: p.attacking,
                defending: p.defending,
                finishing: p.finishing,
                goalkeeping: p.goalkeeping,
            },
            Name(p.name.clone()),
            BirthDate(p.birth.to_date()),
            Morale(70),
            Condition::fit(),
        ));
        match p.club_id.and_then(|cid| club_entity.get(&cid)) {
            Some(&club) => {
                e.insert(Contract { club, until: p.contract_until.to_date(), wage: p.wage });
            }
            None => {
                e.insert((FreeAgent, WageDemand(p.wage)));
            }
        }
    }

    sync_squad_membership(&mut world);
    world
}

/// Write a database to a JSON file.
pub fn save(db: &Database, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::write(path, db.to_json())
}

/// Read a database from a JSON file.
pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Database> {
    let text = std::fs::read_to_string(path)?;
    Database::from_json(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gather_lineups;
    use sim_core::Club;

    #[test]
    fn sample_is_valid() {
        Database::sample().validate().unwrap();
    }

    #[test]
    fn json_round_trips() {
        let db = Database::sample();
        let back = Database::from_json(&db.to_json()).unwrap();
        assert_eq!(db, back);
    }

    #[test]
    fn load_world_builds_a_playable_world() {
        let db = Database::sample();
        let mut world = load_world(&db);

        // All clubs present.
        let clubs = world.query_filtered::<(), With<Club>>().iter(&world).count();
        assert_eq!(clubs, db.clubs.len());

        // Every club fields a lineup (so a season could start).
        let lineups = gather_lineups(&mut world);
        assert_eq!(lineups.len(), db.clubs.len());

        // Free agents made it in.
        let free = world.query_filtered::<(), With<FreeAgent>>().iter(&world).count();
        assert_eq!(free, db.players.iter().filter(|p| p.club_id.is_none()).count());
    }

    #[test]
    fn validate_catches_a_dangling_player_club() {
        let mut db = Database::sample();
        db.players[0].club_id = Some(9999);
        assert!(db.validate().is_err());
    }
}
