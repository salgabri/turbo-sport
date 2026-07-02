//! Tauri backend for the football game. Holds the live world (built from the starting
//! database) and exposes read commands (over the sim-core view DTOs) plus mutating commands
//! that advance time, run the transfer market, play a season, and save/load.

use std::sync::Mutex;

use bevy_ecs::prelude::*;
use football::Season;
use serde::Serialize;
use sim_core::{
    build_daily_schedule, club_views, run_transfer_window, sync_squad_membership, Club, ClubView,
    SimClock, SimSeed, TeamId,
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
fn team_squad(team_id: u32, state: State<AppState>) -> Vec<football::SquadPlayer> {
    football::team_squad_detailed(&mut state.world.lock().unwrap(), team_id)
}

#[tauri::command]
fn market(limit: usize, state: State<AppState>) -> Vec<football::SquadPlayer> {
    football::free_agents_detailed(&mut state.world.lock().unwrap(), limit)
}

/// Build a replayable 2D playback of the managed club's next match (or a friendly if no season
/// is running). Returns null only if there is no opponent in the world.
#[tauri::command]
fn next_match(team_id: u32, state: State<AppState>) -> Option<football::MatchPlayback> {
    football::next_match_playback(&mut state.world.lock().unwrap(), team_id)
}

/// Search the whole player pool, best-overall first, capped. The "Excel over a huge world"
/// surface — filters over every footballer, not just free agents.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
fn search(
    position: Option<u8>,
    min_age: u32,
    max_age: u32,
    min_overall: u8,
    free_only: bool,
    limit: usize,
    state: State<AppState>,
) -> Vec<football::SquadPlayer> {
    let filter = football::SearchFilter { position, min_age, max_age, min_overall, free_only };
    football::search(&mut state.world.lock().unwrap(), filter, limit.clamp(1, 200))
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
    /// Recent form, oldest→newest, as single-char strings ("W"/"D"/"L").
    form: Vec<String>,
}

#[tauri::command]
fn top_scorers(limit: usize, state: State<AppState>) -> Vec<football::ScorerRow> {
    football::top_scorers(&mut state.world.lock().unwrap(), limit.clamp(1, 50))
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
                form: season.form_of(team_id).into_iter().map(String::from).collect(),
            })
            .collect(),
        None => Vec::new(),
    }
}

/// The board's expectation of the managed club and how it's tracking.
#[derive(Serialize)]
struct BoardView {
    /// Human-readable objective, e.g. "Finish in the top 4".
    objective: String,
    /// Target finishing position the board expects.
    target_pos: u32,
    /// Current league position, if a season is running.
    current_pos: Option<u32>,
    /// Board mood word.
    confidence: String,
    on_track: bool,
}

/// The board sets its expectation from the club's squad strength relative to the league (a
/// stronger squad is expected to finish higher) and judges you against the current table.
#[tauri::command]
fn board(team_id: u32, state: State<AppState>) -> BoardView {
    let mut world = state.world.lock().unwrap();
    let clubs = club_views(&mut world);
    let n = clubs.len().max(1);

    // Rank clubs by average squad overall (strongest first); the board expects you to finish at
    // least as high as your squad's strength rank.
    let mut ranked: Vec<(u32, u8)> =
        clubs.iter().filter_map(|c| c.avg_overall.map(|o| (c.team_id, o))).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let strength_rank = ranked.iter().position(|(t, _)| *t == team_id).map_or(n, |i| i + 1);
    let target_pos = strength_rank.clamp(1, n) as u32;

    // Current league position from the standings, if a season is running.
    let current_pos = world.get_resource::<Season>().and_then(|s| {
        s.standings().iter().position(|(t, _)| *t == team_id).map(|i| (i + 1) as u32)
    });

    let on_track = current_pos.map_or(true, |c| c <= target_pos);
    let confidence = match current_pos {
        None => "Pre-season",
        Some(c) if c + 2 <= target_pos => "Delighted",
        Some(c) if c <= target_pos => "Pleased",
        Some(c) if c <= target_pos + 2 => "Concerned",
        Some(_) => "Under pressure",
    }
    .to_string();

    BoardView {
        objective: format!("Finish in the top {target_pos}"),
        target_pos,
        current_pos,
        confidence,
        on_track,
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
    // A new season: zero last year's stats, then a pre-season of training.
    football::reset_tallies(&mut world);
    football::develop(&mut world);
    let today = world.resource::<SimClock>().date();
    let seed = world.get_resource::<SimSeed>().map_or(0, |s| s.0);
    world.insert_resource(Season::new(teams, today, seed, today.year() as u32));
    Ok(())
}

/// Set the managed club's training focus (0 Technical / 1 Physical / 2 Mental, or `None` for
/// balanced) and apply one development step to the world. The UI refreshes to show the change.
#[tauri::command]
fn train_squad(team_id: u32, focus: Option<u8>, state: State<AppState>) {
    let mut world = state.world.lock().unwrap();
    football::set_team_focus(&mut world, team_id, focus);
    football::develop(&mut world);
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
            next_match,
            search,
            current_date,
            season_active,
            standings,
            top_scorers,
            board,
            start_season,
            advance,
            transfer_window,
            train_squad,
            save_game,
            load_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
