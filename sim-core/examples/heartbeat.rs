//! Throwaway proof-of-life for build-order steps 2–3.
//!
//! Spawns a large population of people (birth date, contract, condition, morale) into a
//! `bevy_ecs` world, then runs the standard daily schedule for several seasons. It
//! proves the tick loop runs, the calendar rolls days → seasons, and the lifecycle
//! systems fire: contracts expire into free agency, injuries heal, old players retire.
//! It is **not** a template for real systems — there is no sport logic or economy yet,
//! by design (see `CLAUDE.md`).
//!
//! Run: `cargo run -p sim-core --example heartbeat --release`

use bevy_ecs::prelude::*;
use sim_core::Date;
use sim_core::{
    build_daily_schedule, Balance, BirthDate, Condition, Contract, FreeAgent, Morale, Retired,
    SimClock, SimSeed, WeeklyIncome,
};

fn main() {
    const PEOPLE: usize = 100_000;
    const SEASONS: i32 = 3;

    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
    world.insert_resource(SimSeed(0xC0FFEE));

    // One club to own everyone's contract, with a balance and a flat weekly income.
    // (A single club holding 100k players is obviously unrealistic — this is a stress
    // demo of the payroll mechanic, not a league.)
    let club = world
        .spawn((Balance(50_000_000), WeeklyIncome(120_000_000)))
        .id();

    // Spawn the population as packed component arrays — no per-person heap objects
    // (CLAUDE.md hard constraint #1). Ages 16..=40, contracts ending across the next
    // 1–4 seasons, ~2% carrying a starting injury so recovery has something to do.
    world.spawn_batch((0..PEOPLE).map(move |i| {
        let age = 16 + (i % 25) as i32; // 16..=40
        let birth = Date::new(2025 - age, 1 + (i % 12) as u8, 1 + (i % 28) as u8);
        let until = Date::new(2026 + (i % 4) as i32, 6, 30);
        let injury_days = if i % 50 == 0 { 30 } else { 0 };
        (
            BirthDate(birth),
            Morale(70),
            Condition { fitness: if injury_days > 0 { 60 } else { 90 }, injury_days },
            Contract { club, until, wage: 1000 },
        )
    }));

    let start = *world.resource::<SimClock>();
    let opening_wage_bill: i64 = world.query::<&Contract>().iter(&world).map(|c| i64::from(c.wage)).sum();
    println!(
        "spawned {PEOPLE} people; clock starts {} (season {}/{:02}); opening weekly wage bill {opening_wage_bill}",
        start.date(),
        start.season_start_year(),
        (start.season_start_year() + 1) % 100,
    );

    let mut schedule = build_daily_schedule();

    let mut last_season = start.season_start_year();
    for _ in 0..(365 * SEASONS) {
        schedule.run(&mut world);
        let season = world.resource::<SimClock>().season_start_year();
        if season != last_season {
            println!("  season rollover -> {season}/{:02}", (season + 1) % 100);
            last_season = season;
        }
    }

    // Read the population back via queries — linear scans over packed component arrays.
    let free_agents = world.query_filtered::<(), With<FreeAgent>>().iter(&world).count();
    let retired = world.query_filtered::<(), With<Retired>>().iter(&world).count();
    let injured = world
        .query::<&Condition>()
        .iter(&world)
        .filter(|c| c.is_injured())
        .count();

    let end = *world.resource::<SimClock>();
    let balance = world.get::<Balance>(club).map_or(0, |b| b.0);
    println!(
        "advanced {} days to {}: {free_agents} free agents, {retired} retired, {injured} injured; club balance {balance}",
        end.day_index(),
        end.date(),
    );
}
