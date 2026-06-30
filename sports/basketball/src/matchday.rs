//! Gather rosters from the ECS world and simulate a matchday in parallel — same pipeline
//! shape as football, using the shared `sim_core::seeded_parallel_map`.

use crate::attributes::Baller;
use crate::engine::{simulate_game, GameResult, Roster};
use bevy_ecs::prelude::*;
use rand_pcg::Pcg64Mcg;
use sim_core::{seeded_parallel_map, TeamId};
use std::collections::BTreeMap;

/// A fixture resolved to its two rosters.
#[derive(Clone, Copy, Debug)]
pub struct Fixture {
    pub home: Roster,
    pub away: Roster,
}

/// Average each team's players into a [`Roster`].
pub fn gather_rosters(world: &mut World) -> BTreeMap<u32, Roster> {
    let mut acc: BTreeMap<u32, (f64, f64, f64, f64, u32)> = BTreeMap::new();
    let mut q = world.query::<(&TeamId, &Baller)>();
    for (team, b) in q.iter(world) {
        let e = acc.entry(team.0).or_insert((0.0, 0.0, 0.0, 0.0, 0));
        e.0 += f64::from(b.offense);
        e.1 += f64::from(b.defense);
        e.2 += f64::from(b.three_point);
        e.3 += f64::from(b.rebounding);
        e.4 += 1;
    }
    acc.into_iter()
        .map(|(id, (off, def, three, reb, n))| {
            let n = f64::from(n.max(1));
            (id, Roster::new(off / n, def / n, three / n, reb / n))
        })
        .collect()
}

/// Simulate a matchday in parallel, deterministically seeded by `[season, matchday]`.
pub fn simulate_matchday(
    fixtures: &[Fixture],
    world_seed: u64,
    season: u32,
    matchday: u32,
) -> Vec<GameResult> {
    seeded_parallel_map::<Pcg64Mcg, _, _, _>(
        fixtures,
        world_seed,
        &[u64::from(season), u64::from(matchday)],
        |fx, rng| simulate_game(&fx.home, &fx.away, rng),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_averages_player_abilities() {
        let mut world = World::new();
        world.spawn((TeamId(1), Baller { offense: 60, defense: 50, three_point: 40, rebounding: 50 }));
        world.spawn((TeamId(1), Baller { offense: 80, defense: 50, three_point: 60, rebounding: 50 }));
        world.spawn((TeamId(2), Baller { offense: 30, defense: 30, three_point: 30, rebounding: 30 }));

        let rosters = gather_rosters(&mut world);
        assert_eq!(rosters.len(), 2);
        assert!((rosters[&1].offense - 70.0).abs() < 1e-9);
        assert!((rosters[&2].offense - 30.0).abs() < 1e-9);
    }

    #[test]
    fn matchday_is_deterministic() {
        let fixtures: Vec<Fixture> = (0..16)
            .map(|i| Fixture {
                home: Roster::new(50.0 + f64::from(i), 55.0, 50.0, 55.0),
                away: Roster::new(60.0, 55.0, 50.0, 55.0),
            })
            .collect();
        assert_eq!(
            simulate_matchday(&fixtures, 0xB00, 2025, 4),
            simulate_matchday(&fixtures, 0xB00, 2025, 4)
        );
    }
}
