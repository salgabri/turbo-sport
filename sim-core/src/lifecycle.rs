//! Lifecycle systems: the daily processes that age people, end their contracts, and
//! heal them. All sport-agnostic and all operating generically over the components in
//! [`crate::entity`].
//!
//! Determinism (see `docs/ARCHITECTURE.md`): every system here is a pure function of the
//! world state and the clock — no randomness. The *random* parts of a person's life
//! (does an injury happen in this match? does this 35-year-old retire early?) are driven
//! by events with derived RNG streams at later build steps. Today's retirement rule is a
//! deterministic age threshold, deliberately a placeholder for that.

use crate::entity::{age_years, BirthDate, Condition, Contract, FreeAgent, Retired, WageDemand};
use crate::time::SimClock;
use bevy_ecs::prelude::*;

/// Age at which a person retires under the current placeholder rule. Will become a
/// probabilistic, attribute-aware decision once RNG streams exist (build step 5).
pub const RETIREMENT_AGE: u32 = 38;

/// How much match fitness recovers per rest day, in points.
const FITNESS_RECOVERY_PER_DAY: u8 = 5;

/// Heal one day's worth: tick down injuries, and regenerate fitness on days a person is
/// not injured. Deterministic — injury *occurrence* is match-driven (step 5); recovery
/// is not random. Retired people are skipped.
pub fn recover_condition(mut q: Query<&mut Condition, Without<Retired>>) {
    for mut c in &mut q {
        if c.injury_days > 0 {
            c.injury_days -= 1;
        } else if c.fitness < 100 {
            c.fitness = c.fitness.saturating_add(FITNESS_RECOVERY_PER_DAY).min(100);
        }
    }
}

/// End contracts whose final day has passed: drop the [`Contract`] and mark the person a
/// [`FreeAgent`]. Wages are not touched here — payroll is the economy's concern.
pub fn expire_contracts(clock: Res<SimClock>, mut commands: Commands, q: Query<(Entity, &Contract)>) {
    let today = clock.date();
    for (e, c) in &q {
        if today > c.until {
            // Carry the old wage over as the free agent's asking price for the transfer
            // market.
            commands
                .entity(e)
                .remove::<Contract>()
                .insert(FreeAgent)
                .insert(WageDemand(c.wage));
        }
    }
}

/// On a person's birthday, retire those who have reached [`RETIREMENT_AGE`]. Checking
/// only on the birthday keeps this O(birthdays/day) in effect rather than rescanning
/// everyone's age every day. A retiree also loses any contract and becomes a free agent.
pub fn age_and_retire(
    clock: Res<SimClock>,
    mut commands: Commands,
    q: Query<(Entity, &BirthDate), Without<Retired>>,
) {
    let today = clock.date();
    for (e, birth) in &q {
        let is_birthday = birth.0.month() == today.month() && birth.0.day() == today.day();
        if is_birthday && age_years(birth.0, today) >= RETIREMENT_AGE {
            commands
                .entity(e)
                .insert(Retired)
                .remove::<Contract>()
                .insert(FreeAgent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Morale, PersonBundle};
    use crate::schedule::build_daily_schedule;
    use crate::time::Date;

    /// Build a world starting on `start` with the daily schedule ready.
    fn world_on(start: Date) -> (World, Schedule) {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(start));
        (world, build_daily_schedule())
    }

    fn run_days(world: &mut World, schedule: &mut Schedule, days: u32) {
        for _ in 0..days {
            schedule.run(world);
        }
    }

    #[test]
    fn contract_expires_into_free_agency() {
        let (mut world, mut sched) = world_on(Date::new(2025, 7, 1));
        let club = world.spawn_empty().id();
        let person = world
            .spawn((
                BirthDate(Date::new(2000, 3, 10)),
                Morale(70),
                Condition::fit(),
                Contract { club, until: Date::new(2025, 7, 3), wage: 1000 },
            ))
            .id();

        run_days(&mut world, &mut sched, 2); // now 2025-07-03, still valid (until inclusive)
        assert!(world.get::<Contract>(person).is_some());
        assert!(world.get::<FreeAgent>(person).is_none());

        run_days(&mut world, &mut sched, 1); // now 2025-07-04, past `until`
        assert!(world.get::<Contract>(person).is_none());
        assert!(world.get::<FreeAgent>(person).is_some());
    }

    #[test]
    fn injury_recovers_then_fitness_regenerates() {
        let (mut world, mut sched) = world_on(Date::new(2025, 7, 1));
        let person = world
            .spawn(PersonBundle {
                birth: BirthDate(Date::new(2000, 1, 1)),
                morale: Morale(70),
                condition: Condition { fitness: 50, injury_days: 3 },
            })
            .id();

        run_days(&mut world, &mut sched, 3); // injury_days 3 -> 0
        assert_eq!(world.get::<Condition>(person).unwrap().injury_days, 0);
        assert_eq!(world.get::<Condition>(person).unwrap().fitness, 50); // no regen while healing

        run_days(&mut world, &mut sched, 2); // two rest days: +5, +5
        assert_eq!(world.get::<Condition>(person).unwrap().fitness, 60);
    }

    #[test]
    fn retires_on_birthday_at_threshold() {
        // Born 1987-08-15; on 2025-08-15 they turn 38 == RETIREMENT_AGE.
        let (mut world, mut sched) = world_on(Date::new(2025, 8, 14));
        let club = world.spawn_empty().id();
        let person = world
            .spawn((
                BirthDate(Date::new(1987, 8, 15)),
                Morale(70),
                Condition::fit(),
                Contract { club, until: Date::new(2030, 6, 30), wage: 1000 },
            ))
            .id();

        run_days(&mut world, &mut sched, 1); // 2025-08-15, the birthday
        assert!(world.get::<Retired>(person).is_some());
        assert!(world.get::<Contract>(person).is_none());
        assert!(world.get::<FreeAgent>(person).is_some());
    }
}
