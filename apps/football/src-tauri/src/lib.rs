//! Tauri backend for the football game. Holds the live world (built from the starting
//! database) and exposes read commands (over the sim-core view DTOs) plus mutating commands
//! that advance time, run the transfer market, play a season, and save/load.

use std::sync::Mutex;

use bevy_ecs::prelude::*;
use football::Season;
use serde::Serialize;
use sim_core::{
    build_daily_schedule, club_views, free_agents, run_transfer_window, squad,
    sync_squad_membership, Club, ClubView, PlayerView, SimClock, SimSeed, TeamId,
};
use tauri::State;

/// The live world, behind a mutex so commands can borrow it mutably.
struct AppState {
    world: Mutex<World>,
}

/// Build the starting world from a database (the `TURBO_DB` file, else the sample).
fn starting_world() -> World {
    let db = std::env::var("TURBO_DB")
        .ok()
        .and_then(|path| football::database::load(path).ok())
        .unwrap_or_else(football::database::sample);
    football::load_world(&db)
}

// ---- reads ------------------------------------------------------------------

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

#[tauri::command]
fn current_date(state: State<AppState>) -> String {
    state.world.lock().unwrap().resource::<SimClock>().date().to_string()
}

#[tauri::command]
fn season_active(state: State<AppState>) -> bool {
    state.world.lock().unwrap().get_resource::<Season>().is_some()
}

/// One row of the league table.
#[derive(Serialize)]
struct StandingRow {
    team_id: u32,
    played: u32,
    won: u32,
    drawn: u32,
    lost: u32,
    goals_for: u32,
    goals_against: u32,
    goal_difference: i32,
    points: u32,
}

#[tauri::command]
fn standings(state: State<AppState>) -> Vec<StandingRow> {
    let world = state.world.lock().unwrap();
    match world.get_resource::<Season>() {
        Some(season) => season
            .standings()
            .into_iter()
            .map(|(team_id, r)| StandingRow {
                team_id,
                played: r.played,
                won: r.won,
                drawn: r.drawn,
                lost: r.lost,
                goals_for: r.goals_for,
                goals_against: r.goals_against,
                goal_difference: r.goal_difference(),
                points: r.points,
            })
            .collect(),
        None => Vec::new(),
    }
}

// ---- mutations --------------------------------------------------------------

/// Start a single league among all clubs in the world, kicking off on the current date.
#[tauri::command]
fn start_season(state: State<AppState>) -> Result<(), String> {
    let mut world = state.world.lock().unwrap();
    if world.get_resource::<Season>().is_some() {
        return Err("a season is already running".to_string());
    }
    let mut teams: Vec<u32> =
        world.query_filtered::<&TeamId, With<Club>>().iter(&world).map(|t| t.0).collect();
    teams.sort_unstable();
    if teams.len() < 2 || teams.len() % 2 != 0 {
        return Err(format!("need an even number of clubs (have {})", teams.len()));
    }
    let today = world.resource::<SimClock>().date();
    let seed = world.get_resource::<SimSeed>().map_or(0, |s| s.0);
    world.insert_resource(Season::new(teams, today, seed, today.year() as u32));
    Ok(())
}

/// Advance the simulation `days` days: run the daily schedule (aging, contract expiry, wages)
/// and play any matchday that comes due. Returns the new date.
#[tauri::command]
fn advance(days: u32, state: State<AppState>) -> String {
    let mut world = state.world.lock().unwrap();
    let mut schedule = build_daily_schedule();
    for _ in 0..days {
        schedule.run(&mut world);
        football::play_due_fixtures(&mut world);
    }
    world.resource::<SimClock>().date().to_string()
}

/// Run a transfer window: sign free agents to needy clubs, then sync squad membership.
#[tauri::command]
fn transfer_window(state: State<AppState>) -> u32 {
    let mut world = state.world.lock().unwrap();
    let before = world.query_filtered::<(), With<sim_core::FreeAgent>>().iter(&world).count();
    run_transfer_window(&mut world);
    sync_squad_membership(&mut world);
    let after = world.query_filtered::<(), With<sim_core::FreeAgent>>().iter(&world).count();
    (before - after) as u32
}

#[tauri::command]
fn save_game(path: String, state: State<AppState>) -> Result<(), String> {
    let world = state.world.lock().unwrap();
    football::persistence::save_file(&world, &path).map_err(|e| e.to_string())
}

/// Load a saved game, replacing the live world. An in-progress league season is part of the
/// save and is restored too.
#[tauri::command]
fn load_game(path: String, state: State<AppState>) -> Result<String, String> {
    let loaded = football::persistence::load_file(&path).map_err(|e| e.to_string())?;
    let mut world = state.world.lock().unwrap();
    *world = loaded;
    Ok(world.resource::<SimClock>().date().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState { world: Mutex::new(starting_world()) })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            clubs,
            team_squad,
            market,
            current_date,
            season_active,
            standings,
            start_season,
            advance,
            transfer_window,
            save_game,
            load_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
