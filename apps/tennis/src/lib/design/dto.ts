// Turbo Sport — DTO types crossing the Tauri IPC boundary (tennis event app).
// Mirrors the backend structs in src-tauri/src/lib.rs exactly.

export type PlayerRow = {
  /** 0-indexed seed; display as seed + 1. */
  seed: number;
  name: string;
  serve: number;
  return_game: number;
  baseline: number;
  mental: number;
};

export type MatchRow = {
  winner: string;
  loser: string;
  /** already oriented winner-first, e.g. "2-0". */
  score: string;
};

export type RoundOut = {
  name: string;
  matches: MatchRow[];
};

export type Tourney = {
  champion: string;
  rounds: RoundOut[];
};

// ---- live match playback ----

export type SetScore = {
  home_games: number;
  away_games: number;
};

export type TieEvent = {
  /** 1-indexed set number. */
  set: number;
  /** 1-indexed game number within the whole tie (the clock index). */
  game: number;
  /** 0 = home, 1 = away. */
  side: number;
  title: string;
  sub: string;
};

export type TiePlayback = {
  home_name: string;
  away_name: string;
  /** display seed, 1 = top of this tie. */
  home_seed: number;
  away_seed: number;
  /** 0 = home won, 1 = away won. */
  winner_side: number;
  sets: SetScore[];
  feed: TieEvent[];
};

// ---- UI-side view models ----

export type Screen = "home" | "draw" | "bracket" | "match";
