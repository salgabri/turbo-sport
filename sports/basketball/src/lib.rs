//! `basketball` — a second team sport on top of `sim-core`.
//!
//! Built concretely like football, but it reuses two harvested pieces: the round-robin
//! scheduler (`sim_core::competition`) and the deterministic parallel helper
//! (`sim_core::seeded_parallel_map`). What it keeps to itself: the possession engine and
//! the win/loss standings — basketball has no draws, so it cannot share football's 3/1/0
//! table. The contrast is what justified harvesting the scheduler but not the table.

pub mod attributes;
pub mod engine;
pub mod matchday;
pub mod season;

pub use attributes::{Baller, TeamId};
pub use engine::{simulate_game, GameResult, Roster};
pub use matchday::{gather_rosters, simulate_matchday, Fixture};
pub use season::{play_due_fixtures, Matchday, Season, TeamRecord};
