//! Proof-of-life for build step 5: spawn a league of teams into an ECS world, gather
//! lineups, and simulate a whole matchday in parallel — then prove the result is
//! deterministic by re-running it.
//!
//! Run: `cargo run -p football --example matchday --release`

use bevy_ecs::prelude::*;
use football::{gather_lineups, simulate_matchday, Fixture, Footballer, TeamId};

fn main() {
    const TEAMS: u32 = 20;
    const SQUAD: u32 = 11;
    const WORLD_SEED: u64 = 0xBEEF;
    const SEASON: u32 = 2025;
    const MATCHDAY: u32 = 1;

    // Spawn a league: each team has a base strength, players jitter around it.
    let mut world = World::new();
    for t in 0..TEAMS {
        let base = 45 + (t % 10) as i32 * 3; // 45..=72
        for p in 0..SQUAD {
            let jitter = (p as i32 % 5) - 2; // -2..=2
            let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
            world.spawn((
                TeamId(t),
                Footballer {
                    attacking: rate(base),
                    defending: rate(base - 3),
                    finishing: rate(base + 2),
                    goalkeeping: rate(base - 5),
                },
            ));
        }
    }

    let lineups = gather_lineups(&mut world);

    // Pair consecutive teams into a matchday.
    let fixtures: Vec<Fixture> = (0..TEAMS)
        .step_by(2)
        .map(|t| Fixture { home: lineups[&t], away: lineups[&(t + 1)] })
        .collect();

    let results = simulate_matchday(&fixtures, WORLD_SEED, SEASON, MATCHDAY);

    println!(
        "Matchday {MATCHDAY}, season {SEASON} — {} fixtures simulated across cores:",
        fixtures.len()
    );
    for (i, r) in results.iter().enumerate() {
        let (home, away) = (i as u32 * 2, i as u32 * 2 + 1);
        println!(
            "  team {home:>2} {}-{} team {away:<2}   xG {:.2}-{:.2}  ({} chances)",
            r.home_goals, r.away_goals, r.home_xg, r.away_xg, r.timeline.len(),
        );
    }

    if let Some(r) = results.first() {
        println!("\nTimeline — team 0 vs team 1:");
        for c in &r.timeline {
            let side = if c.home { "team 0" } else { "team 1" };
            let kind = if c.goal {
                "GOAL"
            } else if c.on_target {
                "shot (saved)"
            } else {
                "shot (off target)"
            };
            println!("  {:>2}' {side} {kind}", c.minute);
        }
    }

    // Determinism proof: identical inputs must give identical output.
    let again = simulate_matchday(&fixtures, WORLD_SEED, SEASON, MATCHDAY);
    assert_eq!(results, again, "matchday simulation must be deterministic");
    println!("\ndeterminism check: identical results on re-run [ok]");
}
