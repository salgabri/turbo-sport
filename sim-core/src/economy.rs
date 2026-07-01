//! Sport-agnostic economy basics: club balances, recurring income, and weekly payroll.
//! Build step 4.
//!
//! Scope line (see `CLAUDE.md`): finances and wages only. The transfer market,
//! sponsorship deals, prize money, and detailed budgeting are deliberately absent —
//! they get built once football actually exercises them, not speculatively now. This is
//! where the `wage` recorded on [`crate::entity::Contract`] finally gets *spent*; the
//! contract module stores it, the economy module pays it.

use crate::entity::Contract;
use crate::time::SimClock;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Money, in whole game units — a sim has no need for sub-units (cents). Signed, because
/// running into debt is a normal pressure in this genre, not an error state.
pub type Money = i64;

/// A club's cash balance.
#[derive(Component, Clone, Copy, Debug)]
pub struct Balance(pub Money);

/// Flat recurring weekly income (gate receipts, sponsorship, broadcast — lumped into one
/// number for now). Itemized revenue is a later refinement, not a step-4 concern.
#[derive(Component, Clone, Copy, Debug)]
pub struct WeeklyIncome(pub Money);

/// A player's estimated market/transfer value. Derived deterministically from rating + age via
/// [`value_from`] at spawn (no RNG, so it never depends on scheduling); a later system may
/// revalue it as ratings and age move.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarketValue(pub Money);

/// A deterministic, illustrative market value from an `overall` rating (0..=99) and `age`.
///
/// The curve grows steeply with overall (cubic) and peaks in the low-to-mid twenties, tapering
/// for older players — the usual shape in this genre. Pure: same inputs → same value, always.
pub fn value_from(overall: u8, age: u32) -> Money {
    let t = ((f64::from(overall) - 45.0).max(0.0)) / 45.0; // 0 at 45 ovr, 1 at 90 ovr
    let base = t.powi(3) * 50_000_000.0;
    let age_factor = match age {
        0..=20 => 0.85,
        21..=26 => 1.0,
        27..=30 => 0.8,
        31..=33 => 0.5,
        _ => 0.25,
    };
    (base * age_factor) as Money
}

/// Days between paydays on the fixed calendar.
const PAY_PERIOD_DAYS: u64 = 7;

/// Once a week, credit each club its income and debit its wage bill — the sum of the
/// wages of every player currently contracted to it.
///
/// Deterministic: wages are summed per club into a map keyed by club entity, then
/// applied to each club by key lookup. Nothing here iterates the map's (unordered)
/// entries or uses randomness, so the result is independent of hashing and scheduling.
pub fn weekly_finances(
    clock: Res<SimClock>,
    contracts: Query<&Contract>,
    mut clubs: Query<(Entity, &mut Balance, Option<&WeeklyIncome>)>,
) {
    let day = clock.day_index();
    if day == 0 || !day.is_multiple_of(PAY_PERIOD_DAYS) {
        return;
    }

    let mut wage_bill: HashMap<Entity, Money> = HashMap::new();
    for c in &contracts {
        *wage_bill.entry(c.club).or_default() += Money::from(c.wage);
    }

    for (club, mut balance, income) in &mut clubs {
        let credit = income.map_or(0, |i| i.0);
        let debit = wage_bill.get(&club).copied().unwrap_or(0);
        balance.0 += credit - debit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{BirthDate, Condition, Morale};
    use crate::schedule::build_daily_schedule;
    use crate::time::Date;

    #[test]
    fn payroll_runs_weekly_not_daily() {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
        let mut sched = build_daily_schedule();

        let club = world.spawn((Balance(10_000), WeeklyIncome(1_000))).id();
        for wage in [100u32, 250] {
            world.spawn((
                BirthDate(Date::new(2000, 1, 1)),
                Morale(70),
                Condition::fit(),
                Contract { club, until: Date::new(2030, 6, 30), wage },
            ));
        }

        // Six days in: no payday has occurred.
        for _ in 0..6 {
            sched.run(&mut world);
        }
        assert_eq!(world.get::<Balance>(club).unwrap().0, 10_000);

        // Day 7 is payday: +income 1000, -wages 350.
        sched.run(&mut world);
        assert_eq!(world.get::<Balance>(club).unwrap().0, 10_000 + 1_000 - 350);

        // Day 14: a second payday.
        for _ in 0..7 {
            sched.run(&mut world);
        }
        assert_eq!(world.get::<Balance>(club).unwrap().0, 10_000 + 2 * (1_000 - 350));
    }

    #[test]
    fn expired_contracts_drop_out_of_the_wage_bill() {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2025, 7, 1)));
        let mut sched = build_daily_schedule();

        let club = world.spawn((Balance(0), WeeklyIncome(0))).id();
        // Contract expires after day 3 (2025-07-04), well before the first payday.
        world.spawn((
            BirthDate(Date::new(2000, 1, 1)),
            Morale(70),
            Condition::fit(),
            Contract { club, until: Date::new(2025, 7, 3), wage: 500 },
        ));

        for _ in 0..7 {
            sched.run(&mut world);
        }
        // No contract remained at payday, so nothing was debited.
        assert_eq!(world.get::<Balance>(club).unwrap().0, 0);
    }
}
