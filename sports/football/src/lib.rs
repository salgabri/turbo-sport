//! `football` — the football sport implementation on top of `sim-core`.
//!
//! Build-order step 5. This crate is written **concretely**, not behind shared traits:
//! per `CLAUDE.md` constraint #4, the `MatchEngine` / `CompetitionFormat` abstractions
//! are harvested only after a second sport (cycling) exists to show where the real seams
//! are. Designing them from football alone would bake in football's shape.
//!
//! What's here today: football's player ability schema, the event-tick match engine, and
//! a rayon-parallel matchday runner with deterministic per-match seeding.

pub mod attributes;
pub mod career;
pub mod database;
pub mod engine;
pub mod matchday;
pub mod persistence;
pub mod season;

pub use attributes::Footballer;
pub use career::{generate_prospects, regen_youth};
pub use database::{load_world, sample, Database, FootballAbility, PlayerRecord};
pub use engine::{simulate_match, Chance, Lineup, MatchResult, MATCH_MINUTES};
pub use matchday::{gather_lineups, simulate_matchday, Fixture};
pub use season::{play_due_fixtures, rank_division, Season, TeamRecord};
// Team identity is shared; re-export so `football::TeamId` still resolves.
pub use sim_core::TeamId;
