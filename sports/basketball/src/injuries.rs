//! Match injuries for basketball — the court equivalent of `football::injuries`.
//!
//! Each game, the ~10-man rotation for each side carries a small injury chance drawn from a
//! stream seeded off the fixture coordinates (reproducible). An injury sets a recovery time by
//! severity and drops fitness; `sim-core`'s daily `recover_condition` heals it. Reads basketball
//! attributes only to pick the rotation, so the pure engine is untouched.

use crate::attributes::Baller;
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use sim_core::{Condition, PositionGroup, Retired, TeamId};

const INJURY_CHANCE: f64 = 0.02;
const ROTATION: usize = 10;

/// Severity label for a recovery time in days.
pub fn severity(days: u16) -> &'static str {
    match days {
        0 => "Fit",
        1..=6 => "Minor",
        7..=20 => "Moderate",
        _ => "Serious",
    }
}

/// Roll game injuries for both teams after a played fixture, seeded off its coordinates.
pub fn roll_game_injuries(world: &mut World, home: u32, away: u32, seed: u64) {
    for (team, salt) in [(home, 0u64), (away, 1u64)] {
        let mut rot: Vec<(Entity, u8)> = {
            let mut q = world
                .query_filtered::<(Entity, &TeamId, &Baller, Option<&PositionGroup>, &Condition), Without<Retired>>();
            q.iter(world)
                .filter(|(_, t, _, _, c)| t.0 == team && !c.is_injured())
                .map(|(e, _, b, pos, _)| {
                    let position = pos.map_or(crate::attributes::POS_F, |p| p.0);
                    (e, b.overall(position))
                })
                .collect()
        };
        rot.sort_by_key(|p| std::cmp::Reverse(p.1));
        rot.truncate(ROTATION);

        let mut rng = Pcg64Mcg::seed_from_u64(seed ^ salt.wrapping_mul(0x51ED));
        let mut hits: Vec<(Entity, u16)> = Vec::new();
        for (e, _) in &rot {
            if rng.gen_bool(INJURY_CHANCE) {
                hits.push((*e, rng.gen_range(4u16..=24)));
            }
        }
        for (e, days) in hits {
            if let Some(mut c) = world.get_mut::<Condition>(e) {
                c.injury_days = c.injury_days.max(days);
                c.fitness = c.fitness.min(35);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn injuries_are_deterministic_and_bounded() {
        let db = sample();
        let count = || {
            let mut w = load_world(&db);
            for s in 0..50u64 {
                roll_game_injuries(&mut w, db.clubs[0].id, db.clubs[1].id, s.wrapping_mul(0x9E37));
            }
            let mut q = w.query::<&Condition>();
            q.iter(&w).filter(|c| c.is_injured()).count()
        };
        assert_eq!(count(), count());
        assert!(count() > 0);
    }
}
