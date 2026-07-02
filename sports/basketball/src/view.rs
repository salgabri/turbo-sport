//! Basketball's detailed read layer for the UI — the sport-neutral player row plus the six
//! attributes and the position label, built by reading both the sim-core components and the
//! `Baller` ability so the Tauri layer hands the UI one shape. Mirrors `football::view`.

use crate::attributes::{position_label, Baller};
use bevy_ecs::prelude::*;
use serde::Serialize;
use sim_core::{
    age_years, BirthDate, Club, Condition, Contract, MarketValue, Morale, Name, Nationality,
    PositionGroup, Rating, SimClock, TeamId,
};

/// A basketball player's six attributes as a flat, map-friendly struct.
#[derive(Serialize, Clone, Copy, Debug)]
pub struct Attrs {
    pub ins: u8,
    pub out: u8,
    pub pm: u8,
    pub reb: u8,
    pub def: u8,
    pub ath: u8,
}

impl From<&Baller> for Attrs {
    fn from(b: &Baller) -> Self {
        Attrs { ins: b.ins, out: b.out, pm: b.pm, reb: b.reb, def: b.def, ath: b.ath }
    }
}

/// One player's full row for the basketball squad/profile screens. Field names match the
/// frontend DTO; absent data serializes to `null`.
#[derive(Serialize, Clone, Debug)]
pub struct SquadPlayer {
    pub name: Option<String>,
    pub age: Option<u32>,
    pub wage: Option<u32>,
    pub contract_until: Option<String>,
    pub fitness: Option<u8>,
    pub injured: bool,
    pub morale: Option<u8>,
    pub free_agent: bool,
    pub retired: bool,
    pub nationality: Option<String>,
    pub position_group: Option<String>,
    pub overall: Option<u8>,
    pub potential: Option<u8>,
    pub market_value: Option<i64>,
    pub attrs: Option<Attrs>,
    /// Season games / points so far (present once the player has featured).
    pub games: Option<u16>,
    pub points: Option<u16>,
    /// Days until recovered from an injury, if currently injured.
    pub injury_days: Option<u16>,
}

fn row(world: &World, entity: Entity, today: sim_core::Date) -> SquadPlayer {
    let e = world.entity(entity);
    let contract = e.get::<Contract>();
    let rating = e.get::<Rating>();
    SquadPlayer {
        name: e.get::<Name>().map(|n| n.0.clone()),
        age: e.get::<BirthDate>().map(|b| age_years(b.0, today)),
        wage: contract.map(|c| c.wage),
        contract_until: contract.map(|c| c.until.to_string()),
        fitness: e.get::<Condition>().map(|c| c.fitness),
        injured: e.get::<Condition>().is_some_and(Condition::is_injured),
        morale: e.get::<Morale>().map(|m| m.0),
        free_agent: e.contains::<sim_core::FreeAgent>(),
        retired: e.contains::<sim_core::Retired>(),
        nationality: e.get::<Nationality>().map(|n| n.0.clone()),
        position_group: e.get::<PositionGroup>().map(|p| position_label(p.0).to_string()),
        overall: rating.map(|r| r.overall),
        potential: rating.map(|r| r.potential),
        market_value: e.get::<MarketValue>().map(|v| v.0),
        attrs: e.get::<Baller>().map(Attrs::from),
        games: e.get::<crate::tally::BasketballTally>().map(|t| t.games),
        points: e.get::<crate::tally::BasketballTally>().map(|t| t.points),
        injury_days: e.get::<Condition>().map(|c| c.injury_days).filter(|&d| d > 0),
    }
}

/// One club's roster as detailed basketball rows.
pub fn team_squad(world: &mut World, team_id: u32) -> Vec<SquadPlayer> {
    let today = world.resource::<SimClock>().date();
    let ids: Vec<Entity> = {
        let mut q = world.query_filtered::<(Entity, &TeamId), Without<Club>>();
        q.iter(world).filter(|(_, t)| t.0 == team_id).map(|(e, _)| e).collect()
    };
    ids.into_iter().map(|e| row(world, e, today)).collect()
}

/// A row of the top-scorers chart (points leaders).
#[derive(Serialize, Clone, Debug)]
pub struct ScorerRow {
    pub name: Option<String>,
    pub team_id: Option<u32>,
    pub points: u16,
    pub games: u16,
}

/// The league's leading scorers this season, most points first (ties by fewer games), capped at
/// `limit`. Reads the accumulated [`crate::tally`].
pub fn top_scorers(world: &mut World, limit: usize) -> Vec<ScorerRow> {
    let mut v: Vec<ScorerRow> = {
        let mut q = world.query::<(&Name, Option<&TeamId>, &crate::tally::BasketballTally)>();
        q.iter(world)
            .filter(|(_, _, t)| t.points > 0)
            .map(|(n, team, t)| ScorerRow {
                name: Some(n.0.clone()),
                team_id: team.map(|x| x.0),
                points: t.points,
                games: t.games,
            })
            .collect()
    };
    v.sort_by(|a, b| b.points.cmp(&a.points).then(a.games.cmp(&b.games)).then(a.name.cmp(&b.name)));
    v.truncate(limit);
    v
}

/// Up to `limit` free agents as detailed basketball rows.
pub fn free_agents(world: &mut World, limit: usize) -> Vec<SquadPlayer> {
    use sim_core::{FreeAgent, Retired};
    let today = world.resource::<SimClock>().date();
    let ids: Vec<Entity> = {
        let mut q =
            world.query_filtered::<Entity, (With<FreeAgent>, Without<Club>, Without<Retired>)>();
        q.iter(world).take(limit).collect()
    };
    ids.into_iter().map(|e| row(world, e, today)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn detailed_rows_carry_attributes_and_position() {
        let db = sample();
        let mut world = load_world(&db);
        let rows = team_squad(&mut world, db.clubs[0].id);
        assert!(!rows.is_empty());
        for p in &rows {
            assert!(p.attrs.is_some());
            assert!(p.overall.is_some());
            assert!(p.position_group.is_some());
        }
        assert!(rows.iter().any(|p| p.position_group.as_deref() == Some("C")));
    }
}
