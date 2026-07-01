//! The full composition: clubs as real entities with finances, players on contracts, the
//! transfer market moving free agents between clubs each off-season, squad membership
//! following the contracts, and the league playing out — for several seasons.
//!
//! Everything wired together: sim-core's Club model + economy + transfers + lifecycle +
//! league driver, with football's engine on top.
//!
//! Run: `cargo run -p football --example club_world --release`

use bevy_ecs::prelude::*;
use football::{generate_prospects, play_due_fixtures, Footballer, Season};
use sim_core::{
    build_daily_schedule, index_clubs, run_transfer_window, sync_squad_membership, Balance,
    BirthDate, ClubBundle, Condition, Contract, Date, FreeAgent, Morale, SimClock, SimSeed, TeamId,
};

fn main() {
    const TEAMS: u32 = 20;
    const SQUAD: u32 = 18;
    const SEASONS: u32 = 6;
    const WORLD_SEED: u64 = 0xC1B;

    let mut world = World::new();
    world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
    world.insert_resource(SimSeed(WORLD_SEED));

    // Clubs as entities: identity + finances + squad target.
    let mut club_entities = Vec::new();
    for t in 0..TEAMS {
        let id = world
            .spawn(ClubBundle::new(t, 50_000_000, 500_000, SQUAD))
            .id();
        club_entities.push(id);
    }

    // Initial squads: each player is on a contract to their club (no raw TeamId — sync derives
    // it). Ages spread so players retire over time; contracts expire across the next 3 summers
    // so free agents reach the market each year.
    for (t, &club) in club_entities.iter().enumerate() {
        let base = 50 + (t as i32 % 20);
        for p in 0..SQUAD {
            let age = 18 + (p % 18) as i32;
            let jitter = (p as i32 % 5) - 2;
            let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
            let expiry_year = 2026 + (p % 3) as i32;
            world.spawn((
                Footballer {
                    pac: rate(base), sho: rate(base + 1), pas: rate(base), dri: rate(base),
                    tec: rate(base + 1), def: rate(base - 2), phy: rate(base - 2), vis: rate(base),
                    gk: rate(base - 2),
                },
                BirthDate(Date::new(2025 - age, 6, 1)),
                Morale(70),
                Condition::fit(),
                Contract { club, until: Date::new(expiry_year, 6, 30), wage: 1_000 },
            ));
        }
    }

    // Derive each player's TeamId from their contract, once, before kickoff.
    sync_squad_membership(&mut world);

    let teams: Vec<u32> = (0..TEAMS).collect();
    let mut daily = build_daily_schedule();
    let mut champions: Vec<(u32, u32)> = Vec::new();

    for season_id in 2025..2025 + SEASONS {
        world.insert_resource(Season::new(teams.clone(), Date::new(season_id as i32, 8, 9), WORLD_SEED, season_id));

        let mut guard = 0;
        while !world.resource::<Season>().is_complete() && guard < 400 {
            daily.run(&mut world); // ages players, expires contracts -> free agents, pays wages
            play_due_fixtures(&mut world);
            guard += 1;
        }
        if let Some(champ) = world.resource::<Season>().champion() {
            champions.push((season_id, champ));
        }
        world.remove_resource::<Season>();

        // Off-season: advance through summer so June contract expiries are processed...
        let target = Date::new(world.resource::<SimClock>().date().year(), 7, 15);
        while world.resource::<SimClock>().date() < target {
            daily.run(&mut world);
        }
        // ...then bring youth to market, let clubs sign, and update squad membership.
        let today = world.resource::<SimClock>().date();
        let free_before = count_free_agents(&mut world);
        generate_prospects(&mut world, TEAMS * 4, WORLD_SEED, season_id, today);
        run_transfer_window(&mut world);
        sync_squad_membership(&mut world);
        let signed = (free_before + TEAMS * 4) - count_free_agents(&mut world);
        println!("  off-season {today}: {signed} free agents signed");
    }

    println!("\nChampions:");
    for (year, champ) in &champions {
        println!("  {year}/{:02}: team {champ}", (year + 1) % 100);
    }

    // Finances moved: show the richest and poorest clubs.
    let mut balances: Vec<(u32, i64)> = club_entities
        .iter()
        .map(|&c| {
            let team = world.entity(c).get::<TeamId>().unwrap().0;
            let bal = world.entity(c).get::<Balance>().unwrap().0;
            (team, bal)
        })
        .collect();
    balances.sort_by_key(|&(_, b)| std::cmp::Reverse(b));
    println!("\nClub balances after {SEASONS} seasons (paying wages each week):");
    println!("  richest: team {} = {}", balances[0].0, balances[0].1);
    println!("  poorest: team {} = {}", balances.last().unwrap().0, balances.last().unwrap().1);

    // Every club still fields a squad, fed entirely by the transfer market.
    let registry = index_clubs(&mut world);
    let mut squad_sizes: Vec<usize> = vec![0; TEAMS as usize];
    for t in world.query_filtered::<&TeamId, Without<sim_core::Club>>().iter(&world) {
        squad_sizes[t.0 as usize] += 1;
    }
    let viable = squad_sizes.iter().all(|&n| n > 0);
    println!(
        "\n{} clubs in registry; every club fields a squad: {viable} (squad sizes {}..{})",
        registry.0.len(),
        squad_sizes.iter().min().unwrap(),
        squad_sizes.iter().max().unwrap(),
    );
}

fn count_free_agents(world: &mut World) -> u32 {
    world.query_filtered::<(), With<FreeAgent>>().iter(world).count() as u32
}
