<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import { tierColor } from "../design/color";
  import type { SportTheme } from "../design/theme";
  import type { RiderRow, Screen } from "../design/dto";

  let {
    theme,
    roster,
    hasRaced,
    onNav,
  }: {
    theme: SportTheme;
    roster: RiderRow[];
    hasRaced: boolean;
    onNav: (s: Screen) => void;
  } = $props();

  const DASH = "—";

  // ── best climber (highest climbing attr) ────────────────────────────────
  const bestClimber = $derived.by(() => {
    if (roster.length === 0) return null;
    return roster.reduce((best, r) => (r.climbing > best.climbing ? r : best), roster[0]);
  });

  // ── squad-wide averages, for the team panel ─────────────────────────────
  function mean(sel: (r: RiderRow) => number): number | null {
    if (roster.length === 0) return null;
    return Math.round(roster.reduce((s, r) => s + sel(r), 0) / roster.length);
  }
  const avgClimb = $derived(mean((r) => r.climbing));
  const avgSprint = $derived(mean((r) => r.sprinting));
  const avgTt = $derived(mean((r) => r.time_trial));
  const avgEnd = $derived(mean((r) => r.endurance));

  // ── KPI row ──────────────────────────────────────────────────────────────
  type Kpi = { label: string; value: string; sub: string; subColor: string };
  const kpis = $derived.by<Kpi[]>(() => [
    {
      label: "Riders",
      value: roster.length ? String(roster.length) : DASH,
      sub: "on the team",
      subColor: "#828b96",
    },
    {
      label: "Best Climber",
      value: bestClimber ? bestClimber.name : DASH,
      sub: bestClimber ? `CLM ${bestClimber.climbing}` : "no roster",
      subColor: bestClimber ? theme.accent : "#828b96",
    },
    {
      label: "Grand Tour",
      value: hasRaced ? "Complete" : "Ready",
      sub: hasRaced ? "GC available" : "7 stages await",
      subColor: hasRaced ? "#5ec98a" : "#828b96",
    },
  ]);

  // ── team blend bars ──────────────────────────────────────────────────────
  type Bar = { label: string; short: string; value: number | null };
  const bars = $derived<Bar[]>([
    { label: "Climbing", short: "CLM", value: avgClimb },
    { label: "Sprint", short: "SPR", value: avgSprint },
    { label: "Time Trial", short: "TT", value: avgTt },
    { label: "Endurance", short: "END", value: avgEnd },
  ]);
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
    <!-- ─ TEAM PANEL ─ -->
    <section class="panel">
      <div class="panel-head">
        <div class="panel-title">
          <span class="ttl">The Team</span>
          <span class="pill">{theme.name}</span>
        </div>
      </div>
      <div class="panel-body">
        <p class="lede">
          A pro team lines up for the World Tour — a seven-stage grand tour across flats,
          hills, mountains and a time trial. Each rider brings a blend of four abilities;
          the general classification is decided on cumulative time.
        </p>

        <div class="blend">
          {#each bars as b}
            <div class="blend-row">
              <span class="blend-lbl">{b.label}</span>
              <div class="blend-track">
                {#if b.value != null}
                  <div
                    class="blend-fill"
                    style="width:{b.value}%;background:{tierColor(b.value)}"></div>
                {/if}
              </div>
              <span
                class="blend-val mono"
                style="color:{b.value != null ? tierColor(b.value) : '#5a636e'}">
                {b.value != null ? b.value : DASH}
              </span>
            </div>
          {/each}
        </div>
      </div>
    </section>

    <!-- ─ RIGHT RAIL ─ -->
    <div class="rail">
      <section class="card">
        <div class="card-head">
          <span class="fx-comp">Grand Tour</span>
          <span class="next-tag">NEXT</span>
        </div>
        <div class="cta-body">
          <div class="cta-line">
            {hasRaced
              ? "The tour is done — head to Race to read the classification."
              : "Send the team up the road. Run the Tour to settle the GC."}
          </div>
        </div>
        <div class="fx-foot">
          <button class="ghost-btn" onclick={() => onNav("race")}>
            <Icon name="trophy" size={15} />
            <span>{hasRaced ? "View Race" : "Run the Tour"}</span>
          </button>
        </div>
      </section>

      <section class="card">
        <div class="card-head solo">
          <span class="snap-title">Roster</span>
        </div>
        <div class="cta-body">
          <div class="cta-line">
            {roster.length
              ? `${roster.length} riders on the books. Inspect abilities on the Roster screen.`
              : "Roster loads on start."}
          </div>
        </div>
        <div class="fx-foot">
          <button class="ghost-btn subtle" onclick={() => onNav("roster")}>
            <Icon name="squad" size={15} />
            <span>Open Roster</span>
          </button>
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
    grid-template-columns: repeat(3, 1fr);
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

  /* ── team panel ── */
  .panel {
    flex: 1.7;
    min-width: 0;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 14px;
    overflow: hidden;
  }
  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--line);
  }
  .panel-title {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .panel-title .ttl {
    font-size: 15px;
    font-weight: 700;
  }
  .pill {
    font-size: 11px;
    font-weight: 700;
    font-family: var(--font-mono);
    background: var(--accent-soft);
    color: var(--accent);
    padding: 2px 7px;
    border-radius: 20px;
  }
  .panel-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .lede {
    margin: 0;
    font-size: 13px;
    line-height: 1.55;
    color: var(--muted);
  }

  .blend {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .blend-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .blend-lbl {
    width: 92px;
    flex: none;
    font-size: 12px;
    font-weight: 500;
    color: var(--muted);
  }
  .blend-track {
    flex: 1;
    height: 6px;
    background: var(--line);
    border-radius: 4px;
    overflow: hidden;
  }
  .blend-fill {
    height: 100%;
    border-radius: 4px;
  }
  .blend-val {
    width: 30px;
    text-align: right;
    font-size: 12px;
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
  .next-tag {
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 3px 7px;
    border-radius: 5px;
  }
  .snap-title {
    font-size: 13.5px;
    font-weight: 700;
  }
  .cta-body {
    padding: 14px 15px 4px;
  }
  .cta-line {
    font-size: 12.5px;
    line-height: 1.5;
    color: var(--muted);
  }
  .fx-foot {
    padding: 12px 15px 15px;
  }
  .ghost-btn {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 9px;
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
  .ghost-btn.subtle {
    border-color: var(--line-3);
    color: var(--muted);
  }
  .ghost-btn.subtle:hover {
    border-color: var(--accent-line);
    color: var(--accent);
    background: var(--accent-soft);
  }
</style>
