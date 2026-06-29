//! `sim-core` — the sport-agnostic simulation engine.
//!
//! This crate owns everything a sport-management sim needs that is *not* specific to
//! one sport: the ECS world, the simulation clock and calendar, deterministic seeding,
//! and (later) entity lifecycle, economy, and versioned persistence. Sport crates
//! depend on this crate and implement the sport-specific simulation.
//!
//! Scope discipline (see `CLAUDE.md`): the trait surface that football and cycling will
//! share is **not** defined here yet. It gets harvested from a complete football
//! implementation once a second sport reveals where the real boundaries are. Designing
//! those traits before two sports exist reliably produces the wrong abstraction.
//!
//! What exists today is build-order step 2: a world that can advance simulated time
//! from days to seasons.

pub mod rng;
pub mod time;

pub use rng::{derive_seed, SimSeed};
pub use time::{advance_time, Date, SimClock};
