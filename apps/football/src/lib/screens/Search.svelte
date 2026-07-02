<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import { tierColor, money, posPill } from "../design/color";
  import { posColor, type SportTheme } from "../design/theme";
  import type { PlayerView, SearchArgs } from "../design/dto";

  let {
    theme,
    results,
    onSearch,
    loading = false,
    teamName,
  }: {
    theme: SportTheme;
    results: PlayerView[];
    onSearch: (args: SearchArgs) => void;
    loading?: boolean;
    teamName: (id: number) => string;
  } = $props();

  // ---- filters ----
  let position = $state<number | null>(null); // null = any, else position group index
  let minAge = $state(15);
  let maxAge = $state(40);
  let minOverall = $state(0);
  let freeOnly = $state(false);

  const posPills = $derived<{ i: number | null; label: string }[]>([
    { i: null, label: "All" },
    ...theme.positions.map((p, i) => ({ i, label: p.group })),
  ]);

  function run() {
    onSearch({ position, minAge, maxAge, minOverall, freeOnly });
  }

  const cur = $derived(theme.currency);
  const DASH = "—";
  function posStyle(g: string | null | undefined): string {
    return posPill(g ? posColor(theme, g) : "#7a828d", "sm");
  }
</script>

<div class="wrap">
  <!-- filter bar -->
  <div class="filters">
    <div class="fgroup">
      <span class="flabel mono">POSITION</span>
      <div class="pills">
        {#each posPills as p}
          <div
            class="pill"
            class:on={position === p.i}
            role="button"
            tabindex="0"
            onclick={() => (position = p.i)}
            onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (position = p.i)}>
            {p.label}
          </div>
        {/each}
      </div>
    </div>

    <div class="fgroup">
      <span class="flabel mono">AGE</span>
      <input class="num" type="number" min="15" max="45" bind:value={minAge} />
      <span class="dashsep">–</span>
      <input class="num" type="number" min="15" max="45" bind:value={maxAge} />
    </div>

    <div class="fgroup">
      <span class="flabel mono">MIN OVR</span>
      <input class="num" type="number" min="0" max="99" bind:value={minOverall} />
    </div>

    <label class="chk">
      <input type="checkbox" bind:checked={freeOnly} />
      <span>Free agents only</span>
    </label>

    <div class="spacer"></div>
    <button class="go" disabled={loading} onclick={run}>
      <Icon name="search" size={15} />
      <span>Search</span>
    </button>
  </div>

  <!-- results -->
  <div class="results">
    <div class="rhead">
      <span class="c-pos">POS</span>
      <span class="c-name">PLAYER</span>
      <span class="c-ovr">OVR</span>
      <span class="c-age">AGE</span>
      <span class="c-val">VALUE</span>
      <span class="c-status">STATUS</span>
    </div>
    {#if results.length === 0}
      <div class="empty">Set filters and press Search to scan the world.</div>
    {:else}
      {#each results as p}
        <div class="rrow">
          <span class="c-pos"><span style={posStyle(p.position_group)}>{p.position_group ?? DASH}</span></span>
          <div class="c-name pcol">
            <div class="pname">{p.name ?? DASH}</div>
            <div class="pmeta mono">
              {typeof p.age === "number" ? p.age : DASH}{p.nationality ? " · " + p.nationality : ""}
              {p.team_id != null ? " · " + teamName(p.team_id) : ""}
            </div>
          </div>
          <span class="c-ovr mono" style="color:{typeof p.overall === 'number' ? tierColor(p.overall) : '#5a636e'}">
            {typeof p.overall === "number" ? p.overall : DASH}
          </span>
          <span class="c-age mono">{typeof p.age === "number" ? p.age : DASH}</span>
          <span class="c-val mono">{typeof p.market_value === "number" ? money(p.market_value, cur) : DASH}</span>
          <span class="c-status">{p.free_agent ? "Free agent" : "Contracted"}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .filters {
    flex: none;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    padding: 12px 20px;
    border-bottom: 1px solid #191e25;
    background: #0e1115;
  }
  .fgroup {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .flabel {
    font-size: 9.5px;
    letter-spacing: 0.06em;
    color: #565f6a;
  }
  .pills {
    display: flex;
    align-items: center;
    gap: 3px;
    background: #0f1319;
    border: 1px solid #232a33;
    border-radius: 9px;
    padding: 3px;
  }
  .pill {
    padding: 4px 10px;
    border-radius: 7px;
    font-size: 11px;
    font-weight: 600;
    color: #8b95a1;
    cursor: pointer;
    user-select: none;
  }
  .pill.on {
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent-line);
  }
  .num {
    width: 52px;
    background: #0f1319;
    border: 1px solid #232a33;
    border-radius: 7px;
    color: #d4dae1;
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 5px 7px;
    text-align: right;
  }
  .dashsep {
    color: #5a636e;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: #9aa4b0;
    cursor: pointer;
  }
  .spacer {
    flex: 1;
  }
  .go {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--accent);
    color: #08120c;
    border: none;
    border-radius: 9px;
    font-size: 13px;
    font-weight: 700;
    font-family: inherit;
    cursor: pointer;
  }
  .go:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .go:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .results {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .rhead,
  .rrow {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 20px;
  }
  .rhead {
    position: sticky;
    top: 0;
    background: #12161c;
    border-bottom: 1px solid #232a33;
    font-size: 9.5px;
    letter-spacing: 0.05em;
    color: #616b77;
    font-family: var(--font-mono);
    z-index: 2;
  }
  .rrow {
    border-bottom: 1px solid #171c22;
  }
  .rrow:hover {
    background: #161b22;
  }
  .c-pos {
    width: 44px;
    flex: none;
  }
  .c-name {
    flex: 1;
    min-width: 0;
  }
  .pname {
    font-size: 13.5px;
    font-weight: 600;
    color: #e9edf1;
    white-space: nowrap;
  }
  .pmeta {
    font-size: 11px;
    color: #6b7480;
    margin-top: 1px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .c-ovr {
    width: 44px;
    flex: none;
    text-align: right;
    font-weight: 700;
    font-size: 13px;
  }
  .c-age {
    width: 44px;
    flex: none;
    text-align: right;
    color: #9aa4b0;
    font-size: 12px;
  }
  .c-val {
    width: 84px;
    flex: none;
    text-align: right;
    color: #c2cad3;
    font-size: 12px;
  }
  .c-status {
    width: 96px;
    flex: none;
    text-align: right;
    font-size: 11.5px;
    color: #8b95a1;
  }
  .empty {
    padding: 60px;
    text-align: center;
    color: #5a636e;
    font-family: var(--font-mono);
    font-size: 13px;
  }
</style>
