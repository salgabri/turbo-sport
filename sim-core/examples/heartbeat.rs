//! Throwaway proof-of-life for build-order step 2.
//!
//! Spawns a large batch of entities into a `bevy_ecs` world, then advances the
//! simulation clock through several seasons one day at a time via a `Schedule`. It
//! proves the tick loop runs and the calendar rolls days → seasons. It is **not** a
//! template for real systems — there is no lifecycle, economy, or sport logic here yet,
//! by design (see `CLAUDE.md`, "Start here, then stop").
//!
//! Run: `cargo run -p sim-core --example heartbeat --release`

use bevy_ecs::prelude::*;
use sim_core::{advance_time, Date, SimClock, SimSeed};

/// A couple of plain-data components, just so the entities are real archetypes rather
/// than empty handles. Stand-ins — the actual entity schema is build-order step 3.
#[derive(Component)]
struct Age(u16);

#[derive(Component)]
struct Fitness(u8);

fn main() {
    const ENTITIES: usize = 100_000;
    const SEASONS: i32 = 3;

    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
    world.insert_resource(SimSeed(0xC0FFEE));

    // Spawn the throwaway population as packed component arrays (the whole point of ECS
    // here — no per-entity heap objects).
    world.spawn_batch((0..ENTITIES).map(|i| (Age(16 + (i % 25) as u16), Fitness(100))));

    let start = *world.resource::<SimClock>();
    println!(
        "spawned {ENTITIES} entities; clock starts {} (season {}/{})",
        start.date(),
        start.season_start_year(),
        (start.season_start_year() + 1) % 100,
    );

    // One schedule, one system for now: advance the day. Real subsystems get ordered
    // into this same schedule at later build steps.
    let mut schedule = Schedule::default();
    schedule.add_systems(advance_time);

    // Advance day-by-day for SEASONS fixed years, reporting each season rollover.
    let mut last_season = start.season_start_year();
    for _ in 0..(365 * SEASONS) {
        schedule.run(&mut world);
        let clock = world.resource::<SimClock>();
        let season = clock.season_start_year();
        if season != last_season {
            println!("  season rollover -> {season}/{:02} on {}", (season + 1) % 100, clock.date());
            last_season = season;
        }
    }

    let end = *world.resource::<SimClock>();

    // Read the components back via a query — a linear scan over packed component
    // arrays, which is the whole point of the ECS layout (CLAUDE.md hard constraint #1).
    let mut q = world.query::<(&Age, &Fitness)>();
    let (count, age_sum, fit_sum) = q.iter(&world).fold(
        (0u64, 0u64, 0u64),
        |(c, a, f), (age, fit)| (c + 1, a + u64::from(age.0), f + u64::from(fit.0)),
    );

    println!(
        "advanced {} days to {}; {count} entities resident, mean age {:.1}, mean fitness {:.0}",
        end.day_index(),
        end.date(),
        age_sum as f64 / count as f64,
        fit_sum as f64 / count as f64,
    );
}
