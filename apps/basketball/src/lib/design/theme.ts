// Turbo Sport — per-sport theme config. Basketball for this app; the other
// sports get their own const when this is extracted to packages/ui.
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

export const BASKETBALL: SportTheme = {
  id: "basketball",
  name: "Basketball",
  short: "BBL",
  accent: "#f2913d",
  currency: "$",
  attrGroups: ["Offense", "Defense", "Physical"],
  attributes: [
    { key: "ins", label: "Inside", short: "INS", group: "Offense" },
    { key: "out", label: "Outside", short: "OUT", group: "Offense" },
    { key: "pm", label: "Playmaking", short: "PLY", group: "Offense" },
    { key: "reb", label: "Rebounding", short: "REB", group: "Defense" },
    { key: "def", label: "Defense", short: "DEF", group: "Defense" },
    { key: "ath", label: "Athleticism", short: "ATH", group: "Physical" },
  ],
  positions: [
    { group: "G", color: "#5aa9e6" },
    { group: "F", color: "#5ec98a" },
    { group: "C", color: "#ef6b6b" },
  ],
  matchVariant: "court",
  standingsVariant: "league",
};

/** Colour for a position group; falls back to muted grey. */
export function posColor(theme: SportTheme, group: string | null | undefined): string {
  if (!group) return "#7a828d";
  return theme.positions.find((p) => p.group === group)?.color ?? "#7a828d";
}
