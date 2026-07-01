// Turbo Sport — per-sport theme config. Cycling variant.
// Same SportTheme shape as football; only the sport data differs.
// Schemas taken from the Claude Design comps.

export type AttrDef = { key: string; label: string; short: string; group: string };
export type PosDef = { group: string; color: string };

export type SportTheme = {
  id: string;
  name: string;
  short: string;
  accent: string;
  currency: string;
  attrGroups: string[];
  attributes: AttrDef[];
  positions: PosDef[];
  /** which match/table variants this sport renders */
  matchVariant: "pitch" | "court" | "climb" | "track";
  standingsVariant: "league" | "race";
};

export const CYCLING: SportTheme = {
  id: "cycling",
  name: "Cycling",
  short: "CYC",
  accent: "#f2c14e",
  currency: "€",
  attrGroups: ["Ability"],
  attributes: [
    { key: "climbing", label: "Climbing", short: "CLM", group: "Ability" },
    { key: "sprinting", label: "Sprint", short: "SPR", group: "Ability" },
    { key: "time_trial", label: "Time Trial", short: "TT", group: "Ability" },
    { key: "endurance", label: "Endurance", short: "END", group: "Ability" },
  ],
  positions: [
    { group: "GC", color: "#f2c14e" },
    { group: "CLB", color: "#5ec98a" },
    { group: "SPR", color: "#ef6b6b" },
    { group: "TT", color: "#5aa9e6" },
    { group: "DOM", color: "#8b95a1" },
  ],
  matchVariant: "climb",
  standingsVariant: "race",
};

/** Colour for a position group; falls back to muted grey. */
export function posColor(theme: SportTheme, group: string | null | undefined): string {
  if (!group) return "#7a828d";
  return theme.positions.find((p) => p.group === group)?.color ?? "#7a828d";
}
