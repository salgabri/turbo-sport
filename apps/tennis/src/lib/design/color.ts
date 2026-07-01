// Turbo Sport — pure colour/format helpers, ported verbatim from the design
// comp JS so ratings/attrs/fitness/morale colour exactly as designed.

/** OVR / attribute heat ramp (0–99). */
export function tierColor(v: number): string {
  if (v >= 88) return "#6ee7a8";
  if (v >= 82) return "#a7dd7a";
  if (v >= 74) return "#e6d074";
  if (v >= 66) return "#e2a86a";
  return "#e07e7e";
}

/** Fitness ramp (0–100). */
export function fitColor(v: number): string {
  return v >= 88 ? "#5ec98a" : v >= 72 ? "#b9d97a" : v >= 55 ? "#edb95e" : "#ef6b6b";
}

/** Morale word + colour (0–100). */
export function moraleWord(v: number): { t: string; c: string } {
  if (v >= 85) return { t: "Excellent", c: "#5ec98a" };
  if (v >= 70) return { t: "Good", c: "#7ac88f" };
  if (v >= 55) return { t: "Okay", c: "#edb95e" };
  if (v >= 40) return { t: "Poor", c: "#e0975a" };
  return { t: "Very Poor", c: "#ef6b6b" };
}

/** Semantic tone → colour, matches design's TONE map. */
export const TONE: Record<string, string> = {
  up: "#5ec98a",
  warn: "#edb95e",
  down: "#ef6b6b",
  neutral: "#9aa4b0",
  info: "#5aa9e6",
};

/** Form result chip background/foreground. */
export function formChip(r: string): { bg: string; fg: string } {
  const map: Record<string, [string, string]> = {
    W: ["#173026", "#5ec98a"],
    D: ["#262b33", "#c2cad3"],
    L: ["#331e21", "#ef6b6b"],
  };
  const c = map[r] || ["#1e242c", "#c2cad3"];
  return { bg: c[0], fg: c[1] };
}

/** Money formatter. `cur` is the sport currency symbol (£/$/€). */
export function money(n: number, cur = "£"): string {
  const a = Math.abs(n);
  const s = n < 0 ? "-" : "";
  if (a >= 1e6) return `${s}${cur}${(a / 1e6).toFixed(1)}M`;
  if (a >= 1e3) return `${s}${cur}${(a / 1e3).toFixed(0)}k`;
  return `${s}${cur}${a}`;
}

/** Position-group pill inline style, given the group's hex colour. */
export function posPill(hex: string, size: "sm" | "md" = "sm"): string {
  const dims =
    size === "md"
      ? "min-width:36px;height:22px;padding:0 8px;font-size:11px;border-radius:6px"
      : "min-width:34px;height:20px;padding:0 6px;font-size:10.5px;border-radius:5px";
  return `display:inline-flex;align-items:center;justify-content:center;${dims};font-weight:700;font-family:var(--font-mono);background:${hex}22;color:${hex}`;
}
