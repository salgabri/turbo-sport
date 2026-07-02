//! Match **playback**: turn a single simulated match into a UI-replayable experience.
//!
//! The [`crate::engine`] stays a pure `(lineups, rng) -> result` function — this module reads
//! two real squads out of the world, aggregates their lineups, runs that engine, and then
//! *dresses* the deterministic result into a [`MatchPlayback`]: a formation of dots for the 2D
//! pitch (drawn from the actual best XI), a minute-ordered event feed with named scorers, and
//! the match stat totals. The front-end replays it against a clock, so the "live 2D match" is a
//! deterministic recording, not a second simulation — same seed, same match, every time.
//!
//! All flavour that the engine does not itself produce (which player scored, bookings, corners,
//! fouls) is drawn from a *separate* seeded stream keyed off the match seed, so it never
//! perturbs the goal/xG sequence the engine already fixed.

use crate::attributes::{Footballer, POS_DEF, POS_FWD, POS_GK, POS_MID};
use crate::engine::{simulate_match, Lineup};
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use serde::Serialize;
use sim_core::{derive_seed, Club, Name, PositionGroup, Retired, TeamId};

/// A player marker on the 2D pitch. `x`/`y` are pitch percentages (0..100), oriented so the
/// home side attacks left→right; the away side is mirrored.
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Dot {
    pub n: u32,
    pub x: f32,
    pub y: f32,
}

/// One side of the tie: display name, short crest, and its eleven pitch dots.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Side {
    pub name: String,
    pub crest: String,
    pub dots: Vec<Dot>,
}

/// A moment in the match feed. `side` is 0 = home, 1 = away.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PlayEvent {
    pub minute: u32,
    /// "goal" | "shot" | "card".
    pub kind: String,
    pub side: u8,
    /// Points this event adds to `side`'s score (a football goal is 1; a booking 0). The UI
    /// sums this so the same replay logic serves sports whose events are worth 2 or 3.
    pub points: u8,
    pub title: String,
    pub sub: String,
}

/// A single row of the match-stats panel (final totals; the UI interpolates them over time).
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct StatLine {
    pub label: &'static str,
    pub home: f64,
    pub away: f64,
}

/// Everything the front-end needs to replay one match in 2D.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct MatchPlayback {
    pub home: Side,
    pub away: Side,
    pub final_home: u8,
    pub final_away: u8,
    /// Regulation length in minutes (the clock the UI plays out).
    pub minutes: u32,
    /// Minute-ordered feed of goals and bookings.
    pub events: Vec<PlayEvent>,
    /// Final stat totals for the six stat rows.
    pub stats: Vec<StatLine>,
}

/// A player reduced to what the playback needs.
struct PlayerLite {
    name: String,
    position: u8,
    sho: u8,
    overall: u8,
}

/// Home 4-3-3 formation slots: (position group, x%, y%). Away mirrors x.
const F433: [(u8, f32, f32); 11] = [
    (POS_GK, 7.0, 50.0),
    (POS_DEF, 20.0, 16.0),
    (POS_DEF, 20.0, 39.0),
    (POS_DEF, 20.0, 61.0),
    (POS_DEF, 20.0, 84.0),
    (POS_MID, 38.0, 28.0),
    (POS_MID, 40.0, 50.0),
    (POS_MID, 38.0, 72.0),
    (POS_FWD, 57.0, 24.0),
    (POS_FWD, 60.0, 50.0),
    (POS_FWD, 57.0, 76.0),
];

/// Collect one team's available players (excluding the club entity and retirees).
fn team_players(world: &mut World, team_id: u32) -> Vec<PlayerLite> {
    let mut q = world
        .query_filtered::<(&TeamId, &Footballer, Option<&PositionGroup>, Option<&Name>), (Without<Club>, Without<Retired>)>();
    let mut v: Vec<PlayerLite> = q
        .iter(world)
        .filter(|(t, ..)| t.0 == team_id)
        .map(|(_, f, pos, name)| PlayerLite {
            name: name.map(|n| n.0.clone()).unwrap_or_else(|| "Unknown".into()),
            position: pos.map_or(POS_MID, |p| p.0),
            sho: f.sho,
            overall: f.overall(pos.map_or(POS_MID, |p| p.0)),
        })
        .collect();
    v.sort_by_key(|p| std::cmp::Reverse(p.overall));
    v
}

/// Aggregate a team's players into a [`Lineup`] (mean of the per-player engine aggregates).
fn lineup_of(world: &mut World, team_id: u32) -> Lineup {
    let mut acc = (0.0, 0.0, 0.0, 0.0, 0u32);
    let mut q = world.query_filtered::<(&TeamId, &Footballer), Without<Retired>>();
    for (t, f) in q.iter(world) {
        if t.0 != team_id {
            continue;
        }
        acc.0 += f.attack();
        acc.1 += f.defense();
        acc.2 += f.finishing();
        acc.3 += f.keeper();
        acc.4 += 1;
    }
    let n = f64::from(acc.4.max(1));
    Lineup::new(acc.0 / n, acc.1 / n, acc.2 / n, acc.3 / n)
}

/// Pick the best XI into the 4-3-3 slots, returning pitch dots. `home` sets the orientation.
fn formation_dots(players: &[PlayerLite], home: bool) -> Vec<Dot> {
    let mut used = vec![false; players.len()];
    let mut dots = Vec::with_capacity(11);
    for (slot, &(want_pos, x, y)) in F433.iter().enumerate() {
        // Prefer an unused player of the wanted position; fall back to best remaining.
        let pick = players
            .iter()
            .enumerate()
            .find(|(i, p)| !used[*i] && p.position == want_pos)
            .map(|(i, _)| i)
            .or_else(|| (0..players.len()).find(|&i| !used[i]));
        if let Some(i) = pick {
            used[i] = true;
        }
        let px = if home { x } else { 100.0 - x };
        dots.push(Dot { n: slot as u32 + 1, x: px, y });
    }
    dots
}

/// Choose a scorer for a goal by `players`, weighted by shooting (attackers favoured).
fn pick_scorer(players: &[PlayerLite], rng: &mut Pcg64Mcg) -> String {
    let weighted: Vec<(&PlayerLite, u32)> = players
        .iter()
        .filter(|p| p.position == POS_FWD || p.position == POS_MID)
        .map(|p| (p, u32::from(p.sho).max(1)))
        .collect();
    let pool = if weighted.is_empty() {
        players.iter().map(|p| (p, u32::from(p.sho).max(1))).collect()
    } else {
        weighted
    };
    if pool.is_empty() {
        return "Unknown".into();
    }
    let total: u32 = pool.iter().map(|(_, w)| *w).sum();
    let mut r = rng.gen_range(0..total);
    for (p, w) in &pool {
        if r < *w {
            return p.name.clone();
        }
        r -= *w;
    }
    pool[0].0.name.clone()
}

fn crest_of(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

/// Look up a club's display name by team id.
fn club_name(world: &mut World, team_id: u32) -> String {
    let mut q = world.query_filtered::<(&TeamId, Option<&Name>), With<Club>>();
    q.iter(world)
        .find(|(t, _)| t.0 == team_id)
        .and_then(|(_, n)| n.map(|n| n.0.clone()))
        .unwrap_or_else(|| format!("Team {team_id}"))
}

/// Build a full [`MatchPlayback`] for `home_id` vs `away_id` from their current squads, seeded
/// by `seed` (same seed → identical playback). Pure over the world read; no mutation.
pub fn simulate_match_playback(
    world: &mut World,
    home_id: u32,
    away_id: u32,
    seed: u64,
) -> MatchPlayback {
    let home_players = team_players(world, home_id);
    let away_players = team_players(world, away_id);
    let home_lineup = lineup_of(world, home_id);
    let away_lineup = lineup_of(world, away_id);

    // The match itself: the engine's own deterministic stream.
    let mut match_rng = Pcg64Mcg::seed_from_u64(seed);
    let res = simulate_match(&home_lineup, &away_lineup, &mut match_rng);

    // Flavour (scorers, bookings) on a separate stream so it can't disturb the goals above.
    let mut flavour = Pcg64Mcg::seed_from_u64(derive_seed(seed, &[0xF1A7]));

    let mut events: Vec<PlayEvent> = Vec::new();
    for c in &res.timeline {
        if !c.goal {
            continue;
        }
        let (players, side) = if c.home { (&home_players, 0u8) } else { (&away_players, 1u8) };
        let scorer = pick_scorer(players, &mut flavour);
        events.push(PlayEvent {
            minute: c.minute,
            kind: "goal".into(),
            side,
            points: 1,
            title: format!("GOAL · {scorer}"),
            sub: if c.home { "Home strike".into() } else { "Away strike".into() },
        });
    }

    // A couple of bookings for texture, drawn from the flavour stream.
    let cards = flavour.gen_range(1..=4);
    for _ in 0..cards {
        let minute = flavour.gen_range(5..=88);
        let home_side = flavour.gen_bool(0.5);
        let players = if home_side { &home_players } else { &away_players };
        let name = players
            .iter()
            .filter(|p| p.position == POS_DEF || p.position == POS_MID)
            .min_by_key(|p| p.overall)
            .or_else(|| players.first())
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".into());
        events.push(PlayEvent {
            minute,
            kind: "card".into(),
            side: u8::from(!home_side),
            points: 0,
            title: format!("Yellow card · {name}"),
            sub: "Late challenge".into(),
        });
    }
    events.sort_by_key(|e| e.minute);

    // Stats: shots/on-target/xG from the timeline, possession from the chance share, and a
    // little flavour for corners/fouls.
    let (mut hs, mut as_, mut hot, mut aot) = (0u32, 0u32, 0u32, 0u32);
    for c in &res.timeline {
        if c.home {
            hs += 1;
            hot += u32::from(c.on_target);
        } else {
            as_ += 1;
            aot += u32::from(c.on_target);
        }
    }
    let total_ch = (hs + as_).max(1);
    let poss_home = (f64::from(hs) / f64::from(total_ch) * 100.0).round();
    let corners_h = flavour.gen_range(2..=9);
    let corners_a = flavour.gen_range(2..=9);
    let fouls_h = flavour.gen_range(6..=15);
    let fouls_a = flavour.gen_range(6..=15);
    let stats = vec![
        StatLine { label: "Possession", home: poss_home, away: 100.0 - poss_home },
        StatLine { label: "Shots", home: f64::from(hs), away: f64::from(as_) },
        StatLine { label: "On Target", home: f64::from(hot), away: f64::from(aot) },
        StatLine { label: "Expected Goals", home: (res.home_xg * 10.0).round() / 10.0, away: (res.away_xg * 10.0).round() / 10.0 },
        StatLine { label: "Corners", home: f64::from(corners_h), away: f64::from(corners_a) },
        StatLine { label: "Fouls", home: f64::from(fouls_h), away: f64::from(fouls_a) },
    ];

    let home_name = club_name(world, home_id);
    let away_name = club_name(world, away_id);
    MatchPlayback {
        home: Side { crest: crest_of(&home_name), name: home_name, dots: formation_dots(&home_players, true) },
        away: Side { crest: crest_of(&away_name), name: away_name, dots: formation_dots(&away_players, false) },
        final_home: res.home_goals,
        final_away: res.away_goals,
        minutes: crate::engine::MATCH_MINUTES,
        events,
        stats,
    }
}

/// Convenience: build a playback for a team's next opponent. Uses the in-progress `Season`'s
/// next unplayed fixture when one exists, otherwise stages a friendly against the lowest other
/// team id. Returns `None` only if there is no opponent at all.
///
/// When a season is active the match is seeded **exactly** as the season would seed that fixture
/// when it is played, so the scoreline you watch is the one that gets recorded in the table —
/// the live view is a preview of the real result, not a separate exhibition.
pub fn next_match_playback(world: &mut World, team_id: u32) -> Option<MatchPlayback> {
    let (home, away, seed) = next_fixture(world, team_id)?;
    Some(simulate_match_playback(world, home, away, seed))
}

/// Find `team_id`'s next fixture as `(home, away, seed)`. In-season the seed matches the
/// season's per-fixture stream (see [`sim_core::seeded_parallel_map`]); otherwise it is a
/// stable friendly seed.
/// Summary of a team's next fixture for the Home screen.
#[derive(Serialize, Clone, Debug)]
pub struct FixtureInfo {
    pub comp: String,
    pub date: String,
    pub is_home: bool,
    pub home_id: u32,
    pub away_id: u32,
    pub home_name: String,
    pub away_name: String,
    pub home_crest: String,
    pub away_crest: String,
    pub home_form: Vec<String>,
    pub away_form: Vec<String>,
}

/// The managed team's next fixture with the fields the Home "Next Match" card shows.
pub fn next_fixture_info(world: &mut World, team_id: u32) -> Option<FixtureInfo> {
    let (home, away, coord) = next_fixture(world, team_id)?;
    let (comp, date, home_form, away_form) = match world.get_resource::<crate::season::Season>() {
        Some(s) => {
            let date = if (coord as usize) < s.schedule.len() {
                s.schedule.matchday(coord as usize).date.to_string()
            } else {
                "TBD".to_string()
            };
            (
                format!("League · Matchday {}", coord + 1),
                date,
                s.form_of(home).into_iter().map(String::from).collect(),
                s.form_of(away).into_iter().map(String::from).collect(),
            )
        }
        None => ("Friendly".to_string(), "Pre-season".to_string(), Vec::new(), Vec::new()),
    };
    let home_name = club_name(world, home);
    let away_name = club_name(world, away);
    Some(FixtureInfo {
        comp,
        date,
        is_home: home == team_id,
        home_id: home,
        away_id: away,
        home_crest: crest_of(&home_name),
        away_crest: crest_of(&away_name),
        home_name,
        away_name,
        home_form,
        away_form,
    })
}

fn next_fixture(world: &mut World, team_id: u32) -> Option<(u32, u32, u64)> {
    if let Some(season) = world.get_resource::<crate::season::Season>() {
        let sched = &season.schedule;
        for i in season.schedule.next_index()..sched.len() {
            let md = sched.matchday(i);
            if md.played {
                continue;
            }
            if let Some(k) = md.fixtures.iter().position(|&(h, a)| h == team_id || a == team_id) {
                let (h, a) = md.fixtures[k];
                // Mirror `simulate_matchday` -> `seeded_parallel_map(world_seed, [season, md])`,
                // item index k: derive_seed(derive_seed(world_seed, coords), [k]).
                let base =
                    derive_seed(season.world_seed, &[u64::from(season.season_id), i as u64]);
                return Some((h, a, derive_seed(base, &[k as u64])));
            }
        }
    }
    // No season: friendly against the lowest other club id, `team_id` at home.
    let world_seed = world.get_resource::<sim_core::SimSeed>().map_or(0, |s| s.0);
    let mut clubs: Vec<u32> = {
        let mut q = world.query_filtered::<&TeamId, With<Club>>();
        q.iter(world).map(|t| t.0).collect()
    };
    clubs.sort_unstable();
    let opp = clubs.into_iter().find(|&c| c != team_id)?;
    Some((team_id, opp, derive_seed(world_seed, &[u64::from(team_id), u64::from(opp), 0])))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn playback_has_two_full_lineups_and_matching_score() {
        let db = sample();
        let mut world = load_world(&db);
        let (h, a) = (db.clubs[0].id, db.clubs[1].id);
        let pb = simulate_match_playback(&mut world, h, a, 0xC0FFEE);

        assert_eq!(pb.home.dots.len(), 11);
        assert_eq!(pb.away.dots.len(), 11);
        // Every goal event's count matches the final score.
        let hg = pb.events.iter().filter(|e| e.kind == "goal" && e.side == 0).count();
        let ag = pb.events.iter().filter(|e| e.kind == "goal" && e.side == 1).count();
        assert_eq!(hg, usize::from(pb.final_home));
        assert_eq!(ag, usize::from(pb.final_away));
        // Events are minute-ordered and six stat rows are present.
        assert!(pb.events.windows(2).all(|w| w[0].minute <= w[1].minute));
        assert_eq!(pb.stats.len(), 6);
    }

    #[test]
    fn same_seed_gives_identical_playback() {
        let db = sample();
        let mut world = load_world(&db);
        let (h, a) = (db.clubs[0].id, db.clubs[1].id);
        let p1 = simulate_match_playback(&mut world, h, a, 42);
        let p2 = simulate_match_playback(&mut world, h, a, 42);
        assert_eq!(p1, p2);
    }

    #[test]
    fn next_match_falls_back_to_a_friendly_without_a_season() {
        let db = sample();
        let mut world = load_world(&db);
        let pb = next_match_playback(&mut world, db.clubs[0].id).expect("an opponent exists");
        assert_eq!(pb.home.dots.len(), 11);
    }

    #[test]
    fn watched_next_match_matches_the_season_result() {
        use crate::matchday::{gather_lineups, simulate_matchday, Fixture};
        use crate::season::Season;
        use sim_core::{Club, SimClock, TeamId};

        let db = sample();
        let mut world = load_world(&db);
        let mut teams: Vec<u32> =
            world.query_filtered::<&TeamId, With<Club>>().iter(&world).map(|t| t.0).collect();
        teams.sort_unstable();
        let today = world.resource::<SimClock>().date();
        let world_seed = world.get_resource::<sim_core::SimSeed>().map_or(0, |s| s.0);
        world.insert_resource(Season::new(teams.clone(), today, world_seed, 2025));

        let team = teams[0];
        // Pull the first unplayed matchday containing `team` and its fixture position.
        let (md_index, fixtures, k) = {
            let s = world.resource::<Season>();
            let mut found = None;
            for i in s.schedule.next_index()..s.schedule.len() {
                let md = s.schedule.matchday(i);
                if let Some(k) = md.fixtures.iter().position(|&(h, a)| h == team || a == team) {
                    found = Some((i, md.fixtures.clone(), k));
                    break;
                }
            }
            found.expect("team has a fixture")
        };

        // What the season records for that fixture.
        let (sid, wseed) = {
            let s = world.resource::<Season>();
            (s.season_id, s.world_seed)
        };
        let lineups = gather_lineups(&mut world);
        let fx: Vec<Fixture> = fixtures
            .iter()
            .map(|&(h, a)| Fixture { home: lineups[&h], away: lineups[&a] })
            .collect();
        let recorded = simulate_matchday(&fx, wseed, sid, md_index as u32)[k].clone();

        let pb = next_match_playback(&mut world, team).expect("a fixture playback");
        assert_eq!(pb.final_home, recorded.home_goals, "watched home score == recorded");
        assert_eq!(pb.final_away, recorded.away_goals, "watched away score == recorded");
    }
}
