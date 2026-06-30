//! Football's player ability schema — the sport-specific component that `sim-core`
//! deliberately does not know about (it owns birth date, contract, condition; the sport
//! owns what makes someone good *at football*).
//!
//! Team identity is *not* here: `TeamId` is shared, sport-agnostic, and lives in
//! `sim-core` (football and basketball used identical copies). Only the football-shaped
//! ability set is sport-specific.

use bevy_ecs::prelude::*;

/// A footballer's core abilities, 0..=100 each. A compact set sufficient for the
/// event-tick engine; richer attributes (passing, pace, positioning) get added when the
/// engine grows to need them.
#[derive(Component, Clone, Copy, Debug)]
pub struct Footballer {
    pub attacking: u8,
    pub defending: u8,
    pub finishing: u8,
    pub goalkeeping: u8,
}
