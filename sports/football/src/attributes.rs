//! Football's player ability schema — the sport-specific component that `sim-core`
//! deliberately does not know about (it owns birth date, contract, condition; the sport
//! owns what makes someone good *at football*).
//!
//! Team identity is *not* here: `TeamId` is shared, sport-agnostic, and lives in
//! `sim-core` (football and basketball used identical copies). Only the football-shaped
//! ability set is sport-specific.

use bevy_ecs::prelude::*;
use sim_core::PositionGroup;

/// Football position groups, the opaque [`PositionGroup`] indices the UI maps to labels.
pub const POS_GK: u8 = 0;
pub const POS_DEF: u8 = 1;
pub const POS_MID: u8 = 2;
pub const POS_FWD: u8 = 3;

/// A footballer's abilities, 0..=99 each — the eight outfield attributes shown on the
/// player card (matching the design's radar) plus a dedicated `gk` used only by the engine
/// for keeper strength (goalkeeping is not one of the eight outfield ratings).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Footballer {
    pub pac: u8,
    pub sho: u8,
    pub pas: u8,
    pub dri: u8,
    pub tec: u8,
    pub def: u8,
    pub phy: u8,
    pub vis: u8,
    /// Keeper rating — engine input for the last line, not part of the outfield radar.
    pub gk: u8,
}

impl Footballer {
    fn mean(vals: &[u8]) -> f64 {
        let sum: u32 = vals.iter().map(|&v| u32::from(v)).sum();
        f64::from(sum) / vals.len() as f64
    }

    /// Attacking output the engine's chance model consumes (0..~99).
    pub fn attack(&self) -> f64 {
        Self::mean(&[self.pac, self.dri, self.pas, self.tec, self.sho])
    }
    /// Defensive solidity (0..~99).
    pub fn defense(&self) -> f64 {
        Self::mean(&[self.def, self.phy])
    }
    /// Finishing quality when a chance falls (0..~99).
    pub fn finishing(&self) -> f64 {
        Self::mean(&[self.sho, self.tec])
    }
    /// Keeper strength (0..~99).
    pub fn keeper(&self) -> f64 {
        f64::from(self.gk)
    }

    /// Position-weighted overall rating (0..=99), deterministic and pure. Each position values
    /// a different mix; a keeper's overall is driven by `gk`.
    pub fn overall(&self, position: u8) -> u8 {
        let a = |v: u8| f64::from(v);
        let ovr = match position {
            POS_GK => a(self.gk) * 0.85 + a(self.tec) * 0.08 + a(self.phy) * 0.07,
            POS_DEF => {
                a(self.def) * 0.34
                    + a(self.phy) * 0.20
                    + a(self.pac) * 0.14
                    + a(self.pas) * 0.12
                    + a(self.tec) * 0.10
                    + a(self.vis) * 0.10
            }
            POS_MID => {
                a(self.pas) * 0.20
                    + a(self.tec) * 0.18
                    + a(self.vis) * 0.18
                    + a(self.dri) * 0.16
                    + a(self.pac) * 0.16
                    + a(self.def) * 0.12
            }
            // FWD and anything unknown fall back to an attacking weighting.
            _ => {
                a(self.sho) * 0.30
                    + a(self.pac) * 0.20
                    + a(self.dri) * 0.20
                    + a(self.vis) * 0.16
                    + a(self.tec) * 0.14
            }
        };
        ovr.round().clamp(0.0, 99.0) as u8
    }

    /// Convenience: overall given a [`PositionGroup`] component.
    pub fn overall_for(&self, pos: PositionGroup) -> u8 {
        self.overall(pos.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(base: u8) -> Footballer {
        Footballer {
            pac: base,
            sho: base,
            pas: base,
            dri: base,
            tec: base,
            def: base,
            phy: base,
            vis: base,
            gk: base,
        }
    }

    #[test]
    fn overall_of_a_flat_player_is_that_value() {
        // Every weighting sums to 1, so a player rated `base` everywhere is `base` overall.
        let p = player(70);
        for pos in [POS_GK, POS_DEF, POS_MID, POS_FWD] {
            assert_eq!(p.overall(pos), 70, "position {pos}");
        }
    }

    #[test]
    fn engine_aggregates_stay_on_the_0_99_scale() {
        let p = player(80);
        assert!((p.attack() - 80.0).abs() < 1e-9);
        assert!((p.defense() - 80.0).abs() < 1e-9);
        assert!((p.finishing() - 80.0).abs() < 1e-9);
        assert!((p.keeper() - 80.0).abs() < 1e-9);
    }
}
