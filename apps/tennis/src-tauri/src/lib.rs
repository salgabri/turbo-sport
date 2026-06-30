//! Tauri backend for the tennis game — an event app: load a seeded draw, play a single-
//! elimination tournament, show the bracket and champion. No season/finances.

use std::sync::Mutex;

use bevy_ecs::prelude::*;
use serde::Serialize;
use sim_core::{Name, SimSeed};
use tauri::State;
use tennis::{simulate_tournament, Player, Seed, TennisPlayer};

struct AppState {
    world: Mutex<World>,
}

fn starting_world() -> World {
    let db = std::env::var("TURBO_DB")
        .ok()
        .and_then(|path| tennis::database::load(path).ok())
        .unwrap_or_else(tennis::database::sample);
    tennis::load_world(&db)
}

/// The draw in seed order: (seed, name, engine player).
fn draw_seeded(world: &mut World) -> Vec<(u32, String, Player)> {
    let mut q = world.query::<(&Seed, &TennisPlayer, &Name)>();
    let mut v: Vec<(u32, String, Player)> = q
        .iter(world)
        .map(|(s, p, n)| {
            (
                s.0,
                n.0.clone(),
                Player::new(
                    f64::from(p.serve),
                    f64::from(p.return_game),
                    f64::from(p.baseline),
                    f64::from(p.mental),
                ),
            )
        })
        .collect();
    v.sort_by_key(|(seed, _, _)| *seed);
    v
}

#[derive(Serialize)]
struct PlayerRow {
    seed: u32,
    name: String,
    serve: u8,
    return_game: u8,
    baseline: u8,
    mental: u8,
}

#[tauri::command]
fn draw(state: State<AppState>) -> Vec<PlayerRow> {
    let mut world = state.world.lock().unwrap();
    let mut q = world.query::<(&Seed, &TennisPlayer, &Name)>();
    let mut v: Vec<PlayerRow> = q
        .iter(&world)
        .map(|(s, p, n)| PlayerRow {
            seed: s.0,
            name: n.0.clone(),
            serve: p.serve,
            return_game: p.return_game,
            baseline: p.baseline,
            mental: p.mental,
        })
        .collect();
    v.sort_by_key(|r| r.seed);
    v
}

#[derive(Serialize)]
struct MatchRow {
    winner: String,
    loser: String,
    score: String,
}

#[derive(Serialize)]
struct RoundOut {
    name: String,
    matches: Vec<MatchRow>,
}

#[derive(Serialize)]
struct Tourney {
    champion: String,
    rounds: Vec<RoundOut>,
}

fn round_name(matches: usize) -> String {
    match matches {
        1 => "final".to_string(),
        2 => "semifinals".to_string(),
        4 => "quarterfinals".to_string(),
        n => format!("round of {}", n * 2),
    }
}

#[tauri::command]
fn run_tournament(state: State<AppState>) -> Tourney {
    let mut world = state.world.lock().unwrap();
    let seed = world.get_resource::<SimSeed>().map_or(0, |s| s.0);
    let entries = draw_seeded(&mut world);
    let names: Vec<String> = entries.iter().map(|(_, n, _)| n.clone()).collect();
    let players: Vec<Player> = entries.iter().map(|(_, _, p)| *p).collect();

    let result = simulate_tournament(&players, seed, 1);
    let rounds = result
        .rounds
        .iter()
        .map(|round| RoundOut {
            name: round_name(round.len()),
            matches: round
                .iter()
                .map(|m| {
                    let (ws, ls) = if m.winner == m.a { (m.sets.0, m.sets.1) } else { (m.sets.1, m.sets.0) };
                    let loser = if m.winner == m.a { m.b } else { m.a };
                    MatchRow {
                        winner: names[m.winner].clone(),
                        loser: names[loser].clone(),
                        score: format!("{ws}-{ls}"),
                    }
                })
                .collect(),
        })
        .collect();

    Tourney { champion: names[result.champion].clone(), rounds }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState { world: Mutex::new(starting_world()) })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![draw, run_tournament])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
