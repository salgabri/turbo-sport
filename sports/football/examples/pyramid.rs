//! Promotion and relegation across a 3-division pyramid. Teams' strength is deliberately
//! uncorrelated with their starting division, so a working promotion/relegation system should
//! sort the strong teams up into the top flight over several seasons.
//!
//! Run: `cargo run -p football --example pyramid --release`

use bevy_ecs::prelude::*;
use football::{rank_division, Footballer, TeamId};
use sim_core::Pyramid;

fn main() {
    const DIVISIONS: usize = 3;
    const PER_DIVISION: u32 = 8;
    const TEAMS: u32 = DIVISIONS as u32 * PER_DIVISION; // 24
    const SEASONS: u32 = 12;
    const SWAP: usize = 2; // up/down per division boundary
    const SEED: u64 = 0x9F1D;

    // Each team's quality is a permutation of the field — strength is NOT tied to where a team
    // starts, so the pyramid has to discover the real order.
    let quality: Vec<i32> = (0..TEAMS).map(|t| 36 + ((t * 7 + 3) % TEAMS) as i32 * 2).collect();
    let best_team = (0..TEAMS).max_by_key(|&t| quality[t as usize]).unwrap();

    let mut world = World::new();
    for t in 0..TEAMS {
        let base = quality[t as usize];
        for p in 0..11 {
            let jitter = (p % 5) - 2;
            let rate = |x: i32| (x + jitter).clamp(1, 99) as u8;
            world.spawn((
                TeamId(t),
                Footballer {
                    pac: rate(base), sho: rate(base + 1), pas: rate(base), dri: rate(base),
                    tec: rate(base + 1), def: rate(base - 2), phy: rate(base - 2), vis: rate(base),
                    gk: rate(base - 2),
                },
            ));
        }
    }

    // Start with teams assigned to divisions in id order (uncorrelated with quality).
    let mut pyramid = Pyramid::new(vec![
        (0..8).collect(),
        (8..16).collect(),
        (16..24).collect(),
    ]);

    print!("Top team is {best_team} (quality {}); its division by season: ", quality[best_team as usize]);
    print!("{}", pyramid.division_of(best_team).unwrap() + 1);

    for season in 0..SEASONS {
        let mut orders: Vec<Vec<u32>> = Vec::with_capacity(DIVISIONS);
        for d in 0..pyramid.len() {
            let teams = pyramid.divisions[d].clone();
            orders.push(rank_division(&mut world, &teams, SEED, season * DIVISIONS as u32 + d as u32));
        }
        pyramid.promote_relegate(&orders, SWAP);
        print!(" -> {}", pyramid.division_of(best_team).unwrap() + 1);
    }
    println!();

    println!("\nFinal divisions after {SEASONS} seasons (by avg team quality):");
    for (d, div) in pyramid.divisions.iter().enumerate() {
        let mut teams = div.clone();
        teams.sort_unstable();
        let avg: f64 = div.iter().map(|&t| f64::from(quality[t as usize])).sum::<f64>() / div.len() as f64;
        println!("  Division {}: avg quality {avg:.1}  teams {teams:?}", d + 1);
    }

    // The top flight should now hold the strongest teams.
    let avg_top = pyramid.divisions[0].iter().map(|&t| f64::from(quality[t as usize])).sum::<f64>() / PER_DIVISION as f64;
    let avg_bottom = pyramid.divisions[DIVISIONS - 1].iter().map(|&t| f64::from(quality[t as usize])).sum::<f64>() / PER_DIVISION as f64;
    println!("\nTop flight avg quality {avg_top:.1} vs bottom {avg_bottom:.1} — sorted by merit: {}", avg_top > avg_bottom);
}
