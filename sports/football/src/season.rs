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
use serde::{Deserialize, Serialize};
use sim_core::{run_league_day, Date, League, Schedule};
use std::collections::BTreeMap;

/// One team's running record in the league table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

/// A full league season: its fixture calendar (the shared `sim-core` [`Schedule`]) and its
/// football-specific standings table.
#[derive(Resource, Clone, Debug)]
pub struct Season {
    pub teams: Vec<u32>,
    pub schedule: Schedule,
    pub table: BTreeMap<u32, TeamRecord>,
    pub world_seed: u64,
    pub season_id: u32,
}

impl Season {
    /// Build a season: a double round-robin, one matchday per week from `start`. The
    /// scheduling is the shared sim-core piece; the table rules below are football's own.
    pub fn new(teams: Vec<u32>, start: Date, world_seed: u64, season_id: u32) -> Self {
        let schedule = Schedule::round_robin(&teams, start);
        let table = teams.iter().map(|&t| (t, TeamRecord::default())).collect();
        Season { teams, schedule, table, world_seed, season_id }
    }

    pub fn is_complete(&self) -> bool {
        self.schedule.is_complete()
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

impl League for Season {
    fn schedule(&mut self) -> &mut Schedule {
        &mut self.schedule
    }

    fn play_matchday(&mut self, world: &mut World, index: usize, fixtures: &[(u32, u32)]) {
        let lineups = gather_lineups(world);
        let fx: Vec<Fixture> =
            fixtures.iter().map(|&(h, a)| Fixture { home: lineups[&h], away: lineups[&a] }).collect();
        let results = simulate_matchday(&fx, self.world_seed, self.season_id, index as u32);
        for (k, res) in results.iter().enumerate() {
            let (h, a) = fixtures[k];
            record_result(&mut self.table, h, a, res.home_goals, res.away_goals);
            // Attribute the result to players' season tallies (apps + goals), seeded off the
            // same fixture coordinates so the stat lines are as reproducible as the scoreline.
            let seed =
                sim_core::derive_seed(self.world_seed, &[u64::from(self.season_id), index as u64, k as u64]);
            crate::tally::credit_match(world, h, a, res.home_goals, res.away_goals, seed);
            // Injuries roll on a distinct stream so they don't correlate with the scorers.
            let inj_seed = sim_core::derive_seed(seed, &[0xE]);
            crate::injuries::roll_match_injuries(world, h, a, inj_seed);
        }
    }
}

/// Play every matchday whose date the clock has reached, updating the league table. Call once
/// per simulated day after the daily schedule. The driver mechanism is the shared
/// [`run_league_day`]; this season only provides the per-matchday play above.
pub fn play_due_fixtures(world: &mut World) {
    run_league_day::<Season>(world);
}

/// Simulate a one-off round-robin among `teams` and return the final standings, best-first.
///
/// Unlike [`Season`] this is calendar- and lifecycle-free: it just plays the fixtures and
/// ranks the result. Handy for multi-division setups (each division ranked independently,
/// then fed to a `sim_core::Pyramid` for promotion/relegation). `comp_id` seeds the matches,
/// so give each division-season a distinct value.
pub fn rank_division(world: &mut World, teams: &[u32], world_seed: u64, comp_id: u32) -> Vec<u32> {
    let lineups = gather_lineups(world);
    let schedule = Schedule::round_robin(teams, Date::new(2000, 1, 1)); // dates irrelevant here
    let mut table: BTreeMap<u32, TeamRecord> = teams.iter().map(|&t| (t, TeamRecord::default())).collect();

    for i in 0..schedule.len() {
        let pairs = schedule.matchday(i).fixtures.clone();
        let fx: Vec<Fixture> =
            pairs.iter().map(|&(h, a)| Fixture { home: lineups[&h], away: lineups[&a] }).collect();
        let results = simulate_matchday(&fx, world_seed, comp_id, i as u32);
        for (k, res) in results.iter().enumerate() {
            let (h, a) = pairs[k];
            record_result(&mut table, h, a, res.home_goals, res.away_goals);
        }
    }

    let mut ranked: Vec<(u32, TeamRecord)> = table.into_iter().collect();
    ranked.sort_by(|(_, a), (_, b)| {
        b.points
            .cmp(&a.points)
            .then(b.goal_difference().cmp(&a.goal_difference()))
            .then(b.goals_for.cmp(&a.goals_for))
    });
    ranked.into_iter().map(|(t, _)| t).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::Footballer;
    use sim_core::{build_daily_schedule, SimClock, SimSeed, TeamId};

    /// Spawn `teams` clubs of identical-ish players into a world and return it.
    fn league_world(teams: u32) -> World {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2025, 8, 1)));
        world.insert_resource(SimSeed(0xFEED));
        for t in 0..teams {
            for _ in 0..11 {
                world.spawn((
                    TeamId(t),
                    Footballer {
                        pac: 55,
                        sho: 55,
                        pas: 55,
                        dri: 55,
                        tec: 55,
                        def: 55,
                        phy: 55,
                        vis: 55,
                        gk: 55,
                    },
                ));
            }
        }
        world
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
        assert!(!s2.schedule.is_complete());
        assert!(s2.schedule.matchday(0).date > s1.schedule.matchday(0).date);
    }
}
