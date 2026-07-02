<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import { money, fitColor, moraleWord } from "../design/color";
  import type { SportTheme } from "../design/theme";
  import type { ClubView, PlayerView, StandingRow, Screen } from "../design/dto";

  let {
    theme,
    club,
    squad,
    standings,
    myTeamId,
    teamName,
    seasonActive,
    onNav,
  }: {
    theme: SportTheme;
    club: ClubView | null;
    squad: PlayerView[];
    standings: StandingRow[];
    myTeamId: number | null;
    teamName: (id: number) => string;
    seasonActive: boolean;
    onNav: (s: Screen) => void;
  } = $props();

  // ── ordinal helper (1 -> "1st", 2 -> "2nd", …) ──────────────────────────
  function ordinal(n: number): string {
    const s = ["th", "st", "nd", "rd"];
    const v = n % 100;
    return n + (s[(v - 20) % 10] || s[v] || s[0]);
  }

  // games played is derived from win + loss (basketball has no draws)
  const played = (r: StandingRow): number => r.won + r.lost;

  // ── standings sorted (defensive: sort a copy by win%, then point diff) ───
  const sorted = $derived(
    [...standings].sort(
      (a, b) => b.win_pct - a.win_pct || b.point_diff - a.point_diff || b.won - a.won,
    ),
  );

  // index of my team within the sorted table (−1 if not present)
  const myIdx = $derived(
    myTeamId == null ? -1 : sorted.findIndex((r) => r.team_id === myTeamId),
  );

  const hasSeason = $derived(seasonActive && sorted.length > 0);

  // ── KPI row (6 cards) ────────────────────────────────────────────────────
  type Kpi = {
    label: string;
    value: string;
    sub: string;
    subColor: string;
    hasBar?: boolean;
    barStyle?: string;
  };

  const kpis = $derived.by<Kpi[]>(() => {
    const cur = theme.currency;
    const posValue = hasSeason && myIdx >= 0 ? ordinal(myIdx + 1) : "—";
    const posSub =
      hasSeason && myIdx >= 0
        ? `of ${sorted.length} clubs`
        : "no active season";

    // wage bill / income → bar
    const inc = club?.weekly_income ?? 0;
    const wb = club?.wage_bill ?? 0;
    const ratio = inc > 0 ? Math.min(100, (wb / inc) * 100) : 0;
    const barColor = ratio >= 90 ? "#ef6b6b" : ratio >= 70 ? "#edb95e" : theme.accent;

    return [
      {
        label: "Conf Pos",
        value: posValue,
        sub: posSub,
        subColor: "#828b96",
      },
      {
        label: "Roster Size",
        value: club ? String(club.squad_size) : "—",
        sub: club ? "registered players" : "no club data",
        subColor: "#828b96",
      },
      {
        label: "Balance",
        value: club ? money(club.balance, cur) : "—",
        sub: "transfer budget",
        subColor: club && club.balance < 0 ? "#ef6b6b" : "#828b96",
      },
      {
        label: "Wage Bill",
        value: club ? money(wb, cur) + "/wk" : "—",
        sub: inc > 0 ? `${Math.round(ratio)}% of income` : "wages",
        subColor:
          ratio >= 90 ? "#ef6b6b" : ratio >= 70 ? "#edb95e" : "#828b96",
        hasBar: !!club && inc > 0,
        barStyle: `width:${ratio}%;background:${barColor}`,
      },
      {
        label: "Income",
        value: club ? money(inc, cur) + "/wk" : "—",
        sub: "weekly revenue",
        subColor: "#5ec98a",
      },
      {
        label: "Season",
        value: seasonActive ? "Running" : "Off-season",
        sub: seasonActive ? "games scheduled" : "no fixtures",
        subColor: seasonActive ? "#5ec98a" : "#828b96",
      },
    ];
  });

  // ── standings snapshot: 6 rows centred on my team ────────────────────────
  const snapshotRows = $derived.by(() => {
    if (!hasSeason) return [];
    const N = 6;
    const total = sorted.length;
    let start = 0;
    if (myIdx >= 0) {
      start = Math.max(0, Math.min(myIdx - Math.floor(N / 2), total - N));
    }
    start = Math.max(0, start);
    return sorted.slice(start, start + N).map((r, i) => {
      const pos = start + i + 1;
      const mine = r.team_id === myTeamId;
      return { r, pos, mine };
    });
  });

  // dot colour by table zone: top = accent, bottom 3 = down.
  function zoneDot(pos: number, total: number): string {
    if (pos <= 1) return theme.accent;
    if (pos <= 4) return "#5aa9e6";
    if (pos > total - 3) return "#ef6b6b";
    return "#3a424c";
  }

  // ── squad status ─────────────────────────────────────────────────────────
  const injuredList = $derived(
    squad.filter((p) => p.injured && !p.retired && !p.free_agent),
  );

  const withFitness = $derived(
    squad.filter((p) => !p.retired && p.fitness != null),
  );
  const withMorale = $derived(
    squad.filter((p) => !p.retired && p.morale != null),
  );

  const avgFitness = $derived(
    withFitness.length
      ? Math.round(
          withFitness.reduce((s, p) => s + (p.fitness ?? 0), 0) /
            withFitness.length,
        )
      : null,
  );
  const avgMorale = $derived(
    withMorale.length
      ? Math.round(
          withMorale.reduce((s, p) => s + (p.morale ?? 0), 0) /
            withMorale.length,
        )
      : null,
  );

  type Gauge = {
    label: string;
    valueText: string;
    color: string;
    barStyle: string;
  };
  const gauges = $derived.by<Gauge[]>(() => {
    const out: Gauge[] = [];
    if (avgFitness != null) {
      const c = fitColor(avgFitness);
      out.push({
        label: "Avg Fitness",
        valueText: `${avgFitness}%`,
        color: c,
        barStyle: `width:${avgFitness}%;background:${c}`,
      });
    }
    if (avgMorale != null) {
      const m = moraleWord(avgMorale);
      out.push({
        label: "Squad Morale",
        valueText: m.t,
        color: m.c,
        barStyle: `width:${avgMorale}%;background:${m.c}`,
      });
    }
    return out;
  });

  const hasSquad = $derived(squad.filter((p) => !p.retired).length > 0);
</script>

<div class="home">
  <!-- ══ KPI ROW ══ -->
  <div class="kpi-row">
    {#each kpis as k}
      <div class="kpi">
        <span class="kpi-label">{k.label}</span>
        <span class="kpi-value">{k.value}</span>
        {#if k.hasBar}
          <div class="kpi-bar"><div class="kpi-bar-fill" style={k.barStyle}></div></div>
        {/if}
        <span class="kpi-sub" style="color:{k.subColor}">{k.sub}</span>
      </div>
    {/each}
  </div>

  <!-- ══ MAIN SPLIT ══ -->
  <div class="split">
    <!-- ─ INBOX ─ -->
    <section class="inbox">
      <div class="inbox-head">
        <div class="inbox-title">
          <span class="ttl">Inbox</span>
          <span class="pill-new">0 new</span>
        </div>
        <div class="inbox-tabs">
          <span class="tab on">All</span>
          <span class="tab">Unread</span>
          <div class="filter-btn"><Icon name="filter" size={15} /></div>
        </div>
      </div>
      <div class="empty">No messages yet. Your inbox fills as the season unfolds.</div>
    </section>

    <!-- ─ RIGHT RAIL ─ -->
    <div class="rail">
      <!-- next fixture -->
      <section class="card">
        <div class="card-head">
          <span class="fx-comp">Fixtures</span>
          <span class="next-tag">NEXT</span>
        </div>
        <div class="empty">Fixtures arrive with the schedule view.</div>
        <div class="fx-foot">
          <button class="ghost-btn" onclick={() => onNav("match")}>Go to Match</button>
        </div>
      </section>

      <!-- standings snapshot -->
      <section class="card">
        <div class="card-head">
          <span class="snap-title">Standings</span>
          <div
            class="table-link"
            role="button"
            tabindex="0"
            onclick={() => onNav("table")}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") onNav("table");
            }}>
            <span>Table</span><Icon name="chevron" size={13} />
          </div>
        </div>
        {#if hasSeason}
          <div class="snap-body">
            <div class="snap-colhead">
              <span class="c-pos">#</span>
              <span class="c-club">Club</span>
              <span class="c-a">Pl</span>
              <span class="c-b">Pct</span>
            </div>
            {#each snapshotRows as row}
              <div class="snap-row" class:mine={row.mine}>
                <span class="c-pos mono">{row.pos}</span>
                <span
                  class="c-club club-name"
                  style="font-weight:{row.mine ? 700 : 500};color:{row.mine
                    ? 'var(--text)'
                    : 'var(--text-3)'}">
                  <span
                    class="dot"
                    style="background:{zoneDot(row.pos, sorted.length)}"></span>
                  {teamName(row.r.team_id)}
                </span>
                <span class="c-a mono">{played(row.r)}</span>
                <span
                  class="c-b mono"
                  style="color:{row.mine ? 'var(--accent)' : 'var(--text-2)'}">
                  {row.r.win_pct.toFixed(3)}
                </span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty">Standings appear once a season is under way.</div>
        {/if}
      </section>

      <!-- squad status -->
      <section class="card">
        <div class="card-head solo">
          <span class="snap-title">Roster Status</span>
        </div>
        {#if hasSquad}
          <div class="status-body">
            {#if injuredList.length}
              <div class="alerts">
                {#each injuredList.slice(0, 4) as p}
                  <div class="alert">
                    <div class="alert-chip">
                      <Icon name="injury" size={15} />
                    </div>
                    <div class="alert-text">
                      <span class="alert-name">{p.name ?? "Unknown"}</span>
                      <span class="alert-note">
                        — {p.fitness != null
                          ? `${Math.round(p.fitness)}% fit`
                          : "out"}</span>
                    </div>
                    <span class="alert-tag">INJ</span>
                  </div>
                {/each}
              </div>
              <div class="divider"></div>
            {/if}
            {#if gauges.length}
              {#each gauges as g}
                <div class="gauge">
                  <div class="gauge-head">
                    <span class="gauge-label">{g.label}</span>
                    <span class="gauge-value mono" style="color:{g.color}">
                      {g.valueText}
                    </span>
                  </div>
                  <div class="gauge-track">
                    <div class="gauge-fill" style={g.barStyle}></div>
                  </div>
                </div>
              {/each}
            {:else}
              <div class="empty sm">Fitness and morale data pending.</div>
            {/if}
          </div>
        {:else}
          <div class="empty">Roster status appears once players are on the books.</div>
        {/if}
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
    grid-template-columns: repeat(6, 1fr);
    gap: 12px;
  }
  .kpi {
    position: relative;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 0;
    padding: 13px 14px;
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-height: 92px;
  }
  .kpi::before {
    content: "";
    position: absolute;
    top: -1px;
    left: -1px;
    width: 9px;
    height: 9px;
    border-top: 2px solid var(--accent);
    border-left: 2px solid var(--accent);
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
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
  }
  .kpi-bar {
    height: 4px;
    background: var(--line);
    border-radius: 0;
    overflow: hidden;
    margin-top: -1px;
  }
  .kpi-bar-fill {
    height: 100%;
    border-radius: 0;
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

  /* ── inbox ── */
  .inbox {
    flex: 1.7;
    min-width: 0;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 0;
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
    border-radius: 0;
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
    border-radius: 0;
    cursor: pointer;
  }
  .tab:hover {
    background: #1b212a;
  }
  .tab.on {
    font-weight: 600;
    color: #0e1115;
    background: var(--accent);
  }
  .filter-btn {
    width: 30px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0;
    color: var(--muted-2);
    cursor: pointer;
  }
  .filter-btn:hover {
    background: #1b212a;
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
    border-radius: 0;
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

  /* next fixture */
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
    border-radius: 0;
  }
  .fx-foot {
    padding: 0 15px 15px;
  }
  .ghost-btn {
    width: 100%;
    text-align: center;
    padding: 9px;
    border: 1px solid var(--accent-line);
    color: var(--accent);
    border-radius: 0;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    background: none;
    font-family: inherit;
  }
  .ghost-btn:hover {
    background: var(--accent-soft);
  }

  /* standings snapshot */
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
  .snap-colhead {
    display: flex;
    padding: 6px 8px;
    font-size: 9.5px;
    color: var(--dim-2);
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .snap-row {
    display: flex;
    align-items: center;
    padding: 6px 8px;
    border-radius: 0;
  }
  .snap-row.mine {
    background: var(--accent-soft);
    box-shadow: inset 2px 0 0 var(--accent);
  }
  .c-pos {
    width: 22px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--muted-3);
  }
  .c-club {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .club-name .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex: none;
  }
  .c-a {
    width: 34px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--muted-3);
  }
  .c-b {
    width: 52px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 12.5px;
    font-weight: 600;
  }

  /* squad status */
  .status-body {
    padding: 13px 15px;
    display: flex;
    flex-direction: column;
    gap: 11px;
  }
  .alerts {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .alert {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .alert-chip {
    width: 28px;
    height: 28px;
    flex: none;
    border-radius: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #331e21;
    color: var(--down);
  }
  .alert-text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .alert-name {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text);
  }
  .alert-note {
    font-size: 11.5px;
    color: #828b96;
  }
  .alert-tag {
    font-size: 10.5px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--down);
    background: #331e21;
    padding: 2px 7px;
    border-radius: 0;
    flex: none;
  }
  .divider {
    height: 1px;
    background: var(--line);
  }
  .gauge-head {
    display: flex;
    justify-content: space-between;
    margin-bottom: 5px;
  }
  .gauge-label {
    font-size: 11.5px;
    color: var(--muted);
    font-weight: 500;
  }
  .gauge-value {
    font-size: 11.5px;
    font-weight: 700;
  }
  .gauge-track {
    height: 6px;
    background: var(--line);
    border-radius: 0;
    overflow: hidden;
  }
  .gauge-fill {
    height: 100%;
    border-radius: 0;
  }

  /* empty state (consistent) */
  .empty {
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--faint);
    padding: 48px 24px;
  }
  .empty.sm {
    padding: 20px 24px;
  }
</style>
