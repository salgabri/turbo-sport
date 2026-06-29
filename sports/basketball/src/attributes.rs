//! Basketball's player ability schema — sport-specific, like football's and cycling's.

use bevy_ecs::prelude::*;

/// A basketball player's core abilities, 0..=100 each.
#[derive(Component, Clone, Copy, Debug)]
pub struct Baller {
    pub offense: u8,
    pub defense: u8,
    pub three_point: u8,
    pub rebounding: u8,
}

/// Which team a player belongs to.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TeamId(pub u32);
