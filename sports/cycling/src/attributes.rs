//! Cycling's rider ability schema — the sport-specific components `sim-core` does not
//! know about. Deliberately cycling-shaped (climbing, sprinting, time-trial), nothing
//! like football's; the contrast is the point (constraint #4), and it's what will reveal
//! the right shared boundary when the trait surface is harvested.

use bevy_ecs::prelude::*;

/// A rider's core abilities, 0..=100 each. Unlike football, the unit of competition is
/// the individual — there is no lineup to aggregate.
#[derive(Component, Clone, Copy, Debug)]
pub struct Rider {
    pub climbing: u8,
    pub sprinting: u8,
    pub time_trial: u8,
    pub endurance: u8,
}

/// The terrain profile of a stage, which decides what abilities matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageType {
    Flat,
    Hilly,
    Mountain,
    TimeTrial,
}
