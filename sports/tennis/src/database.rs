//! Tennis's starting database — a second **individual** sport on the shared
//! [`sim_core::Database`] core, with a twist: a tennis player carries a *seed* (tournament
//! ranking) in addition to abilities, so its loader inserts two components from one ability
//! record — `TennisPlayer` and `Seed`. This shows the per-sport spawn hook is free to insert
//! whatever the sport needs — the core stays oblivious.
//!
//! No clubs or divisions: a flat roster of seeded individuals (`club_id: None`).

use crate::attributes::{Seed, TennisPlayer};
use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};
use sim_core::database::{DbDate, PersonRecord};

/// A tennis player's database fields: abilities plus the tournament seed (0 = top seed).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TennisDbAbility {
    pub serve: u8,
    pub return_game: u8,
    pub baseline: u8,
    pub mental: u8,
    pub seed: u32,
}

/// A tennis starting database (a flat seeded roster — no clubs/divisions).
pub type Database = sim_core::Database<TennisDbAbility>;

/// Build a world of seeded players from a database (inserts both `TennisPlayer` and `Seed`).
pub fn load_world(db: &Database) -> World {
    sim_core::database::load_world(db, |e, a: &TennisDbAbility| {
        e.insert((
            TennisPlayer {
                serve: a.serve,
                return_game: a.return_game,
                baseline: a.baseline,
                mental: a.mental,
            },
            Seed(a.seed),
        ));
    })
}

/// Write a tennis database to a JSON file.
pub fn save(db: &Database, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    sim_core::database::save(db, path)
}

/// Read a tennis database from a JSON file.
pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Database> {
    sim_core::database::load::<TennisDbAbility>(path)
}

/// A sample 32-player draw, seeded by strength.
pub fn sample() -> Database {
    const FIRST: [&str; 8] = ["Novak", "Carlos", "Jannik", "Daniil", "Stefanos", "Andrey", "Casper", "Taylor"];
    const LAST: [&str; 8] = ["Djokovic", "Alcaraz", "Sinner", "Medvedev", "Tsitsipas", "Rublev", "Ruud", "Fritz"];
    let start = DbDate { year: 2025, month: 1, day: 1 };

    let players = (0..32u32)
        .map(|s| {
            let base = 90 - (s as i32) * 2; // seed 0 ~90 down to ~28
            let jitter = (s % 5) as i32 - 2;
            let adj = |x: i32| (x + jitter).clamp(1, 99) as u8;
            PersonRecord {
                name: format!("{} {}", FIRST[(s as usize) % 8], LAST[(s as usize / 2) % 8]),
                club_id: None,
                birth: DbDate { year: 2025 - (19 + (s % 17) as i32), month: 1 + (s % 12) as u8, day: 1 + (s % 28) as u8 },
                wage: 0,
                contract_until: start,
                ability: TennisDbAbility {
                    serve: adj(base),
                    return_game: adj(base - 2),
                    baseline: adj(base - 1),
                    mental: adj(base - 3),
                    seed: s,
                },
            }
        })
        .collect();

    Database {
        name: "Sample Draw".into(),
        start_date: start,
        seed: 0x7E115,
        divisions: vec![],
        clubs: vec![],
        players,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gather_draw, simulate_tournament};

    #[test]
    fn sample_is_valid_and_round_trips() {
        let db = sample();
        db.validate().unwrap();
        assert_eq!(Database::from_json(&db.to_json()).unwrap(), db);
    }

    #[test]
    fn load_world_builds_a_playable_draw() {
        let db = sample();
        let mut world = load_world(&db);
        let draw = gather_draw(&mut world);
        assert_eq!(draw.len(), db.players.len());

        // The loaded draw can run a tournament to a champion.
        let result = simulate_tournament(&draw, 1, 1);
        assert!(result.champion < draw.len());
    }
}
