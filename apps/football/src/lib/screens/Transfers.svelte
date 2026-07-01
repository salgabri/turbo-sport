<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import { money, tierColor, posPill } from "../design/color";
  import { FOOTBALL, posColor, type SportTheme } from "../design/theme";
  import type { ClubView, PlayerView } from "../design/dto";

  let {
    theme,
    market,
    club,
  }: { theme: SportTheme; market: PlayerView[]; club: ClubView | null } = $props();

  const cur = $derived(theme.currency ?? FOOTBALL.currency);

  // ---- client-side state ----
  const POS_FILTERS = ["All", "GK", "DEF", "MID", "FWD"] as const;
  let posFilter = $state<(typeof POS_FILTERS)[number]>("All");
  let shortlist = $state<string[]>([]);

  // ---- stat cards ----
  const balance = $derived(club?.balance ?? 0);
  const wageBill = $derived(club?.wage_bill ?? 0);
  const income = $derived(club?.weekly_income ?? 0);
  // Wage bar = wage_bill / weekly_income, clamped 0–100.
  const wagePct = $derived(
    income > 0 ? Math.min(100, Math.max(0, (wageBill / income) * 100)) : 0,
  );

  // ---- targets: free agents from the market prop ----
  const targets = $derived(
    market
      .filter((p) => p.free_agent)
      .filter((p) => posFilter === "All" || p.position_group === posFilter),
  );

  function meta(p: PlayerView): string {
    const bits: string[] = [];
    if (p.age != null) bits.push(`${p.age}y`);
    const loc = p.nationality ?? (p.free_agent ? "Free agent" : null);
    if (loc) bits.push(loc);
    return bits.join(" · ") || "—";
  }

  function shortMeta(name: string): { club: string; value: string } {
    const p = market.find((m) => m.name === name);
    return {
      club: p?.nationality ?? "Free agent",
      value: p?.market_value != null ? money(p.market_value, cur) : "—",
    };
  }

  function toggle(name: string | null) {
    if (!name) return;
    if (shortlist.includes(name)) shortlist = shortlist.filter((n) => n !== name);
    else shortlist = [...shortlist, name];
  }
  function remove(name: string) {
    shortlist = shortlist.filter((n) => n !== name);
  }
</script>

<div class="wrap">
  <!-- ===== stat cards ===== -->
  <div class="cards">
    <div class="card">
      <div class="card-label">Transfer Budget</div>
      <div class="card-big">{money(balance, cur)}</div>
      <div class="bar">
        <div
          class="bar-fill"
          style="width:72%;background:linear-gradient(90deg,var(--accent),var(--accent))"></div>
      </div>
      <div class="card-sub">Available to spend</div>
    </div>

    <div class="card">
      <div class="card-label">Wage Budget</div>
      <div class="card-big">{money(wageBill, cur)} / {money(income, cur)}</div>
      <div class="bar">
        <div
          class="bar-fill"
          style="width:{wagePct}%;background:{wagePct > 90 ? 'var(--down)' : wagePct > 70 ? 'var(--warn)' : 'var(--accent)'}"></div>
      </div>
      <div class="card-sub">Weekly commitment</div>
    </div>

    <div class="card shortlist-card">
      <div class="card-label">Shortlist</div>
      <div class="card-big">
        <span style="color:var(--accent)">{shortlist.length}</span>
        <span class="card-big-unit">targets</span>
      </div>
    </div>
  </div>

  <!-- ===== filter bar ===== -->
  <div class="filters">
    <span class="fl-label">Position</span>
    <div class="pills">
      {#each POS_FILTERS as f (f)}
        <button
          class="pill"
          class:on={posFilter === f}
          onclick={() => (posFilter = f)}>{f}</button>
      {/each}
    </div>
    <div class="divider"></div>
    <div class="chip-group">
      <span class="chip-label">Age</span><span class="chip">18–32</span>
    </div>
    <div class="chip-group">
      <span class="chip-label">Max fee</span><span class="chip">{money(balance, cur)}</span>
    </div>
    <div class="chip-group">
      <span class="chip-label">Min rating</span><span class="chip">74</span>
    </div>
  </div>

  <!-- ===== table + shortlist ===== -->
  <div class="grid">
    <section class="table">
      <div class="thead">
        <span class="th-pos">POS</span>
        <span class="th-target">TARGET</span>
        <span class="th-ovr">OVR</span>
        <span class="th-value">VALUE</span>
        <span class="th-interest">INTEREST</span>
        <span class="th-add"></span>
      </div>

      {#if targets.length === 0}
        <div class="empty">No transfer targets available.</div>
      {:else}
        {#each targets as t (t.name)}
          {@const inList = t.name != null && shortlist.includes(t.name)}
          <div class="row">
            <span class="td-pos">
              {#if t.position_group}
                <span style={posPill(posColor(theme, t.position_group))}>{t.position_group}</span>
              {:else}
                <span class="dash-pill">—</span>
              {/if}
            </span>
            <div class="td-target">
              <div class="t-name">{t.name ?? "—"}</div>
              <div class="t-meta">{meta(t)}</div>
            </div>
            <span
              class="td-ovr"
              style="color:{t.overall != null ? tierColor(t.overall) : 'var(--dim)'}"
              >{t.overall ?? "—"}</span>
            <span class="td-value"
              >{t.market_value != null ? money(t.market_value, cur) : "—"}</span>
            <span class="td-interest">—</span>
            <span class="td-add">
              <button
                class="addbtn"
                class:added={inList}
                onclick={() => toggle(t.name)}
                aria-label={inList ? "Remove from shortlist" : "Add to shortlist"}>
                {#if inList}
                  <Icon name="check" size={15} />
                {:else}
                  <span class="plus">+</span>
                {/if}
              </button>
            </span>
          </div>
        {/each}
      {/if}
    </section>

    <aside class="aside">
      <div class="aside-head">
        <span class="aside-title">Shortlist</span>
        <span class="aside-count">{shortlist.length}</span>
      </div>
      {#if shortlist.length > 0}
        <div class="sl-list">
          {#each shortlist as name (name)}
            {@const m = shortMeta(name)}
            <div class="sl-row">
              <div class="sl-info">
                <div class="sl-name">{name}</div>
                <div class="sl-meta">{m.club} · {m.value}</div>
              </div>
              <button class="sl-remove" onclick={() => remove(name)} aria-label="Remove">
                <Icon name="x" size={14} />
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <div class="aside-empty">Add targets from the list to build your shortlist.</div>
      {/if}
    </aside>
  </div>
</div>

<style>
  .wrap {
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1200px;
  }

  /* ---- stat cards ---- */
  .cards {
    display: flex;
    gap: 12px;
  }
  .card {
    flex: 1;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 13px 15px;
  }
  .shortlist-card {
    width: 200px;
    flex: none;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .card-label {
    font-size: 10px;
    letter-spacing: 0.08em;
    font-weight: 600;
    color: var(--dim-2);
    font-family: var(--font-mono);
    text-transform: uppercase;
  }
  .card-big {
    font-size: 24px;
    font-weight: 800;
    letter-spacing: -0.02em;
    margin: 5px 0 8px;
  }
  .shortlist-card .card-big {
    margin: 5px 0 0;
  }
  .card-big-unit {
    font-size: 14px;
    color: var(--muted-3);
    font-weight: 500;
  }
  .bar {
    height: 5px;
    background: var(--line);
    border-radius: 3px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    border-radius: 3px;
  }
  .card-sub {
    font-size: 11px;
    color: #828b96;
    margin-top: 7px;
  }

  /* ---- filter bar ---- */
  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 9px 14px;
  }
  .fl-label {
    font-size: 10px;
    color: var(--dim-2);
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .pill {
    font-family: inherit;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--muted-2);
    background: #0f1319;
    border: 1px solid var(--line);
    padding: 4px 11px;
    border-radius: 7px;
    cursor: pointer;
  }
  .pill:hover {
    color: var(--text-3);
    border-color: #2f3742;
  }
  .pill.on {
    background: var(--accent-soft);
    border-color: var(--accent-line);
    color: var(--accent);
  }
  .divider {
    width: 1px;
    height: 22px;
    background: #2a323c;
    margin: 0 3px;
  }
  .chip-group {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .chip-label {
    font-size: 11px;
    color: var(--muted-2);
  }
  .chip {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-3);
    background: #0f1319;
    border: 1px solid var(--line);
    padding: 3px 8px;
    border-radius: 6px;
  }

  /* ---- table + aside ---- */
  .grid {
    display: flex;
    gap: 16px;
    align-items: start;
  }
  .table {
    flex: 1;
    min-width: 0;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 14px;
    overflow: hidden;
  }
  .thead {
    display: flex;
    align-items: center;
    padding: 9px 14px;
    background: var(--panel-2);
    border-bottom: 1px solid var(--line);
  }
  .thead span {
    font-size: 9.5px;
    color: var(--dim-2);
    font-family: var(--font-mono);
  }
  .th-pos {
    width: 30px;
    flex: none;
  }
  .th-target {
    flex: 1;
    min-width: 0;
    letter-spacing: 0.05em;
  }
  .th-ovr {
    width: 38px;
    flex: none;
    text-align: center;
  }
  .th-value {
    width: 60px;
    flex: none;
    text-align: right;
  }
  .th-interest {
    width: 72px;
    flex: none;
    text-align: center;
  }
  .th-add {
    width: 40px;
    flex: none;
  }

  .row {
    display: flex;
    align-items: center;
    padding: 9px 14px;
    border-bottom: 1px solid var(--line-2);
  }
  .row:hover {
    background: var(--hover-2);
  }
  .td-pos {
    width: 30px;
    flex: none;
  }
  .dash-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 34px;
    height: 20px;
    padding: 0 6px;
    border-radius: 5px;
    font-size: 10.5px;
    font-weight: 700;
    font-family: var(--font-mono);
    background: #1b212a;
    color: var(--dim);
  }
  .td-target {
    flex: 1;
    min-width: 0;
    padding-right: 8px;
  }
  .t-name {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .t-meta {
    font-size: 10.5px;
    color: var(--dim);
    font-family: var(--font-mono);
    margin-top: 1px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .td-ovr {
    width: 38px;
    flex: none;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 700;
  }
  .td-value {
    width: 60px;
    flex: none;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-2);
  }
  .td-interest {
    width: 72px;
    flex: none;
    text-align: center;
    font-size: 11px;
    font-weight: 600;
    color: var(--dim);
  }
  .td-add {
    width: 40px;
    flex: none;
    display: flex;
    justify-content: center;
  }
  .addbtn {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 7px;
    background: #1b212a;
    border: 1px solid #2a323c;
    color: var(--muted-2);
    cursor: pointer;
    padding: 0;
  }
  .addbtn:hover {
    border-color: var(--accent-line);
    color: var(--accent);
  }
  .addbtn.added {
    background: var(--accent-soft);
    border-color: var(--accent-line);
    color: var(--accent);
  }
  .plus {
    font-size: 17px;
    font-weight: 600;
    line-height: 1;
  }

  .empty {
    padding: 48px 18px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--faint);
  }

  /* ---- shortlist aside ---- */
  .aside {
    width: 200px;
    flex: none;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 14px;
    overflow: hidden;
  }
  .aside-head {
    padding: 12px 15px;
    border-bottom: 1px solid var(--line);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .aside-title {
    font-size: 13px;
    font-weight: 700;
  }
  .aside-count {
    font-size: 11px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent);
    background: var(--accent-soft);
    padding: 2px 7px;
    border-radius: 20px;
  }
  .sl-list {
    padding: 6px;
  }
  .sl-row {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 9px 10px;
    border-radius: 9px;
  }
  .sl-row:hover {
    background: var(--hover);
  }
  .sl-info {
    flex: 1;
    min-width: 0;
  }
  .sl-name {
    font-size: 12.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sl-meta {
    font-size: 10.5px;
    color: var(--dim);
    font-family: var(--font-mono);
  }
  .sl-remove {
    width: 24px;
    height: 24px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    color: var(--dim);
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
  }
  .sl-remove:hover {
    background: #331e21;
    color: var(--down);
  }
  .aside-empty {
    padding: 26px 18px;
    text-align: center;
    color: var(--faint);
    font-size: 12px;
    line-height: 1.5;
  }
</style>
