<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import type { SportTheme } from "../design/theme";
  import type { GcRow } from "../design/dto";

  let {
    theme,
    gc,
    busy,
    onRun,
  }: {
    theme: SportTheme;
    gc: GcRow[];
    busy: boolean;
    onRun: () => void;
  } = $props();

  const LEADER = "#f2c14e"; // gold — matches cycling accent

  // "+M:SS" from gap seconds; the leader shows an em dash.
  function gapStr(rank: number, secs: number): string {
    if (rank === 1) return "—";
    const s = Math.max(0, Math.round(secs));
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `+${m}:${String(sec).padStart(2, "0")}`;
  }

  // Podium tone for the rank chip: 1 gold, 2/3 accent-ish, rest muted.
  function rankColor(rank: number): string {
    if (rank === 1) return LEADER;
    if (rank <= 3) return "#5aa9e6";
    return "#3a424c";
  }

  const raced = $derived(gc.length > 0);
  const winner = $derived(gc[0]?.name ?? "—");
</script>

<div class="page">
  <section class="card">
    <!-- head -->
    <div class="card-head">
      <div>
        <div class="title">General Classification</div>
        <div class="sub mono">
          {raced ? `${gc.length} finishers · winner ${winner}` : "7-stage grand tour"}
        </div>
      </div>
      <button class="run-btn" disabled={busy} onclick={onRun}>
        <Icon name="trophy" size={15} />
        <span>{busy ? "Racing…" : raced ? "Re-run the Tour" : "Run the Tour"}</span>
      </button>
    </div>

    {#if !raced}
      <div class="empty-page">
        The road is empty. Press <b>Run the Tour</b> to settle the classification.
      </div>
    {:else}
      <div class="col-head">
        <span class="ch-pos">#</span>
        <span class="ch-rider">RIDER</span>
        <span class="ch-gap mono">GAP</span>
      </div>

      <div class="rows">
        {#each gc as r}
          <div class="row" class:leader={r.rank === 1}>
            <span class="pos-wrap">
              <span class="dot" style="background:{rankColor(r.rank)}"></span>
              <span class="pos mono">{r.rank}</span>
            </span>
            <span
              class="rider"
              style="font-weight:{r.rank === 1 ? 700 : 500};color:{r.rank === 1
                ? LEADER
                : 'var(--text)'}">{r.name}</span>
            <span
              class="gap mono"
              style="color:{r.rank === 1 ? LEADER : 'var(--text-3)'}">
              {gapStr(r.rank, r.gap_secs)}
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
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
    margin-top: 2px;
  }

  .run-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 15px;
    background: var(--accent);
    color: #1a1405;
    font-weight: 700;
    font-size: 12.5px;
    border: none;
    border-radius: 0;
    cursor: pointer;
    box-shadow: 0 0 18px var(--accent-soft);
    font-family: inherit;
    flex: none;
  }
  .run-btn:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .run-btn:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .empty-page {
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--faint);
    padding: 60px 22px;
  }
  .empty-page b {
    color: var(--accent);
    font-weight: 600;
  }

  .col-head {
    display: flex;
    align-items: center;
    padding: 9px 16px;
    background: #12161c;
    border-bottom: 1px solid #232a33;
  }
  .ch-pos {
    width: 46px;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .ch-rider {
    flex: 1;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .ch-gap {
    width: 90px;
    text-align: right;
    font-size: 9.5px;
    color: #616b77;
    letter-spacing: 0.05em;
  }

  .rows {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    padding: 10px 16px;
    border-bottom: 1px solid #232a33;
  }
  .row:hover {
    background: #161b22;
  }
  .row.leader {
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .pos-wrap {
    width: 46px;
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
  }
  .pos {
    font-size: 12.5px;
    color: #9aa4b0;
  }
  .rider {
    flex: 1;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .gap {
    width: 90px;
    text-align: right;
    font-size: 12.5px;
    font-weight: 600;
  }
</style>
