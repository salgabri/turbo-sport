// Turbo Sport — per-sport theme config. Tennis event app.
// Schema mirrors football's SportTheme so the shared design foundation applies
// verbatim; only the sport-specific values differ.

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
  matchVariant: "pitch" | "court" | "climb" | "track" | "bracket";
  standingsVariant: "league" | "race";
};

export const TENNIS: SportTheme = {
  id: "tennis",
  name: "Tennis",
  short: "TEN",
  accent: "#ef5a5a",
  currency: "$",
  attrGroups: ["Ability"],
  attributes: [
    { key: "serve", label: "Serve", short: "SRV", group: "Ability" },
    { key: "return_game", label: "Return", short: "RET", group: "Ability" },
    { key: "baseline", label: "Baseline", short: "BAS", group: "Ability" },
    { key: "mental", label: "Mental", short: "MEN", group: "Ability" },
  ],
  positions: [
    { group: "S", color: "#ef5a5a" },
    { group: "B", color: "#5ec98a" },
    { group: "A", color: "#5aa9e6" },
  ],
  matchVariant: "bracket",
  standingsVariant: "race",
};

/** Colour for a position group; falls back to muted grey. */
export function posColor(theme: SportTheme, group: string | null | undefined): string {
  if (!group) return "#7a828d";
  return theme.positions.find((p) => p.group === group)?.color ?? "#7a828d";
}
