//! Proof-of-life for build step 7A: spawn a peloton into an ECS world and simulate a
//! multi-stage grand tour, printing the general classification — then re-run to prove
//! determinism.
//!
//! Run: `cargo run -p cycling --example grand_tour --release`

use bevy_ecs::prelude::*;
use cycling::{gather_riders, simulate_race, Race, Rider, StageType};

fn main() {
    const RIDERS: u32 = 176; // a realistic grand-tour peloton
    const WORLD_SEED: u64 = 0x5EED;
    const RACE_ID: u32 = 1;

    // Spawn a varied peloton: pure climbers, sprinters, rouleurs, all-rounders.
    let mut world = World::new();
    for i in 0..RIDERS {
        let archetype = i % 4;
        let r = match archetype {
            0 => Rider { climbing: 85, sprinting: 45, time_trial: 70, endurance: 80 }, // climber
            1 => Rider { climbing: 45, sprinting: 88, time_trial: 55, endurance: 72 }, // sprinter
            2 => Rider { climbing: 60, sprinting: 60, time_trial: 82, endurance: 78 }, // TT specialist
            _ => Rider { climbing: 75, sprinting: 68, time_trial: 75, endurance: 82 }, // all-rounder
        };
        // Light per-rider variation so no two are identical.
        let jitter = (i % 7) as i8 - 3;
        let adj = |x: u8| (x as i16 + jitter as i16).clamp(1, 99) as u8;
        world.spawn(Rider {
            climbing: adj(r.climbing),
            sprinting: adj(r.sprinting),
            time_trial: adj(r.time_trial),
            endurance: adj(r.endurance),
        });
    }

    let riders = gather_riders(&mut world);

    let race = Race {
        stages: vec![
            StageType::Flat,
            StageType::Hilly,
            StageType::Mountain,
            StageType::TimeTrial,
            StageType::Mountain,
            StageType::Flat,
            StageType::Mountain,
        ],
    };

    let gc = simulate_race(&riders, &race, WORLD_SEED, RACE_ID);

    println!(
        "{}-stage grand tour, {} riders (stages simulated in parallel). Final GC top 10:",
        race.stages.len(),
        riders.len()
    );
    let winner = gc[0].total_secs;
    for (pos, entry) in gc.iter().take(10).enumerate() {
        let gap = entry.total_secs - winner;
        let r = &riders[entry.rider_index];
        println!(
            "  {:>2}. rider {:>3}  +{:>4.0}s   (clb {} spr {} tt {} end {})",
            pos + 1,
            entry.rider_index,
            gap,
            r.climbing,
            r.sprinting,
            r.time_trial,
            r.endurance,
        );
    }

    let again = simulate_race(&riders, &race, WORLD_SEED, RACE_ID);
    assert_eq!(gc, again, "race simulation must be deterministic");
    println!("\ndeterminism check: identical GC on re-run [ok]");
}
