//! Football's starting database: the sport-agnostic [`sim_core::Database`] core specialized to
//! football's ability record.
//!
//! The structure — clubs, divisions, contracts, the sample generator's shape — is shared
//! (sim-core). The football parts are [`FootballAbility`] (each player's attributes, position,
//! potential and nationality, flattened into the JSON so the file stays flat) and how to turn
//! it into the runtime components on load: the `Footballer` ability plus the sport-neutral
//! [`sim_core`] facts every card needs — position group, overall/potential rating, market
//! value and nationality. A second sport reuses the same core with its own ability type.

use crate::attributes::{Footballer, POS_DEF, POS_FWD, POS_GK, POS_MID};
use bevy_ecs::prelude::World;
use serde::{Deserialize, Serialize};
use sim_core::database::{ClubRecord, DbDate, DivisionRecord, PersonRecord};
use sim_core::{
    age_years, value_from, BirthDate, MarketValue, Nationality, PositionGroup, Rating, SimClock,
};

/// Football's per-player fields, flattened into each player in the database JSON. The eight
/// outfield attributes are required; the rest carry `#[serde(default)]` so hand-authored or
/// older JSON without them still loads.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FootballAbility {
    pub pac: u8,
    pub sho: u8,
    pub pas: u8,
    pub dri: u8,
    pub tec: u8,
    pub def: u8,
    pub phy: u8,
    pub vis: u8,
    #[serde(default)]
    pub gk: u8,
    /// Position group index: 0 GK, 1 DEF, 2 MID, 3 FWD.
    #[serde(default)]
    pub position: u8,
    /// Peak potential rating (0..=99); clamped up to the current overall on load.
    #[serde(default)]
    pub potential: u8,
    #[serde(default)]
    pub nationality: String,
}

/// A football starting database.
pub type Database = sim_core::Database<FootballAbility>;
/// A football player record.
pub type PlayerRecord = PersonRecord<FootballAbility>;

/// Build a ready-to-play football world from a database. Each player gets the `Footballer`
/// ability plus the sport-neutral rating/position/value/nationality the UI reads — all derived
/// deterministically from the authored attributes (no RNG), so the world is reproducible.
pub fn load_world(db: &Database) -> World {
    sim_core::database::load_world(db, |e, a: &FootballAbility| {
        let f = Footballer {
            pac: a.pac,
            sho: a.sho,
            pas: a.pas,
            dri: a.dri,
            tec: a.tec,
            def: a.def,
            phy: a.phy,
            vis: a.vis,
            gk: a.gk,
        };
        let overall = f.overall(a.position);
        let potential = a.potential.max(overall);
        let today = e.world_scope(|w| w.resource::<SimClock>().date());
        let age = e.get::<BirthDate>().map_or(25, |b| age_years(b.0, today));
        e.insert((
            f,
            PositionGroup(a.position),
            Rating { overall, potential },
            MarketValue(value_from(overall, age)),
        ));
        if !a.nationality.is_empty() {
            e.insert(Nationality(a.nationality.clone()));
        }
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

/// The position group for the `slot`-th player in a 16-player squad: 2 GK, 5 DEF, 6 MID, 3 FWD.
fn slot_position(slot: u32) -> u8 {
    match slot {
        0..=1 => POS_GK,
        2..=6 => POS_DEF,
        7..=12 => POS_MID,
        _ => POS_FWD,
    }
}

/// Attributes for a player of a given position around a `base` rating, tilted so the position's
/// key attributes are strongest. Deterministic (varies only by `base` and `slot`).
fn ability_for(position: u8, base: u8, slot: u32, nat: &str) -> FootballAbility {
    let b = base;
    let up = |v: u8, d: u8| v.saturating_add(d).min(99);
    let dn = |v: u8, d: u8| v.saturating_sub(d);
    let wiggle = ((slot * 7) % 9) as u8; // small deterministic per-slot variation
    let mut a = FootballAbility {
        pac: b,
        sho: b,
        pas: b,
        dri: b,
        tec: b,
        def: b,
        phy: b,
        vis: b,
        gk: dn(b, 20),
        position,
        potential: up(b, 3 + wiggle),
        nationality: nat.to_string(),
    };
    match position {
        POS_GK => {
            a.gk = up(b, 6);
            a.sho = dn(b, 30);
            a.dri = dn(b, 20);
            a.pac = dn(b, 15);
            a.def = up(b, 2);
        }
        POS_DEF => {
            a.def = up(b, 8);
            a.phy = up(b, 6);
            a.sho = dn(b, 18);
            a.vis = dn(b, 6);
        }
        POS_MID => {
            a.pas = up(b, 7);
            a.vis = up(b, 6);
            a.tec = up(b, 4);
            a.def = dn(b, 4);
        }
        _ => {
            a.sho = up(b, 8);
            a.pac = up(b, 6);
            a.dri = up(b, 5);
            a.def = dn(b, 20);
        }
    }
    a
}

/// A small, valid sample database: two divisions of six clubs, squads of named players with
/// positions/attributes/nationalities, plus a handful of free agents.
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
    const NATS: [&str; 12] = [
        "ENG", "ESP", "ITA", "GER", "POR", "FRA", "CRO", "POL", "AUT", "MEX", "BRA", "SUI",
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
                let position = slot_position(p);
                let nat = NATS[(p as usize + id as usize) % NATS.len()];
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
                    ability: ability_for(position, base, p, nat),
                });
            }
            club_ids.push(id);
            id += 1;
        }
        divisions.push(DivisionRecord { name: format!("Division {}", div + 1), tier: div + 1, club_ids });
    }

    for k in 0..8u32 {
        let base = (40 + (k * 4) % 25) as u8;
        let position = slot_position(k % 16);
        players.push(PersonRecord {
            name: format!("{} {}", FIRST[(k as usize + 3) % 10], LAST[(k as usize + 5) % 12]),
            club_id: None,
            birth: DbDate { year: 2025 - (18 + (k % 15) as i32), month: 3, day: 12 },
            wage: 800 + u32::from(base) * 40,
            contract_until: start,
            ability: ability_for(position, base, k, NATS[k as usize % NATS.len()]),
        });
    }

    Database { name: "Sample League".into(), start_date: start, seed: 0xDA7A, divisions, clubs, players }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gather_lineups;
    use bevy_ecs::prelude::*;
    use sim_core::{Club, FreeAgent, Rating};

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
    fn load_world_builds_a_playable_world_with_ratings() {
        let db = sample();
        let mut world = load_world(&db);

        let clubs = world.query_filtered::<(), With<Club>>().iter(&world).count();
        assert_eq!(clubs, db.clubs.len());

        let lineups = gather_lineups(&mut world);
        assert_eq!(lineups.len(), db.clubs.len());

        // Every player got a rating authored from their attributes.
        let rated = world.query::<&Rating>().iter(&world).count();
        assert_eq!(rated, db.players.len());
        assert!(world.query::<&Rating>().iter(&world).all(|r| r.overall > 0 && r.potential >= r.overall));

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
