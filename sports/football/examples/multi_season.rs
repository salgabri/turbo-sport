//! Proof-of-life for multi-season continuity: a league run for several seasons. Players
//! age and retire (lifecycle), youth are regenerated each off-season, and a fresh season's
//! fixtures are scheduled — all on one continuous clock. Prints the champion each year plus
//! the squad turnover that kept it going.
//!
//! Run: `cargo run -p football --example multi_season --release`

use bevy_ecs::prelude::*;
use football::{gather_lineups, play_due_fixtures, regen_youth, Footballer, Season, TeamId};
use sim_core::{
    age_years, build_daily_schedule, BirthDate, Condition, Date, Morale, Retired, SimClock, SimSeed,
};

fn main() {
    const TEAMS: u32 = 20;
    const SQUAD_TARGET: usize = 16;
    const SEASONS: u32 = 6;
    const WORLD_SEED: u64 = 0xCA7;

    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
    world.insert_resource(SimSeed(WORLD_SEED));

    // Initial squads: ages spread 18..=37 so retirements start almost immediately.
    for t in 0..TEAMS {
        let base = 50 + (t % 20) as i32;
        for p in 0..SQUAD_TARGET {
            let age = 18 + (p % 20) as i32;
            let jitter = (p as i32 % 5) - 2;
            let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
            world.spawn((
                TeamId(t),
                Footballer {
                    pac: rate(base), sho: rate(base + 1), pas: rate(base), dri: rate(base),
                    tec: rate(base + 1), def: rate(base - 2), phy: rate(base - 2), vis: rate(base),
                    gk: rate(base - 2),
                },
                BirthDate(Date::new(2025 - age, 6, 1)),
                Morale(70),
                Condition::fit(),
            ));
        }
    }

    let teams: Vec<u32> = (0..TEAMS).collect();
    let mut daily = build_daily_schedule();
    let mut season_start = Date::new(2025, 8, 9);
    let mut champions: Vec<(u32, u32)> = Vec::new();

    for season_id in 2025..2025 + SEASONS {
        world.insert_resource(Season::new(teams.clone(), season_start, WORLD_SEED, season_id));

        let mut guard = 0;
        while !world.resource::<Season>().is_complete() && guard < 400 {
            daily.run(&mut world);
            play_due_fixtures(&mut world);
            guard += 1;
        }

        if let Some(champ) = world.resource::<Season>().champion() {
            champions.push((season_id, champ));
        }

        // Off-season: regenerate youth to replace retirees, then schedule next August.
        let today = world.resource::<SimClock>().date();
        regen_youth(&mut world, &teams, SQUAD_TARGET, WORLD_SEED, season_id, today);
        season_start = Date::new(today.year(), 8, 9);
    }

    // Report champions per season.
    println!("Champions over {SEASONS} seasons:");
    for (year, champ) in &champions {
        println!("  {year}/{:02}: team {champ}", (year + 1) % 100);
    }

    // Report the resulting population: how lifecycle + regen reshaped it.
    let today = world.resource::<SimClock>().date();
    let total = world.query::<&TeamId>().iter(&world).count();
    let retired = world.query_filtered::<&TeamId, With<Retired>>().iter(&world).count();
    let active = total - retired;
    let ages: Vec<u32> = {
        let mut q = world.query_filtered::<&BirthDate, Without<Retired>>();
        q.iter(&world).map(|b| age_years(b.0, today)).collect()
    };
    let mean_age = ages.iter().sum::<u32>() as f64 / ages.len().max(1) as f64;

    println!(
        "\nAfter {SEASONS} seasons ({}): {total} players ever, {active} active, {retired} retired; mean active age {mean_age:.1}",
        today
    );

    // Sanity: every team still fields a squad.
    let lineups = gather_lineups(&mut world);
    println!("All {} teams still field a lineup: {}", lineups.len(), lineups.len() == TEAMS as usize);
}
