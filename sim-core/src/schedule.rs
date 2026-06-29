//! Assembly of the daily simulation schedule from the cross-cutting subsystems.
//!
//! Kept in its own module so no subsystem (lifecycle, economy, …) has to depend on
//! another merely to be scheduled together — they each just expose systems, and this
//! module wires them into one ordered tick.

use crate::economy::weekly_finances;
use crate::lifecycle::{age_and_retire, expire_contracts, recover_condition};
use crate::time::advance_time;
use bevy_ecs::prelude::*;

/// Build the schedule that simulates exactly one day per `run(&mut world)`.
///
/// The order is fixed with `.chain()` for determinism (see `docs/ARCHITECTURE.md`): the
/// clock advances first so every later system sees the new date; per-person lifecycle
/// runs next; club finances run last, so a contract that expired today is already gone
/// from the wage bill. Competition and AI systems slot into this same sequence at their
/// build steps.
pub fn build_daily_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            advance_time,
            recover_condition,
            expire_contracts,
            age_and_retire,
            weekly_finances,
        )
            .chain(),
    );
    schedule
}
