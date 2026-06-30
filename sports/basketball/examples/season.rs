//! Proof-of-life: a 16-team basketball league simulated day by day across a full season;
//! prints the final win/loss standings.
//!
//! Run: `cargo run -p basketball --example season --release`

use bevy_ecs::prelude::*;
use basketball::{play_due_fixtures, Baller, Season, TeamId};
use sim_core::{build_daily_schedule, Date, SimClock, SimSeed};

fn main() {
    const TEAMS: u32 = 16;
    const ROSTER: u32 = 10;
    const WORLD_SEED: u64 = 0x5C0;

    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(Date::new(2025, 10, 1)));
    world.insert_resource(SimSeed(WORLD_SEED));

    for t in 0..TEAMS {
        let base = 48 + (t % 16) as i32;
        for p in 0..ROSTER {
            let jitter = (p as i32 % 5) - 2;
            let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
            world.spawn((
                TeamId(t),
                Baller {
                    offense: rate(base),
                    defense: rate(base - 1),
                    three_point: rate(base - 4),
                    rebounding: rate(base),
                },
            ));
        }
    }

    world.insert_resource(Season::new((0..TEAMS).collect(), Date::new(2025, 10, 8), WORLD_SEED, 2025));

    let mut daily = build_daily_schedule();
    let mut days = 0u32;
    while !world.resource::<Season>().is_complete() && days < 400 {
        daily.run(&mut world);
        play_due_fixtures(&mut world);
        days += 1;
    }

    let season = world.resource::<Season>();
    println!(
        "Basketball season 2025: {} games each, {days} simulated days. Final standings:",
        season.schedule.len()
    );
    println!("  seed  team    W    L    Pct    PF     PA   Diff");
    for (pos, (team, r)) in season.standings().iter().enumerate() {
        println!(
            "  {:>4}  {:>4}  {:>3}  {:>3}  {:>5.3}  {:>5} {:>5}  {:>+5}",
            pos + 1,
            team,
            r.won,
            r.lost,
            r.win_pct(),
            r.points_for,
            r.points_against,
            r.point_diff(),
        );
    }
    if let Some(champ) = season.champion() {
        println!("\nBest record: team {champ}");
    }
}
