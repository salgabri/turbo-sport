//! The event-tick match engine: a **pure** function from two lineups and a seeded RNG
//! to a result with a timeline of moments.
//!
//! "Pure" is deliberate and load-bearing (see `docs/ARCHITECTURE.md`): the simulation
//! touches no ECS world and no shared state, so a matchday's fixtures can be run across
//! cores with `rayon` without locks, and — because each match's RNG is seeded from the
//! world seed plus stable coordinates — the result is identical no matter how threads
//! are scheduled. Determinism and parallelism, together.
//!
//! Fidelity: minute-by-minute *chances* (not full player positioning). Each simulated
//! minute, each side may create a chance; a chance has a goal probability (its xG) and
//! may or may not be on target. Enough to produce a scoreline, an xG figure, and a
//! timeline of moments to surface — the "Excel with moments" target — while staying
//! cheap enough to multiply across thousands of concurrent matches.

use rand::Rng;

/// Minutes simulated per match.
pub const MATCH_MINUTES: u32 = 90;

/// Per-minute base rate at which a side creates a chance, before the attack/defense
/// ratio scales it. Tuned so two average sides produce a realistic shot count.
const CHANCE_RATE: f64 = 0.28;

/// Baseline conversion of a chance, before the finishing/keeper ratio scales it. Tuned
/// so two even sides (ratio 0.5) convert ~0.11 per chance — a realistic ~2.8 goals/game.
const BASE_CONVERSION: f64 = 0.22;

/// Probability a non-scoring chance was still on target (flavour for the timeline).
const OFF_SCORE_ON_TARGET: f64 = 0.40;

/// A team's aggregate ratings for a single match (0..~100 each). Aggregate rather than
/// per-player for now; deepening to eleven players is a later refinement that does not
/// change this engine's shape.
#[derive(Clone, Copy, Debug)]
pub struct Lineup {
    pub attack: f64,
    pub defense: f64,
    pub finishing: f64,
    pub keeper: f64,
}

impl Lineup {
    pub fn new(attack: f64, defense: f64, finishing: f64, keeper: f64) -> Self {
        Self { attack, defense, finishing, keeper }
    }
}

/// One chance in the match timeline — a "moment".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chance {
    pub minute: u32,
    /// True if the home side created it.
    pub home: bool,
    pub on_target: bool,
    pub goal: bool,
}

/// The outcome of a single match.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchResult {
    pub home_goals: u8,
    pub away_goals: u8,
    /// Expected goals — the summed goal probability of every chance created.
    pub home_xg: f64,
    pub away_xg: f64,
    pub timeline: Vec<Chance>,
}

/// Probability that a side with `attack` creates a chance in a given minute against
/// `opp_defense`. Clamped into a sane range so extreme ratings can't break it.
fn chance_prob(attack: f64, opp_defense: f64) -> f64 {
    let share = attack / (attack + opp_defense).max(1.0);
    (CHANCE_RATE * share).clamp(0.0, 0.95)
}

/// Probability a created chance is scored (its xG), given the attacker's finishing and
/// the defending keeper.
fn goal_prob(finishing: f64, keeper: f64) -> f64 {
    let share = finishing / (finishing + keeper).max(1.0);
    (BASE_CONVERSION * share).clamp(0.0, 0.95)
}

/// Simulate one match. Deterministic for a given `rng` state — seed the `rng` from the
/// world seed plus stable coordinates to make a whole matchday reproducible.
pub fn simulate_match(home: &Lineup, away: &Lineup, rng: &mut impl Rng) -> MatchResult {
    let p_home = chance_prob(home.attack, away.defense);
    let p_away = chance_prob(away.attack, home.defense);
    let xg_home_per = goal_prob(home.finishing, away.keeper);
    let xg_away_per = goal_prob(away.finishing, home.keeper);

    let mut res = MatchResult {
        home_goals: 0,
        away_goals: 0,
        home_xg: 0.0,
        away_xg: 0.0,
        timeline: Vec::new(),
    };

    for minute in 1..=MATCH_MINUTES {
        // Home side, then away side — fixed order keeps the RNG sequence deterministic.
        for home_side in [true, false] {
            let (p_chance, xg) = if home_side {
                (p_home, xg_home_per)
            } else {
                (p_away, xg_away_per)
            };
            if !rng.gen_bool(p_chance) {
                continue;
            }
            let goal = rng.gen_bool(xg);
            let on_target = goal || rng.gen_bool(OFF_SCORE_ON_TARGET);
            if home_side {
                res.home_xg += xg;
                res.home_goals += u8::from(goal);
            } else {
                res.away_xg += xg;
                res.away_goals += u8::from(goal);
            }
            res.timeline.push(Chance { minute, home: home_side, on_target, goal });
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    fn even() -> Lineup {
        Lineup::new(50.0, 50.0, 50.0, 50.0)
    }

    #[test]
    fn identical_seed_gives_identical_match() {
        let (h, a) = (even(), even());
        let mut r1 = Pcg64Mcg::seed_from_u64(12345);
        let mut r2 = Pcg64Mcg::seed_from_u64(12345);
        assert_eq!(simulate_match(&h, &a, &mut r1), simulate_match(&h, &a, &mut r2));
    }

    #[test]
    fn timeline_goals_match_the_scoreline() {
        let (h, a) = (even(), even());
        let mut rng = Pcg64Mcg::seed_from_u64(99);
        let res = simulate_match(&h, &a, &mut rng);
        let home_goal_events = res.timeline.iter().filter(|c| c.home && c.goal).count();
        let away_goal_events = res.timeline.iter().filter(|c| !c.home && c.goal).count();
        assert_eq!(home_goal_events, res.home_goals as usize);
        assert_eq!(away_goal_events, res.away_goals as usize);
        // A goal is always on target.
        assert!(res.timeline.iter().all(|c| !c.goal || c.on_target));
    }

    #[test]
    fn average_scoring_is_in_a_plausible_range() {
        let (h, a) = (even(), even());
        let mut rng = Pcg64Mcg::seed_from_u64(7);
        let n = 4000;
        let total: u32 =
            (0..n).map(|_| {
                let r = simulate_match(&h, &a, &mut rng);
                u32::from(r.home_goals) + u32::from(r.away_goals)
            }).sum();
        let mean = f64::from(total) / f64::from(n);
        assert!((1.5..4.5).contains(&mean), "mean total goals was {mean}");
    }

    #[test]
    fn stronger_team_scores_more_on_average() {
        let strong = Lineup::new(75.0, 70.0, 70.0, 70.0);
        let weak = Lineup::new(40.0, 35.0, 40.0, 35.0);
        let mut rng = Pcg64Mcg::seed_from_u64(2024);
        let n = 3000;
        let (mut strong_goals, mut weak_goals) = (0u32, 0u32);
        for _ in 0..n {
            let r = simulate_match(&strong, &weak, &mut rng);
            strong_goals += u32::from(r.home_goals);
            weak_goals += u32::from(r.away_goals);
        }
        assert!(strong_goals > weak_goals * 2, "strong {strong_goals} vs weak {weak_goals}");
    }
}
