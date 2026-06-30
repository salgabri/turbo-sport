//! Tauri backend: holds the simulation world and exposes the sim-core read layer as
//! commands the Svelte frontend calls. Only summarized DTOs cross the IPC boundary — never
//! raw entity rows.

use std::sync::Mutex;

use bevy_ecs::prelude::World;
use football::{generate_prospects, Footballer};
use sim_core::{
    club_views, free_agents, squad, sync_squad_membership, BirthDate, ClubBundle, ClubView,
    Condition, Contract, Date, Morale, PlayerView, SimClock, SimSeed,
};
use tauri::State;

/// The live world, behind a mutex so commands can borrow it mutably (queries need `&mut`).
struct AppState {
    world: Mutex<World>,
}

/// Build a small demo league: 20 clubs with finances, contracted squads, a pool of free-agent
/// prospects, and membership synced from contracts.
fn demo_world() -> World {
    const TEAMS: u32 = 20;
    const SQUAD: u32 = 18;
    const SEED: u64 = 0xC1B;

    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(Date::new(2026, 1, 15)));
    world.insert_resource(SimSeed(SEED));

    let mut clubs = Vec::new();
    for t in 0..TEAMS {
        clubs.push(world.spawn(ClubBundle::new(t, 50_000_000, 500_000, SQUAD)).id());
    }
    for (t, &club) in clubs.iter().enumerate() {
        let base = 50 + (t as i32 % 20);
        for p in 0..SQUAD {
            let age = 18 + (p % 18) as i32;
            let jitter = (p as i32 % 5) - 2;
            let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
            let injury = if p % 9 == 0 { 14 } else { 0 };
            world.spawn((
                Footballer {
                    attacking: rate(base),
                    defending: rate(base - 2),
                    finishing: rate(base + 1),
                    goalkeeping: rate(base - 3),
                },
                BirthDate(Date::new(2026 - age, 6, 1)),
                Morale(70),
                Condition { fitness: if injury > 0 { 60 } else { 95 }, injury_days: injury },
                Contract {
                    club,
                    until: Date::new(2028 + (p % 3) as i32, 6, 30),
                    wage: 1_000 + (p as u32 % 5) * 250,
                },
            ));
        }
    }

    // A pool of free agents so the transfer-market view has content.
    generate_prospects(&mut world, 40, SEED, 2026, Date::new(2026, 1, 15));

    sync_squad_membership(&mut world);
    world
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
        .manage(AppState { world: Mutex::new(demo_world()) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![clubs, team_squad, market])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
