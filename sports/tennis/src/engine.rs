//! The tennis match engine: a **pure**, seeded best-of-three-sets simulation.
//!
//! Shape note: like cycling this is about individuals, but unlike both prior sports the
//! outcome is a *winner advancing* (head-to-head, single result) — which is what a knockout
//! bracket needs. No scoreline to accumulate, no points table; just who progresses.

use rand::Rng;

/// A player's ratings for a match (0..~100 each).
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub serve: f64,
    pub return_game: f64,
    pub baseline: f64,
    pub mental: f64,
}

impl Player {
    pub fn new(serve: f64, return_game: f64, baseline: f64, mental: f64) -> Self {
        Self { serve, return_game, baseline, mental }
    }

    /// Overall match strength. Serve, return, and baseline dominate; mental is a smaller
    /// factor here (it gets its real say in deciding sets).
    fn strength(&self) -> f64 {
        0.3 * self.serve + 0.3 * self.return_game + 0.3 * self.baseline + 0.1 * self.mental
    }
}

/// The result of a match: which input won (0 = first, 1 = second) and the sets won by each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchOutcome {
    pub winner: usize,
    pub sets: (u8, u8),
}

/// Simulate a best-of-three-sets match. Deterministic for a given `rng`. The deciding set
/// (one set all) tilts slightly toward the stronger-minded player.
pub fn simulate_match(p1: &Player, p2: &Player, rng: &mut impl Rng) -> MatchOutcome {
    // Each point of strength difference is ~1% of set-win probability around an even 0.5.
    let edge = 0.01 * (p1.strength() - p2.strength());
    let (mut a, mut b) = (0u8, 0u8);
    while a < 2 && b < 2 {
        let deciding = a == 1 && b == 1;
        let mut p = 0.5 + edge;
        if deciding {
            p += 0.0015 * (p1.mental - p2.mental);
        }
        if rng.gen_bool(p.clamp(0.05, 0.95)) {
            a += 1;
        } else {
            b += 1;
        }
    }
    MatchOutcome { winner: if a > b { 0 } else { 1 }, sets: (a, b) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    fn player(rating: f64) -> Player {
        Player::new(rating, rating, rating, rating)
    }

    #[test]
    fn identical_seed_gives_identical_match() {
        let (p1, p2) = (player(60.0), player(55.0));
        let mut r1 = Pcg64Mcg::seed_from_u64(11);
        let mut r2 = Pcg64Mcg::seed_from_u64(11);
        assert_eq!(simulate_match(&p1, &p2, &mut r1), simulate_match(&p1, &p2, &mut r2));
    }

    #[test]
    fn winner_always_has_two_sets() {
        let (p1, p2) = (player(60.0), player(50.0));
        let mut rng = Pcg64Mcg::seed_from_u64(5);
        for _ in 0..500 {
            let o = simulate_match(&p1, &p2, &mut rng);
            let winning_sets = if o.winner == 0 { o.sets.0 } else { o.sets.1 };
            assert_eq!(winning_sets, 2);
        }
    }

    #[test]
    fn stronger_player_wins_more() {
        let (strong, weak) = (player(75.0), player(45.0));
        let mut rng = Pcg64Mcg::seed_from_u64(2024);
        let mut strong_wins = 0;
        for _ in 0..500 {
            if simulate_match(&strong, &weak, &mut rng).winner == 0 {
                strong_wins += 1;
            }
        }
        assert!(strong_wins > 400, "strong won {strong_wins}/500");
    }
}
