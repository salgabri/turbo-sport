//! Multi-season continuity: youth regeneration.
//!
//! Over many seasons the lifecycle systems age players and retire them at ~38; without new
//! blood every squad would eventually empty out. `regen_youth` is the off-season intake: it
//! tops each team back up to a target size with 16–18-year-olds, generated deterministically
//! so a save reproduces the same prospects.
//!
//! Football-specific because the new players need football abilities. The *generic* parts of
//! a person (birth date, condition, morale) come from `sim-core`; only the `Footballer`
//! attributes are added here — the same split the schema has everywhere.

use crate::attributes::{Footballer, TeamId};
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use sim_core::{derive_seed, BirthDate, Condition, Date, Morale, Retired};
use std::collections::HashMap;

/// Top every team up to `target` active (non-retired) players with newly generated youth.
///
/// New players are 16–18 years old with modest abilities (40–65). Each intake slot is seeded
/// from `(season_id, team, slot)`, so the regeneration is fully deterministic. `today` dates
/// the birthdays relative to the current calendar.
pub fn regen_youth(
    world: &mut World,
    teams: &[u32],
    target: usize,
    world_seed: u64,
    season_id: u32,
    today: Date,
) {
    // Count active (non-retired) players already on each team.
    let mut active: HashMap<u32, usize> = HashMap::new();
    {
        let mut q = world.query_filtered::<&TeamId, Without<Retired>>();
        for t in q.iter(world) {
            *active.entry(t.0).or_default() += 1;
        }
    }

    for &team in teams {
        let have = active.get(&team).copied().unwrap_or(0);
        for slot in have..target {
            let seed = derive_seed(world_seed, &[u64::from(season_id), u64::from(team), slot as u64]);
            let mut rng = Pcg64Mcg::seed_from_u64(seed);
            let age = rng.gen_range(16i32..=18);
            let birth = Date::new(
                today.year() - age,
                1 + rng.gen_range(0u8..12),
                1 + rng.gen_range(0u8..28),
            );
            let ability = Footballer {
                attacking: rng.gen_range(40u8..=65),
                defending: rng.gen_range(40u8..=65),
                finishing: rng.gen_range(40u8..=65),
                goalkeeping: rng.gen_range(40u8..=65),
            };
            world.spawn((TeamId(team), ability, BirthDate(birth), Morale(70), Condition::fit()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_count(world: &mut World, team: u32) -> usize {
        let mut q = world.query_filtered::<&TeamId, Without<Retired>>();
        q.iter(world).filter(|t| t.0 == team).count()
    }

    #[test]
    fn tops_each_team_up_to_target() {
        let mut world = World::new();
        // Team 0 starts with 2 players, team 1 with none.
        world.spawn((TeamId(0), Morale(70)));
        world.spawn((TeamId(0), Morale(70)));

        regen_youth(&mut world, &[0, 1], 5, 42, 2025, Date::new(2026, 7, 1));

        assert_eq!(active_count(&mut world, 0), 5);
        assert_eq!(active_count(&mut world, 1), 5);
    }

    #[test]
    fn retired_players_do_not_count_toward_the_target() {
        let mut world = World::new();
        world.spawn((TeamId(0), Morale(70))); // 1 active
        world.spawn((TeamId(0), Morale(70), Retired)); // retired, ignored

        regen_youth(&mut world, &[0], 4, 1, 2025, Date::new(2026, 7, 1));

        // Topped up to 4 *active*; the retired one is extra.
        assert_eq!(active_count(&mut world, 0), 4);
    }

    #[test]
    fn is_deterministic() {
        let build = || {
            let mut world = World::new();
            regen_youth(&mut world, &[0], 3, 7, 2025, Date::new(2026, 7, 1));
            let mut q = world.query::<&Footballer>();
            let mut abilities: Vec<(u8, u8, u8, u8)> =
                q.iter(&world).map(|f| (f.attacking, f.defending, f.finishing, f.goalkeeping)).collect();
            abilities.sort_unstable();
            abilities
        };
        assert_eq!(build(), build());
    }
}
