// Turbo Sport — DTO types crossing the Tauri IPC boundary.
// Fields marked "(phase N)" are not produced by the backend yet; screens treat
// them as optional and fall back to a tasteful empty state until wired up.

export type ClubView = {
  team_id: number;
  name: string | null;
  balance: number;
  weekly_income: number;
  wage_bill: number;
  squad_size: number;
  // extensions (phase 2)
  avg_overall?: number | null;
  best_overall?: number | null;
  squad_value?: number | null;
};

export type PlayerView = {
  name: string | null;
  team_id: number | null;
  age: number | null;
  wage: number | null;
  contract_until: string | null;
  fitness: number | null;
  injured: boolean;
  morale: number | null;
  free_agent: boolean;
  retired: boolean;
  // extensions (phase 2/3) — sport attribute column zipped on by team_squad
  nationality?: string | null;
  position_group?: string | null;
  shirt?: number | null;
  overall?: number | null;
  potential?: number | null;
  market_value?: number | null;
  attrs?: Record<string, number> | null;
  // season tally (phase 3)
  apps?: number | null;
  goals?: number | null;
  assists?: number | null;
  rating?: number | null;
  captain?: boolean;
  suspended?: boolean;
  signed?: string | null;
};

export type BoardView = {
  objective: string;
  target_pos: number;
  current_pos: number | null;
  confidence: string;
  on_track: boolean;
};

export type StandingRow = {
  team_id: number;
  played: number;
  won: number;
  drawn: number;
  lost: number;
  goals_for: number;
  goals_against: number;
  goal_difference: number;
  points: number;
  form?: string[] | null; // (phase 3) last-N W/D/L
};

// ---- UI-side view models (composed in +page.svelte from the DTOs above) ----

export type Screen = "home" | "squad" | "profile" | "table" | "transfers" | "search" | "match";

export type SearchArgs = {
  position: number | null;
  minAge: number;
  maxAge: number;
  minOverall: number;
  freeOnly: boolean;
};

export type Kpi = {
  label: string;
  value: string;
  sub?: string;
  tone?: string;
  barPct?: number;
};

export type InboxItem = {
  type: string;
  icon: string;
  color: string;
  subject: string;
  preview: string;
  time: string;
  unread: boolean;
};

// ---- Live 2D match playback (from the `next_match` command) ----

export type Dot = { n: number; x: number; y: number };
export type MatchSide = { name: string; crest: string; dots: Dot[] };
export type PlayEvent = {
  minute: number;
  kind: string; // "goal" | "card" | "score" | "foul"
  side: number; // 0 home, 1 away
  points: number; // score contribution (football goal = 1, basketball basket = 2/3)
  title: string;
  sub: string;
};
export type StatLine = { label: string; home: number; away: number };
export type MatchPlayback = {
  home: MatchSide;
  away: MatchSide;
  final_home: number;
  final_away: number;
  minutes: number;
  events: PlayEvent[];
  stats: StatLine[];
};
