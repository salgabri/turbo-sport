//! Match injuries: a deterministic chance that a player who featured picks up a knock.
//!
//! `sim-core` already models [`Condition`] (match fitness + days until recovered) and heals it
//! daily; nothing *inflicted* injuries until now. Here each matchday, the ~16 players who
//! featured for each side each carry a small injury chance, drawn from a stream seeded off the
//! fixture's coordinates so the injury list is as reproducible as the scoreline. An injury drops
//! the player's fitness and sets a recovery time by severity; the sport reads football
//! attributes only to pick who featured, so the pure engine is untouched.

use crate::attributes::Footballer;
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use sim_core::{Condition, PositionGroup, Retired, TeamId};

/// Per-player, per-match probability of a fresh injury.
const INJURY_CHANCE: f64 = 0.015;
/// Roughly the number of players who feature (starters + subs).
const FEATURED: usize = 16;

/// Severity label for a recovery time in days.
pub fn severity(days: u16) -> &'static str {
    match days {
        0 => "Fit",
        1..=6 => "Minor",
        7..=20 => "Moderate",
        _ => "Serious",
    }
}

/// Roll match injuries for both teams after a played fixture. `seed` should come from the
/// fixture's match coordinates so a save reproduces the same injuries.
pub fn roll_match_injuries(world: &mut World, home: u32, away: u32, seed: u64) {
    for (team, salt) in [(home, 0u64), (away, 1u64)] {
        // The players who featured: the highest-rated, currently-available squad members.
        let mut featured: Vec<(Entity, u8)> = {
            let mut q = world
                .query_filtered::<(Entity, &TeamId, &Footballer, Option<&PositionGroup>, &Condition), Without<Retired>>();
            q.iter(world)
                .filter(|(_, t, _, _, c)| t.0 == team && !c.is_injured())
                .map(|(e, _, f, pos, _)| {
                    let position = pos.map_or(crate::attributes::POS_MID, |p| p.0);
                    (e, f.overall(position))
                })
                .collect()
        };
        featured.sort_by_key(|p| std::cmp::Reverse(p.1));
        featured.truncate(FEATURED);

        let mut rng = Pcg64Mcg::seed_from_u64(seed ^ salt.wrapping_mul(0x51ED));
        let mut hits: Vec<(Entity, u16)> = Vec::new();
        for (e, _) in &featured {
            if rng.gen_bool(INJURY_CHANCE) {
                hits.push((*e, rng.gen_range(4u16..=28)));
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
            // Force injuries by rolling many "matches" with different seeds.
            for s in 0..40u64 {
                roll_match_injuries(&mut w, db.clubs[0].id, db.clubs[1].id, s.wrapping_mul(0x9E37));
            }
            let mut q = w.query::<&Condition>();
            q.iter(&w).filter(|c| c.is_injured()).count()
        };
        let a = count();
        let b = count();
        assert_eq!(a, b, "same seeds -> same injuries");
        assert!(a > 0, "some injuries should occur over 40 matchdays");
    }

    #[test]
    fn severity_labels_scale_with_days() {
        assert_eq!(severity(0), "Fit");
        assert_eq!(severity(3), "Minor");
        assert_eq!(severity(14), "Moderate");
        assert_eq!(severity(25), "Serious");
    }
}
