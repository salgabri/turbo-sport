//! `cycling` — the second sport, on top of `sim-core`. Build step 7A.
//!
//! Built **concretely**, mirroring football's structure (attributes / engine / runner)
//! on purpose: the duplication and the *differences* between the two crates are the raw
//! material for harvesting the shared trait surface in step 7B. Per constraint #4, no
//! shared traits are introduced here — that comes only now that a second sport exists.

pub mod attributes;
pub mod database;
pub mod race;
pub mod stage;

pub use attributes::{Rider, StageType};
pub use database::{load_world, sample, CyclingAbility, Database};
pub use race::{gather_riders, simulate_race, GcEntry, Race};
pub use stage::{simulate_stage, StageTime};
