//! Sport-agnostic starting-database core.
//!
//! Every sport-game seeds a new save from a *database* (editable, human-authored pre-game
//! data) — distinct from a save (a binary in-play snapshot). The *structure* is shared across
//! sports: a named dataset, clubs, divisions, and people with contracts. The only per-person
//! piece that differs by sport is the **ability** record, so the database is generic over it:
//! `Database<A>`.
//!
//! - Team sports (football, basketball) fill in clubs and divisions and contract their people.
//! - Individual sports can leave clubs/divisions empty and put everything on the people.
//!
//! `#[serde(flatten)]` inlines the ability fields into each person, so a sport's JSON stays
//! flat and natural (e.g. `attacking`/`defending` sit right on the player, not nested).

use crate::club::{sync_squad_membership, ClubBundle};
use crate::entity::{BirthDate, Condition, Contract, FreeAgent, Morale, Name, WageDemand};
use crate::rng::SimSeed;
use crate::time::{Date, SimClock};
use bevy_ecs::prelude::*;
use bevy_ecs::world::EntityWorldMut;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A calendar date in the database — a serializable mirror of [`Date`].
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl DbDate {
    pub fn to_date(self) -> Date {
        Date::new(self.year, self.month, self.day)
    }
}

impl From<Date> for DbDate {
    fn from(d: Date) -> Self {
        Self { year: d.year(), month: d.month(), day: d.day() }
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

/// A division (one rung of a league pyramid) and its member clubs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DivisionRecord {
    pub name: String,
    pub tier: u32,
    pub club_ids: Vec<u32>,
}

/// A person in the database. The sport-neutral fields are explicit; the sport's per-person
/// data (`A` — abilities and any sport-specific extras) is flattened inline. `club_id` `None`
/// means a free agent / unattached individual.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PersonRecord<A> {
    pub name: String,
    pub club_id: Option<u32>,
    pub birth: DbDate,
    pub wage: u32,
    pub contract_until: DbDate,
    #[serde(flatten)]
    pub ability: A,
}

/// A complete starting database, generic over a sport's ability type `A`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Database<A> {
    pub name: String,
    pub start_date: DbDate,
    pub seed: u64,
    pub divisions: Vec<DivisionRecord>,
    pub clubs: Vec<ClubRecord>,
    pub players: Vec<PersonRecord<A>>,
}

impl<A> Database<A> {
    /// Referential-integrity check: unique club ids, and every player/division reference
    /// points at a club that exists. Independent of the ability type.
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
}

impl<A: Serialize> Database<A> {
    /// Serialize to pretty JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a Database always serializes")
    }
}

impl<A: for<'de> Deserialize<'de>> Database<A> {
    /// Parse from JSON.
    pub fn from_json(s: &str) -> serde_json::Result<Database<A>> {
        serde_json::from_str(s)
    }
}

/// Build a ready-to-play ECS world from a database: spawn named clubs, then people (their
/// sport-neutral components, their sport ability via `spawn_ability`, and a contract or free-
/// agent status), then derive squad membership from contracts.
///
/// `spawn_ability` is the only sport-specific hook — it inserts the sport's ability component
/// from the person's `ability` record, so `sim-core` never names `Footballer`/`Baller`/etc.
pub fn load_world<A>(
    db: &Database<A>,
    mut spawn_ability: impl FnMut(&mut EntityWorldMut, &A),
) -> World {
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
            Name(p.name.clone()),
            BirthDate(p.birth.to_date()),
            Morale(70),
            Condition::fit(),
        ));
        spawn_ability(&mut e, &p.ability);
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
pub fn save<A: Serialize>(db: &Database<A>, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::write(path, db.to_json())
}

/// Read a database from a JSON file.
pub fn load<A: for<'de> Deserialize<'de>>(path: impl AsRef<std::path::Path>) -> std::io::Result<Database<A>> {
    let text = std::fs::read_to_string(path)?;
    Database::from_json(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::club::Club;
    use crate::entity::TeamId;

    // A stand-in sport ability for testing the generic. Must be a struct with named fields —
    // serde(flatten) inlines a map, not a scalar.
    #[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    struct Skill {
        rating: u8,
    }

    fn sample() -> Database<Skill> {
        Database {
            name: "Test".into(),
            start_date: DbDate { year: 2025, month: 7, day: 1 },
            seed: 1,
            divisions: vec![DivisionRecord { name: "D1".into(), tier: 1, club_ids: vec![0, 1] }],
            clubs: vec![
                ClubRecord { id: 0, name: "A".into(), balance: 1000, weekly_income: 100, squad_target: 2 },
                ClubRecord { id: 1, name: "B".into(), balance: 1000, weekly_income: 100, squad_target: 2 },
            ],
            players: vec![
                PersonRecord {
                    name: "P0".into(),
                    club_id: Some(0),
                    birth: DbDate { year: 2000, month: 1, day: 1 },
                    wage: 50,
                    contract_until: DbDate { year: 2030, month: 6, day: 30 },
                    ability: Skill { rating: 70 },
                },
                PersonRecord {
                    name: "Free".into(),
                    club_id: None,
                    birth: DbDate { year: 2002, month: 1, day: 1 },
                    wage: 40,
                    contract_until: DbDate { year: 2025, month: 7, day: 1 },
                    ability: Skill { rating: 55 },
                },
            ],
        }
    }

    #[test]
    fn json_round_trips_and_validates() {
        let db = sample();
        db.validate().unwrap();
        assert_eq!(Database::<Skill>::from_json(&db.to_json()).unwrap(), db);
    }

    #[test]
    fn flatten_keeps_ability_inline() {
        // The ability field sits directly on the player JSON, not nested under "ability".
        let json = sample().to_json();
        assert!(json.contains("\"rating\": 70"), "ability should be flattened inline:\n{json}");
        assert!(!json.contains("\"ability\""), "no nested ability object");
    }

    #[test]
    fn load_world_spawns_clubs_people_and_abilities() {
        let db = sample();
        let mut world = load_world(&db, |e, a: &Skill| {
            e.insert(*a);
        });

        assert_eq!(world.query_filtered::<(), With<Club>>().iter(&world).count(), 2);
        assert_eq!(world.query::<&Skill>().iter(&world).count(), 2);
        // The contracted player got their club's TeamId via sync (excluding the club entity,
        // which carries its own TeamId).
        let on_team = world
            .query_filtered::<&TeamId, Without<Club>>()
            .iter(&world)
            .filter(|t| t.0 == 0)
            .count();
        assert_eq!(on_team, 1);
    }
}
