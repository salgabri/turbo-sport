//! Proof-of-life for the season loop: a 20-team league simulated day by day across a full
//! season, driven by the calendar — prints the final table and champion.
//!
//! Run: `cargo run -p football --example league_season --release`

use bevy_ecs::prelude::*;
use football::{play_due_fixtures, Footballer, Season, TeamId};
use sim_core::{build_daily_schedule, Date, SimClock, SimSeed};

fn main() {
    const TEAMS: u32 = 20;
    const SQUAD: u32 = 11;
    const WORLD_SEED: u64 = 0x1EA6; // "league"
    const SEASON_ID: u32 = 2025;

    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(Date::new(2025, 8, 1)));
    world.insert_resource(SimSeed(WORLD_SEED));

    // Spawn 20 clubs of varying strength (base 50..69), players jittered around it.
    for t in 0..TEAMS {
        let base = 50 + (t % 20) as i32;
        for p in 0..SQUAD {
            let jitter = (p as i32 % 5) - 2;
            let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
            world.spawn((
                TeamId(t),
                Footballer {
                    attacking: rate(base),
                    defending: rate(base - 2),
                    finishing: rate(base + 1),
                    goalkeeping: rate(base - 3),
                },
            ));
        }
    }

    world.insert_resource(Season::new(
        (0..TEAMS).collect(),
        Date::new(2025, 8, 9), // first matchday
        WORLD_SEED,
        SEASON_ID,
    ));

    // Run day by day: advance sim-core's daily systems, then play any due matchday.
    let mut daily = build_daily_schedule();
    let mut days = 0u32;
    while !world.resource::<Season>().is_complete() && days < 400 {
        daily.run(&mut world);
        play_due_fixtures(&mut world);
        days += 1;
    }

    let season = world.resource::<Season>();
    println!(
        "Season {SEASON_ID}: {} matchdays over {days} simulated days. Final table:",
        season.matchdays.len()
    );
    println!("  pos  team    P   W   D   L   GF   GA   GD   Pts");
    for (pos, (team, r)) in season.standings().iter().enumerate() {
        println!(
            "  {:>3}  {:>4}  {:>3} {:>3} {:>3} {:>3} {:>4} {:>4} {:>+4}  {:>4}",
            pos + 1,
            team,
            r.played,
            r.won,
            r.drawn,
            r.lost,
            r.goals_for,
            r.goals_against,
            r.goal_difference(),
            r.points,
        );
    }
    if let Some(champ) = season.champion() {
        println!("\nChampions: team {champ}");
    }
}
