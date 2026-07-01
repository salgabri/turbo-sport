// Turbo Sport — per-sport theme config. Football for now; the other three
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

export const FOOTBALL: SportTheme = {
  id: "football",
  name: "Football",
  short: "FBL",
  accent: "#4fd08a",
  currency: "£",
  attrGroups: ["Technical", "Physical", "Mental"],
  attributes: [
    { key: "pac", label: "Pace", short: "PAC", group: "Physical" },
    { key: "sho", label: "Shooting", short: "SHO", group: "Technical" },
    { key: "pas", label: "Passing", short: "PAS", group: "Technical" },
    { key: "dri", label: "Dribbling", short: "DRI", group: "Technical" },
    { key: "tec", label: "Technique", short: "TEC", group: "Technical" },
    { key: "def", label: "Defending", short: "DEF", group: "Mental" },
    { key: "phy", label: "Strength", short: "PHY", group: "Physical" },
    { key: "vis", label: "Vision", short: "VIS", group: "Mental" },
  ],
  positions: [
    { group: "GK", color: "#edb95e" },
    { group: "DEF", color: "#5aa9e6" },
    { group: "MID", color: "#5ec98a" },
    { group: "FWD", color: "#ef6b6b" },
  ],
  matchVariant: "pitch",
  standingsVariant: "league",
};

/** Colour for a position group; falls back to muted grey. */
export function posColor(theme: SportTheme, group: string | null | undefined): string {
  if (!group) return "#7a828d";
  return theme.positions.find((p) => p.group === group)?.color ?? "#7a828d";
}
