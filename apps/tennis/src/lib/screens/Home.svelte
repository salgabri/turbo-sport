<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import { tierColor } from "../design/color";
  import type { SportTheme } from "../design/theme";
  import type { PlayerRow, Tourney, Screen } from "../design/dto";

  let {
    theme,
    draw,
    result,
    onNav,
  }: {
    theme: SportTheme;
    draw: PlayerRow[];
    result: Tourney | null;
    onNav: (s: Screen) => void;
  } = $props();

  const DASH = "—";

  function ovr(p: PlayerRow): number {
    return Math.round((p.serve + p.return_game + p.baseline + p.mental) / 4);
  }

  // top seed = seed 0 (draw is delivered in seed order)
  const topSeed = $derived(
    [...draw].sort((a, b) => a.seed - b.seed)[0] ?? null,
  );

  type Kpi = {
    label: string;
    value: string;
    sub: string;
    subColor: string;
  };

  const kpis = $derived.by<Kpi[]>(() => {
    const played = result != null;
    return [
      {
        label: "Field Size",
        value: draw.length ? String(draw.length) : DASH,
        sub: "single elimination",
        subColor: "#828b96",
      },
      {
        label: "Top Seed",
        value: topSeed ? topSeed.name : DASH,
        sub: topSeed ? `#1 · OVR ${ovr(topSeed)}` : "no draw loaded",
        subColor: topSeed ? theme.accent : "#828b96",
      },
      {
        label: "Rounds",
        value: draw.length ? String(Math.max(0, Math.round(Math.log2(draw.length)))) : DASH,
        sub: "to the title",
        subColor: "#828b96",
      },
      {
        label: "Champion",
        value: result ? result.champion : "TBD",
        sub: played ? "tournament complete" : "not played yet",
        subColor: played ? theme.accent : "#828b96",
      },
    ];
  });

  const hasDraw = $derived(draw.length > 0);
</script>

<div class="home">
  <!-- ══ KPI ROW ══ -->
  <div class="kpi-row">
    {#each kpis as k}
      <div class="kpi">
        <span class="kpi-label">{k.label}</span>
        <span class="kpi-value">{k.value}</span>
        <span class="kpi-sub" style="color:{k.subColor}">{k.sub}</span>
      </div>
    {/each}
  </div>

  <!-- ══ MAIN SPLIT ══ -->
  <div class="split">
    <!-- ─ OVERVIEW ─ -->
    <section class="inbox">
      <div class="inbox-head">
        <div class="inbox-title">
          <span class="ttl">Tournament</span>
          <span class="pill-new">{hasDraw ? `${draw.length} players` : "empty"}</span>
        </div>
        <div class="inbox-tabs">
          <span class="tab on">Overview</span>
        </div>
      </div>

      {#if hasDraw}
        <div class="overview-body">
          <div class="lead-line">
            <span class="lead-label">A single-elimination bracket is ready.</span>
            <span class="lead-note">
              {draw.length} seeded players. Head to the Bracket to play it out and crown a champion.
            </span>
          </div>
          <div class="cta-row">
            <button class="cta" onclick={() => onNav("bracket")}>
              <Icon name="play" size={15} />
              <span>Play the tournament</span>
            </button>
            <button class="ghost-btn wide" onclick={() => onNav("draw")}>View the draw</button>
          </div>
          {#if result}
            <div class="done-banner">
              <span class="done-icon"><Icon name="trophy" size={16} /></span>
              <span class="done-text">
                Champion: <b>{result.champion}</b>
              </span>
            </div>
          {/if}
        </div>
      {:else}
        <div class="empty">No draw loaded yet.</div>
      {/if}
    </section>

    <!-- ─ RIGHT RAIL ─ -->
    <div class="rail">
      <section class="card">
        <div class="card-head">
          <span class="fx-comp">Top Seeds</span>
          <div
            class="table-link"
            role="button"
            tabindex="0"
            onclick={() => onNav("draw")}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") onNav("draw");
            }}>
            <span>Draw</span><Icon name="chevron" size={13} />
          </div>
        </div>
        {#if hasDraw}
          <div class="snap-body">
            {#each [...draw].sort((a, b) => a.seed - b.seed).slice(0, 6) as p}
              <div class="snap-row">
                <span class="c-pos mono">#{p.seed + 1}</span>
                <span class="c-club">{p.name}</span>
                <span class="c-b mono" style="color:{tierColor(ovr(p))}">{ovr(p)}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty">Seeds appear once a draw is loaded.</div>
        {/if}
      </section>

      <section class="card">
        <div class="card-head solo">
          <span class="snap-title">How it works</span>
        </div>
        <div class="status-body">
          <div class="note-line">Each match is best-of sets, decided by serve, return, baseline and mental ability.</div>
          <div class="note-line">Play the bracket to advance winners round by round until one player remains.</div>
        </div>
      </section>
    </div>
  </div>
</div>

<style>
  .home {
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1180px;
  }

  /* ── KPI row ── */
  .kpi-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
  }
  .kpi {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 13px;
    padding: 13px 14px;
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-height: 92px;
  }
  .kpi-label {
    font-size: 10px;
    letter-spacing: 0.09em;
    font-weight: 600;
    color: var(--dim-2);
    font-family: var(--font-mono);
    text-transform: uppercase;
  }
  .kpi-value {
    font-size: 23px;
    font-weight: 800;
    letter-spacing: -0.02em;
    line-height: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .kpi-sub {
    font-size: 11.5px;
    font-weight: 500;
  }

  /* ── main split ── */
  .split {
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }

  /* ── overview panel ── */
  .inbox {
    flex: 1.7;
    min-width: 0;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 14px;
    overflow: hidden;
  }
  .inbox-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--line);
  }
  .inbox-title {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .inbox-title .ttl {
    font-size: 15px;
    font-weight: 700;
  }
  .pill-new {
    font-size: 11px;
    font-weight: 700;
    font-family: var(--font-mono);
    background: var(--accent-soft);
    color: var(--accent);
    padding: 2px 7px;
    border-radius: 20px;
  }
  .inbox-tabs {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .tab {
    font-size: 11.5px;
    font-weight: 500;
    color: var(--muted-2);
    padding: 4px 10px;
    border-radius: 7px;
  }
  .tab.on {
    font-weight: 600;
    color: #0e1115;
    background: var(--accent);
  }

  .overview-body {
    padding: 20px 18px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .lead-line {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .lead-label {
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
  }
  .lead-note {
    font-size: 12.5px;
    color: var(--muted);
    line-height: 1.5;
  }
  .cta-row {
    display: flex;
    gap: 10px;
  }
  .cta {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 18px;
    background: var(--accent);
    color: #08120c;
    font-weight: 700;
    font-size: 13px;
    border: none;
    border-radius: 9px;
    cursor: pointer;
    box-shadow: 0 0 20px var(--accent-soft);
    font-family: inherit;
  }
  .cta:hover {
    filter: brightness(1.08);
  }
  .done-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border: 1px solid var(--accent-line);
    background: var(--accent-soft);
    border-radius: 10px;
  }
  .done-icon {
    display: flex;
    color: var(--accent);
  }
  .done-text {
    font-size: 13px;
    color: var(--text-2);
  }
  .done-text b {
    color: var(--accent);
    font-weight: 700;
  }

  /* ── right rail ── */
  .rail {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .card {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 14px;
    overflow: hidden;
  }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 15px;
    border-bottom: 1px solid var(--line);
  }
  .card-head.solo {
    justify-content: flex-start;
  }
  .fx-comp {
    font-size: 11px;
    font-weight: 600;
    color: var(--muted-3);
    font-family: var(--font-mono);
    letter-spacing: 0.04em;
  }
  .snap-title {
    font-size: 13.5px;
    font-weight: 700;
  }
  .table-link {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 11px;
    color: var(--muted-2);
    cursor: pointer;
    font-family: var(--font-mono);
  }
  .table-link:hover {
    color: var(--accent);
  }
  .snap-body {
    padding: 5px 8px 8px;
  }
  .snap-row {
    display: flex;
    align-items: center;
    padding: 6px 8px;
    border-radius: 7px;
  }
  .c-pos {
    width: 34px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--muted-3);
  }
  .c-club {
    flex: 1;
    font-size: 12.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-3);
  }
  .c-b {
    width: 40px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 12.5px;
    font-weight: 700;
  }
  .ghost-btn {
    text-align: center;
    padding: 10px 16px;
    border: 1px solid var(--accent-line);
    color: var(--accent);
    border-radius: 9px;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    background: none;
    font-family: inherit;
  }
  .ghost-btn:hover {
    background: var(--accent-soft);
  }
  .status-body {
    padding: 13px 15px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .note-line {
    font-size: 12px;
    color: var(--muted);
    line-height: 1.5;
  }

  /* empty state */
  .empty {
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--faint);
    padding: 48px 24px;
  }
</style>
