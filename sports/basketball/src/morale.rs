//! Morale dynamics for basketball — the court equivalent of `football::morale`.
//!
//! Each game nudges both rosters' morale: a win lifts it, a loss dents it (no draws). A plain
//! deterministic clamp-shift applied after the result, so a save reproduces the same mood.

use bevy_ecs::prelude::*;
use sim_core::{Morale, Retired, TeamId};

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

/// Apply a game result to both rosters' morale: winners up, losers down.
pub fn apply_game_morale(world: &mut World, home: u32, away: u32, home_pts: u16, away_pts: u16) {
    let (home_d, away_d) = if home_pts > away_pts { (4, -4) } else { (-4, 4) };
    shift(world, home, home_d);
    shift(world, away, away_d);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    fn mean(world: &mut World, team: u32) -> f64 {
        let mut q = world.query::<(&TeamId, &Morale)>();
        let v: Vec<u8> = q.iter(world).filter(|(t, _)| t.0 == team).map(|(_, m)| m.0).collect();
        v.iter().map(|&m| f64::from(m)).sum::<f64>() / v.len().max(1) as f64
    }

    #[test]
    fn a_win_lifts_a_loss_dents() {
        let db = sample();
        let mut world = load_world(&db);
        let (h, a) = (db.clubs[0].id, db.clubs[1].id);
        let (bh, ba) = (mean(&mut world, h), mean(&mut world, a));
        apply_game_morale(&mut world, h, a, 110, 98);
        assert!(mean(&mut world, h) > bh);
        assert!(mean(&mut world, a) < ba);
    }
}
