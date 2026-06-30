//! Turning a set of fixtures into results — the gather → parallel-simulate → (apply)
//! pipeline.
//!
//! `gather_lineups` reads the ECS world single-threaded into plain data; `simulate_matchday`
//! runs the pure engine across cores via `sim_core::seeded_parallel_map`, which owns the
//! determinism contract (per-fixture seeding, no shared RNG). Writing results back into the
//! world (standings, condition, injuries) is single-threaded and belongs to later steps.

use crate::attributes::Footballer;
use crate::engine::{simulate_match, Lineup, MatchResult};
use bevy_ecs::prelude::*;
use rand_pcg::Pcg64Mcg;
use sim_core::{seeded_parallel_map, Retired, TeamId};
use std::collections::BTreeMap;

/// A fixture, fully resolved to its two lineups so simulating it needs no ECS access.
#[derive(Clone, Copy, Debug)]
pub struct Fixture {
    pub home: Lineup,
    pub away: Lineup,
}

/// Aggregate every team's players into a per-team [`Lineup`] by averaging their
/// abilities. Ordered by team id (`BTreeMap`) for reproducible iteration.
pub fn gather_lineups(world: &mut World) -> BTreeMap<u32, Lineup> {
    let mut acc: BTreeMap<u32, (f64, f64, f64, f64, u32)> = BTreeMap::new();
    // Retired players keep their TeamId but are no longer available to play.
    let mut q = world.query_filtered::<(&TeamId, &Footballer), Without<Retired>>();
    for (team, f) in q.iter(world) {
        let e = acc.entry(team.0).or_insert((0.0, 0.0, 0.0, 0.0, 0));
        e.0 += f64::from(f.attacking);
        e.1 += f64::from(f.defending);
        e.2 += f64::from(f.finishing);
        e.3 += f64::from(f.goalkeeping);
        e.4 += 1;
    }
    acc.into_iter()
        .map(|(id, (atk, def, fin, gk, n))| {
            let n = f64::from(n.max(1));
            (id, Lineup::new(atk / n, def / n, fin / n, gk / n))
        })
        .collect()
}

/// Simulate a whole matchday in parallel. The `[season, matchday]` coordinates name the
/// stream group; the shared helper seeds each fixture so the result is identical on every
/// run, on any number of cores.
pub fn simulate_matchday(
    fixtures: &[Fixture],
    world_seed: u64,
    season: u32,
    matchday: u32,
) -> Vec<MatchResult> {
    seeded_parallel_map::<Pcg64Mcg, _, _, _>(
        fixtures,
        world_seed,
        &[u64::from(season), u64::from(matchday)],
        |fx, rng| simulate_match(&fx.home, &fx.away, rng),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lineup(base: f64) -> Lineup {
        Lineup::new(base, base, base, base)
    }

    #[test]
    fn matchday_is_deterministic() {
        // Determinism across cores is guaranteed by the shared helper (tested in
        // sim-core); here we just confirm a matchday is reproducible end to end.
        let fixtures: Vec<Fixture> = (0..32)
            .map(|i| Fixture { home: lineup(40.0 + f64::from(i)), away: lineup(70.0 - f64::from(i)) })
            .collect();
        let a = simulate_matchday(&fixtures, 0xABCD, 2025, 7);
        let b = simulate_matchday(&fixtures, 0xABCD, 2025, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn gather_averages_player_abilities() {
        let mut world = World::new();
        // Team 1: two players, attacking 60 and 80 -> mean 70.
        world.spawn((TeamId(1), Footballer { attacking: 60, defending: 50, finishing: 50, goalkeeping: 50 }));
        world.spawn((TeamId(1), Footballer { attacking: 80, defending: 50, finishing: 50, goalkeeping: 50 }));
        world.spawn((TeamId(2), Footballer { attacking: 30, defending: 30, finishing: 30, goalkeeping: 30 }));

        let lineups = gather_lineups(&mut world);
        assert_eq!(lineups.len(), 2);
        assert!((lineups[&1].attack - 70.0).abs() < 1e-9);
        assert!((lineups[&2].attack - 30.0).abs() < 1e-9);
    }
}
