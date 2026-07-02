//! Per-player season statistics: appearances and goals, credited deterministically after each
//! match the season plays.
//!
//! The engine produces a scoreline; this attributes it. The starting XI (the 11 highest-rated
//! available players) each get an appearance, and the team's goals are handed to scorers picked
//! — weighted by shooting, attackers favoured — from a stream seeded off the fixture's match
//! coordinates, so the same season produces the same scorers every run. Football-specific
//! because it reads football attributes; kept out of the pure engine so determinism there is
//! untouched.

use crate::attributes::{Footballer, POS_FWD, POS_MID};
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use sim_core::{PositionGroup, Retired, TeamId};
use std::collections::HashMap;

/// One player's running season tally.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FootballTally {
    pub apps: u16,
    pub goals: u16,
}

/// A player reduced to what tally-crediting needs.
struct XiPlayer {
    entity: Entity,
    sho: u8,
    position: u8,
    overall: u8,
}

/// The starting XI for `team_id`: the 11 highest-rated available players.
fn team_xi(world: &mut World, team_id: u32) -> Vec<XiPlayer> {
    let mut v: Vec<XiPlayer> = {
        let mut q = world
            .query_filtered::<(Entity, &TeamId, &Footballer, Option<&PositionGroup>), Without<Retired>>();
        q.iter(world)
            .filter(|(_, t, ..)| t.0 == team_id)
            .map(|(entity, _, f, pos)| {
                let position = pos.map_or(POS_MID, |p| p.0);
                XiPlayer { entity, sho: f.sho, position, overall: f.overall(position) }
            })
            .collect()
    };
    v.sort_by_key(|p| std::cmp::Reverse(p.overall));
    v.truncate(11);
    v
}

/// Pick a scorer from the XI, weighted by shooting; forwards and midfielders favoured.
fn pick_scorer(xi: &[XiPlayer], rng: &mut Pcg64Mcg) -> Entity {
    let attackers: Vec<&XiPlayer> =
        xi.iter().filter(|p| p.position == POS_FWD || p.position == POS_MID).collect();
    let pool = if attackers.is_empty() { xi.iter().collect() } else { attackers };
    let total: u32 = pool.iter().map(|p| u32::from(p.sho).max(1)).sum();
    let mut r = rng.gen_range(0..total);
    for p in &pool {
        let w = u32::from(p.sho).max(1);
        if r < w {
            return p.entity;
        }
        r -= w;
    }
    pool[0].entity
}

/// Credit one played fixture into both teams' player tallies. `seed` should be derived from the
/// fixture's match coordinates so scorers are reproducible.
pub fn credit_match(world: &mut World, home: u32, away: u32, home_goals: u8, away_goals: u8, seed: u64) {
    let mut credits: HashMap<Entity, (u16, u16)> = HashMap::new();
    for (team, goals) in [(home, home_goals), (away, away_goals)] {
        let xi = team_xi(world, team);
        if xi.is_empty() {
            continue;
        }
        for p in &xi {
            credits.entry(p.entity).or_default().0 += 1; // appearance
        }
        let mut rng = Pcg64Mcg::seed_from_u64(seed ^ u64::from(team).wrapping_mul(0x9E37));
        for _ in 0..goals {
            let scorer = pick_scorer(&xi, &mut rng);
            credits.entry(scorer).or_default().1 += 1;
        }
    }
    for (e, (apps, goals)) in credits {
        let mut em = world.entity_mut(e);
        match em.get_mut::<FootballTally>() {
            Some(mut t) => {
                t.apps += apps;
                t.goals += goals;
            }
            None => {
                em.insert(FootballTally { apps, goals });
            }
        }
    }
}

/// Zero every player's tally — call when a new season begins.
pub fn reset_tallies(world: &mut World) {
    let mut q = world.query::<&mut FootballTally>();
    for mut t in q.iter_mut(world) {
        *t = FootballTally::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};
    use sim_core::TeamId;

    #[test]
    fn crediting_a_result_records_apps_and_goals() {
        let db = sample();
        let mut world = load_world(&db);
        let team = db.clubs[0].id;
        credit_match(&mut world, team, db.clubs[1].id, 3, 1, 0xABCD);

        // Home team: 11 appearances and exactly 3 goals distributed among them.
        let (mut apps, mut goals) = (0u32, 0u32);
        let mut q = world.query::<(&TeamId, &FootballTally)>();
        for (t, tal) in q.iter(&world) {
            if t.0 == team {
                apps += u32::from(tal.apps);
                goals += u32::from(tal.goals);
            }
        }
        assert_eq!(apps, 11, "the XI each get an appearance");
        assert_eq!(goals, 3, "the three goals are attributed");
    }

    #[test]
    fn crediting_is_deterministic() {
        let db = sample();
        let scorers = || {
            let mut w = load_world(&db);
            credit_match(&mut w, db.clubs[0].id, db.clubs[1].id, 4, 2, 0x1234);
            let mut q = w.query::<(&sim_core::Name, &FootballTally)>();
            let mut out: Vec<(String, u16)> =
                q.iter(&w).filter(|(_, t)| t.goals > 0).map(|(n, t)| (n.0.clone(), t.goals)).collect();
            out.sort();
            out
        };
        assert_eq!(scorers(), scorers());
    }
}
