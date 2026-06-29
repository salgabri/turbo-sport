//! The basketball game engine: a **pure**, seeded possession-by-possession simulation.
//!
//! Same purity/determinism discipline as the other sports. Shape note: like football this
//! is two teams head-to-head, but the result is a high score with **no draws** — a tie goes
//! to overtime until it breaks. That "no draws" rule is exactly why basketball can't share
//! football's 3/1/0 standings, only the round-robin scheduler.

use rand::Rng;

/// Regulation possessions per team.
const REGULATION: u32 = 100;
/// Possessions per overtime period, per team.
const OVERTIME: u32 = 12;
/// Scoring rate scalar — tuned so two even sides score ~105.
const SCORE_BASE: f64 = 0.9;

/// A team's aggregate ratings for one game (0..~100 each).
#[derive(Clone, Copy, Debug)]
pub struct Roster {
    pub offense: f64,
    pub defense: f64,
    pub three_point: f64,
    pub rebounding: f64,
}

impl Roster {
    pub fn new(offense: f64, defense: f64, three_point: f64, rebounding: f64) -> Self {
        Self { offense, defense, three_point, rebounding }
    }
}

/// Final score of a game. Guaranteed `home_points != away_points` (overtime breaks ties).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameResult {
    pub home_points: u16,
    pub away_points: u16,
}

/// Possession count for a team, nudged by the rebounding edge over the opponent.
fn possession_count(base: u32, rebounding: f64, opp_rebounding: f64) -> u32 {
    (f64::from(base) + (rebounding - opp_rebounding) * 0.2).round().clamp(1.0, f64::from(base) * 2.0)
        as u32
}

/// Points a team scores over `n` possessions against `opp_defense`.
fn score_possessions(offense: f64, opp_defense: f64, three_point: f64, n: u32, rng: &mut impl Rng) -> u16 {
    let p_score = (SCORE_BASE * offense / (offense + opp_defense).max(1.0)).clamp(0.0, 0.95);
    let p_three = (0.25 + 0.0025 * three_point).clamp(0.0, 0.9);
    let mut points = 0u16;
    for _ in 0..n {
        if rng.gen_bool(p_score) {
            points += if rng.gen_bool(p_three) { 3 } else { 2 };
        }
    }
    points
}

/// Simulate a full game (regulation plus any overtime). Deterministic for a given `rng`.
pub fn simulate_game(home: &Roster, away: &Roster, rng: &mut impl Rng) -> GameResult {
    let home_poss = possession_count(REGULATION, home.rebounding, away.rebounding);
    let away_poss = possession_count(REGULATION, away.rebounding, home.rebounding);
    let mut h = score_possessions(home.offense, away.defense, home.three_point, home_poss, rng);
    let mut a = score_possessions(away.offense, home.defense, away.three_point, away_poss, rng);
    while h == a {
        h += score_possessions(home.offense, away.defense, home.three_point, OVERTIME, rng);
        a += score_possessions(away.offense, home.defense, away.three_point, OVERTIME, rng);
    }
    GameResult { home_points: h, away_points: a }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    fn even() -> Roster {
        Roster::new(55.0, 55.0, 50.0, 55.0)
    }

    #[test]
    fn identical_seed_gives_identical_game() {
        let (h, a) = (even(), even());
        let mut r1 = Pcg64Mcg::seed_from_u64(7);
        let mut r2 = Pcg64Mcg::seed_from_u64(7);
        assert_eq!(simulate_game(&h, &a, &mut r1), simulate_game(&h, &a, &mut r2));
    }

    #[test]
    fn never_a_draw() {
        let (h, a) = (even(), even());
        let mut rng = Pcg64Mcg::seed_from_u64(1);
        for _ in 0..500 {
            let r = simulate_game(&h, &a, &mut rng);
            assert_ne!(r.home_points, r.away_points);
        }
    }

    #[test]
    fn scoring_is_in_a_basketball_range() {
        let (h, a) = (even(), even());
        let mut rng = Pcg64Mcg::seed_from_u64(3);
        let n = 1000;
        let total: u32 = (0..n)
            .map(|_| {
                let r = simulate_game(&h, &a, &mut rng);
                u32::from(r.home_points) + u32::from(r.away_points)
            })
            .sum();
        let mean = f64::from(total) / f64::from(n);
        assert!((180.0..240.0).contains(&mean), "mean combined points was {mean}");
    }

    #[test]
    fn stronger_offense_wins_more() {
        let strong = Roster::new(75.0, 60.0, 65.0, 60.0);
        let weak = Roster::new(45.0, 50.0, 45.0, 50.0);
        let mut rng = Pcg64Mcg::seed_from_u64(2024);
        let mut strong_wins = 0;
        for _ in 0..500 {
            let r = simulate_game(&strong, &weak, &mut rng);
            if r.home_points > r.away_points {
                strong_wins += 1;
            }
        }
        assert!(strong_wins > 400, "strong team won {strong_wins}/500");
    }
}
