//! Tauri backend: holds the simulation world (built from the starting database) and exposes
//! the sim-core read layer as commands the Svelte frontend calls. Only summarized DTOs cross
//! the IPC boundary — never raw entity rows.

use std::sync::Mutex;

use bevy_ecs::prelude::World;
use football::Database;
use sim_core::{club_views, free_agents, squad, ClubView, PlayerView};
use tauri::State;

/// The live world, behind a mutex so commands can borrow it mutably (queries need `&mut`).
struct AppState {
    world: Mutex<World>,
}

/// Build the starting world from a database: the `TURBO_DB` env var points at a JSON database
/// file (as authored/edited in the editor app); otherwise the built-in sample is used.
fn starting_world() -> World {
    let db = std::env::var("TURBO_DB")
        .ok()
        .and_then(|path| football::database::load(path).ok())
        .unwrap_or_else(Database::sample);
    football::load_world(&db)
}

#[tauri::command]
fn clubs(state: State<AppState>) -> Vec<ClubView> {
    club_views(&mut state.world.lock().unwrap())
}

#[tauri::command]
fn team_squad(team_id: u32, state: State<AppState>) -> Vec<PlayerView> {
    squad(&mut state.world.lock().unwrap(), team_id)
}

#[tauri::command]
fn market(limit: usize, state: State<AppState>) -> Vec<PlayerView> {
    free_agents(&mut state.world.lock().unwrap(), limit)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState { world: Mutex::new(starting_world()) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![clubs, team_squad, market])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
