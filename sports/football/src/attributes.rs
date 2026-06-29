//! Football's player ability schema — the sport-specific components that `sim-core`
//! deliberately does not know about (it owns birth date, contract, condition; the sport
//! owns what makes someone good *at football*).
//!
//! This is intentionally football-shaped, not a generalised "attributes" abstraction.
//! Per `CLAUDE.md` constraint #4, the shared trait surface is harvested only once a
//! second sport (cycling) exists to reveal where the real boundaries are — so cycling
//! will define its own climbing/sprinting components rather than reusing these.

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

/// Which team a player belongs to. A lightweight integer id for now; once clubs are full
/// entities with squads this becomes a relation to the club entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TeamId(pub u32);
