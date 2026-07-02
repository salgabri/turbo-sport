//! Stage **playback**: turn a single simulated mountain stage into a UI-replayable
//! experience.
//!
//! The [`crate::stage`] engine stays a pure `(riders, seed) -> times` function — this module
//! reads every rider out of the world, runs that engine for the deterministic finishing
//! order, and then *dresses* the result into a [`StagePlayback`]: a ranked gap table and a
//! climb profile the front-end replays against a clock. So the "live stage" is a deterministic
//! recording, not a second simulation — same seed, same stage, every time.
//!
//! Nothing here perturbs the engine's own stream: the finishing times come straight from
//! `simulate_stage`, and the (purely deterministic) climb-profile polygon needs no RNG at all.
//! Where synthesis were ever needed it would go on a *separate* seeded stream keyed off the
//! stage seed via [`sim_core::derive_seed`], exactly as football's playback does.

use crate::attributes::{Rider, StageType};
use crate::stage::simulate_stage;
use bevy_ecs::prelude::*;
use serde::Serialize;
use sim_core::Name;

/// One rider's line in the ranked gap table. `gap_secs` is seconds behind the stage winner
/// (the leader is `0.0`).
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct RiderGap {
    pub rank: u32,
    pub name: String,
    pub gap_secs: f64,
}

/// Everything the front-end needs to replay one mountain stage in 2D.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct StagePlayback {
    pub stage_name: String,
    /// Total distance of the (final climb) in km — the clock counts this down to 0.
    pub km_total: f64,
    /// Average gradient label, e.g. `"8.4%"`.
    pub gradient: String,
    /// SVG polygon points for the climb profile, in a `0..100` x by `0..46` y viewBox.
    /// SVG y is top-down, so the summit finish ends at a *small* y.
    pub profile: Vec<[f32; 2]>,
    /// Riders ranked by finishing time, fastest first.
    pub riders: Vec<RiderGap>,
    /// The stage winner's name.
    pub winner: String,
}

/// Collect every rider from the world with their display name, in a stable world order (so the
/// `rider_index` returned by the engine lines up with these names).
fn riders_with_names(world: &mut World) -> Vec<(String, Rider)> {
    let mut q = world.query::<(&Name, &Rider)>();
    q.iter(world)
        .map(|(n, r)| (n.0.clone(), *r))
        .collect()
}

/// Build the deterministic climb profile: a jagged ascending line across the viewBox, then
/// closed back along the bottom so it fills as a polygon. SVG y is top-down, so `y` falls from
/// ~40 (valley) to ~6 (summit) as `x` runs 0→100. No RNG — a fixed, reproducible silhouette.
fn climb_profile() -> Vec<[f32; 2]> {
    // Ten points forming a rising, jagged ascent (x, y). y trends down (climbs) with small kicks.
    const TOP: [[f32; 2]; 10] = [
        [0.0, 40.0],
        [11.0, 37.0],
        [22.0, 38.5],
        [33.0, 31.0],
        [44.0, 27.5],
        [55.0, 24.0],
        [66.0, 25.5],
        [77.0, 16.0],
        [88.0, 11.0],
        [100.0, 6.0],
    ];
    let mut pts: Vec<[f32; 2]> = TOP.to_vec();
    // Close the polygon along the bottom edge (46 is the viewBox floor).
    pts.push([100.0, 46.0]);
    pts.push([0.0, 46.0]);
    pts
}

/// Build a [`StagePlayback`] for the whole peloton, seeded by `seed` (same seed → identical
/// playback). Runs the pure stage engine once and ranks the result; pure over the world read.
pub fn simulate_stage_playback(world: &mut World, seed: u64) -> StagePlayback {
    let pairs = riders_with_names(world);
    let riders: Vec<Rider> = pairs.iter().map(|(_, r)| *r).collect();

    // The stage itself: the engine's own deterministic stream. race_id / stage_index = 0.
    let mut times = simulate_stage(&riders, StageType::Mountain, seed, 0, 0);
    // Fastest first; total_cmp is a stable, panic-free ordering of the finite f64 times.
    times.sort_by(|a, b| a.secs.total_cmp(&b.secs));

    let winner_secs = times.first().map_or(0.0, |t| t.secs);
    let winner = times
        .first()
        .map_or_else(|| "—".to_string(), |t| pairs[t.rider_index].0.clone());

    let gaps: Vec<RiderGap> = times
        .iter()
        .enumerate()
        .map(|(i, t)| RiderGap {
            rank: i as u32 + 1,
            name: pairs[t.rider_index].0.clone(),
            gap_secs: t.secs - winner_secs,
        })
        .collect();

    StagePlayback {
        stage_name: "Queen Stage · Summit Finish".into(),
        km_total: 12.0,
        gradient: "8.4%".into(),
        profile: climb_profile(),
        riders: gaps,
        winner,
    }
}

/// Convenience: build a stage playback using the world's master seed (0 if unset).
pub fn next_stage_playback(world: &mut World) -> StagePlayback {
    let seed = world.get_resource::<sim_core::SimSeed>().map_or(0, |s| s.0);
    simulate_stage_playback(world, seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn same_seed_gives_identical_playback() {
        let db = sample();
        let mut world = load_world(&db);
        let p1 = simulate_stage_playback(&mut world, 0xC0FFEE);
        let p2 = simulate_stage_playback(&mut world, 0xC0FFEE);
        assert_eq!(p1, p2);
    }

    #[test]
    fn gaps_are_sorted_non_negative_with_a_zero_leader() {
        let db = sample();
        let mut world = load_world(&db);
        let pb = simulate_stage_playback(&mut world, 42);

        // One line per rider, ranked 1..=n.
        assert_eq!(pb.riders.len(), db.players.len());
        for (i, r) in pb.riders.iter().enumerate() {
            assert_eq!(r.rank, i as u32 + 1);
        }
        // Leader gap is exactly 0 and names the winner.
        assert_eq!(pb.riders[0].gap_secs, 0.0);
        assert_eq!(pb.riders[0].name, pb.winner);
        // Gaps are non-negative and monotonically non-decreasing (sorted ascending).
        assert!(pb.riders.iter().all(|r| r.gap_secs >= 0.0));
        assert!(pb
            .riders
            .windows(2)
            .all(|w| w[0].gap_secs <= w[1].gap_secs));
        // The closed profile polygon has the two floor points appended.
        assert_eq!(pb.profile.last(), Some(&[0.0, 46.0]));
    }
}
