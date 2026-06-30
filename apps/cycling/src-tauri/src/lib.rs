//! Tauri backend for the cycling game — an event app: load a peloton, run a grand tour,
//! show the general classification. No season/finances (cycling is individual).

use std::sync::Mutex;

use bevy_ecs::prelude::*;
use cycling::{simulate_race, Race, Rider, StageType};
use serde::Serialize;
use sim_core::{Name, SimSeed};
use tauri::State;

struct AppState {
    world: Mutex<World>,
}

fn starting_world() -> World {
    let db = std::env::var("TURBO_DB")
        .ok()
        .and_then(|path| cycling::database::load(path).ok())
        .unwrap_or_else(cycling::database::sample);
    cycling::load_world(&db)
}

#[derive(Serialize)]
struct RiderRow {
    name: String,
    climbing: u8,
    sprinting: u8,
    time_trial: u8,
    endurance: u8,
}

/// Rider + name pairs, in a stable world order.
fn riders_with_names(world: &mut World) -> Vec<(String, Rider)> {
    let mut q = world.query::<(&Name, &Rider)>();
    q.iter(world).map(|(n, r)| (n.0.clone(), *r)).collect()
}

#[tauri::command]
fn roster(state: State<AppState>) -> Vec<RiderRow> {
    riders_with_names(&mut state.world.lock().unwrap())
        .into_iter()
        .map(|(name, r)| RiderRow {
            name,
            climbing: r.climbing,
            sprinting: r.sprinting,
            time_trial: r.time_trial,
            endurance: r.endurance,
        })
        .collect()
}

#[derive(Serialize)]
struct GcRow {
    rank: usize,
    name: String,
    gap_secs: i64,
}

#[tauri::command]
fn run_tour(state: State<AppState>) -> Vec<GcRow> {
    let mut world = state.world.lock().unwrap();
    let seed = world.get_resource::<SimSeed>().map_or(0, |s| s.0);
    let pairs = riders_with_names(&mut world);
    let riders: Vec<Rider> = pairs.iter().map(|(_, r)| *r).collect();

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
    let gc = simulate_race(&riders, &race, seed, 1);
    let winner = gc.first().map_or(0.0, |e| e.total_secs);
    gc.iter()
        .enumerate()
        .map(|(i, e)| GcRow {
            rank: i + 1,
            name: pairs[e.rider_index].0.clone(),
            gap_secs: (e.total_secs - winner) as i64,
        })
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState { world: Mutex::new(starting_world()) })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![roster, run_tour])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
