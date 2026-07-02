<script lang="ts">
  import { FOOTBALL, type SportTheme } from "../design/theme";
  import { formChip } from "../design/color";
  import type { StandingRow, ScorerRow } from "../design/dto";

  let {
    theme,
    standings,
    teamName,
    myTeamId,
    scorers = [],
  }: {
    theme: SportTheme;
    standings: StandingRow[];
    teamName: (id: number) => string;
    myTeamId: number | null;
    scorers?: ScorerRow[];
  } = $props();

  // Playoff-zone blue (design 481 / 496).
  const PLAYOFF = "#5aa9e6";
  const RELEGATION = "#ef6b6b";
  const NEUTRAL_DOT = "#2f3742";

  // Numeric columns after CLUB (design 489). FORM omitted — form is null now.
  const COLS: { l: string; w: string }[] = [
    { l: "Pl", w: "40px" },
    { l: "W", w: "40px" },
    { l: "D", w: "40px" },
    { l: "L", w: "40px" },
    { l: "GF", w: "44px" },
    { l: "GA", w: "44px" },
    { l: "GD", w: "48px" },
    { l: "Pts", w: "52px" },
  ];

  const n = $derived(standings.length);

  function zoneDot(pos: number): string {
    if (pos <= 2) return theme.accent; // promotion
    if (pos <= 6) return PLAYOFF; // playoff
    if (pos > n - 3) return RELEGATION; // relegation (last 3)
    return NEUTRAL_DOT;
  }

  type Cell = { text: string; bold?: boolean };

  const rows = $derived(
    standings.map((r, i) => {
      const pos = i + 1;
      const mine = r.team_id === myTeamId;
      const cells: Cell[] = [
        { text: String(r.played) },
        { text: String(r.won) },
        { text: String(r.drawn) },
        { text: String(r.lost) },
        { text: String(r.goals_for) },
        { text: String(r.goals_against) },
        {
          text: (r.goal_difference > 0 ? "+" : "") + String(r.goal_difference),
        },
        { text: String(r.points), bold: true },
      ];
      return {
        pos,
        mine,
        dotColor: zoneDot(pos),
        club: teamName(r.team_id),
        cells,
        form: (r.form ?? []).map((c) => ({ c, ...formChip(c) })),
      };
    }),
  );
</script>

{#if standings.length === 0}
  <div class="empty-page">
    No season running — start a season and advance time.
  </div>
{:else}
  <div class="page">
    <section class="card">
      <!-- Header (design 478-483) -->
      <div class="card-head">
        <div>
          <div class="title">League Table</div>
          <div class="sub">{n} clubs</div>
        </div>
        <div class="legend">
          <div class="legend-item">
            <span class="swatch" style="background:{theme.accent}"></span>
            <span class="legend-label">Promotion</span>
          </div>
          <div class="legend-item">
            <span class="swatch" style="background:{PLAYOFF}"></span>
            <span class="legend-label">Playoff</span>
          </div>
        </div>
      </div>

      <div class="scroll-x">
        <div class="table-min">
          <!-- Column header (design 486-490) -->
          <div class="col-head">
            <span class="ch-pos">#</span>
            <span class="ch-club">CLUB</span>
            {#each COLS as c}
              <span class="ch-num" style="width:{c.w}">{c.l}</span>
            {/each}
            <span class="ch-form">FORM</span>
          </div>

          <!-- Rows (design 492-504) -->
          {#each rows as r}
            <div class="row" class:mine={r.mine}>
              <span class="pos">{r.pos}</span>
              <div class="club">
                <span class="dot" style="background:{r.dotColor}"></span>
                <span
                  class="club-name"
                  style="font-weight:{r.mine ? 700 : 500};color:{r.mine
                    ? 'var(--accent)'
                    : 'var(--text)'}">{r.club}</span
                >
              </div>
              {#each r.cells as cell, ci}
                <span
                  class="num"
                  style="width:{COLS[ci].w};{cell.bold
                    ? 'color:var(--text);font-weight:700'
                    : ''}">{cell.text}</span
                >
              {/each}
              <span class="form">
                {#each r.form as f}
                  <span class="chip" style="background:{f.bg};color:{f.fg}">{f.c}</span>
                {/each}
              </span>
            </div>
          {/each}
        </div>
      </div>
    </section>

    {#if scorers.length > 0}
      <section class="card">
        <div class="card-head">
          <div>
            <div class="title">Top Scorers</div>
            <div class="sub">this season</div>
          </div>
        </div>
        {#each scorers as s, i}
          <div class="srow">
            <span class="srank">{i + 1}</span>
            <div class="splayer">
              <div class="sname">{s.name ?? "—"}</div>
              <div class="steam">{s.team_id != null ? teamName(s.team_id) : "—"}</div>
            </div>
            <span class="sapps">{s.apps} apps</span>
            <span class="sgoals">{s.goals}</span>
          </div>
        {/each}
      </section>
    {/if}
  </div>
{/if}

<style>
  .empty-page {
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--faint);
    padding: 60px 22px;
  }

  .page {
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1180px;
  }

  .card {
    background: #14181e;
    border: 1px solid #232a33;
    border-radius: 0;
    overflow: hidden;
  }

  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 14px 16px;
    border-bottom: 1px solid #232a33;
  }
  .title {
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
  }
  .sub {
    font-size: 11.5px;
    color: #7a828d;
    font-family: var(--font-mono);
    margin-top: 2px;
  }

  .legend {
    display: flex;
    gap: 14px;
  }
  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .swatch {
    width: 9px;
    height: 9px;
    border-radius: 0;
    flex: none;
  }
  .legend-label {
    font-size: 11px;
    color: #8b95a1;
  }

  .scroll-x {
    overflow-x: auto;
  }
  .table-min {
    min-width: 640px;
  }

  .col-head {
    display: flex;
    align-items: center;
    padding: 9px 16px;
    background: #12161c;
    border-bottom: 1px solid #232a33;
  }
  .ch-pos {
    width: 38px;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    text-align: center;
  }
  .ch-club {
    flex: 1;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .ch-num {
    text-align: center;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
  }
  .ch-form {
    width: 118px;
    text-align: center;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .form {
    width: 118px;
    display: flex;
    gap: 3px;
    justify-content: center;
  }
  .chip {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-size: 9.5px;
    font-weight: 700;
  }

  .row {
    display: flex;
    align-items: center;
    padding: 9px 16px;
    border-bottom: 1px solid #232a33;
  }
  .row.mine {
    background: var(--accent-soft);
  }

  .pos {
    width: 38px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: #9aa4b0;
  }
  .club {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
  }
  .club-name {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .num {
    text-align: center;
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: var(--text-3);
  }

  /* top scorers */
  .srow {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 9px 16px;
    border-bottom: 1px solid #171c22;
  }
  .srank {
    width: 20px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: #616b77;
    text-align: center;
  }
  .splayer {
    flex: 1;
    min-width: 0;
  }
  .sname {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
  }
  .steam {
    font-size: 11px;
    color: #6b7480;
    font-family: var(--font-mono);
  }
  .sapps {
    font-size: 11px;
    color: #7a828d;
    font-family: var(--font-mono);
  }
  .sgoals {
    width: 40px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 14px;
    font-weight: 700;
    color: var(--accent);
  }
</style>
