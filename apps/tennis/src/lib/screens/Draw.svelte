<script lang="ts">
  import { tierColor } from "../design/color";
  import type { SportTheme } from "../design/theme";
  import type { PlayerRow } from "../design/dto";

  let {
    theme,
    draw,
  }: {
    theme: SportTheme;
    draw: PlayerRow[];
  } = $props();

  const GOLD = "#e6c34a";

  function ovr(p: PlayerRow): number {
    return Math.round((p.serve + p.return_game + p.baseline + p.mental) / 4);
  }

  // seed-ordered copy
  const rows = $derived([...draw].sort((a, b) => a.seed - b.seed));

  // heat columns, in order (short label + accessor)
  const heatCols: { label: string; get: (p: PlayerRow) => number }[] = [
    { label: "SRV", get: (p) => p.serve },
    { label: "RET", get: (p) => p.return_game },
    { label: "BAS", get: (p) => p.baseline },
    { label: "MEN", get: (p) => p.mental },
  ];

  // footer metrics
  const avgOvr = $derived(
    rows.length ? Math.round(rows.reduce((s, p) => s + ovr(p), 0) / rows.length) : "—",
  );
  const bestSrv = $derived(rows.length ? Math.max(...rows.map((p) => p.serve)) : "—");

  const empty = $derived(rows.length === 0);
</script>

<div class="squad">
  <!-- top bar -->
  <div class="topbar">
    <div class="segset">
      <div class="seg on">Attributes</div>
    </div>
    <div class="right">
      <span class="count">{rows.length} players · seeded draw</span>
    </div>
  </div>

  <!-- scroll body -->
  <div class="body">
    {#if empty}
      <div class="empty">No players in the draw.</div>
    {:else}
      <!-- sticky header -->
      <div class="header">
        <div class="lead">
          <span class="h-seed">SEED</span>
          <span class="h-player">PLAYER</span>
        </div>
        <div class="h-cells">
          <span class="h-col" style="width:52px">OVR</span>
          {#each heatCols as col}
            <span class="h-col" style="width:50px">{col.label}</span>
          {/each}
        </div>
      </div>

      {#each rows as p (p.seed)}
        <div class="row">
          <div class="lead">
            <span
              class="seed-badge"
              class:top={p.seed === 0}
              style={p.seed === 0
                ? `background:${GOLD}22;color:${GOLD};border-color:${GOLD}55`
                : ""}>
              #{p.seed + 1}
            </span>
            <div class="pcol">
              <div class="pname-line">
                <span class="pname">{p.name}</span>
              </div>
              <div class="meta">Seed {p.seed + 1}</div>
            </div>
          </div>
          <div class="cells">
            <span
              class="cell ovr"
              style="width:52px;color:{tierColor(ovr(p))}">{ovr(p)}</span>
            {#each heatCols as col}
              <span
                class="cell heat"
                style="width:50px;color:{tierColor(col.get(p))}">{col.get(p)}</span>
            {/each}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- footer bar -->
  <div class="footer">
    <span>FIELD <b>{rows.length}</b></span>
    <span>AVG OVR <b>{avgOvr}</b></span>
    <span>TOP SERVE <b>{bestSrv}</b></span>
    <span class="sel-lbl">TOP SEED <b class="sel-name">{rows[0]?.name ?? "None"}</b></span>
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
  .segset {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #0f1319;
    border: 1px solid #232a33;
    border-radius: 9px;
    padding: 3px;
  }
  .seg {
    padding: 5px 12px;
    border-radius: 7px;
    font-size: 12px;
    font-weight: 600;
    background: transparent;
    color: #8b95a1;
    user-select: none;
  }
  .seg.on {
    background: var(--accent);
    color: #0a0c0f;
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
    gap: 12px;
  }
  .h-seed {
    width: 46px;
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
  .seed-badge {
    min-width: 40px;
    height: 22px;
    padding: 0 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-weight: 700;
    font-size: 11.5px;
    border-radius: 6px;
    background: #1b212a;
    border: 1px solid #2a323c;
    color: var(--muted-3);
  }
  .seed-badge.top {
    font-weight: 800;
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
  .meta {
    font-size: 11px;
    color: #6b7480;
    font-family: var(--font-mono);
    margin-top: 1px;
  }
  .cells {
    flex: none;
    display: flex;
    align-items: center;
  }
  .cell {
    text-align: right;
    padding: 0 5px;
    white-space: nowrap;
    font-family: var(--font-mono);
  }
  .cell.ovr {
    font-weight: 700;
    font-size: 13.5px;
  }
  .cell.heat {
    font-weight: 600;
    font-size: 12.5px;
  }

  /* ---- empty state ---- */
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
  .sel-lbl {
    margin-left: auto;
  }
  .sel-name {
    color: var(--accent) !important;
  }
</style>
