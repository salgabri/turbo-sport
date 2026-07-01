//! Basketball's starting database — the **second** sport on the shared
//! [`sim_core::Database`] core, which is what justifies the generalization. Only the ability
//! record ([`BasketballAbility`]) and how to spawn the runtime components are basketball's; the
//! clubs/divisions/contracts structure, JSON I/O, validation, and the world loader are all
//! shared with football.

use crate::attributes::{Baller, POS_C, POS_F, POS_G};
use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};
use sim_core::database::{ClubRecord, DbDate, DivisionRecord, PersonRecord};
use sim_core::{
    age_years, value_from, BirthDate, MarketValue, Nationality, PositionGroup, Rating, SimClock,
};

/// Basketball's per-player fields (flattened into each player in the JSON). The six attributes
/// are required; position/potential/nationality carry `#[serde(default)]` for older or hand
/// authored JSON.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BasketballAbility {
    pub ins: u8,
    pub out: u8,
    pub pm: u8,
    pub reb: u8,
    pub def: u8,
    pub ath: u8,
    /// Position group index: 0 G, 1 F, 2 C.
    #[serde(default)]
    pub position: u8,
    #[serde(default)]
    pub potential: u8,
    #[serde(default)]
    pub nationality: String,
}

/// A basketball starting database.
pub type Database = sim_core::Database<BasketballAbility>;

/// Build a ready-to-play basketball world. Each player gets the `Baller` ability plus the
/// sport-neutral rating/position/value/nationality the UI reads, derived deterministically.
pub fn load_world(db: &Database) -> World {
    sim_core::database::load_world(db, |e, a: &BasketballAbility| {
        let b = Baller { ins: a.ins, out: a.out, pm: a.pm, reb: a.reb, def: a.def, ath: a.ath };
        let overall = b.overall(a.position);
        let potential = a.potential.max(overall);
        let today = e.world_scope(|w| w.resource::<SimClock>().date());
        let age = e.get::<BirthDate>().map_or(25, |bd| age_years(bd.0, today));
        e.insert((
            b,
            PositionGroup(a.position),
            Rating { overall, potential },
            MarketValue(value_from(overall, age)),
        ));
        if !a.nationality.is_empty() {
            e.insert(Nationality(a.nationality.clone()));
        }
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

/// Position for the `slot`-th player in a 10-man roster: 4 G, 3 F, 3 C.
fn slot_position(slot: u32) -> u8 {
    match slot {
        0..=3 => POS_G,
        4..=6 => POS_F,
        _ => POS_C,
    }
}

/// Attributes for a player of a given position around a `base` rating, tilted to the position.
fn ability_for(position: u8, base: u8, nat: &str) -> BasketballAbility {
    let up = |v: u8, d: u8| v.saturating_add(d).min(99);
    let dn = |v: u8, d: u8| v.saturating_sub(d);
    let mut a = BasketballAbility {
        ins: base,
        out: base,
        pm: base,
        reb: base,
        def: base,
        ath: base,
        position,
        potential: up(base, 4),
        nationality: nat.to_string(),
    };
    match position {
        POS_G => {
            a.pm = up(base, 8);
            a.out = up(base, 6);
            a.reb = dn(base, 12);
            a.ins = dn(base, 4);
        }
        POS_C => {
            a.reb = up(base, 9);
            a.ins = up(base, 6);
            a.out = dn(base, 16);
            a.pm = dn(base, 12);
        }
        _ => {
            a.ins = up(base, 4);
            a.reb = up(base, 3);
        }
    }
    a
}

/// A small sample league: one division of eight clubs with named rosters.
pub fn sample() -> Database {
    const FIRST: [&str; 8] = ["Jay", "Marcus", "Tyler", "Andre", "Kobe", "Luka", "Nikola", "Trae"];
    const LAST: [&str; 8] = ["Johnson", "Williams", "Brooks", "Carter", "Davis", "Young", "Reed", "Foster"];
    const CITIES: [&str; 8] =
        ["Capital", "Harbor", "Summit", "Valley", "Metro", "Coastal", "Granite", "Delta"];
    const NATS: [&str; 8] = ["USA", "CAN", "ESP", "FRA", "SRB", "SLO", "GRE", "AUS"];
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
            let position = slot_position(p);
            let nat = NATS[(p as usize + id as usize) % NATS.len()];
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
                ability: ability_for(position, base, nat),
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
    use sim_core::{Club, Rating};

    #[test]
    fn sample_is_valid_and_round_trips() {
        let db = sample();
        db.validate().unwrap();
        assert_eq!(Database::from_json(&db.to_json()).unwrap(), db);
    }

    #[test]
    fn load_world_builds_a_playable_world_with_ratings() {
        let db = sample();
        let mut world = load_world(&db);
        assert_eq!(world.query_filtered::<(), With<Club>>().iter(&world).count(), db.clubs.len());
        assert_eq!(gather_rosters(&mut world).len(), db.clubs.len());
        let rated = world.query::<&Rating>().iter(&world).count();
        assert_eq!(rated, db.players.len());
        assert!(world.query::<&Rating>().iter(&world).all(|r| r.overall > 0 && r.potential >= r.overall));
    }
}
