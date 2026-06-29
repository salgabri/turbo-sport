//! Sport-agnostic competition scheduling.
//!
//! Harvested once a second team sport (basketball) needed the *identical* round-robin
//! fixturing that football already had. Round-robin pairing is pure combinatorics on
//! participant ids — it has no sport-specific meaning — so it belongs here.
//!
//! What stays in the sport crates: the **standings rules**. They genuinely differ — football
//! awards 3/1/0 with draws and ranks on goal difference; basketball is win/loss with no
//! draws, ranked on win percentage. Forcing those into one shared table would repeat the
//! premature-abstraction mistake constraint #4 warns about, so only the scheduler is shared.
//! (A knockout/bracket format — tennis — is a different scheduler entirely and lives in that
//! crate; it shares nothing with round-robin.)

use crate::time::Date;

/// Single round-robin via the circle method: each participant meets every other once.
/// Requires an even participant count. Returns rounds of `(home, away)` id pairs; the home
/// side alternates by round so no participant is always at home.
pub fn single_round_robin(participants: &[u32]) -> Vec<Vec<(u32, u32)>> {
    let n = participants.len();
    assert!(n >= 2 && n.is_multiple_of(2), "round-robin needs an even participant count >= 2");
    let mut arr = participants.to_vec();
    let mut rounds = Vec::with_capacity(n - 1);
    for r in 0..(n - 1) {
        let mut round = Vec::with_capacity(n / 2);
        for i in 0..n / 2 {
            let (a, b) = (arr[i], arr[n - 1 - i]);
            if r.is_multiple_of(2) {
                round.push((a, b));
            } else {
                round.push((b, a));
            }
        }
        rounds.push(round);
        // Rotate all but the first entry (circle method).
        arr[1..].rotate_right(1);
    }
    rounds
}

/// Double round-robin: the single schedule, then the same fixtures with venues swapped, so
/// every ordered `(home, away)` pairing occurs exactly once.
pub fn double_round_robin(participants: &[u32]) -> Vec<Vec<(u32, u32)>> {
    let first = single_round_robin(participants);
    let second: Vec<Vec<(u32, u32)>> =
        first.iter().map(|round| round.iter().map(|&(h, a)| (a, h)).collect()).collect();
    first.into_iter().chain(second).collect()
}

/// Place `rounds` onto the calendar, one round per week starting at `start`.
pub fn schedule_weekly(rounds: Vec<Vec<(u32, u32)>>, start: Date) -> Vec<(Date, Vec<(u32, u32)>)> {
    rounds
        .into_iter()
        .enumerate()
        .map(|(i, round)| (start.add_days(i as u32 * 7), round))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn double_round_robin_covers_every_ordered_pair_once() {
        let teams: Vec<u32> = (0..6).collect();
        let rounds = double_round_robin(&teams);
        assert_eq!(rounds.len(), 2 * (6 - 1));

        let mut games = HashMap::<(u32, u32), u32>::new();
        for round in &rounds {
            assert_eq!(round.len(), 3); // 6 participants -> 3 games per round
            for &pair in round {
                *games.entry(pair).or_default() += 1;
            }
        }
        for &h in &teams {
            for &a in &teams {
                if h != a {
                    assert_eq!(games.get(&(h, a)).copied().unwrap_or(0), 1, "{h} vs {a}");
                }
            }
        }
    }

    #[test]
    fn single_round_robin_gives_each_pair_once_and_full_rounds() {
        let teams: Vec<u32> = (0..8).collect();
        let rounds = single_round_robin(&teams);
        assert_eq!(rounds.len(), 7); // n-1 rounds
        let mut seen = HashMap::<(u32, u32), u32>::new();
        for round in &rounds {
            assert_eq!(round.len(), 4); // 8 teams -> 4 games per round
            for &(h, a) in round {
                // Count the unordered pairing.
                let key = if h < a { (h, a) } else { (a, h) };
                *seen.entry(key).or_default() += 1;
            }
        }
        // Every unordered pair meets exactly once in a single round-robin.
        for h in 0..8u32 {
            for a in (h + 1)..8 {
                assert_eq!(seen.get(&(h, a)).copied().unwrap_or(0), 1, "{h}-{a}");
            }
        }
    }

    #[test]
    fn schedule_weekly_spaces_rounds_seven_days_apart() {
        let rounds = single_round_robin(&[0, 1, 2, 3]);
        let scheduled = schedule_weekly(rounds, Date::new(2025, 8, 1));
        assert_eq!(scheduled[0].0, Date::new(2025, 8, 1));
        assert_eq!(scheduled[1].0, Date::new(2025, 8, 8));
        assert_eq!(scheduled[2].0, Date::new(2025, 8, 15));
    }
}
