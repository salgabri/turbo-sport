//! Single-elimination bracket — the genuinely new competition format.
//!
//! Nothing here is shared with the round-robin leagues: seeded placement, round-by-round
//! elimination, winners feeding the next round. It does reuse the one piece that *is*
//! universal — `sim_core::seeded_parallel_map` — because the matches within a single round
//! are independent and run in parallel, deterministically seeded by `[tournament, round]`.
//! That a knockout and a league share the parallel helper but no competition logic is the
//! whole point of harvesting infrastructure rather than a `CompetitionFormat` trait.

use crate::attributes::{Seed, TennisPlayer};
use crate::engine::{simulate_match, Player};
use bevy_ecs::prelude::*;
use rand_pcg::Pcg64Mcg;
use sim_core::seeded_parallel_map;

/// One match in the bracket: the two player indices (into the draw), the winner's index,
/// and the set score.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BracketMatch {
    pub a: usize,
    pub b: usize,
    pub winner: usize,
    pub sets: (u8, u8),
}

/// A completed tournament: every round's matches and the champion's draw index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tournament {
    pub rounds: Vec<Vec<BracketMatch>>,
    pub champion: usize,
}

/// Standard bracket seeding for a power-of-two field: returns seed indices in bracket
/// order, so the top seeds only meet in later rounds. For n=4 -> `[0, 3, 1, 2]`.
fn seeding_order(n: usize) -> Vec<usize> {
    let mut seeds = vec![0usize];
    let mut size = 1;
    while size < n {
        size *= 2;
        let mut next = Vec::with_capacity(size);
        for &s in &seeds {
            next.push(s);
            next.push(size - 1 - s);
        }
        seeds = next;
    }
    seeds
}

/// Read the draw out of the ECS world: tennis players ordered by their seed (top seed
/// first). The returned `Vec` index *is* the seed rank used for bracket placement.
pub fn gather_draw(world: &mut World) -> Vec<Player> {
    let mut seeded: Vec<(u32, Player)> = Vec::new();
    let mut q = world.query::<(&Seed, &TennisPlayer)>();
    for (seed, p) in q.iter(world) {
        seeded.push((
            seed.0,
            Player::new(
                f64::from(p.serve),
                f64::from(p.return_game),
                f64::from(p.baseline),
                f64::from(p.mental),
            ),
        ));
    }
    seeded.sort_by_key(|(s, _)| *s);
    seeded.into_iter().map(|(_, p)| p).collect()
}

/// Run a full single-elimination tournament. `players` is the field ordered by seed
/// (index 0 = top seed); the count must be a power of two. Deterministic for a given seed.
pub fn simulate_tournament(players: &[Player], world_seed: u64, tournament_id: u32) -> Tournament {
    let n = players.len();
    assert!(n >= 2 && n.is_power_of_two(), "draw size must be a power of two >= 2");

    let mut alive = seeding_order(n); // player indices in bracket order
    let mut rounds = Vec::new();
    let mut round_no = 0u32;

    while alive.len() > 1 {
        let pairs: Vec<(usize, usize)> = alive.chunks(2).map(|c| (c[0], c[1])).collect();
        // Matches in a round are independent -> simulate in parallel, deterministically.
        let outcomes = seeded_parallel_map::<Pcg64Mcg, _, _, _>(
            &pairs,
            world_seed,
            &[u64::from(tournament_id), u64::from(round_no)],
            |&(a, b), rng| simulate_match(&players[a], &players[b], rng),
        );

        let mut round = Vec::with_capacity(pairs.len());
        let mut winners = Vec::with_capacity(pairs.len());
        for (&(a, b), outcome) in pairs.iter().zip(outcomes) {
            let winner = if outcome.winner == 0 { a } else { b };
            round.push(BracketMatch { a, b, winner, sets: outcome.sets });
            winners.push(winner);
        }
        rounds.push(round);
        alive = winners;
        round_no += 1;
    }

    Tournament { champion: alive[0], rounds }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(rating: f64) -> Player {
        Player::new(rating, rating, rating, rating)
    }

    #[test]
    fn seeding_places_top_seeds_apart() {
        assert_eq!(seeding_order(2), vec![0, 1]);
        assert_eq!(seeding_order(4), vec![0, 3, 1, 2]);
        assert_eq!(seeding_order(8), vec![0, 7, 3, 4, 1, 6, 2, 5]);
    }

    #[test]
    fn tournament_reduces_to_one_champion_and_is_deterministic() {
        let players: Vec<Player> = (0..32).map(|i| player(80.0 - f64::from(i))).collect();
        let a = simulate_tournament(&players, 0xACE, 1);
        let b = simulate_tournament(&players, 0xACE, 1);
        assert_eq!(a, b, "same seed must give the same bracket");
        // 32 -> 5 rounds (16, 8, 4, 2, 1 matches).
        assert_eq!(a.rounds.len(), 5);
        assert_eq!(a.rounds[0].len(), 16);
        assert_eq!(a.rounds.last().unwrap().len(), 1);
        assert!(a.champion < 32);
    }

    #[test]
    fn a_dominant_top_seed_usually_wins() {
        // Top seed far stronger than the field.
        let mut players: Vec<Player> = (0..16).map(|_| player(50.0)).collect();
        players[0] = player(95.0);
        let mut titles = 0;
        for t in 0..200 {
            if simulate_tournament(&players, 1234, t).champion == 0 {
                titles += 1;
            }
        }
        assert!(titles > 150, "dominant top seed won {titles}/200");
    }
}
