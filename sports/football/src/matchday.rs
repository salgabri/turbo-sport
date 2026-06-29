//! Turning a set of fixtures into results — the gather → parallel-simulate → (apply)
//! pipeline.
//!
//! `gather_lineups` reads the ECS world single-threaded into plain data; `simulate_matchday`
//! runs the pure engine across cores with `rayon`. Writing results back into the world
//! (standings, condition, injuries) is single-threaded and belongs to later steps. The
//! split is what lets the parallel section stay lock-free and deterministic.

use crate::attributes::{Footballer, TeamId};
use crate::engine::{simulate_match, Lineup, MatchResult};
use bevy_ecs::prelude::*;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use sim_core::derive_seed;
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
    let mut q = world.query::<(&TeamId, &Footballer)>();
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

/// Seed for one fixture: the world seed folded with stable coordinates. Identical across
/// runs and independent of `rayon`'s scheduling.
fn fixture_seed(world_seed: u64, season: u32, matchday: u32, index: usize) -> u64 {
    derive_seed(world_seed, &[u64::from(season), u64::from(matchday), index as u64])
}

/// Simulate a whole matchday in parallel. Because every fixture owns an independently
/// seeded RNG and the engine is pure, the result vector is byte-for-byte identical on
/// every run, on any number of cores.
pub fn simulate_matchday(
    fixtures: &[Fixture],
    world_seed: u64,
    season: u32,
    matchday: u32,
) -> Vec<MatchResult> {
    fixtures
        .par_iter()
        .enumerate()
        .map(|(i, fx)| {
            let mut rng = Pcg64Mcg::seed_from_u64(fixture_seed(world_seed, season, matchday, i));
            simulate_match(&fx.home, &fx.away, &mut rng)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lineup(base: f64) -> Lineup {
        Lineup::new(base, base, base, base)
    }

    /// Single-threaded reference using the same per-fixture seeds.
    fn simulate_serial(
        fixtures: &[Fixture],
        world_seed: u64,
        season: u32,
        matchday: u32,
    ) -> Vec<MatchResult> {
        fixtures
            .iter()
            .enumerate()
            .map(|(i, fx)| {
                let mut rng = Pcg64Mcg::seed_from_u64(fixture_seed(world_seed, season, matchday, i));
                simulate_match(&fx.home, &fx.away, &mut rng)
            })
            .collect()
    }

    #[test]
    fn parallel_matches_serial_and_is_repeatable() {
        let fixtures: Vec<Fixture> = (0..32)
            .map(|i| Fixture { home: lineup(40.0 + i as f64), away: lineup(70.0 - i as f64) })
            .collect();

        let par = simulate_matchday(&fixtures, 0xABCD, 2025, 7);
        let serial = simulate_serial(&fixtures, 0xABCD, 2025, 7);
        let par_again = simulate_matchday(&fixtures, 0xABCD, 2025, 7);

        assert_eq!(par, serial, "parallel result must equal the serial reference");
        assert_eq!(par, par_again, "matchday must be reproducible run-to-run");
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
