//! Per-player season statistics for basketball: games played and points scored.
//!
//! The engine yields a final score; this attributes it. The ~10-man rotation (the highest-rated
//! available players) each get a game, and the team's points are split across them in proportion
//! to their scoring (inside + outside + playmaking) — a deterministic, RNG-free distribution, so
//! the same season produces the same stat lines every run. Mirrors `football::tally`.

use crate::attributes::Baller;
use bevy_ecs::prelude::*;
use sim_core::{PositionGroup, Retired, TeamId};
use std::collections::HashMap;

/// One player's running season tally.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BasketballTally {
    pub games: u16,
    pub points: u16,
}

/// Rotation size that features in a game.
const ROTATION: usize = 10;

struct RotationPlayer {
    entity: Entity,
    offense: u32,
    overall: u8,
}

fn rotation(world: &mut World, team_id: u32) -> Vec<RotationPlayer> {
    let mut v: Vec<RotationPlayer> = {
        let mut q = world
            .query_filtered::<(Entity, &TeamId, &Baller, Option<&PositionGroup>), Without<Retired>>();
        q.iter(world)
            .filter(|(_, t, ..)| t.0 == team_id)
            .map(|(entity, _, b, pos)| {
                let position = pos.map_or(crate::attributes::POS_F, |p| p.0);
                let offense = u32::from(b.ins) + u32::from(b.out) + u32::from(b.pm);
                RotationPlayer { entity, offense: offense.max(1), overall: b.overall(position) }
            })
            .collect()
    };
    v.sort_by_key(|p| std::cmp::Reverse(p.overall));
    v.truncate(ROTATION);
    v
}

/// Credit one played game to both teams' player tallies.
pub fn credit_game(world: &mut World, home: u32, away: u32, home_pts: u16, away_pts: u16) {
    let mut credits: HashMap<Entity, (u16, u16)> = HashMap::new();
    for (team, pts) in [(home, home_pts), (away, away_pts)] {
        let rot = rotation(world, team);
        if rot.is_empty() {
            continue;
        }
        let total_off: u32 = rot.iter().map(|p| p.offense).sum::<u32>().max(1);
        let mut assigned: u16 = 0;
        for p in &rot {
            credits.entry(p.entity).or_default().0 += 1; // a game played
            let share = (u32::from(pts) * p.offense / total_off) as u16;
            credits.entry(p.entity).or_default().1 += share;
            assigned += share;
        }
        // Hand any rounding remainder to the top scorer (highest offense).
        if let Some(top) = rot.iter().max_by_key(|p| p.offense) {
            credits.entry(top.entity).or_default().1 += pts.saturating_sub(assigned);
        }
    }
    for (e, (games, points)) in credits {
        let mut em = world.entity_mut(e);
        match em.get_mut::<BasketballTally>() {
            Some(mut t) => {
                t.games += games;
                t.points += points;
            }
            None => {
                em.insert(BasketballTally { games, points });
            }
        }
    }
}

/// Zero every player's tally — call when a new season begins.
pub fn reset_tallies(world: &mut World) {
    let mut q = world.query::<&mut BasketballTally>();
    for mut t in q.iter_mut(world) {
        *t = BasketballTally::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn crediting_a_game_records_games_and_all_points() {
        let db = sample();
        let mut world = load_world(&db);
        let team = db.clubs[0].id;
        credit_game(&mut world, team, db.clubs[1].id, 110, 98);

        let (mut games, mut points) = (0u32, 0u32);
        let mut q = world.query::<(&TeamId, &BasketballTally)>();
        for (t, tal) in q.iter(&world) {
            if t.0 == team {
                games += u32::from(tal.games);
                points += u32::from(tal.points);
            }
        }
        assert_eq!(points, 110, "every point is attributed");
        assert!(games >= 10, "the rotation each played");
    }

    #[test]
    fn distribution_is_deterministic() {
        let db = sample();
        let tally = || {
            let mut w = load_world(&db);
            credit_game(&mut w, db.clubs[0].id, db.clubs[1].id, 104, 101);
            let mut q = w.query::<(&sim_core::Name, &BasketballTally)>();
            let mut v: Vec<(String, u16)> =
                q.iter(&w).filter(|(_, t)| t.points > 0).map(|(n, t)| (n.0.clone(), t.points)).collect();
            v.sort();
            v
        };
        assert_eq!(tally(), tally());
    }
}
