//! Scale & speed spike for the core thesis: build a **100k-player** world (hundreds of
//! divisions), then time the hot paths — a full parallel matchday, a whole season, and one
//! daily lifecycle tick over every entity.
//!
//! This is the load-bearing product claim (simulate a far larger world, dramatically faster),
//! so it needs a number, not a promise. Run in release for a meaningful figure:
//!
//!   cargo run -p football --example scale --release
//!   TURBO_SCALE_PLAYERS=250000 cargo run -p football --example scale --release
//!
//! Determinism holds at any scale: each match is seeded from `(world_seed, season, matchday,
//! index)` with no shared RNG across the rayon batch.

use football::database::{Database, FootballAbility};
use football::matchday::Fixture;
use football::{gather_lineups, load_world, simulate_matchday};
use sim_core::database::{ClubRecord, DbDate, DivisionRecord, PersonRecord};
use sim_core::{build_daily_schedule, single_round_robin};
use std::time::Instant;

const SQUAD: u32 = 25;
const TEAMS_PER_DIV: u32 = 20;

/// Procedurally build a big football database: `divisions` divisions of 20 clubs, each with a
/// `SQUAD`-man roster. Deterministic — no RNG, just spread-out ratings.
fn big_db(divisions: u32) -> Database {
    let teams = divisions * TEAMS_PER_DIV;
    let start = DbDate { year: 2025, month: 7, day: 1 };
    let mut clubs = Vec::with_capacity(teams as usize);
    let mut players = Vec::with_capacity((teams * SQUAD) as usize);
    let mut div_recs = Vec::with_capacity(divisions as usize);

    for d in 0..divisions {
        let mut ids = Vec::with_capacity(TEAMS_PER_DIV as usize);
        for t in 0..TEAMS_PER_DIV {
            let id = d * TEAMS_PER_DIV + t;
            let strength = 45 + ((id * 7) % 40) as i32 - (d as i32 % 10);
            clubs.push(ClubRecord {
                id,
                name: format!("Club {id}"),
                balance: 10_000_000,
                weekly_income: 200_000,
                squad_target: SQUAD,
            });
            for p in 0..SQUAD {
                let base = (strength + ((p * 5 + id * 3) % 22) as i32 - 10).clamp(30, 95) as u8;
                let position = (p % 4) as u8; // 0 GK .. 3 FWD, cheap spread
                let age = 17 + ((p * 3 + id) % 18) as i32;
                players.push(PersonRecord {
                    name: format!("P{id}-{p}"),
                    club_id: Some(id),
                    birth: DbDate { year: 2025 - age, month: 1 + (p % 12) as u8, day: 1 + (p % 28) as u8 },
                    wage: 1_000 + u32::from(base) * 40,
                    contract_until: DbDate { year: 2027 + (p % 3) as i32, month: 6, day: 30 },
                    ability: FootballAbility {
                        pac: base,
                        sho: base.saturating_sub(2),
                        pas: base,
                        dri: base,
                        tec: base.saturating_sub(1),
                        def: base.saturating_sub(3),
                        phy: base,
                        vis: base.saturating_sub(2),
                        gk: base.saturating_sub(10),
                        position,
                        potential: base.saturating_add(4).min(99),
                        nationality: "ENG".into(),
                    },
                });
            }
            ids.push(id);
        }
        div_recs.push(DivisionRecord { name: format!("Division {d}"), tier: d + 1, club_ids: ids });
    }

    Database { name: "Scale Test".into(), start_date: start, seed: 0x5CA1E, divisions: div_recs, clubs, players }
}

fn main() {
    let target: u32 = std::env::var("TURBO_SCALE_PLAYERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    // Round the player target up to whole divisions of 20 clubs x SQUAD players.
    let per_div = TEAMS_PER_DIV * SQUAD;
    let divisions = target.div_ceil(per_div).max(1);
    let teams = divisions * TEAMS_PER_DIV;
    let players = teams * SQUAD;

    println!("== Turbo Sport — scale & speed spike ==");
    println!("target players: {target} -> {divisions} divisions x {TEAMS_PER_DIV} clubs x {SQUAD} = {players} players ({teams} clubs)\n");

    // --- build the world ---
    let t = Instant::now();
    let db = big_db(divisions);
    let build_db = t.elapsed();

    let t = Instant::now();
    let mut world = load_world(&db);
    let build_world = t.elapsed();
    let entities = world.iter_entities().count();

    // --- read the lineups once (single-threaded gather) ---
    let t = Instant::now();
    let lineups = gather_lineups(&mut world);
    let gather = t.elapsed();

    // --- schedule: a single round-robin per division; combine each round across all divisions ---
    let rounds_per_div: Vec<Vec<Vec<(u32, u32)>>> = (0..divisions)
        .map(|d| {
            let ids: Vec<u32> = (0..TEAMS_PER_DIV).map(|t| d * TEAMS_PER_DIV + t).collect();
            single_round_robin(&ids)
        })
        .collect();
    let n_rounds = rounds_per_div[0].len();

    let fixtures_for_round = |m: usize| -> Vec<Fixture> {
        rounds_per_div
            .iter()
            .flat_map(|div| div[m].iter())
            .map(|&(h, a)| Fixture { home: lineups[&h], away: lineups[&a] })
            .collect()
    };

    // --- one full matchday across every division, in parallel ---
    let md0 = fixtures_for_round(0);
    let matches_per_day = md0.len();
    let t = Instant::now();
    let r0 = simulate_matchday(&md0, db.seed, 2025, 0);
    let one_matchday = t.elapsed();
    let md0_goals: u32 = r0.iter().map(|r| u32::from(r.home_goals) + u32::from(r.away_goals)).sum();

    // --- a whole season (every round) ---
    let t = Instant::now();
    let mut season_goals: u64 = 0;
    let mut season_matches: u64 = 0;
    for m in 0..n_rounds {
        let fx = fixtures_for_round(m);
        let res = simulate_matchday(&fx, db.seed, 2025, m as u32);
        season_matches += res.len() as u64;
        season_goals += res.iter().map(|r| u64::from(r.home_goals) + u64::from(r.away_goals)).sum::<u64>();
    }
    let full_season = t.elapsed();

    // --- one daily lifecycle tick over the whole world ---
    let mut daily = build_daily_schedule();
    let t = Instant::now();
    daily.run(&mut world);
    let one_tick = t.elapsed();

    // --- report ---
    let per_s = |n: u64, d: std::time::Duration| {
        let s = d.as_secs_f64();
        if s > 0.0 { (n as f64 / s) as u64 } else { 0 }
    };
    println!("world");
    println!("  build database        {build_db:>10.2?}");
    println!("  load into ECS         {build_world:>10.2?}   ({entities} entities)");
    println!("  gather all lineups    {gather:>10.2?}\n");
    println!("match simulation (rayon across all divisions at once)");
    println!("  one matchday          {one_matchday:>10.2?}   {matches_per_day} matches  ({} matches/s, {md0_goals} goals)", per_s(matches_per_day as u64, one_matchday));
    println!("  full season           {full_season:>10.2?}   {season_matches} matches over {n_rounds} rounds  ({} matches/s, {season_goals} goals)", per_s(season_matches, full_season));
    println!("\nlifecycle");
    println!("  one daily tick        {one_tick:>10.2?}   ({} entities/s over aging/contracts/wages)", per_s(entities as u64, one_tick));

    // Sanity: re-running a matchday gives an identical result (determinism at scale).
    let again = simulate_matchday(&md0, db.seed, 2025, 0);
    assert_eq!(again, r0, "matchday must be identical on re-run (determinism)");
    println!("\ndeterminism: matchday re-run identical — OK");
}
