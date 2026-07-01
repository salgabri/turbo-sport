//! The UI-agnostic read layer: small, summarized, serializable snapshots of the world for a
//! UI to display.
//!
//! This is the boundary the performance thesis cares about: the UI must **never** receive
//! 100k rows. Every function here returns a bounded, summarized view — a handful of clubs, one
//! club's squad, a capped free-agent list. The DTOs derive `serde` so they cross any boundary
//! (a Tauri IPC call, a Bevy resource, a test) unchanged.
//!
//! Sport-agnostic by design: these expose the generic facts every sport shares (finances,
//! ages, contracts, condition). A sport-specific view — a football league *table* with its own
//! points rules — is built in the sport crate from its own `Season`, not here.

use crate::club::Club;
use crate::economy::{Balance, MarketValue, WeeklyIncome};
use crate::entity::{
    age_years, BirthDate, Condition, Contract, FreeAgent, Morale, Name, Nationality, PositionGroup,
    Rating, Retired, TeamId,
};
use crate::time::{Date, SimClock};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A one-line summary of a club for a finances/overview table.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClubView {
    pub team_id: u32,
    pub name: Option<String>,
    pub balance: i64,
    pub weekly_income: i64,
    /// Sum of the wages of the club's contracted players.
    pub wage_bill: i64,
    /// Number of contracted players.
    pub squad_size: u32,
    /// Mean `overall` of contracted players with a [`Rating`], if any.
    pub avg_overall: Option<u8>,
    /// Best `overall` among contracted players with a [`Rating`], if any.
    pub best_overall: Option<u8>,
    /// Sum of contracted players' [`MarketValue`], if any carry one.
    pub squad_value: Option<i64>,
}

/// A one-row summary of a player for a squad / free-agent table.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub name: Option<String>,
    pub team_id: Option<u32>,
    pub age: Option<u32>,
    pub wage: Option<u32>,
    /// Contract end date as `YYYY-MM-DD`, if contracted.
    pub contract_until: Option<String>,
    pub fitness: Option<u8>,
    pub injured: bool,
    pub morale: Option<u8>,
    pub free_agent: bool,
    pub retired: bool,
    /// Nationality code/name, if authored.
    pub nationality: Option<String>,
    /// Opaque sport position-group index (the sport/UI maps it to a label).
    pub position_group: Option<u8>,
    /// Current overall rating (0..=99), if the sport has computed one.
    pub overall: Option<u8>,
    /// Peak potential rating (0..=99).
    pub potential: Option<u8>,
    /// Estimated market value.
    pub market_value: Option<i64>,
}

fn player_view(world: &World, entity: Entity, today: Date) -> PlayerView {
    let e = world.entity(entity);
    let contract = e.get::<Contract>();
    let rating = e.get::<Rating>();
    PlayerView {
        name: e.get::<Name>().map(|n| n.0.clone()),
        team_id: e.get::<TeamId>().map(|t| t.0),
        age: e.get::<BirthDate>().map(|b| age_years(b.0, today)),
        wage: contract.map(|c| c.wage),
        contract_until: contract.map(|c| c.until.to_string()),
        fitness: e.get::<Condition>().map(|c| c.fitness),
        injured: e.get::<Condition>().is_some_and(Condition::is_injured),
        morale: e.get::<Morale>().map(|m| m.0),
        free_agent: e.contains::<FreeAgent>(),
        retired: e.contains::<Retired>(),
        nationality: e.get::<Nationality>().map(|n| n.0.clone()),
        position_group: e.get::<PositionGroup>().map(|p| p.0),
        overall: rating.map(|r| r.overall),
        potential: rating.map(|r| r.potential),
        market_value: e.get::<MarketValue>().map(|v| v.0),
    }
}

/// Summaries of every club, ordered by team id. Bounded by the number of clubs (tens), so it
/// is safe to hand to a UI whole.
pub fn club_views(world: &mut World) -> Vec<ClubView> {
    // Per-club aggregate over the club's contracted players: squad size, wage bill, rating
    // and value roll-ups. Rating/value are read from the same entity as the contract.
    #[derive(Default)]
    struct Agg {
        squad_size: u32,
        wage_bill: i64,
        ovr_sum: u32,
        ovr_count: u32,
        ovr_best: u8,
        value_sum: i64,
        has_value: bool,
    }
    let mut agg: HashMap<Entity, Agg> = HashMap::new();
    {
        let mut q = world.query::<(&Contract, Option<&Rating>, Option<&MarketValue>)>();
        for (c, rating, value) in q.iter(world) {
            let a = agg.entry(c.club).or_default();
            a.squad_size += 1;
            a.wage_bill += i64::from(c.wage);
            if let Some(r) = rating {
                a.ovr_sum += u32::from(r.overall);
                a.ovr_count += 1;
                a.ovr_best = a.ovr_best.max(r.overall);
            }
            if let Some(v) = value {
                a.value_sum += v.0;
                a.has_value = true;
            }
        }
    }

    let mut views = Vec::new();
    let mut q = world
        .query_filtered::<(Entity, &TeamId, Option<&Name>, &Balance, &WeeklyIncome), With<Club>>();
    for (entity, team, name, balance, income) in q.iter(world) {
        let a = agg.remove(&entity).unwrap_or_default();
        views.push(ClubView {
            team_id: team.0,
            name: name.map(|n| n.0.clone()),
            balance: balance.0,
            weekly_income: income.0,
            wage_bill: a.wage_bill,
            squad_size: a.squad_size,
            avg_overall: (a.ovr_count > 0).then(|| (a.ovr_sum / a.ovr_count) as u8),
            best_overall: (a.ovr_count > 0).then_some(a.ovr_best),
            squad_value: a.has_value.then_some(a.value_sum),
        });
    }
    views.sort_by_key(|v| v.team_id);
    views
}

/// One club's squad (players currently tagged with `team_id`, excluding club entities).
/// Bounded by squad size (tens).
pub fn squad(world: &mut World, team_id: u32) -> Vec<PlayerView> {
    let today = world.resource::<SimClock>().date();
    let ids: Vec<Entity> = {
        let mut q = world.query_filtered::<(Entity, &TeamId), Without<Club>>();
        q.iter(world).filter(|(_, t)| t.0 == team_id).map(|(e, _)| e).collect()
    };
    ids.into_iter().map(|e| player_view(world, e, today)).collect()
}

/// Up to `limit` free agents (excluding retired). The cap is the whole point — a real save can
/// have thousands of free agents and a UI shows one page at a time.
pub fn free_agents(world: &mut World, limit: usize) -> Vec<PlayerView> {
    let today = world.resource::<SimClock>().date();
    let ids: Vec<Entity> = {
        let mut q =
            world.query_filtered::<Entity, (With<FreeAgent>, Without<Club>, Without<Retired>)>();
        q.iter(world).take(limit).collect()
    };
    ids.into_iter().map(|e| player_view(world, e, today)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::club::ClubBundle;
    use crate::entity::WageDemand;

    fn world_on(date: Date) -> World {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(date));
        world
    }

    #[test]
    fn club_view_summarizes_finances_and_squad() {
        let mut world = world_on(Date::new(2026, 1, 1));
        let club = world.spawn(ClubBundle::new(0, 5_000, 200, 20)).id();
        world.spawn(Contract { club, until: Date::new(2030, 1, 1), wage: 100 });
        world.spawn(Contract { club, until: Date::new(2030, 1, 1), wage: 150 });

        let views = club_views(&mut world);
        assert_eq!(views.len(), 1);
        assert_eq!(
            views[0],
            ClubView {
                team_id: 0,
                name: None,
                balance: 5_000,
                weekly_income: 200,
                wage_bill: 250,
                squad_size: 2,
                avg_overall: None,
                best_overall: None,
                squad_value: None,
            }
        );
    }

    #[test]
    fn squad_lists_players_with_computed_age() {
        let mut world = world_on(Date::new(2026, 6, 1));
        world.spawn((TeamId(3), BirthDate(Date::new(2000, 6, 1)), Morale(70), Condition::fit()));
        world.spawn((TeamId(3), BirthDate(Date::new(2006, 1, 1)), Morale(70), Condition { fitness: 80, injury_days: 4 }));
        world.spawn((TeamId(9), BirthDate(Date::new(1999, 1, 1)), Morale(70), Condition::fit())); // other team

        let mut s = squad(&mut world, 3);
        s.sort_by_key(|p| p.age);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].age, Some(20)); // born 2006
        assert_eq!(s[1].age, Some(26)); // born 2000-06-01, on birthday
        assert!(s[0].injured); // injury_days 4
        assert!(!s[1].injured);
    }

    #[test]
    fn free_agents_are_capped_at_the_limit() {
        let mut world = world_on(Date::new(2026, 1, 1));
        for _ in 0..5 {
            world.spawn((FreeAgent, WageDemand(100)));
        }
        assert_eq!(free_agents(&mut world, 3).len(), 3);
        assert_eq!(free_agents(&mut world, 10).len(), 5);
        assert!(free_agents(&mut world, 10).iter().all(|p| p.free_agent));
    }
}
