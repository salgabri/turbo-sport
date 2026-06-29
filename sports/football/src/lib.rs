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
pub mod engine;
pub mod matchday;

pub use attributes::{Footballer, TeamId};
pub use engine::{simulate_match, Chance, Lineup, MatchResult, MATCH_MINUTES};
pub use matchday::{gather_lineups, simulate_matchday, Fixture};
