//! Tauri backend for the basketball game. Same shape as the football app, with basketball's
//! engine, win/loss standings, and save format.

use std::sync::Mutex;

use basketball::Season;
use bevy_ecs::prelude::*;
use serde::Serialize;
use sim_core::{
    build_daily_schedule, club_views, run_transfer_window, sync_squad_membership, Club, ClubView,
    SimClock, SimSeed, TeamId,
};
use tauri::State;

struct AppState {
    world: Mutex<World>,
}

fn starting_world() -> World {
    let db = std::env::var("TURBO_DB")
        .ok()
        .and_then(|path| basketball::database::load(path).ok())
        .unwrap_or_else(basketball::database::sample);
    basketball::load_world(&db)
}

#[tauri::command]
fn clubs(state: State<AppState>) -> Vec<ClubView> {
    club_views(&mut state.world.lock().unwrap())
}

#[tauri::command]
fn team_squad(team_id: u32, state: State<AppState>) -> Vec<basketball::SquadPlayer> {
    basketball::team_squad_detailed(&mut state.world.lock().unwrap(), team_id)
}

#[tauri::command]
fn market(limit: usize, state: State<AppState>) -> Vec<basketball::SquadPlayer> {
    basketball::free_agents_detailed(&mut state.world.lock().unwrap(), limit)
}

#[tauri::command]
fn current_date(state: State<AppState>) -> String {
    state.world.lock().unwrap().resource::<SimClock>().date().to_string()
}

#[tauri::command]
fn season_active(state: State<AppState>) -> bool {
    state.world.lock().unwrap().get_resource::<Season>().is_some()
}

/// One row of the basketball standings (win/loss, no draws).
#[derive(Serialize)]
struct StandingRow {
    team_id: u32,
    won: u32,
    lost: u32,
    win_pct: f64,
    points_for: u32,
    points_against: u32,
    point_diff: i32,
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
                won: r.won,
                lost: r.lost,
                win_pct: r.win_pct(),
                points_for: r.points_for,
                points_against: r.points_against,
                point_diff: r.point_diff(),
            })
            .collect(),
        None => Vec::new(),
    }
}

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

#[tauri::command]
fn advance(days: u32, state: State<AppState>) -> String {
    let mut world = state.world.lock().unwrap();
    let mut schedule = build_daily_schedule();
    for _ in 0..days {
        schedule.run(&mut world);
        basketball::play_due_fixtures(&mut world);
    }
    world.resource::<SimClock>().date().to_string()
}

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
    basketball::persistence::save_file(&world, &path).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_game(path: String, state: State<AppState>) -> Result<String, String> {
    let loaded = basketball::persistence::load_file(&path).map_err(|e| e.to_string())?;
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
