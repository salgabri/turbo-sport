//! Deterministic parallel simulation — the one piece of cross-sport infrastructure
//! harvested from football and cycling (build step 7B).
//!
//! Both sports independently needed the identical pattern: run a pure per-item simulation
//! across cores with `rayon`, giving each item an independently *seeded* RNG so the result
//! is the same regardless of thread scheduling. That determinism contract — seed derived
//! from the world seed plus stable coordinates plus the item index, never shared between
//! threads — is subtle enough to deserve a single audited home instead of a copy in every
//! sport.
//!
//! What is deliberately **not** here: a `MatchEngine` / `CompetitionFormat` trait. Once a
//! second sport existed, football (two lineups → one result) and cycling (N riders → N
//! times, accumulated into a general classification) turned out to have genuinely
//! different engine and standings shapes. The illustrative `MatchEngine { simulate(home,
//! away) }` once sketched in `docs/ARCHITECTURE.md` is football-shaped — cycling has no
//! home/away and cannot implement it. Forcing both into one trait now would be exactly the
//! premature abstraction constraint #4 warns against, so the sports keep their own concrete
//! engines and only this infrastructure is shared.

use crate::rng::derive_seed;
use rand_core::SeedableRng;
use rayon::prelude::*;

/// Run `simulate` over every item in parallel, giving each item its own deterministically
/// seeded RNG.
///
/// Item `i` is simulated with an RNG seeded from
/// `derive_seed(derive_seed(world_seed, coords), &[i])`. No RNG is shared between items, so
/// the output is identical on every run and on any number of cores, and element order
/// matches input order. `coords` names the stream group (e.g. `[season, matchday]` or
/// `[race_id, stage]`); the item index distinguishes items within it.
///
/// `R` is the RNG engine (e.g. `rand_pcg::Pcg64Mcg`), chosen by the caller so the engine
/// stays a sport-level decision rather than a `sim-core` one.
pub fn seeded_parallel_map<R, In, Out, F>(
    items: &[In],
    world_seed: u64,
    coords: &[u64],
    simulate: F,
) -> Vec<Out>
where
    R: SeedableRng,
    In: Sync,
    Out: Send,
    F: Fn(&In, &mut R) -> Out + Sync,
{
    let base = derive_seed(world_seed, coords);
    items
        .par_iter()
        .enumerate()
        .map(|(i, item)| {
            let mut rng = R::seed_from_u64(derive_seed(base, &[i as u64]));
            simulate(item, &mut rng)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::RngCore;
    use rand_pcg::Pcg64Mcg;

    // Capture each item's index and a draw from its stream.
    fn probe(items: &[u32], seed: u64, coords: &[u64]) -> Vec<(u32, u64)> {
        seeded_parallel_map::<Pcg64Mcg, _, _, _>(items, seed, coords, |x, rng| (*x, rng.next_u64()))
    }

    #[test]
    fn is_deterministic_and_order_preserving() {
        let items: Vec<u32> = (0..256).collect();
        let a = probe(&items, 42, &[1, 2]);
        let b = probe(&items, 42, &[1, 2]);
        assert_eq!(a, b, "same inputs must give identical output on every run");
        assert!(
            a.iter().map(|(x, _)| *x).eq(items.iter().copied()),
            "output order must match input order"
        );
    }

    #[test]
    fn streams_differ_by_coords_seed_and_index() {
        let items: Vec<u32> = (0..16).collect();
        let base = probe(&items, 42, &[1, 2]);
        assert_ne!(base, probe(&items, 42, &[1, 3]), "different coords -> different streams");
        assert_ne!(base, probe(&items, 43, &[1, 2]), "different world seed -> different streams");
        // Adjacent items draw different values (per-index seeding, not one shared stream).
        assert_ne!(base[0].1, base[1].1);
    }
}
