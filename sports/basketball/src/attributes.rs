//! Basketball's player ability schema — sport-specific. Team identity (`TeamId`) is shared
//! and lives in `sim-core`; only the ability set is basketball's own.

use bevy_ecs::prelude::*;
use sim_core::PositionGroup;

/// Basketball position groups, the opaque [`PositionGroup`] indices the UI maps to labels.
pub const POS_G: u8 = 0;
pub const POS_F: u8 = 1;
pub const POS_C: u8 = 2;

/// The short label for a basketball position group index.
pub fn position_label(position: u8) -> &'static str {
    match position {
        POS_G => "G",
        POS_F => "F",
        _ => "C",
    }
}

/// A basketball player's abilities, 0..=99 each — the six design attributes shown on the
/// player card.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Baller {
    /// Inside scoring.
    pub ins: u8,
    /// Outside / three-point shooting.
    pub out: u8,
    /// Playmaking.
    pub pm: u8,
    /// Rebounding.
    pub reb: u8,
    /// Perimeter/interior defense.
    pub def: u8,
    /// Athleticism.
    pub ath: u8,
}

impl Baller {
    fn mean(vals: &[u8]) -> f64 {
        let sum: u32 = vals.iter().map(|&v| u32::from(v)).sum();
        f64::from(sum) / vals.len() as f64
    }

    /// Aggregate offense the game engine consumes (0..~99).
    pub fn offense(&self) -> f64 {
        Self::mean(&[self.ins, self.out, self.pm])
    }
    /// Aggregate defense (0..~99).
    pub fn defense(&self) -> f64 {
        Self::mean(&[self.def, self.ath])
    }
    /// Three-point tendency/quality (0..~99).
    pub fn three_point(&self) -> f64 {
        f64::from(self.out)
    }
    /// Rebounding (0..~99).
    pub fn rebounding(&self) -> f64 {
        f64::from(self.reb)
    }

    /// Position-weighted overall rating (0..=99), deterministic and pure.
    pub fn overall(&self, position: u8) -> u8 {
        let a = |v: u8| f64::from(v);
        let ovr = match position {
            POS_G => {
                a(self.pm) * 0.26
                    + a(self.out) * 0.22
                    + a(self.def) * 0.16
                    + a(self.ath) * 0.14
                    + a(self.ins) * 0.12
                    + a(self.reb) * 0.10
            }
            POS_C => {
                a(self.reb) * 0.26
                    + a(self.ins) * 0.24
                    + a(self.def) * 0.20
                    + a(self.ath) * 0.16
                    + a(self.out) * 0.08
                    + a(self.pm) * 0.06
            }
            // Forward and anything unknown: balanced weighting.
            _ => {
                a(self.ins) * 0.20
                    + a(self.def) * 0.18
                    + a(self.reb) * 0.18
                    + a(self.out) * 0.16
                    + a(self.ath) * 0.16
                    + a(self.pm) * 0.12
            }
        };
        ovr.round().clamp(0.0, 99.0) as u8
    }

    /// Convenience: overall given a [`PositionGroup`] component.
    pub fn overall_for(&self, pos: PositionGroup) -> u8 {
        self.overall(pos.0)
    }
}
