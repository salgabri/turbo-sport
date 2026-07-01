//! Sport-agnostic entity schema: the components every person in the simulation has,
//! regardless of sport, plus a bundle to spawn one.
//!
//! Scope line (see `CLAUDE.md`): only the *sport-neutral* facts live here — when a
//! person was born, their contract, their physical condition, morale. Sport-specific
//! ability attributes (a footballer's passing, a cyclist's climbing) belong to the
//! sport crate's own schema, not to `sim-core`. The `wage` on [`Contract`] is stored
//! as plain data here; actually *paying* it is the economy's job (build step 4), not
//! this module's.

use crate::time::Date;
use bevy_ecs::prelude::*;

/// When this person was born. Age is derived from this and the world clock rather than
/// stored, so it can never drift out of sync with simulated time. See [`age_years`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BirthDate(pub Date);

/// An employment contract tying a person to a club until a date.
///
/// `wage` is recorded here but intentionally **not** processed in this module —
/// payroll, budgets, and the transfer market are the economy (build step 4). Keeping
/// the field as data avoids a second source of truth later without pulling economy
/// logic into lifecycle.
#[derive(Component, Clone, Copy, Debug)]
pub struct Contract {
    /// The employing club entity.
    pub club: Entity,
    /// Last day the contract is valid; on/after the following day the person is free.
    pub until: Date,
    /// Weekly wage, in the game's money unit. Data only — see the type docs.
    pub wage: u32,
}

/// Morale, 0..=100. Stored generically; the dynamics that move it (results, playing
/// time, transfers) are driven by later subsystems, not by lifecycle.
#[derive(Component, Clone, Copy, Debug)]
pub struct Morale(pub u8);

/// Physical condition: match fitness and any active injury.
#[derive(Component, Clone, Copy, Debug)]
pub struct Condition {
    /// Match fitness, 0..=100.
    pub fitness: u8,
    /// Days remaining until recovered; 0 means available to play.
    pub injury_days: u16,
}

impl Condition {
    /// A fully fit, uninjured condition.
    pub fn fit() -> Self {
        Self { fitness: 100, injury_days: 0 }
    }

    pub fn is_injured(&self) -> bool {
        self.injury_days > 0
    }
}

/// Marker: this person currently has no contract.
#[derive(Component, Clone, Copy, Debug)]
pub struct FreeAgent;

/// Marker: this person has retired from active play.
#[derive(Component, Clone, Copy, Debug)]
pub struct Retired;

/// A person's or club's display name. The simulation doesn't need it, but it is carried for
/// the UI and authored in the starting database. Not `Copy` (owns a `String`).
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct Name(pub String);

/// Which team/club an entity belongs to, by stable integer id. Sport-agnostic team
/// identity — the team sports (football, basketball, …) all tag their players with this
/// rather than each defining their own identical id type.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TeamId(pub u32);

/// A club's desired squad size. The transfer system signs free agents to clubs that are
/// below this target.
#[derive(Component, Clone, Copy, Debug)]
pub struct SquadTarget(pub u32);

/// A free agent's asking weekly wage, set when their contract expires (carried over from
/// the old wage). The transfer system signs free agents whose demand a club can afford.
#[derive(Component, Clone, Copy, Debug)]
pub struct WageDemand(pub u32);

/// A person's nationality, as an authored code or name. Sport-neutral flavour carried for the
/// UI (and future eligibility rules); no simulation system reads it today. Owns a `String`.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct Nationality(pub String);

/// An **opaque** position/role group index into the *sport's* own position table. `sim-core`
/// deliberately does not name the meanings — football maps `0..=3` to GK/DEF/MID/FWD,
/// basketball to G/F/C, cycling to GC/CLB/SPR/TT/DOM — the sport crate and the UI own that
/// mapping. Kept here (not in a sport crate) because lineup/formation concepts are genuinely
/// cross-sport, and it round-trips with the generic save.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionGroup(pub u8);

/// A player's aggregate rating: current `overall` and peak `potential`, each 0..=99.
///
/// `sim-core` stores the *shape* but never computes it — each sport derives `overall` from its
/// own attributes (position-weighted) at spawn, the same way it inserts its ability component.
/// A later deterministic growth system can move `overall` toward `potential` with age.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rating {
    pub overall: u8,
    pub potential: u8,
}

/// The minimal set of components every simulated person shares. Sport crates spawn this
/// alongside their own ability components; a [`Contract`] is added separately because a
/// free agent is a person without one.
#[derive(Bundle)]
pub struct PersonBundle {
    pub birth: BirthDate,
    pub morale: Morale,
    pub condition: Condition,
}

/// Whole years elapsed from `birth` to `today` on the fixed calendar. Clamped at 0 so a
/// birth date in the future (bad data) yields 0 rather than a negative age.
pub fn age_years(birth: Date, today: Date) -> u32 {
    let mut years = today.year() - birth.year();
    if (today.month(), today.day()) < (birth.month(), birth.day()) {
        years -= 1;
    }
    years.max(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_counts_only_completed_years() {
        let birth = Date::new(1990, 6, 15);
        assert_eq!(age_years(birth, Date::new(2026, 6, 14)), 35); // day before birthday
        assert_eq!(age_years(birth, Date::new(2026, 6, 15)), 36); // on birthday
        assert_eq!(age_years(birth, Date::new(2026, 6, 16)), 36);
    }

    #[test]
    fn future_birth_clamps_to_zero() {
        assert_eq!(age_years(Date::new(2030, 1, 1), Date::new(2026, 1, 1)), 0);
    }
}
