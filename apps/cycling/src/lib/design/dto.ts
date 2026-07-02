// Turbo Sport — DTO types crossing the Tauri IPC boundary (cycling app).
// Cycling is an event app: a roster of riders + a general classification.

export type RiderRow = {
  name: string;
  climbing: number;
  sprinting: number;
  time_trial: number;
  endurance: number;
};

export type GcRow = {
  rank: number;
  name: string;
  gap_secs: number;
};

// ---- live stage playback (the "watch" experience) ----

export type RiderGap = {
  rank: number;
  name: string;
  gap_secs: number;
};

export type StagePlayback = {
  stage_name: string;
  km_total: number;
  gradient: string;
  profile: [number, number][];
  riders: RiderGap[];
  winner: string;
};

// ---- UI-side view models ----

export type Screen = "home" | "roster" | "race" | "stage";

export type Kpi = {
  label: string;
  value: string;
  sub?: string;
  tone?: string;
  barPct?: number;
};
