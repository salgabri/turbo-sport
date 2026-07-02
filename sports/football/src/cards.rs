//! Bookings & suspensions: yellow cards accumulate into a ban.
//!
//! Each matchday, players who featured pick up the odd booking (deterministic, stream seeded off
//! the fixture coordinates). Reach [`YELLOW_BAN`] yellows and a one-match ban is triggered (the
//! tally resets); a rare straight red bans for two. A banned player's count winds down one match
//! at a time. Mirrors `injuries` — a deterministic step in the season play path, engine
//! untouched.

use crate::attributes::Footballer;
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use sim_core::{Retired, TeamId};

/// Yellow cards that trigger a one-match ban.
pub const YELLOW_BAN: u8 = 5;
const YELLOW_CHANCE: f64 = 0.10;
const RED_CHANCE: f64 = 0.01;
const FEATURED: usize = 16;

/// Booking record for a player this season.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cards {
    pub yellows: u8,
    /// Matches still to be served.
    pub ban: u8,
}

/// Advance one matchday for a team: banned players serve a match, then the players who featured
/// risk fresh bookings. Seeded off the fixture coordinates.
pub fn roll_match_cards(world: &mut World, home: u32, away: u32, seed: u64) {
    for (team, salt) in [(home, 0u64), (away, 1u64)] {
        // Serve outstanding bans (one match each).
        let banned: Vec<Entity> = {
            let mut q = world.query::<(Entity, &TeamId, &Cards)>();
            q.iter(world).filter(|(_, t, c)| t.0 == team && c.ban > 0).map(|(e, ..)| e).collect()
        };
        for e in banned {
            if let Some(mut c) = world.get_mut::<Cards>(e) {
                c.ban = c.ban.saturating_sub(1);
            }
        }

        // The players who featured (highest-rated available), then roll bookings.
        let mut featured: Vec<Entity> = {
            let mut q = world
                .query_filtered::<(Entity, &TeamId, &Footballer, Option<&Cards>), Without<Retired>>();
            let mut v: Vec<(Entity, u8)> = q
                .iter(world)
                .filter(|(_, t, _, c)| t.0 == team && c.map_or(0, |c| c.ban) == 0)
                .map(|(e, _, f, _)| (e, f.overall(crate::attributes::POS_MID)))
                .collect();
            v.sort_by_key(|p| std::cmp::Reverse(p.1));
            v.truncate(FEATURED);
            v.into_iter().map(|(e, _)| e).collect()
        };

        let mut rng = Pcg64Mcg::seed_from_u64(seed ^ salt.wrapping_mul(0xCA5D));
        let bookings: Vec<(Entity, bool)> = featured
            .drain(..)
            .filter_map(|e| {
                if rng.gen_bool(RED_CHANCE) {
                    Some((e, true)) // red
                } else if rng.gen_bool(YELLOW_CHANCE) {
                    Some((e, false)) // yellow
                } else {
                    None
                }
            })
            .collect();

        for (e, red) in bookings {
            let mut em = world.entity_mut(e);
            let mut c = em.get_mut::<Cards>().map(|c| *c).unwrap_or_default();
            if red {
                c.ban = c.ban.max(2);
            } else {
                c.yellows += 1;
                if c.yellows >= YELLOW_BAN {
                    c.yellows -= YELLOW_BAN;
                    c.ban = c.ban.max(1);
                }
            }
            em.insert(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn bookings_accumulate_and_bans_are_deterministic() {
        let db = sample();
        let bans = || {
            let mut w = load_world(&db);
            for s in 0..40u64 {
                roll_match_cards(&mut w, db.clubs[0].id, db.clubs[1].id, s.wrapping_mul(0x1234));
            }
            let mut q = w.query::<&Cards>();
            (
                q.iter(&w).filter(|c| c.ban > 0).count(),
                q.iter(&w).map(|c| u32::from(c.yellows)).sum::<u32>(),
            )
        };
        let a = bans();
        assert_eq!(a, bans(), "same seeds -> same bookings");
        assert!(a.1 > 0, "yellows accumulate over a season");
    }
}
