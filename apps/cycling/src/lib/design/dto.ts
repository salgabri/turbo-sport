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

// ---- UI-side view models ----

export type Screen = "home" | "roster" | "race";

export type Kpi = {
  label: string;
  value: string;
  sub?: string;
  tone?: string;
  barPct?: number;
};
