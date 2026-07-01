//! A basketball league season. Same calendar-driven structure as football's, but the
//! standings are **win/loss** (no draws — see the engine) ranked on win percentage, which
//! is why the table rules live here per-sport while only the round-robin scheduler is shared
//! from `sim-core`.
//!
//! The fixture calendar and matchday progression are the shared `sim-core` [`Schedule`];
//! only the win/loss table fold below is basketball's own. (The remaining driver shell —
//! gather → play due → fold — still mirrors football's; a generic driver could harvest it
//! later, but the result→table fold differs per sport, so it stays concrete for now.)

use crate::matchday::{gather_rosters, simulate_matchday, Fixture};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use sim_core::{run_league_day, Date, League, Schedule};
use std::collections::BTreeMap;

/// A team's win/loss record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamRecord {
    pub won: u32,
    pub lost: u32,
    pub points_for: u32,
    pub points_against: u32,
}

impl TeamRecord {
    pub fn games(&self) -> u32 {
        self.won + self.lost
    }
    /// Win percentage in [0, 1]; 0 if no games played.
    pub fn win_pct(&self) -> f64 {
        if self.games() == 0 {
            0.0
        } else {
            f64::from(self.won) / f64::from(self.games())
        }
    }
    pub fn point_diff(&self) -> i32 {
        self.points_for as i32 - self.points_against as i32
    }
}

#[derive(Resource, Clone, Debug)]
pub struct Season {
    pub teams: Vec<u32>,
    pub schedule: Schedule,
    pub table: BTreeMap<u32, TeamRecord>,
    pub world_seed: u64,
    pub season_id: u32,
}

impl Season {
    pub fn new(teams: Vec<u32>, start: Date, world_seed: u64, season_id: u32) -> Self {
        let schedule = Schedule::round_robin(&teams, start);
        let table = teams.iter().map(|&t| (t, TeamRecord::default())).collect();
        Season { teams, schedule, table, world_seed, season_id }
    }

    pub fn is_complete(&self) -> bool {
        self.schedule.is_complete()
    }

    /// Standings by win percentage, then point differential.
    pub fn standings(&self) -> Vec<(u32, TeamRecord)> {
        let mut v: Vec<(u32, TeamRecord)> = self.table.iter().map(|(&t, &r)| (t, r)).collect();
        v.sort_by(|(_, a), (_, b)| {
            b.win_pct().total_cmp(&a.win_pct()).then(b.point_diff().cmp(&a.point_diff()))
        });
        v
    }

    pub fn champion(&self) -> Option<u32> {
        self.standings().first().map(|(t, _)| *t)
    }

    pub fn next_season(&self, start: Date) -> Season {
        Season::new(self.teams.clone(), start, self.world_seed, self.season_id + 1)
    }
}

fn record_result(table: &mut BTreeMap<u32, TeamRecord>, home: u32, away: u32, hp: u16, ap: u16) {
    let h = table.entry(home).or_default();
    h.points_for += u32::from(hp);
    h.points_against += u32::from(ap);
    if hp > ap {
        h.won += 1;
    } else {
        h.lost += 1;
    }
    let a = table.entry(away).or_default();
    a.points_for += u32::from(ap);
    a.points_against += u32::from(hp);
    if ap > hp {
        a.won += 1;
    } else {
        a.lost += 1;
    }
}

impl League for Season {
    fn schedule(&mut self) -> &mut Schedule {
        &mut self.schedule
    }

    fn play_matchday(&mut self, world: &mut World, index: usize, fixtures: &[(u32, u32)]) {
        let rosters = gather_rosters(world);
        let fx: Vec<Fixture> =
            fixtures.iter().map(|&(h, a)| Fixture { home: rosters[&h], away: rosters[&a] }).collect();
        let results = simulate_matchday(&fx, self.world_seed, self.season_id, index as u32);
        for (k, res) in results.iter().enumerate() {
            let (h, a) = fixtures[k];
            record_result(&mut self.table, h, a, res.home_points, res.away_points);
        }
    }
}

/// Play every matchday whose date the clock has reached, updating the table. Call once per
/// simulated day. The driver is the shared [`run_league_day`]; this season provides only the
/// per-matchday play above.
pub fn play_due_fixtures(world: &mut World) {
    run_league_day::<Season>(world);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::Baller;
    use sim_core::{build_daily_schedule, SimClock, SimSeed, TeamId};

    fn league_world(teams: u32) -> World {
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2025, 10, 1)));
        world.insert_resource(SimSeed(0xBA11));
        for t in 0..teams {
            for _ in 0..10 {
                world.spawn((
                    TeamId(t),
                    Baller { ins: 55, out: 50, pm: 55, reb: 55, def: 55, ath: 55 },
                ));
            }
        }
        world
    }

    #[test]
    fn a_full_season_plays_every_game_with_no_draws() {
        let teams = 6;
        let mut world = league_world(teams);
        world.insert_resource(Season::new((0..teams).collect(), Date::new(2025, 10, 8), 0xB0B, 2025));

        let mut daily = build_daily_schedule();
        for _ in 0..120 {
            daily.run(&mut world);
            play_due_fixtures(&mut world);
        }

        let season = world.resource::<Season>();
        assert!(season.is_complete());
        // Every team played 2*(n-1) games; wins + losses account for all of them.
        for (_, r) in season.standings() {
            assert_eq!(r.games(), 2 * (teams - 1));
        }
        // Total wins == total losses == total games (no draws possible).
        let total_games: u32 = season.standings().iter().map(|(_, r)| r.games()).sum::<u32>() / 2;
        let total_wins: u32 = season.standings().iter().map(|(_, r)| r.won).sum();
        assert_eq!(total_wins, total_games);
        assert!(season.champion().is_some());
    }
}
