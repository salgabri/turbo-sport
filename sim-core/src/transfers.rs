//! The transfer market — a sport-agnostic free-agent signing system with a simple AI.
//!
//! This is where the economy stops being inert: free agents (produced by contract expiry
//! in [`crate::lifecycle`]) get signed by clubs that *need* players and can *afford* them.
//! It is sport-neutral on purpose — it reasons about contracts, wages, and squad sizes, not
//! about football or basketball — so every sport gets a transfer market for free.
//!
//! The AI is deliberately simple and **deterministic** (no RNG, stable tie-breaks): it is a
//! greedy matching, not a market simulation. A richer model (bidding, valuations, player
//! preferences) is a later refinement; this establishes the loop and the data flow.

use crate::entity::{Contract, FreeAgent, Retired, SquadTarget, WageDemand};
use crate::economy::WeeklyIncome;
use crate::time::{Date, SimClock};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Length of a contract offered to a signed free agent, in (fixed-calendar) years.
const CONTRACT_YEARS: i32 = 2;

/// Working tally for a club during a window: current squad size, current weekly wage bill,
/// desired squad size, and the weekly income that bounds the affordable wage budget.
#[derive(Clone, Copy)]
struct ClubState {
    squad: u32,
    wage_bill: u32,
    target: u32,
    income: u32,
}

/// Run one transfer window: sign free agents to clubs that need them and can afford them.
///
/// Greedy and deterministic. Free agents are considered best-first (highest wage demand,
/// breaking ties by entity), and each is offered to the club with the greatest squad need
/// it can afford (breaking ties by lower wage bill, then entity). A club signs only while
/// below its [`SquadTarget`] and while its wage bill plus the new wage stays within its
/// [`WeeklyIncome`]. Retired players are never signed.
pub fn run_transfer_window(world: &mut World) {
    let today = world.resource::<SimClock>().date();

    // Current squad size and wage bill per club, from existing contracts.
    let mut squads: HashMap<Entity, (u32, u32)> = HashMap::new();
    {
        let mut q = world.query::<&Contract>();
        for c in q.iter(world) {
            let e = squads.entry(c.club).or_insert((0, 0));
            e.0 += 1;
            e.1 += c.wage;
        }
    }

    // Clubs that can sign (have a squad target + an income), in a stable order.
    let mut clubs: Vec<Entity> = Vec::new();
    let mut state: HashMap<Entity, ClubState> = HashMap::new();
    {
        let mut q = world.query::<(Entity, &SquadTarget, &WeeklyIncome)>();
        for (e, target, income) in q.iter(world) {
            let (squad, wage_bill) = squads.get(&e).copied().unwrap_or((0, 0));
            clubs.push(e);
            state.insert(
                e,
                ClubState { squad, wage_bill, target: target.0, income: income.0.max(0) as u32 },
            );
        }
    }
    clubs.sort_unstable();

    // Free agents (excluding retired), best-first by wage demand then entity.
    let mut free_agents: Vec<(Entity, u32)> = Vec::new();
    {
        let mut q = world.query_filtered::<(Entity, &WageDemand), (With<FreeAgent>, Without<Retired>)>();
        for (e, demand) in q.iter(world) {
            free_agents.push((e, demand.0));
        }
    }
    free_agents.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    // Greedy assignment.
    let until = Date::new(today.year() + CONTRACT_YEARS, today.month(), today.day());
    let mut signings: Vec<(Entity, Entity, u32)> = Vec::new();
    for (player, demand) in free_agents {
        let mut chosen: Option<(Entity, u32, u32)> = None; // (club, need, wage_bill)
        for &club in &clubs {
            let cs = state[&club];
            if cs.squad < cs.target && cs.wage_bill + demand <= cs.income {
                let need = cs.target - cs.squad;
                let better = match chosen {
                    None => true,
                    Some((_, best_need, best_bill)) => {
                        need > best_need || (need == best_need && cs.wage_bill < best_bill)
                    }
                };
                if better {
                    chosen = Some((club, need, cs.wage_bill));
                }
            }
        }
        if let Some((club, _, _)) = chosen {
            let cs = state.get_mut(&club).unwrap();
            cs.squad += 1;
            cs.wage_bill += demand;
            signings.push((player, club, demand));
        }
    }

    // Apply: turn each signed free agent into a contracted player.
    for (player, club, wage) in signings {
        world
            .entity_mut(player)
            .remove::<FreeAgent>()
            .remove::<WageDemand>()
            .insert(Contract { club, until, wage });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SimSeed;

    fn world_on(date: Date) -> World {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(date));
        world.insert_resource(SimSeed(1));
        world
    }

    fn is_signed(world: &World, player: Entity) -> bool {
        let e = world.entity(player);
        e.get::<Contract>().is_some() && e.get::<FreeAgent>().is_none()
    }

    #[test]
    fn signs_free_agents_to_a_needy_affordable_club() {
        let mut world = world_on(Date::new(2026, 6, 1));
        let _club = world.spawn((SquadTarget(2), WeeklyIncome(1_000))).id();
        let p1 = world.spawn((FreeAgent, WageDemand(300))).id();
        let p2 = world.spawn((FreeAgent, WageDemand(250))).id();

        run_transfer_window(&mut world);

        assert!(is_signed(&world, p1));
        assert!(is_signed(&world, p2));
        // Contract end date is CONTRACT_YEARS out.
        assert_eq!(world.entity(p1).get::<Contract>().unwrap().until, Date::new(2028, 6, 1));
    }

    #[test]
    fn respects_the_squad_target() {
        let mut world = world_on(Date::new(2026, 6, 1));
        world.spawn((SquadTarget(1), WeeklyIncome(10_000)));
        let p1 = world.spawn((FreeAgent, WageDemand(300))).id();
        let p2 = world.spawn((FreeAgent, WageDemand(300))).id();

        run_transfer_window(&mut world);

        // Exactly one signs (target is 1); the other stays free.
        assert_ne!(is_signed(&world, p1), is_signed(&world, p2));
    }

    #[test]
    fn a_club_that_cannot_afford_signs_nobody() {
        let mut world = world_on(Date::new(2026, 6, 1));
        world.spawn((SquadTarget(3), WeeklyIncome(100)));
        let p = world.spawn((FreeAgent, WageDemand(500))).id();

        run_transfer_window(&mut world);

        assert!(!is_signed(&world, p), "broke club must not sign");
    }

    #[test]
    fn retired_free_agents_are_not_signed() {
        let mut world = world_on(Date::new(2026, 6, 1));
        world.spawn((SquadTarget(3), WeeklyIncome(10_000)));
        let p = world.spawn((FreeAgent, WageDemand(100), Retired)).id();

        run_transfer_window(&mut world);

        assert!(!is_signed(&world, p), "retired players must not be signed");
    }

    #[test]
    fn wage_bill_accumulates_so_a_club_stops_when_budget_runs_out() {
        let mut world = world_on(Date::new(2026, 6, 1));
        // Target 5 but income only covers two 400-wage players.
        world.spawn((SquadTarget(5), WeeklyIncome(900)));
        let players: Vec<Entity> =
            (0..5).map(|_| world.spawn((FreeAgent, WageDemand(400))).id()).collect();

        run_transfer_window(&mut world);

        let signed = players.iter().filter(|&&p| is_signed(&world, p)).count();
        assert_eq!(signed, 2, "only two 400-wage players fit in a 900 budget");
    }
}
