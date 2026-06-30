//! Sport-agnostic league scheduling and progression.
//!
//! Harvested once a second league sport (basketball) duplicated football's season plumbing.
//! What is genuinely shared is the *fixture calendar* and the *progression cursor* — build a
//! round-robin, place it on weeks, and play matchdays as their dates arrive. What is **not**
//! shared, and stays in the sport crates, is the standings table: football is 3/1/0 with
//! draws and goal difference, basketball is win/loss on win percentage. So this module owns
//! the [`Schedule`]; each sport's `Season` embeds one alongside its own table.

use crate::competition::{double_round_robin, schedule_weekly};
use crate::time::Date;

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
