//! Morale dynamics: match results nudge every squad member's morale.
//!
//! `sim-core` stores [`Morale`] but nothing moved it — it sat at its authored value. Here each
//! played match shifts both squads: a win lifts morale, a defeat dents it, a draw barely moves
//! it. It is a plain deterministic delta (no RNG), applied single-threaded after the result, so
//! a save reproduces the same dressing-room mood. Bigger drivers (playing time, board
//! confidence, personalities) are later refinements.

use bevy_ecs::prelude::*;
use sim_core::{Morale, Retired, TeamId};

/// Clamp-shift one team's morale by `delta`.
fn shift(world: &mut World, team: u32, delta: i16) {
    let ids: Vec<Entity> = {
        let mut q = world.query_filtered::<(Entity, &TeamId), Without<Retired>>();
        q.iter(world).filter(|(_, t)| t.0 == team).map(|(e, _)| e).collect()
    };
    for e in ids {
        if let Some(mut m) = world.get_mut::<Morale>(e) {
            m.0 = (i16::from(m.0) + delta).clamp(0, 100) as u8;
        }
    }
}

/// Apply a result to both squads' morale: winners up, losers down, draws near-neutral.
pub fn apply_match_morale(world: &mut World, home: u32, away: u32, home_goals: u8, away_goals: u8) {
    let (home_d, away_d) = match home_goals.cmp(&away_goals) {
        std::cmp::Ordering::Greater => (4, -5),
        std::cmp::Ordering::Equal => (1, -1),
        std::cmp::Ordering::Less => (-5, 4),
    };
    shift(world, home, home_d);
    shift(world, away, away_d);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    fn mean_morale(world: &mut World, team: u32) -> f64 {
        let mut q = world.query::<(&TeamId, &Morale)>();
        let vals: Vec<u8> = q.iter(world).filter(|(t, _)| t.0 == team).map(|(_, m)| m.0).collect();
        vals.iter().map(|&m| f64::from(m)).sum::<f64>() / vals.len().max(1) as f64
    }

    #[test]
    fn a_win_lifts_morale_a_loss_dents_it() {
        let db = sample();
        let mut world = load_world(&db);
        let (h, a) = (db.clubs[0].id, db.clubs[1].id);
        let before_h = mean_morale(&mut world, h);
        let before_a = mean_morale(&mut world, a);
        apply_match_morale(&mut world, h, a, 3, 0);
        assert!(mean_morale(&mut world, h) > before_h);
        assert!(mean_morale(&mut world, a) < before_a);
    }

    #[test]
    fn morale_stays_in_range() {
        let db = sample();
        let mut world = load_world(&db);
        let (h, a) = (db.clubs[0].id, db.clubs[1].id);
        for _ in 0..60 {
            apply_match_morale(&mut world, h, a, 0, 5); // home hammered repeatedly
        }
        assert!((0.0..=100.0).contains(&mean_morale(&mut world, h)));
    }
}
