//! Tennis's player ability schema. Individuals, not teams — and they carry a tournament
//! seed rather than a team id.

use bevy_ecs::prelude::*;

/// A tennis player's core abilities, 0..=100 each.
#[derive(Component, Clone, Copy, Debug)]
pub struct TennisPlayer {
    pub serve: u8,
    pub return_game: u8,
    pub baseline: u8,
    /// Composure in tight moments — nudges deciding sets.
    pub mental: u8,
}

/// Tournament seeding, 0 = top seed. Determines bracket placement.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Seed(pub u32);
