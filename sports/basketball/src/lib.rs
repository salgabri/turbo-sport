//! `basketball` — a second team sport on top of `sim-core`.
//!
//! Built concretely like football, but it reuses two harvested pieces: the round-robin
//! scheduler (`sim_core::competition`) and the deterministic parallel helper
//! (`sim_core::seeded_parallel_map`). What it keeps to itself: the possession engine and
//! the win/loss standings — basketball has no draws, so it cannot share football's 3/1/0
//! table. The contrast is what justified harvesting the scheduler but not the table.

pub mod attributes;
pub mod database;
pub mod engine;
pub mod injuries;
pub mod matchday;
pub mod morale;
pub mod persistence;
pub mod playback;
pub mod season;
pub mod tally;
pub mod view;

pub use attributes::Baller;
pub use injuries::{roll_game_injuries, severity};
pub use morale::apply_game_morale;
pub use playback::{next_match_playback, simulate_match_playback, MatchPlayback};
pub use tally::{credit_game, reset_tallies, BasketballTally};
pub use view::{
    free_agents as free_agents_detailed, team_squad as team_squad_detailed, top_scorers, Attrs,
    ScorerRow, SquadPlayer,
};
pub use database::{load_world, sample, BasketballAbility, Database};
pub use engine::{simulate_game, GameResult, Roster};
pub use matchday::{gather_rosters, simulate_matchday, Fixture};
pub use season::{play_due_fixtures, Season, TeamRecord};
// Team identity is shared; re-export so `basketball::TeamId` still resolves.
pub use sim_core::TeamId;
