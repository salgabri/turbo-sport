//! Deterministic seeding.
//!
//! The simulation is deterministic (see `docs/ARCHITECTURE.md`, "Determinism"): the
//! same save + the same seed + the same input reproduce the same result on every run.
//! The way that survives `rayon`'s nondeterministic scheduling is to never share a
//! mutable RNG across parallel work. Instead, every random stream is *derived* from the
//! world's master seed plus stable coordinates (season, matchday, match id, …), so each
//! unit of parallel work owns an independent, reproducible stream and execution order
//! cannot affect any outcome.
//!
//! This module deliberately holds only the master seed and the seed-derivation recipe.
//! Actual RNG engines (and the choice of `rand`/`rand_pcg`/etc.) belong to the match
//! engine at its build-order step, not to the foundation.

use bevy_ecs::prelude::*;

/// The world's master seed. Everything random derives from this; persist it in the
/// save so a loaded game keeps reproducing the same outcomes.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimSeed(pub u64);

/// splitmix64 finalizer. Chosen because it is tiny, dependency-free, and **stable**:
/// the bit pattern is fixed by these constants, not by a library version or the host
/// platform — essential for cross-version, cross-machine reproducibility.
#[inline]
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Derive a per-stream seed from the master seed and a list of stable coordinates.
///
/// This is the concrete form of the recipe in `CLAUDE.md`
/// (`seed = hash(world_seed, season, matchday, match_id)`), generalised to any number
/// of coordinates so different subsystems can name their streams however fits. It is a
/// pure function: same inputs → same output, always.
///
/// ```
/// use sim_core::rng::derive_seed;
/// // Order matters; distinct coordinates give distinct streams.
/// let a = derive_seed(42, &[/* season */ 2026, /* matchday */ 3, /* match */ 17]);
/// let b = derive_seed(42, &[2026, 3, 18]);
/// assert_ne!(a, b);
/// assert_eq!(a, derive_seed(42, &[2026, 3, 17])); // reproducible
/// ```
pub fn derive_seed(world_seed: u64, coords: &[u64]) -> u64 {
    let mut acc = mix(world_seed);
    for (i, &c) in coords.iter().enumerate() {
        // Fold the position in so [1, 2] and [2, 1] (and trailing zeros) differ.
        acc = mix(acc ^ mix(c.wrapping_add(i as u64).wrapping_add(1)));
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_position_sensitive() {
        assert_eq!(derive_seed(7, &[1, 2, 3]), derive_seed(7, &[1, 2, 3]));
        assert_ne!(derive_seed(7, &[1, 2, 3]), derive_seed(7, &[3, 2, 1]));
        assert_ne!(derive_seed(7, &[1, 2]), derive_seed(7, &[1, 2, 0]));
        assert_ne!(derive_seed(7, &[1]), derive_seed(8, &[1]));
    }
}
