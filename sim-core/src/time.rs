//! Simulation time: a calendar date, the world clock, and the day-advancing system.
//!
//! Design notes:
//! - The calendar is a **fixed 365-day proleptic year with no leap years**. A sport
//!   sim does not need astronomical accuracy, and a fixed calendar is fully
//!   deterministic and trivially reproducible — which the determinism requirement
//!   (see `docs/ARCHITECTURE.md`) cares about far more than Feb 29th existing.
//! - "Season" is modelled the football way: a season is named by the calendar year it
//!   starts in and runs July 1 → June 30. Other sports can reinterpret this later; it
//!   is not yet a shared trait.

use bevy_ecs::prelude::*;
use std::fmt;

/// Days in each month, January = index 0. No leap years (see module docs).
const MONTH_DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// A calendar date in the simulation. Plain value type — cheap to copy and store as a
/// component if a sport ever needs per-entity dates (e.g. birth date, contract end).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Date {
    year: i32,
    /// 1..=12
    month: u8,
    /// 1..=days_in_month
    day: u8,
}

impl Date {
    /// Construct a date. Panics on an out-of-range month or day — dates are
    /// constructed from controlled inputs (schedules, save data), never user text, so
    /// an invalid date is a programmer error worth catching loudly.
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        assert!((1..=12).contains(&month), "month out of range: {month}");
        let dim = MONTH_DAYS[(month - 1) as usize];
        assert!((1..=dim).contains(&day), "day {day} out of range for month {month}");
        Self { year, month, day }
    }

    pub fn year(self) -> i32 {
        self.year
    }
    pub fn month(self) -> u8 {
        self.month
    }
    pub fn day(self) -> u8 {
        self.day
    }

    /// The next calendar day, rolling month and year over as needed.
    #[must_use]
    pub fn succ(self) -> Date {
        let dim = MONTH_DAYS[(self.month - 1) as usize];
        if self.day < dim {
            Date { day: self.day + 1, ..self }
        } else if self.month < 12 {
            Date { month: self.month + 1, day: 1, ..self }
        } else {
            Date { year: self.year + 1, month: 1, day: 1 }
        }
    }

    /// The calendar year in which this date's *season* began. Football convention:
    /// July starts a new season, so e.g. 2026-03-14 belongs to the 2025/26 season and
    /// returns 2025, while 2026-08-01 belongs to 2026/27 and returns 2026.
    pub fn season_start_year(self) -> i32 {
        if self.month >= 7 {
            self.year
        } else {
            self.year - 1
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// The world clock: the single source of truth for "what day is it" during play.
///
/// Stored as an ECS resource. Systems read it with `Res<SimClock>` and the tick system
/// advances it with `ResMut<SimClock>`. `day_index` is a monotonic counter from world
/// start — handy as a stable, calendar-independent ordinate (e.g. for scheduling or as
/// a component of a derived RNG seed).
#[derive(Resource, Clone, Copy, Debug)]
pub struct SimClock {
    date: Date,
    day_index: u64,
}

impl SimClock {
    /// Start the clock on `date` at day index 0.
    pub fn starting_on(date: Date) -> Self {
        Self { date, day_index: 0 }
    }

    /// Reconstruct a clock from saved parts. Used by persistence on load: `day_index` is
    /// the monotonic counter captured at save time, restored verbatim rather than
    /// recomputed from `date`.
    pub fn from_parts(date: Date, day_index: u64) -> Self {
        Self { date, day_index }
    }

    pub fn date(&self) -> Date {
        self.date
    }

    /// Days elapsed since the world started (0 on the first day).
    pub fn day_index(&self) -> u64 {
        self.day_index
    }

    /// The starting year of the current season (see [`Date::season_start_year`]).
    pub fn season_start_year(&self) -> i32 {
        self.date.season_start_year()
    }

    /// Advance exactly one day. The whole sim's notion of time moving forward funnels
    /// through here, so there is one place to reason about ordering and determinism.
    pub fn advance_one_day(&mut self) {
        self.date = self.date.succ();
        self.day_index += 1;
    }
}

/// The tick system: advance the world clock by one day.
///
/// Registered into a `Schedule`; one `schedule.run(&mut world)` = one simulated day.
/// Lifecycle, economy, and competition systems will later be ordered relative to this
/// in the same schedule — for now it is the entire tick.
pub fn advance_time(mut clock: ResMut<SimClock>) {
    clock.advance_one_day();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls_month_and_year() {
        assert_eq!(Date::new(2026, 1, 31).succ(), Date::new(2026, 2, 1));
        assert_eq!(Date::new(2026, 2, 28).succ(), Date::new(2026, 3, 1)); // no leap day
        assert_eq!(Date::new(2026, 12, 31).succ(), Date::new(2027, 1, 1));
    }

    #[test]
    fn season_boundary_is_july() {
        assert_eq!(Date::new(2026, 6, 30).season_start_year(), 2025);
        assert_eq!(Date::new(2026, 7, 1).season_start_year(), 2026);
    }

    #[test]
    fn one_fixed_year_is_365_days() {
        let mut clock = SimClock::starting_on(Date::new(2025, 7, 1));
        for _ in 0..365 {
            clock.advance_one_day();
        }
        assert_eq!(clock.date(), Date::new(2026, 7, 1));
        assert_eq!(clock.day_index(), 365);
    }
}
