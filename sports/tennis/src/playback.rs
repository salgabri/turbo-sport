//! Match **playback**: turn a single simulated tie into a UI-replayable experience.
//!
//! The [`crate::engine`] stays a pure `(p1, p2, rng) -> MatchOutcome` function — this module
//! reads the two strongest players out of the world, runs that engine for a deterministic
//! result, and then *dresses* it into a [`TiePlayback`]: a per-set game score and a
//! game-by-game event feed. The front-end replays it against a clock, so the "live tie" is a
//! deterministic recording, not a second simulation — same seed, same tie, every time.
//!
//! All flavour the engine does not itself produce (which side wins each set, how many games,
//! the odd 7-5) is drawn from a *separate* seeded stream keyed off the tie seed, so it never
//! perturbs the winner / set count the engine already fixed.

use crate::attributes::TennisPlayer;
use crate::engine::{simulate_match, Player};
use bevy_ecs::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use serde::Serialize;
use sim_core::{derive_seed, Name};

/// One completed set: games won by each side.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetScore {
    pub home_games: u8,
    pub away_games: u8,
}

/// A moment in the tie feed — emitted once per game. `side` is 0 = home, 1 = away.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TieEvent {
    /// 1-indexed set number this game belongs to.
    pub set: u32,
    /// 1-indexed game number within the whole tie (the UI's clock index).
    pub game: u32,
    pub side: u8,
    pub title: String,
    pub sub: String,
}

/// Everything the front-end needs to replay one tie as a scoreboard + feed.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TiePlayback {
    pub home_name: String,
    pub away_name: String,
    /// Display seed (1 = top of this tie, 2 = the other side).
    pub home_seed: u32,
    pub away_seed: u32,
    /// 0 = home won, 1 = away won.
    pub winner_side: u8,
    pub sets: Vec<SetScore>,
    pub feed: Vec<TieEvent>,
}

/// A player reduced to what the playback needs: display name + engine ratings.
struct PlayerLite {
    name: String,
    player: Player,
    /// Sum of ratings — the strength proxy used to pick the featured pair.
    strength: u32,
}

/// Gather every `(Name, TennisPlayer)` in the world, strongest (by rating sum) first.
fn ranked_players(world: &mut World) -> Vec<PlayerLite> {
    let mut q = world.query::<(&TennisPlayer, Option<&Name>)>();
    let mut v: Vec<PlayerLite> = q
        .iter(world)
        .map(|(t, name)| PlayerLite {
            name: name.map(|n| n.0.clone()).unwrap_or_else(|| "Unknown".into()),
            player: Player::new(
                f64::from(t.serve),
                f64::from(t.return_game),
                f64::from(t.baseline),
                f64::from(t.mental),
            ),
            strength: u32::from(t.serve) + u32::from(t.return_game) + u32::from(t.baseline) + u32::from(t.mental),
        })
        .collect();
    v.sort_by(|a, b| b.strength.cmp(&a.strength).then_with(|| a.name.cmp(&b.name)));
    v
}

/// Build a full [`TiePlayback`] for the two strongest players in the world, seeded by `seed`
/// (same seed → identical playback). Returns `None` if fewer than two players exist.
pub fn featured_match_playback(world: &mut World, seed: u64) -> Option<TiePlayback> {
    let ranked = ranked_players(world);
    if ranked.len() < 2 {
        return None;
    }
    let home = &ranked[0];
    let away = &ranked[1];

    // The tie itself: the engine's own deterministic stream.
    let mut match_rng = Pcg64Mcg::seed_from_u64(seed);
    let outcome = simulate_match(&home.player, &away.player, &mut match_rng);
    let winner_side = outcome.winner as u8;

    // Flavour (set order, game counts) on a separate stream so it can't disturb the outcome.
    let mut flavour = Pcg64Mcg::seed_from_u64(derive_seed(seed, &[0x7e]));

    // Set-winner sequence consistent with the outcome: the winner takes exactly 2 sets, the
    // total is sets.0 + sets.1. A 2-0 is [winner, winner]; a 2-1 puts the loser's win in the
    // middle: [winner, loser, winner].
    let total_sets = outcome.sets.0 + outcome.sets.1;
    let set_winners: Vec<u8> = if total_sets == 2 {
        vec![winner_side, winner_side]
    } else {
        vec![winner_side, 1 - winner_side, winner_side]
    };

    let mut sets: Vec<SetScore> = Vec::with_capacity(set_winners.len());
    let mut feed: Vec<TieEvent> = Vec::new();
    let mut game_index: u32 = 0;

    for (i, &set_winner) in set_winners.iter().enumerate() {
        let set_no = i as u32 + 1;
        // Winner's game count: usually 6, occasionally a 7-5.
        let (winner_games, loser_games): (u8, u8) = if flavour.gen_bool(0.18) {
            (7, 5)
        } else {
            (6, flavour.gen_range(2..=4))
        };
        let (home_games, away_games) = if set_winner == 0 {
            (winner_games, loser_games)
        } else {
            (loser_games, winner_games)
        };
        sets.push(SetScore { home_games, away_games });

        // Emit one event per game. Order games so the set winner reaches their total last:
        // interleave loser games among the winner's until the loser is exhausted, then the
        // winner closes it out. `running_*` reflect the score *after* the game just played.
        let (mut running_home, mut running_away) = (0u8, 0u8);
        let mut wg = 0u8; // winner games played so far
        let mut lg = 0u8; // loser games played so far
        while running_home < home_games || running_away < away_games {
            // Give the loser a game roughly every other game while they have some left; always
            // let the winner take the final game of the set.
            let winner_left = if set_winner == 0 { home_games - running_home } else { away_games - running_away };
            let loser_left = if set_winner == 0 { away_games - running_away } else { home_games - running_home };
            let winner_plays = loser_left == 0 || winner_left == 1 || (wg <= lg && winner_left > 0);
            let this_side = if winner_plays { set_winner } else { 1 - set_winner };
            if winner_plays {
                wg += 1;
            } else {
                lg += 1;
            }
            if this_side == 0 {
                running_home += 1;
            } else {
                running_away += 1;
            }
            game_index += 1;
            let scorer = if this_side == 0 { &home.name } else { &away.name };
            feed.push(TieEvent {
                set: set_no,
                game: game_index,
                side: this_side,
                title: format!("Game, {scorer}"),
                sub: format!("{running_home}-{running_away}, Set {set_no}"),
            });
        }
    }

    Some(TiePlayback {
        home_name: home.name.clone(),
        away_name: away.name.clone(),
        home_seed: 1,
        away_seed: 2,
        winner_side,
        sets,
        feed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, sample};

    #[test]
    fn same_seed_gives_identical_playback() {
        let db = sample();
        let mut world = load_world(&db);
        let p1 = featured_match_playback(&mut world, 42).expect("two players exist");
        let p2 = featured_match_playback(&mut world, 42).expect("two players exist");
        assert_eq!(p1, p2);
    }

    #[test]
    fn winner_holds_two_sets_and_per_set_games_are_consistent() {
        let db = sample();
        let mut world = load_world(&db);
        for seed in 0..200u64 {
            let pb = featured_match_playback(&mut world, seed).expect("two players exist");

            // Winner takes exactly two sets.
            let winner = pb.winner_side;
            let sets_won = pb
                .sets
                .iter()
                .filter(|s| if winner == 0 { s.home_games > s.away_games } else { s.away_games > s.home_games })
                .count();
            assert_eq!(sets_won, 2, "seed {seed}: winner should hold 2 sets");
            assert!(pb.sets.len() == 2 || pb.sets.len() == 3, "seed {seed}: best-of-3");

            // Every set has a legal game score: winner > loser and either 6-x or 7-5/7-6.
            for s in &pb.sets {
                let (hi, lo) = (s.home_games.max(s.away_games), s.home_games.min(s.away_games));
                assert!(hi > lo, "seed {seed}: a set must have a winner");
                let legal = (hi == 6 && lo <= 4) || (hi == 7 && (lo == 5 || lo == 6));
                assert!(legal, "seed {seed}: illegal set {hi}-{lo}");
            }

            // The feed emits one event per game and its games are sequentially indexed. Each
            // set's per-side event counts match that set's game score.
            let total_games: u32 = pb.sets.iter().map(|s| u32::from(s.home_games) + u32::from(s.away_games)).sum();
            assert_eq!(pb.feed.len() as u32, total_games, "seed {seed}: one event per game");
            for (i, ev) in pb.feed.iter().enumerate() {
                assert_eq!(ev.game, i as u32 + 1, "seed {seed}: game index is sequential");
            }
            for (i, s) in pb.sets.iter().enumerate() {
                let set_no = i as u32 + 1;
                let h = pb.feed.iter().filter(|e| e.set == set_no && e.side == 0).count();
                let a = pb.feed.iter().filter(|e| e.set == set_no && e.side == 1).count();
                assert_eq!(h, usize::from(s.home_games), "seed {seed}: set {set_no} home games");
                assert_eq!(a, usize::from(s.away_games), "seed {seed}: set {set_no} away games");
            }
        }
    }

    #[test]
    fn none_when_fewer_than_two_players() {
        let mut world = World::new();
        assert!(featured_match_playback(&mut world, 0).is_none());
    }
}
