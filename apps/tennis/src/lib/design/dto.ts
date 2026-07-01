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

// ---- UI-side view models ----

export type Screen = "home" | "draw" | "bracket";
