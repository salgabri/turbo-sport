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
use crate::economy::{Balance, WeeklyIncome};
use crate::entity::{age_years, BirthDate, Condition, Contract, FreeAgent, Morale, Retired, TeamId};
use crate::time::{Date, SimClock};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A one-line summary of a club for a finances/overview table.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClubView {
    pub team_id: u32,
    pub balance: i64,
    pub weekly_income: i64,
    /// Sum of the wages of the club's contracted players.
    pub wage_bill: i64,
    /// Number of contracted players.
    pub squad_size: u32,
}

/// A one-row summary of a player for a squad / free-agent table.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
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
}

fn player_view(world: &World, entity: Entity, today: Date) -> PlayerView {
    let e = world.entity(entity);
    let contract = e.get::<Contract>();
    PlayerView {
        team_id: e.get::<TeamId>().map(|t| t.0),
        age: e.get::<BirthDate>().map(|b| age_years(b.0, today)),
        wage: contract.map(|c| c.wage),
        contract_until: contract.map(|c| c.until.to_string()),
        fitness: e.get::<Condition>().map(|c| c.fitness),
        injured: e.get::<Condition>().is_some_and(Condition::is_injured),
        morale: e.get::<Morale>().map(|m| m.0),
        free_agent: e.contains::<FreeAgent>(),
        retired: e.contains::<Retired>(),
    }
}

/// Summaries of every club, ordered by team id. Bounded by the number of clubs (tens), so it
/// is safe to hand to a UI whole.
pub fn club_views(world: &mut World) -> Vec<ClubView> {
    // Aggregate contracts per club entity (squad size + wage bill).
    let mut agg: HashMap<Entity, (u32, i64)> = HashMap::new();
    {
        let mut q = world.query::<&Contract>();
        for c in q.iter(world) {
            let e = agg.entry(c.club).or_insert((0, 0));
            e.0 += 1;
            e.1 += i64::from(c.wage);
        }
    }

    let mut views = Vec::new();
    let mut q = world.query_filtered::<(Entity, &TeamId, &Balance, &WeeklyIncome), With<Club>>();
    for (entity, team, balance, income) in q.iter(world) {
        let (squad_size, wage_bill) = agg.get(&entity).copied().unwrap_or((0, 0));
        views.push(ClubView {
            team_id: team.0,
            balance: balance.0,
            weekly_income: income.0,
            wage_bill,
            squad_size,
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
            ClubView { team_id: 0, balance: 5_000, weekly_income: 200, wage_bill: 250, squad_size: 2 }
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
