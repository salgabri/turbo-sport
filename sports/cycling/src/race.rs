//! A stage race and its general classification (GC).
//!
//! Where football's competition is a matchday of independent head-to-head results, a
//! cycling race is a *sequence* of stages whose times **accumulate** per rider into the
//! GC. Same gather-from-ECS → simulate → standings pipeline as football, but the
//! standings logic is fundamentally different — another seam for the harvest step.

use crate::attributes::{Rider, StageType};
use crate::stage::simulate_stage;
use bevy_ecs::prelude::*;

/// A stage race: an ordered list of stage profiles.
pub struct Race {
    pub stages: Vec<StageType>,
}

/// One rider's standing in the general classification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GcEntry {
    pub rider_index: usize,
    pub total_secs: f64,
}

/// Collect every rider from the ECS world into a start list. Order is the world's entity
/// iteration order and is stable for a given world, which keeps seeding reproducible.
pub fn gather_riders(world: &mut World) -> Vec<Rider> {
    let mut q = world.query::<&Rider>();
    q.iter(world).copied().collect()
}

/// Simulate a whole race and return the GC, sorted ascending by total time (leader
/// first). Stages run in order (the GC is their cumulative sum); within each stage the
/// riders are simulated in parallel. Fully deterministic for a given seed.
pub fn simulate_race(riders: &[Rider], race: &Race, world_seed: u64, race_id: u32) -> Vec<GcEntry> {
    let mut totals = vec![0.0f64; riders.len()];
    for (s, &stage) in race.stages.iter().enumerate() {
        for t in simulate_stage(riders, stage, world_seed, race_id, s as u32) {
            totals[t.rider_index] += t.secs;
        }
    }

    let mut gc: Vec<GcEntry> = totals
        .iter()
        .enumerate()
        .map(|(i, &total_secs)| GcEntry { rider_index: i, total_secs })
        .collect();
    // total_cmp gives a stable, panic-free ordering of the finite f64 times.
    gc.sort_by(|a, b| a.total_secs.total_cmp(&b.total_secs));
    gc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rider(climbing: u8, sprinting: u8, time_trial: u8, endurance: u8) -> Rider {
        Rider { climbing, sprinting, time_trial, endurance }
    }

    fn grand_tour() -> Race {
        Race {
            stages: vec![
                StageType::Flat,
                StageType::Hilly,
                StageType::Mountain,
                StageType::TimeTrial,
                StageType::Mountain,
            ],
        }
    }

    #[test]
    fn race_is_deterministic() {
        let riders: Vec<Rider> = (0..40).map(|i| rider(50 + (i % 40) as u8, 55, 50, 60)).collect();
        let a = simulate_race(&riders, &grand_tour(), 123, 9);
        let b = simulate_race(&riders, &grand_tour(), 123, 9);
        assert_eq!(a, b);
    }

    #[test]
    fn gc_total_is_the_sum_of_stage_times() {
        let riders = [rider(70, 60, 65, 70), rider(55, 80, 60, 65)];
        let race = grand_tour();
        let gc = simulate_race(&riders, &race, 42, 1);

        // Recompute each rider's total independently and compare.
        let mut expected = vec![0.0f64; riders.len()];
        for (s, &stage) in race.stages.iter().enumerate() {
            for t in simulate_stage(&riders, stage, 42, 1, s as u32) {
                expected[t.rider_index] += t.secs;
            }
        }
        for entry in &gc {
            assert_eq!(entry.total_secs, expected[entry.rider_index]);
        }
    }

    #[test]
    fn all_rounder_wins_a_balanced_grand_tour() {
        // A strong all-rounder should beat a pure sprinter over mixed terrain.
        let all_rounder = rider(80, 70, 80, 85);
        let sprinter = rider(45, 92, 55, 70);
        let gc = simulate_race(&[all_rounder, sprinter], &grand_tour(), 2024, 1);
        assert_eq!(gc[0].rider_index, 0, "all-rounder should lead the GC");
    }
}
