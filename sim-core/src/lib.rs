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

pub mod club;
pub mod competition;
pub mod economy;
pub mod entity;
pub mod league;
pub mod lifecycle;
pub mod persistence;
pub mod pyramid;
pub mod rng;
pub mod schedule;
pub mod sim;
pub mod time;
pub mod transfers;

pub use club::{index_clubs, sync_squad_membership, Club, ClubBundle, ClubRegistry};
pub use competition::{double_round_robin, schedule_weekly, single_round_robin};
pub use economy::{Balance, Money, WeeklyIncome};
pub use entity::{
    age_years, BirthDate, Condition, Contract, FreeAgent, Morale, PersonBundle, Retired,
    SquadTarget, TeamId, WageDemand,
};
pub use league::{run_league_day, League, Matchday, Schedule};
pub use lifecycle::RETIREMENT_AGE;
pub use persistence::{
    read_save, write_save, BincodeCodec, SaveCodec, SaveData, SaveError, FORMAT_VERSION,
};
pub use pyramid::Pyramid;
pub use rng::{derive_seed, SimSeed};
pub use schedule::build_daily_schedule;
pub use sim::seeded_parallel_map;
pub use time::{advance_time, Date, SimClock};
pub use transfers::run_transfer_window;
