//! Sport-agnostic league scheduling and progression.
//!
//! Harvested once a second league sport (basketball) duplicated football's season plumbing.
//! What is genuinely shared is the *fixture calendar* and the *progression cursor* — build a
//! round-robin, place it on weeks, and play matchdays as their dates arrive. What is **not**
//! shared, and stays in the sport crates, is the standings table: football is 3/1/0 with
//! draws and goal difference, basketball is win/loss on win percentage. So this module owns
//! the [`Schedule`]; each sport's `Season` embeds one alongside its own table.

use crate::competition::{double_round_robin, schedule_weekly};
use crate::time::{Date, SimClock};
use bevy_ecs::prelude::*;

/// A scheduled round of fixtures on a date. `fixtures` are `(home, away)` team-id pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matchday {
    pub date: Date,
    pub fixtures: Vec<(u32, u32)>,
    pub played: bool,
}

/// A league's fixture calendar plus a cursor to the next unplayed matchday.
#[derive(Clone, Debug)]
pub struct Schedule {
    matchdays: Vec<Matchday>,
    next: usize,
}

impl Schedule {
    /// A double round-robin (everyone plays everyone home and away), one matchday per week
    /// starting on `start`.
    pub fn round_robin(teams: &[u32], start: Date) -> Self {
        let matchdays = schedule_weekly(double_round_robin(teams), start)
            .into_iter()
            .map(|(date, fixtures)| Matchday { date, fixtures, played: false })
            .collect();
        Self { matchdays, next: 0 }
    }

    pub fn len(&self) -> usize {
        self.matchdays.len()
    }

    pub fn is_empty(&self) -> bool {
        self.matchdays.is_empty()
    }

    /// True once every matchday has been played.
    pub fn is_complete(&self) -> bool {
        self.next >= self.matchdays.len()
    }

    /// The cursor: index of the next unplayed matchday. Exposed for persistence.
    pub fn next_index(&self) -> usize {
        self.next
    }

    /// Reconstruct a schedule from saved matchdays and cursor — used by a sport's save/load to
    /// persist an in-progress season (the runtime fields are otherwise private).
    pub fn from_parts(matchdays: Vec<Matchday>, next: usize) -> Self {
        Self { matchdays, next }
    }

    /// The matchday at `index` (e.g. to read its fixtures or date).
    pub fn matchday(&self, index: usize) -> &Matchday {
        &self.matchdays[index]
    }

    /// Mark and return the indices of every matchday due on/before `today` that has not been
    /// played, advancing the cursor past them. The caller plays the returned matchdays.
    pub fn take_due(&mut self, today: Date) -> Vec<usize> {
        let mut due = Vec::new();
        while self.next < self.matchdays.len()
            && !self.matchdays[self.next].played
            && self.matchdays[self.next].date <= today
        {
            let i = self.next;
            self.matchdays[i].played = true;
            self.next += 1;
            due.push(i);
        }
        due
    }
}

/// A sport's league season, drivable by [`run_league_day`].
///
/// This is the harvested half of what football and basketball duplicated: the calendar
/// progression (owned here) is shared, while the parts that genuinely differ — gathering a
/// lineup, simulating, and folding results into the sport's own standings table — are the
/// sport's [`play_matchday`](League::play_matchday) implementation. Same split as the rest of
/// the shared layer: centralize the subtle mechanism (the `resource_scope` + `take_due` dance
/// and the matchday-index seeding convention), keep the domain logic concrete per sport.
pub trait League: Resource {
    /// This season's fixture calendar.
    fn schedule(&mut self) -> &mut Schedule;

    /// Play matchday `index` with the given `fixtures`: simulate them and fold the results
    /// into this season's standings. `world` is available (this season resource has been
    /// taken out of it) for gathering lineups.
    fn play_matchday(&mut self, world: &mut World, index: usize, fixtures: &[(u32, u32)]);
}

/// Drive a league for one simulated day: play every matchday whose date the clock has
/// reached, in order. A no-op if the season resource is absent. Call once per day, after the
/// daily schedule. The whole `resource_scope`/`take_due` mechanism lives here so each sport's
/// driver is a one-liner over its [`League`] impl.
pub fn run_league_day<S: League>(world: &mut World) {
    if world.get_resource::<S>().is_none() {
        return;
    }
    let today = world.resource::<SimClock>().date();
    world.resource_scope(|world, mut season: Mut<S>| {
        for index in season.schedule().take_due(today) {
            let fixtures = season.schedule().matchday(index).fixtures.clone();
            season.play_matchday(world, index, &fixtures);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_builds_a_full_double_schedule() {
        let s = Schedule::round_robin(&[0, 1, 2, 3], Date::new(2025, 8, 1));
        assert_eq!(s.len(), 2 * (4 - 1)); // 6 matchdays
        assert!(!s.is_complete());
        assert_eq!(s.matchday(0).date, Date::new(2025, 8, 1));
    }

    #[test]
    fn take_due_releases_matchdays_as_dates_arrive() {
        let mut s = Schedule::round_robin(&[0, 1, 2, 3], Date::new(2025, 8, 1));

        // Before the first date: nothing due.
        assert!(s.take_due(Date::new(2025, 7, 31)).is_empty());
        // On the first date: exactly matchday 0.
        assert_eq!(s.take_due(Date::new(2025, 8, 1)), vec![0]);
        // Two weeks on: matchdays 1 and 2 (Aug 8 and Aug 15) both become due.
        assert_eq!(s.take_due(Date::new(2025, 8, 15)), vec![1, 2]);
        // Already-released matchdays are not handed out again.
        assert!(s.take_due(Date::new(2025, 8, 15)).is_empty());
    }

    #[test]
    fn completes_after_all_matchdays_taken() {
        let mut s = Schedule::round_robin(&[0, 1], Date::new(2025, 8, 1));
        let _ = s.take_due(Date::new(2026, 1, 1)); // far future: take everything
        assert!(s.is_complete());
    }
}
