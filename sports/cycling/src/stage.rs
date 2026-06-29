//! The stage engine: a **pure**, seeded simulation of one stage that produces a finishing
//! time per rider.
//!
//! Same purity/determinism discipline as football's match engine (no ECS, no shared
//! state, seeded RNG) — but a deliberately different *shape*: the input is N riders (not
//! two lineups), and the output is a time per rider (not a scoreline). That structural
//! mismatch is exactly what the trait-harvest step needs to see.

use crate::attributes::{Rider, StageType};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use sim_core::derive_seed;

/// One rider's result in a stage: their index in the start list and elapsed seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageTime {
    pub rider_index: usize,
    pub secs: f64,
}

/// How much each ability contributes on a given terrain (weights sum to 1.0).
fn performance(r: &Rider, stage: StageType) -> f64 {
    let (c, s, tt, e) = (
        f64::from(r.climbing),
        f64::from(r.sprinting),
        f64::from(r.time_trial),
        f64::from(r.endurance),
    );
    match stage {
        StageType::Flat => 0.6 * s + 0.4 * e,
        StageType::Hilly => 0.4 * c + 0.3 * s + 0.3 * e,
        StageType::Mountain => 0.7 * c + 0.3 * e,
        StageType::TimeTrial => 0.8 * tt + 0.2 * e,
    }
}

/// `(base seconds, ability spread, random noise amplitude)` for a stage type. A stronger
/// rider finishes up to `spread` seconds faster; `noise` is the per-stage randomness.
fn stage_params(stage: StageType) -> (f64, f64, f64) {
    match stage {
        StageType::Flat => (14_400.0, 600.0, 90.0),     // ~4h, bunch finishes (small gaps)
        StageType::Hilly => (14_400.0, 900.0, 120.0),   // ~4h
        StageType::Mountain => (18_000.0, 1_500.0, 150.0), // ~5h, big GC gaps
        StageType::TimeTrial => (3_600.0, 1_200.0, 60.0),  // ~1h, ability dominates
    }
}

/// Simulate one stage for the whole start list, in parallel. Each rider's RNG is seeded
/// from the world seed plus `(race_id, stage_index, rider_index)`, so the times are
/// identical regardless of how `rayon` schedules the riders.
pub fn simulate_stage(
    riders: &[Rider],
    stage: StageType,
    world_seed: u64,
    race_id: u32,
    stage_index: u32,
) -> Vec<StageTime> {
    let (base, spread, noise) = stage_params(stage);
    riders
        .par_iter()
        .enumerate()
        .map(|(i, r)| {
            let seed =
                derive_seed(world_seed, &[u64::from(race_id), u64::from(stage_index), i as u64]);
            let mut rng = Pcg64Mcg::seed_from_u64(seed);
            let secs = base - (performance(r, stage) / 100.0) * spread + rng.gen_range(-noise..noise);
            StageTime { rider_index: i, secs }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rider(climbing: u8, sprinting: u8, time_trial: u8, endurance: u8) -> Rider {
        Rider { climbing, sprinting, time_trial, endurance }
    }

    fn simulate_serial(
        riders: &[Rider],
        stage: StageType,
        world_seed: u64,
        race_id: u32,
        stage_index: u32,
    ) -> Vec<StageTime> {
        let (base, spread, noise) = stage_params(stage);
        riders
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let seed =
                    derive_seed(world_seed, &[u64::from(race_id), u64::from(stage_index), i as u64]);
                let mut rng = Pcg64Mcg::seed_from_u64(seed);
                let secs =
                    base - (performance(r, stage) / 100.0) * spread + rng.gen_range(-noise..noise);
                StageTime { rider_index: i, secs }
            })
            .collect()
    }

    #[test]
    fn parallel_matches_serial_and_repeats() {
        let riders: Vec<Rider> =
            (0..64).map(|i| rider(40 + (i % 50) as u8, 50, 50, 60)).collect();
        let par = simulate_stage(&riders, StageType::Mountain, 0xC0FFEE, 1, 3);
        let serial = simulate_serial(&riders, StageType::Mountain, 0xC0FFEE, 1, 3);
        let again = simulate_stage(&riders, StageType::Mountain, 0xC0FFEE, 1, 3);
        assert_eq!(par, serial);
        assert_eq!(par, again);
    }

    #[test]
    fn climber_beats_sprinter_in_the_mountains() {
        let climber = rider(90, 40, 50, 75);
        let sprinter = rider(40, 90, 50, 70);
        let riders = [climber, sprinter];
        let mut climber_wins = 0;
        for stage_index in 0..200 {
            let t = simulate_stage(&riders, StageType::Mountain, 7, 1, stage_index);
            if t[0].secs < t[1].secs {
                climber_wins += 1;
            }
        }
        assert!(climber_wins > 180, "climber won {climber_wins}/200 mountain stages");
    }
}
