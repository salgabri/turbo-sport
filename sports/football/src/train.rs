//! Player development: a deterministic, age-curved training step.
//!
//! Youngsters below their potential improve — biased by the manager's [`TrainingFocus`] group,
//! or by their position when focus is "balanced" — and players in their thirties decline
//! physically. It is a **pure function of age + attributes** (no RNG), so a save develops
//! identically on every run — determinism holds for progression exactly as it does for matches.
//!
//! Football-specific because it nudges football attributes; the generic pieces (age, the opaque
//! `TrainingFocus`/`PositionGroup` group indices) live in `sim-core`. Ratings and market values
//! are recomputed from the new attributes so the whole card stays consistent.

use crate::attributes::{Footballer, POS_DEF, POS_GK, POS_MID};
use bevy_ecs::prelude::*;
use sim_core::{
    age_years, value_from, BirthDate, MarketValue, PositionGroup, Rating, SimClock, TrainingFocus,
};

/// Football [`TrainingFocus`] group indices (match the design's attribute groups).
pub const FOCUS_TECHNICAL: u8 = 0;
pub const FOCUS_PHYSICAL: u8 = 1;
pub const FOCUS_MENTAL: u8 = 2;

/// Age past which a player no longer improves.
const PEAK_AGE: u32 = 30;
/// Age at which physical decline begins.
const DECLINE_AGE: u32 = 31;

fn bump(v: &mut u8, d: i16) {
    *v = (i16::from(*v) + d).clamp(1, 99) as u8;
}

/// Apply one development step to every footballer in the world. Deterministic. Call it on a
/// cadence (e.g. monthly, or at each season boundary) from the host loop.
pub fn develop(world: &mut World) {
    let today = world.resource::<SimClock>().date();
    let mut q = world.query::<(
        &mut Footballer,
        &BirthDate,
        &mut Rating,
        Option<&PositionGroup>,
        Option<&TrainingFocus>,
        Option<&mut MarketValue>,
    )>();
    for (mut f, birth, mut rating, pos, focus, value) in q.iter_mut(world) {
        let age = age_years(birth.0, today);
        let position = pos.map_or(POS_MID, |p| p.0);
        develop_one(&mut f, &mut rating, position, focus.map(|x| x.0), age);
        if let Some(mut mv) = value {
            mv.0 = value_from(rating.overall, age);
        }
    }
}

/// The per-player rule. `rating.overall` is recomputed and clamped to `potential` as its ceiling.
fn develop_one(f: &mut Footballer, rating: &mut Rating, position: u8, focus: Option<u8>, age: u32) {
    if age < PEAK_AGE && rating.overall < rating.potential {
        match focus {
            Some(FOCUS_TECHNICAL) => {
                bump(&mut f.sho, 1);
                bump(&mut f.pas, 1);
                bump(&mut f.dri, 1);
                bump(&mut f.tec, 1);
            }
            Some(FOCUS_PHYSICAL) => {
                bump(&mut f.pac, 1);
                bump(&mut f.phy, 1);
            }
            Some(FOCUS_MENTAL) => {
                bump(&mut f.def, 1);
                bump(&mut f.vis, 1);
            }
            // balanced: lean into the attributes the position values most
            _ => match position {
                POS_GK => {
                    bump(&mut f.gk, 1);
                    bump(&mut f.tec, 1);
                }
                POS_DEF => {
                    bump(&mut f.def, 1);
                    bump(&mut f.phy, 1);
                }
                POS_MID => {
                    bump(&mut f.pas, 1);
                    bump(&mut f.vis, 1);
                }
                _ => {
                    bump(&mut f.sho, 1);
                    bump(&mut f.pac, 1);
                }
            },
        }
    } else if age >= DECLINE_AGE {
        let d = if age >= 34 { -2 } else { -1 };
        bump(&mut f.pac, d);
        bump(&mut f.phy, d);
    }
    rating.overall = f.overall(position).min(rating.potential);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(v: u8) -> Footballer {
        Footballer { pac: v, sho: v, pas: v, dri: v, tec: v, def: v, phy: v, vis: v, gk: v }
    }

    #[test]
    fn young_player_grows_toward_potential_and_stops() {
        let mut f = flat(70);
        let mut r = Rating { overall: 70, potential: 80 };
        for _ in 0..60 {
            develop_one(&mut f, &mut r, POS_MID, None, 20);
        }
        assert!(r.overall > 70, "a youngster should improve");
        assert!(r.overall <= r.potential, "overall never exceeds potential");
        assert!(r.overall >= 79, "reaches near its potential given enough steps: {}", r.overall);
    }

    #[test]
    fn focus_biases_which_attributes_grow() {
        let mut f = flat(60);
        let mut r = Rating { overall: 60, potential: 99 };
        for _ in 0..5 {
            develop_one(&mut f, &mut r, POS_MID, Some(FOCUS_PHYSICAL), 19);
        }
        assert!(f.pac > 60 && f.phy > 60, "physical focus raises pace/strength");
        assert_eq!(f.tec, 60, "a non-physical attribute is untouched by physical focus");
    }

    #[test]
    fn veteran_declines() {
        let mut f = flat(82);
        let mut r = Rating { overall: 82, potential: 82 };
        let before = r.overall;
        for _ in 0..6 {
            develop_one(&mut f, &mut r, POS_MID, None, 35);
        }
        assert!(r.overall < before, "a 35-year-old should decline: {} !< {before}", r.overall);
    }

    #[test]
    fn peak_player_is_stable() {
        let mut f = flat(78);
        let mut r = Rating { overall: 78, potential: 90 };
        develop_one(&mut f, &mut r, POS_MID, None, 30); // exactly peak age, below potential
        assert_eq!(r.overall, 78, "a peak-age player neither grows nor declines this step");
    }
}
