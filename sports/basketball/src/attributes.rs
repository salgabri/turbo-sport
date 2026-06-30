//! Basketball's player ability schema — sport-specific. Team identity (`TeamId`) is shared
//! and lives in `sim-core`; only the ability set is basketball's own.

use bevy_ecs::prelude::*;

/// A basketball player's core abilities, 0..=100 each.
#[derive(Component, Clone, Copy, Debug)]
pub struct Baller {
    pub offense: u8,
    pub defense: u8,
    pub three_point: u8,
    pub rebounding: u8,
}
