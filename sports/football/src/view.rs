//! Football's detailed read layer for the UI.
//!
//! `sim_core::view` gives the sport-neutral player row (name, age, contract, condition,
//! rating, position index, value, nationality). The football screens also want the eight
//! attributes behind the rating and the position *label* — sport-specific data `sim-core`
//! cannot name. This module builds the richer per-player row by reading both the sim-core
//! components and the `Footballer` ability, so the Tauri layer hands the UI one shape.

use crate::attributes::{position_label, Footballer};
use bevy_ecs::prelude::*;
use serde::Serialize;
use sim_core::{
    age_years, BirthDate, Club, Condition, Contract, MarketValue, Morale, Name, Nationality,
    PositionGroup, Rating, SimClock, TeamId,
};

/// A footballer's eight outfield attributes plus keeper, as a flat map-friendly struct.
#[derive(Serialize, Clone, Copy, Debug)]
pub struct Attrs {
    pub pac: u8,
    pub sho: u8,
    pub pas: u8,
    pub dri: u8,
    pub tec: u8,
    pub def: u8,
    pub phy: u8,
    pub vis: u8,
    pub gk: u8,
}

impl From<&Footballer> for Attrs {
    fn from(f: &Footballer) -> Self {
        Attrs {
            pac: f.pac,
            sho: f.sho,
            pas: f.pas,
            dri: f.dri,
            tec: f.tec,
            def: f.def,
            phy: f.phy,
            vis: f.vis,
            gk: f.gk,
        }
    }
}

/// One player's full row for the football squad/profile screens. Field names match the
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
    /// Position group label ("GK"/"DEF"/"MID"/"FWD").
    pub position_group: Option<String>,
    pub overall: Option<u8>,
    pub potential: Option<u8>,
    pub market_value: Option<i64>,
    pub attrs: Option<Attrs>,
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
        attrs: e.get::<Footballer>().map(Attrs::from),
    }
}

/// One club's squad as detailed football rows (players tagged with `team_id`, excluding the
/// club entity). Bounded by squad size.
pub fn team_squad(world: &mut World, team_id: u32) -> Vec<SquadPlayer> {
    let today = world.resource::<SimClock>().date();
    let ids: Vec<Entity> = {
        let mut q = world.query_filtered::<(Entity, &TeamId), Without<Club>>();
        q.iter(world).filter(|(_, t)| t.0 == team_id).map(|(e, _)| e).collect()
    };
    ids.into_iter().map(|e| row(world, e, today)).collect()
}

/// Up to `limit` free agents as detailed football rows (excluding retired). Same cap rationale
/// as `sim_core::free_agents`.
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
        let team = db.clubs[0].id;
        let rows = team_squad(&mut world, team);
        assert!(!rows.is_empty());
        // Every contracted player has attributes, a rating, a position label and a value.
        for p in &rows {
            assert!(p.attrs.is_some(), "attrs present");
            assert!(p.overall.is_some(), "overall present");
            assert!(p.position_group.is_some(), "position present");
            assert!(p.market_value.is_some(), "value present");
        }
        // At least one keeper in the squad.
        assert!(rows.iter().any(|p| p.position_group.as_deref() == Some("GK")));
    }
}
