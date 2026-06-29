//! Proof-of-life: a 32-player single-elimination draw spawned into an ECS world, the whole
//! bracket simulated (each round in parallel), printed round by round.
//!
//! Run: `cargo run -p tennis --example grand_slam --release`

use bevy_ecs::prelude::*;
use tennis::{gather_draw, simulate_tournament, Seed, TennisPlayer};

fn main() {
    const DRAW: u32 = 32;
    const WORLD_SEED: u64 = 0x6A3E; // "gate"
    const TOURNAMENT_ID: u32 = 1;

    // Spawn a seeded field: ability decreases with seed number, with light jitter.
    let mut world = World::new();
    for s in 0..DRAW {
        let base = 90 - (s as i32) * 2; // seed 0 ~90 down to ~28
        let jitter = (s % 5) as i32 - 2;
        let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
        world.spawn((
            Seed(s),
            TennisPlayer {
                serve: rate(base),
                return_game: rate(base - 2),
                baseline: rate(base - 1),
                mental: rate(base - 3),
            },
        ));
    }

    let draw = gather_draw(&mut world);
    let result = simulate_tournament(&draw, WORLD_SEED, TOURNAMENT_ID);

    let round_name = |remaining: usize| -> String {
        match remaining {
            1 => "Final".to_string(),
            2 => "Semifinals".to_string(),
            4 => "Quarterfinals".to_string(),
            n => format!("Round of {}", n * 2),
        }
    };

    println!("Grand slam: {DRAW}-player draw, {} rounds:", result.rounds.len());
    for round in &result.rounds {
        println!("\n{} ({} matches):", round_name(round.len()), round.len());
        for m in round {
            let (ws, ls) = if m.winner == m.a { (m.sets.0, m.sets.1) } else { (m.sets.1, m.sets.0) };
            let loser = if m.winner == m.a { m.b } else { m.a };
            println!("  seed {:>2} def. seed {:>2}  {}-{}", m.winner, loser, ws, ls);
        }
    }

    println!("\nChampion: seed {}", result.champion);

    // Determinism proof.
    let again = simulate_tournament(&draw, WORLD_SEED, TOURNAMENT_ID);
    assert_eq!(result, again, "tournament must be deterministic");
    println!("determinism check: identical bracket on re-run [ok]");
}
