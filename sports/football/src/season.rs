//! A league season: round-robin fixtures placed on the calendar, a points table, and a
//! driver that plays each matchday as the simulation clock reaches its date.
//!
//! Built concretely in the football crate, not as a sport-agnostic `CompetitionFormat`
//! trait — cycling already models its competition completely differently (a stage race
//! accumulating into a GC), so there is no shared shape to extract yet (constraint #4 and
//! the step-7B harvest lesson). A league table of points is football's own construct.
//!
//! Dependency direction stays correct: this drives `sim-core`'s clock and uses football's
//! own match engine; `sim-core` never references football. The driver
//! [`play_due_fixtures`] is a plain function the host loop calls after advancing a day.
//!
//! Not modelled yet (deliberately): promotion/relegation, which needs multiple divisions.
//! Single league for now.

use crate::matchday::{gather_lineups, simulate_matchday, Fixture};
use bevy_ecs::prelude::*;
use sim_core::{Date, SimClock};
use std::collections::BTreeMap;

/// One team's running record in the league table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeamRecord {
    pub played: u32,
    pub won: u32,
    pub drawn: u32,
    pub lost: u32,
    pub goals_for: u32,
    pub goals_against: u32,
    pub points: u32,
}

impl TeamRecord {
    pub fn goal_difference(&self) -> i32 {
        self.goals_for as i32 - self.goals_against as i32
    }
}

/// A scheduled round of fixtures on a specific date.
#[derive(Clone, Debug)]
pub struct Matchday {
    pub date: Date,
    /// `(home_team, away_team)` pairs by `TeamId` value.
    pub fixtures: Vec<(u32, u32)>,
    pub played: bool,
}

/// A full league season: its fixture calendar and standings.
#[derive(Resource, Clone, Debug)]
pub struct Season {
    pub teams: Vec<u32>,
    pub matchdays: Vec<Matchday>,
    /// Index of the next unplayed matchday.
    pub next: usize,
    pub table: BTreeMap<u32, TeamRecord>,
    pub world_seed: u64,
    pub season_id: u32,
}

impl Season {
    /// Build a season: a double round-robin (everyone plays everyone home and away),
    /// one matchday per week starting on `start`. Requires an even number of teams.
    pub fn new(teams: Vec<u32>, start: Date, world_seed: u64, season_id: u32) -> Self {
        assert!(teams.len() >= 2 && teams.len().is_multiple_of(2), "need an even team count >= 2");
        let matchdays = double_round_robin(&teams)
            .into_iter()
            .enumerate()
            .map(|(i, fixtures)| Matchday { date: start.add_days(i as u32 * 7), fixtures, played: false })
            .collect();
        let table = teams.iter().map(|&t| (t, TeamRecord::default())).collect();
        Season { teams, matchdays, next: 0, table, world_seed, season_id }
    }

    pub fn is_complete(&self) -> bool {
        self.next >= self.matchdays.len()
    }

    /// Standings ordered by points, then goal difference, then goals scored.
    pub fn standings(&self) -> Vec<(u32, TeamRecord)> {
        let mut v: Vec<(u32, TeamRecord)> = self.table.iter().map(|(&t, &r)| (t, r)).collect();
        v.sort_by(|(_, a), (_, b)| {
            b.points
                .cmp(&a.points)
                .then(b.goal_difference().cmp(&a.goal_difference()))
                .then(b.goals_for.cmp(&a.goals_for))
        });
        v
    }

    /// The current leader / final champion, if any teams exist.
    pub fn champion(&self) -> Option<u32> {
        self.standings().first().map(|(t, _)| *t)
    }

    /// Begin the following season with the same teams, fixtures regenerated from `start`.
    pub fn next_season(&self, start: Date) -> Season {
        Season::new(self.teams.clone(), start, self.world_seed, self.season_id + 1)
    }
}

/// Apply a single result to both teams' records.
fn record_result(table: &mut BTreeMap<u32, TeamRecord>, home: u32, away: u32, hg: u8, ag: u8) {
    use std::cmp::Ordering::{Equal, Greater, Less};
    let h = table.entry(home).or_default();
    h.played += 1;
    h.goals_for += u32::from(hg);
    h.goals_against += u32::from(ag);
    match hg.cmp(&ag) {
        Greater => {
            h.won += 1;
            h.points += 3;
        }
        Equal => {
            h.drawn += 1;
            h.points += 1;
        }
        Less => h.lost += 1,
    }
    let a = table.entry(away).or_default();
    a.played += 1;
    a.goals_for += u32::from(ag);
    a.goals_against += u32::from(hg);
    match ag.cmp(&hg) {
        Greater => {
            a.won += 1;
            a.points += 3;
        }
        Equal => {
            a.drawn += 1;
            a.points += 1;
        }
        Less => a.lost += 1,
    }
}

/// Play every matchday whose date the clock has reached but which hasn't been played yet,
/// updating the league table. Call once per simulated day, after the daily schedule.
///
/// Reads the [`Season`] and [`SimClock`] resources and the football players in the world;
/// each matchday is simulated in parallel and deterministically (seeded by the season's
/// seed and the matchday index).
pub fn play_due_fixtures(world: &mut World) {
    if world.get_resource::<Season>().is_none() {
        return;
    }
    world.resource_scope(|world, mut season: Mut<Season>| {
        let today = world.resource::<SimClock>().date();
        loop {
            let i = season.next;
            if i >= season.matchdays.len() || season.matchdays[i].played || season.matchdays[i].date > today {
                break;
            }
            let pairs = season.matchdays[i].fixtures.clone();
            let lineups = gather_lineups(world);
            let fixtures: Vec<Fixture> =
                pairs.iter().map(|&(h, a)| Fixture { home: lineups[&h], away: lineups[&a] }).collect();
            let results = simulate_matchday(&fixtures, season.world_seed, season.season_id, i as u32);
            for (k, res) in results.iter().enumerate() {
                let (h, a) = pairs[k];
                record_result(&mut season.table, h, a, res.home_goals, res.away_goals);
            }
            season.matchdays[i].played = true;
            season.next += 1;
        }
    });
}

/// Single round-robin via the circle method (each team plays each other once). Even N.
fn single_round_robin(teams: &[u32]) -> Vec<Vec<(u32, u32)>> {
    let n = teams.len();
    let mut arr = teams.to_vec();
    let mut rounds = Vec::with_capacity(n - 1);
    for r in 0..(n - 1) {
        let mut day = Vec::with_capacity(n / 2);
        for i in 0..n / 2 {
            let (a, b) = (arr[i], arr[n - 1 - i]);
            // Alternate home/away by round so no team is always at home.
            if r.is_multiple_of(2) {
                day.push((a, b));
            } else {
                day.push((b, a));
            }
        }
        rounds.push(day);
        // Rotate all but the first entry (circle method).
        arr[1..].rotate_right(1);
    }
    rounds
}

/// Double round-robin: the single schedule, then the same fixtures with venues swapped.
fn double_round_robin(teams: &[u32]) -> Vec<Vec<(u32, u32)>> {
    let first = single_round_robin(teams);
    let second: Vec<Vec<(u32, u32)>> =
        first.iter().map(|round| round.iter().map(|&(h, a)| (a, h)).collect()).collect();
    first.into_iter().chain(second).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{Footballer, TeamId};
    use sim_core::{build_daily_schedule, SimSeed};

    /// Spawn `teams` clubs of identical-ish players into a world and return it.
    fn league_world(teams: u32) -> World {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2025, 8, 1)));
        world.insert_resource(SimSeed(0xFEED));
        for t in 0..teams {
            for _ in 0..11 {
                world.spawn((
                    TeamId(t),
                    Footballer { attacking: 55, defending: 55, finishing: 55, goalkeeping: 55 },
                ));
            }
        }
        world
    }

    #[test]
    fn double_round_robin_has_every_pair_twice() {
        let teams: Vec<u32> = (0..6).collect();
        let rounds = double_round_robin(&teams);
        assert_eq!(rounds.len(), 2 * (6 - 1)); // 10 matchdays

        let mut games = std::collections::HashMap::<(u32, u32), u32>::new();
        for round in &rounds {
            assert_eq!(round.len(), 3); // 6 teams -> 3 games per round
            for &(h, a) in round {
                *games.entry((h, a)).or_default() += 1;
            }
        }
        // Each ordered pairing (home, away) occurs exactly once across the season.
        for &h in &teams {
            for &a in &teams {
                if h != a {
                    assert_eq!(games.get(&(h, a)).copied().unwrap_or(0), 1, "{h} vs {a}");
                }
            }
        }
    }

    #[test]
    fn a_full_season_plays_every_fixture() {
        let teams = 8;
        let mut world = league_world(teams);
        world.insert_resource(Season::new((0..teams).collect(), Date::new(2025, 8, 8), 0xABC, 2025));

        let mut daily = build_daily_schedule();
        // 2*(8-1) = 14 matchdays, weekly -> ~98 days; run comfortably past that.
        for _ in 0..130 {
            daily.run(&mut world);
            play_due_fixtures(&mut world);
        }

        let season = world.resource::<Season>();
        assert!(season.is_complete(), "all matchdays should have been played");
        // Every team played 2*(n-1) games.
        for (_, rec) in season.standings() {
            assert_eq!(rec.played, 2 * (teams - 1));
        }
        // Points conservation: each game awards 3 (decisive) or 2 (draw) total points.
        let total_games: u32 = season.standings().iter().map(|(_, r)| r.played).sum::<u32>() / 2;
        let total_points: u32 = season.standings().iter().map(|(_, r)| r.points).sum();
        let draws: u32 = season.standings().iter().map(|(_, r)| r.drawn).sum::<u32>() / 2;
        assert_eq!(total_points, 3 * (total_games - draws) + 2 * draws);
        assert!(season.champion().is_some());
    }

    #[test]
    fn rollover_resets_table_and_moves_dates() {
        let s1 = Season::new((0..4).collect(), Date::new(2025, 8, 8), 7, 2025);
        let s2 = s1.next_season(Date::new(2026, 8, 8));
        assert_eq!(s2.season_id, 2026);
        assert!(s2.table.values().all(|r| *r == TeamRecord::default()));
        assert_eq!(s2.next, 0);
        assert!(s2.matchdays[0].date > s1.matchdays[0].date);
    }
}
