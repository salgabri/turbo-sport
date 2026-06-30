//! `tennis` — an individual sport with a knockout competition, on top of `sim-core`.
//!
//! The fourth sport, chosen for structural contrast: where football/basketball are team
//! leagues and cycling is an individual stage race, tennis is an individual single-
//! elimination bracket. It shares only `sim-core`'s deterministic parallel helper (matches
//! within a round are independent), and nothing of the leagues' competition logic — which
//! is the evidence that the right shared layer was infrastructure, not a competition trait.

pub mod attributes;
pub mod bracket;
pub mod database;
pub mod engine;

pub use attributes::{Seed, TennisPlayer};
pub use database::{load_world, sample, Database, TennisDbAbility};
pub use bracket::{gather_draw, simulate_tournament, BracketMatch, Tournament};
pub use engine::{simulate_match, MatchOutcome, Player};
