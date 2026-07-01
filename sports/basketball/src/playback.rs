//! Basketball match **playback** — the court equivalent of `football::playback`.
//!
//! The possession engine ([`crate::engine`]) is left pure and only yields a final score, so
//! this module runs it for the deterministic result and then *synthesises* a plausible scoring
//! timeline (baskets worth 1/2/3, spread across the 48 minutes, with named scorers) that sums
//! exactly to that score. All of that dressing is drawn from a **separate** seeded stream keyed
//! off the match seed, so it never disturbs the engine's result — same seed, same game.

use crate::attributes::{Baller, POS_C, POS_F, POS_G};
use crate::engine::{simulate_game, Roster};
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use serde::Serialize;
use sim_core::{derive_seed, Club, Name, PositionGroup, Retired, TeamId};

/// A player marker on the 2D court (percent coords; home attacks left→right, away mirrored).
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Dot {
    pub n: u32,
    pub x: f32,
    pub y: f32,
}

/// One side: display name, short crest, and its five court dots.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Side {
    pub name: String,
    pub crest: String,
    pub dots: Vec<Dot>,
}

/// A moment in the game feed. `side` 0 = home, 1 = away; `points` is the basket value.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PlayEvent {
    pub minute: u32,
    pub kind: String,
    pub side: u8,
    pub points: u8,
    pub title: String,
    pub sub: String,
}

/// A single row of the stats panel (final totals; the UI interpolates them over time).
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct StatLine {
    pub label: &'static str,
    pub home: f64,
    pub away: f64,
}

/// Everything the front-end needs to replay one game in 2D. Same shape as football's so the
/// shared `Match.svelte` replays either sport.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct MatchPlayback {
    pub home: Side,
    pub away: Side,
    pub final_home: u8,
    pub final_away: u8,
    pub minutes: u32,
    pub events: Vec<PlayEvent>,
    pub stats: Vec<StatLine>,
}

struct PlayerLite {
    name: String,
    position: u8,
    off: u8,
    out: u8,
    overall: u8,
}

/// Home five-out formation slots: (position group, x%, y%). Away mirrors x.
const COURT5: [(u8, f32, f32); 5] = [
    (POS_G, 22.0, 30.0),
    (POS_G, 30.0, 62.0),
    (POS_F, 42.0, 22.0),
    (POS_F, 42.0, 74.0),
    (POS_C, 56.0, 50.0),
];

const GAME_MINUTES: u32 = 48;

fn team_players(world: &mut World, team_id: u32) -> Vec<PlayerLite> {
    let mut q = world
        .query_filtered::<(&TeamId, &Baller, Option<&PositionGroup>, Option<&Name>), (Without<Club>, Without<Retired>)>();
    let mut v: Vec<PlayerLite> = q
        .iter(world)
        .filter(|(t, ..)| t.0 == team_id)
        .map(|(_, b, pos, name)| {
            let position = pos.map_or(POS_F, |p| p.0);
            PlayerLite {
                name: name.map(|n| n.0.clone()).unwrap_or_else(|| "Unknown".into()),
                position,
                off: ((u16::from(b.ins) + u16::from(b.out) + u16::from(b.pm)) / 3) as u8,
                out: b.out,
                overall: b.overall(position),
            }
        })
        .collect();
    v.sort_by_key(|p| std::cmp::Reverse(p.overall));
    v
}

fn roster_of(world: &mut World, team_id: u32) -> Roster {
    let mut acc = (0.0, 0.0, 0.0, 0.0, 0u32);
    let mut q = world.query_filtered::<(&TeamId, &Baller), Without<Retired>>();
    for (t, b) in q.iter(world) {
        if t.0 != team_id {
            continue;
        }
        acc.0 += b.offense();
        acc.1 += b.defense();
        acc.2 += b.three_point();
        acc.3 += b.rebounding();
        acc.4 += 1;
    }
    let n = f64::from(acc.4.max(1));
    Roster::new(acc.0 / n, acc.1 / n, acc.2 / n, acc.3 / n)
}

fn court_dots(players: &[PlayerLite], home: bool) -> Vec<Dot> {
    let mut used = vec![false; players.len()];
    let mut dots = Vec::with_capacity(5);
    for (slot, &(want, x, y)) in COURT5.iter().enumerate() {
        let pick = players
            .iter()
            .enumerate()
            .find(|(i, p)| !used[*i] && p.position == want)
            .map(|(i, _)| i)
            .or_else(|| (0..players.len()).find(|&i| !used[i]));
        if let Some(i) = pick {
            used[i] = true;
        }
        dots.push(Dot { n: slot as u32 + 1, x: if home { x } else { 100.0 - x }, y });
    }
    dots
}

/// Pick a scorer, weighted by scoring; three-pointers favour outside shooters.
fn pick_scorer(players: &[PlayerLite], three: bool, rng: &mut Pcg64Mcg) -> String {
    if players.is_empty() {
        return "Unknown".into();
    }
    let weight = |p: &PlayerLite| u32::from(if three { p.out } else { p.off }).max(1);
    let total: u32 = players.iter().map(weight).sum();
    let mut r = rng.gen_range(0..total);
    for p in players {
        let w = weight(p);
        if r < w {
            return p.name.clone();
        }
        r -= w;
    }
    players[0].name.clone()
}

/// Break a team's `points` into a minute-ordered list of baskets `(minute, value)` that sums
/// exactly to `points`. Three-point frequency scales with the team's outside shooting.
fn baskets(points: u32, three_rate: f64, rng: &mut Pcg64Mcg) -> Vec<(u32, u8)> {
    let mut out = Vec::new();
    let mut remaining = points;
    while remaining > 0 {
        let v: u8 = if remaining >= 3 && rng.gen_bool(three_rate) {
            3
        } else if remaining >= 2 {
            2
        } else {
            1
        };
        out.push((rng.gen_range(0..GAME_MINUTES), v));
        remaining -= u32::from(v);
    }
    out.sort_by_key(|b| b.0);
    out
}

fn crest_of(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn club_name(world: &mut World, team_id: u32) -> String {
    let mut q = world.query_filtered::<(&TeamId, Option<&Name>), With<Club>>();
    q.iter(world)
        .find(|(t, _)| t.0 == team_id)
        .and_then(|(_, n)| n.map(|n| n.0.clone()))
        .unwrap_or_else(|| format!("Team {team_id}"))
}

/// Build a full [`MatchPlayback`] for `home_id` vs `away_id` from their rosters, seeded by
/// `seed` (same seed → identical playback).
pub fn simulate_match_playback(
    world: &mut World,
    home_id: u32,
    away_id: u32,
    seed: u64,
) -> MatchPlayback {
    let home_players = team_players(world, home_id);
    let away_players = team_players(world, away_id);
    let home_roster = roster_of(world, home_id);
    let away_roster = roster_of(world, away_id);

    let mut game_rng = Pcg64Mcg::seed_from_u64(seed);
    let res = simulate_game(&home_roster, &away_roster, &mut game_rng);

    let mut flavour = Pcg64Mcg::seed_from_u64(derive_seed(seed, &[0xBA5]));

    // Three-point rate scales with each side's mean outside rating.
    let three_rate = |players: &[PlayerLite]| {
        if players.is_empty() {
            return 0.28;
        }
        let mean: f64 =
            players.iter().map(|p| f64::from(p.out)).sum::<f64>() / players.len() as f64;
        (0.18 + mean / 400.0).clamp(0.18, 0.5)
    };

    let mut events: Vec<PlayEvent> = Vec::new();
    let mut push_side = |players: &[PlayerLite], side: u8, pts: u32, flavour: &mut Pcg64Mcg| {
        for (minute, v) in baskets(pts, three_rate(players), flavour) {
            let scorer = pick_scorer(players, v == 3, flavour);
            let label = if v == 1 { "FT".to_string() } else { format!("{v}PT") };
            events.push(PlayEvent {
                minute,
                kind: "score".into(),
                side,
                points: v,
                title: format!("{label} · {scorer}"),
                sub: if side == 0 { "Home basket".into() } else { "Away basket".into() },
            });
        }
    };
    push_side(&home_players, 0, u32::from(res.home_points), &mut flavour);
    push_side(&away_players, 1, u32::from(res.away_points), &mut flavour);
    events.sort_by_key(|e| e.minute);

    let threes_h = events.iter().filter(|e| e.side == 0 && e.points == 3).count() as f64;
    let threes_a = events.iter().filter(|e| e.side == 1 && e.points == 3).count() as f64;
    let stats = vec![
        StatLine { label: "Points", home: f64::from(res.home_points), away: f64::from(res.away_points) },
        StatLine { label: "Rebounds", home: f64::from(flavour.gen_range(32..48)), away: f64::from(flavour.gen_range(32..48)) },
        StatLine { label: "Assists", home: f64::from(flavour.gen_range(18..30)), away: f64::from(flavour.gen_range(18..30)) },
        StatLine { label: "Field Goal %", home: f64::from(flavour.gen_range(42..53)), away: f64::from(flavour.gen_range(42..53)) },
        StatLine { label: "3-Pointers", home: threes_h, away: threes_a },
        StatLine { label: "Turnovers", home: f64::from(flavour.gen_range(8..16)), away: f64::from(flavour.gen_range(8..16)) },
    ];

    let home_name = club_name(world, home_id);
    let away_name = club_name(world, away_id);
    MatchPlayback {
        home: Side { crest: crest_of(&home_name), name: home_name, dots: court_dots(&home_players, true) },
        away: Side { crest: crest_of(&away_name), name: away_name, dots: court_dots(&away_players, false) },
        final_home: res.home_points.min(255) as u8,
        final_away: res.away_points.min(255) as u8,
        minutes: GAME_MINUTES,
        events,
        stats,
    }
}

/// Build a playback for a team's next opponent: the in-progress `Season`'s next unplayed fixture
/// if any, otherwise a friendly against the lowest other club id.
pub fn next_match_playback(world: &mut World, team_id: u32) -> Option<MatchPlayback> {
    let world_seed = world.get_resource::<sim_core::SimSeed>().map_or(0, |s| s.0);
    let (home, away, coord) = next_fixture(world, team_id)?;
    let seed = derive_seed(world_seed, &[u64::from(home), u64::from(away), coord, 0xB]);
    Some(simulate_match_playback(world, home, away, seed))
}

fn next_fixture(world: &mut World, team_id: u32) -> Option<(u32, u32, u64)> {
    if let Some(season) = world.get_resource::<crate::season::Season>() {
        let sched = &season.schedule;
        for i in season.schedule.next_index()..sched.len() {
            let md = sched.matchday(i);
            if md.played {
                continue;
            }
            if let Some(&(h, a)) = md.fixtures.iter().find(|&&(h, a)| h == team_id || a == team_id) {
                return Some((h, a, i as u64));
            }
        }
    }
    let mut clubs: Vec<u32> = {
        let mut q = world.query_filtered::<&TeamId, With<Club>>();
        q.iter(world).map(|t| t.0).collect()
    };
    clubs.sort_unstable();
    let opp = clubs.into_iter().find(|&c| c != team_id)?;
    Some((team_id, opp, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn playback_scores_sum_to_the_final_and_five_a_side() {
        let db = sample();
        let mut world = load_world(&db);
        let (h, a) = (db.clubs[0].id, db.clubs[1].id);
        let pb = simulate_match_playback(&mut world, h, a, 0xCAFE);

        assert_eq!(pb.home.dots.len(), 5);
        assert_eq!(pb.away.dots.len(), 5);
        let hp: u32 = pb.events.iter().filter(|e| e.side == 0).map(|e| u32::from(e.points)).sum();
        let ap: u32 = pb.events.iter().filter(|e| e.side == 1).map(|e| u32::from(e.points)).sum();
        assert_eq!(hp, u32::from(pb.final_home));
        assert_eq!(ap, u32::from(pb.final_away));
        assert!(pb.events.windows(2).all(|w| w[0].minute <= w[1].minute));
        assert_eq!(pb.stats.len(), 6);
    }

    #[test]
    fn same_seed_gives_identical_playback() {
        let db = sample();
        let mut world = load_world(&db);
        let (h, a) = (db.clubs[0].id, db.clubs[1].id);
        assert_eq!(
            simulate_match_playback(&mut world, h, a, 9),
            simulate_match_playback(&mut world, h, a, 9)
        );
    }
}
