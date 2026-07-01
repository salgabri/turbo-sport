<script lang="ts">
  import { tierColor, posPill } from "../design/color";
  import { posColor, type SportTheme } from "../design/theme";
  import type { RiderRow } from "../design/dto";

  let {
    theme,
    roster,
    count,
  }: {
    theme: SportTheme;
    roster: RiderRow[];
    count: number;
  } = $props();

  const DASH = "—";

  // ── column set (sticky header) ─────────────────────────────────────────────
  type Col = { label: string; width: string };
  const cols: Col[] = [
    { label: "OVR", width: "50px" },
    { label: "CLM", width: "50px" },
    { label: "SPR", width: "50px" },
    { label: "TT", width: "50px" },
    { label: "END", width: "50px" },
  ];

  // ── derived helpers ────────────────────────────────────────────────────────
  // Position pill: pick the highest of the 4 attrs -> CLB / SPR / TT / GC.
  // GC only "wins" ties as the all-rounder crown; DOM is the fallback floor.
  function riderPos(r: RiderRow): string {
    const opts: [string, number][] = [
      ["CLB", r.climbing],
      ["SPR", r.sprinting],
      ["TT", r.time_trial],
      ["GC", r.endurance],
    ];
    let best = opts[0];
    for (const o of opts) if (o[1] > best[1]) best = o;
    // no rider stands out at all -> domestique
    if (best[1] < 55) return "DOM";
    return best[0];
  }

  function overall(r: RiderRow): number {
    return Math.round((r.climbing + r.sprinting + r.time_trial + r.endurance) / 4);
  }

  // ── cell style builders (mirror football Squad) ───────────────────────────
  const base = (width: string) =>
    `text-align:right;width:${width};padding:0 5px;white-space:nowrap;`;
  const mono = "font-family:var(--font-mono);";

  function ovrCell(width: string, v: number) {
    return {
      text: String(v),
      style: base(width) + mono + `font-weight:700;font-size:13.5px;color:${tierColor(v)}`,
    };
  }
  function heatCell(width: string, v: number) {
    return {
      text: String(v),
      style: base(width) + mono + `font-weight:600;font-size:12.5px;color:${tierColor(v)}`,
    };
  }

  function cellsFor(r: RiderRow) {
    return [
      ovrCell("50px", overall(r)),
      heatCell("50px", r.climbing),
      heatCell("50px", r.sprinting),
      heatCell("50px", r.time_trial),
      heatCell("50px", r.endurance),
    ];
  }

  const posMuted = "#7a828d";
  function posStyleFor(r: RiderRow) {
    const g = riderPos(r);
    const hex = g === "DOM" ? posMuted : posColor(theme, g);
    return posPill(hex, "sm");
  }

  // ── footer metrics ─────────────────────────────────────────────────────────
  const avgOvr = $derived(
    roster.length
      ? Math.round(roster.reduce((s, r) => s + overall(r), 0) / roster.length).toString()
      : DASH,
  );
  const topOvr = $derived(
    roster.length ? Math.max(...roster.map(overall)).toString() : DASH,
  );

  const empty = $derived(roster.length === 0);
</script>

<div class="squad">
  <!-- top bar -->
  <div class="topbar">
    <div class="lead-note">
      <span class="note-title">Team Roster</span>
      <span class="note-sub mono">abilities &amp; rated overall</span>
    </div>
    <div class="right">
      <span class="count">{count} riders</span>
    </div>
  </div>

  <!-- scroll body -->
  <div class="body">
    {#if empty}
      <div class="empty">No riders on the team.</div>
    {:else}
      <!-- sticky header -->
      <div class="header">
        <div class="lead">
          <span class="h-pos">POS</span>
          <span class="h-player">RIDER</span>
        </div>
        <div class="h-cells">
          {#each cols as col}
            <span class="h-col" style="width:{col.width}">{col.label}</span>
          {/each}
        </div>
      </div>

      {#each roster as r}
        <div class="row">
          <div class="lead">
            <span style={posStyleFor(r)}>{riderPos(r)}</span>
            <div class="pcol">
              <div class="pname-line">
                <span class="pname">{r.name}</span>
              </div>
            </div>
          </div>
          <div class="cells">
            {#each cellsFor(r) as cell}
              <span style={cell.style}>{cell.text}</span>
            {/each}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- footer bar -->
  <div class="footer">
    <span>SIZE <b>{count}</b></span>
    <span>AVG OVR <b>{avgOvr}</b></span>
    <span>TOP OVR <b>{topOvr}</b></span>
  </div>
</div>

<style>
  .squad {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  /* ---- top bar ---- */
  .topbar {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 11px 20px;
    border-bottom: 1px solid #191e25;
    background: #0e1115;
  }
  .lead-note {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .note-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text);
  }
  .note-sub {
    font-size: 11px;
    color: #616b77;
  }
  .right {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .count {
    font-size: 11.5px;
    color: #616b77;
    font-family: var(--font-mono);
  }

  /* ---- scroll body ---- */
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  /* ---- sticky header ---- */
  .header {
    display: flex;
    align-items: center;
    padding: 9px 20px;
    position: sticky;
    top: 0;
    background: #12161c;
    border-bottom: 1px solid #232a33;
    z-index: 2;
  }
  .lead {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .h-pos {
    width: 34px;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .h-player {
    flex: 1;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .h-cells {
    flex: none;
    display: flex;
  }
  .h-col {
    text-align: right;
    padding: 0 5px;
    font-size: 9.5px;
    letter-spacing: 0.05em;
    color: #616b77;
    font-family: var(--font-mono);
    text-transform: uppercase;
    white-space: nowrap;
  }

  /* ---- rows ---- */
  .row {
    display: flex;
    align-items: center;
    padding: 9px 20px;
    border-bottom: 1px solid #171c22;
  }
  .row:hover {
    background: #161b22;
  }
  .pcol {
    flex: 1;
    min-width: 0;
  }
  .pname-line {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .pname {
    font-size: 13.5px;
    font-weight: 600;
    color: #e9edf1;
    white-space: nowrap;
  }
  .cells {
    flex: none;
    display: flex;
    align-items: center;
  }

  /* ---- empty ---- */
  .empty {
    padding: 70px;
    text-align: center;
    color: #5a636e;
    font-family: var(--font-mono);
    font-size: 13px;
  }

  /* ---- footer ---- */
  .footer {
    flex: none;
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 10px 20px;
    border-top: 1px solid #191e25;
    background: #0e1115;
    font-family: var(--font-mono);
    font-size: 11px;
    color: #6b7480;
    letter-spacing: 0.03em;
  }
  .footer b {
    color: #d4dae1;
    font-weight: 600;
  }
</style>
