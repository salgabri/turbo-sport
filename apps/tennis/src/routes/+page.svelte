<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type PlayerRow = { seed: number; name: string; serve: number; return_game: number; baseline: number; mental: number };
  type MatchRow = { winner: string; loser: string; score: string };
  type RoundOut = { name: string; matches: MatchRow[] };
  type Tourney = { champion: string; rounds: RoundOut[] };

  let draw = $state<PlayerRow[]>([]);
  let result = $state<Tourney | null>(null);
  let busy = $state(false);

  async function loadDraw() {
    draw = await invoke<PlayerRow[]>("draw");
  }
  async function play() {
    busy = true;
    try {
      result = await invoke<Tourney>("run_tournament");
    } finally {
      busy = false;
    }
  }

  onMount(loadDraw);
</script>

<div class="wrap">
  <header>
    <h1>Turbo Tennis</h1>
    <div class="meta">{draw.length}-player draw · single elimination</div>
    <button disabled={busy} onclick={play}>{busy ? "playing…" : "play tournament"}</button>
    {#if result}<span class="champ"><i class="ti ti-trophy" aria-hidden="true"></i> champion: {result.champion}</span>{/if}
  </header>

  <div class="grid">
    <section>
      <h2>Bracket</h2>
      {#if !result}
        <p class="empty">Press “play tournament” to run the draw.</p>
      {:else}
        {#each result.rounds as round}
          <div class="round">
            <div class="rname">{round.name}</div>
            {#each round.matches as m}
              <div class="match"><span class="w">{m.winner}</span> def. <span class="l">{m.loser}</span> <span class="sc">{m.score}</span></div>
            {/each}
          </div>
        {/each}
      {/if}
    </section>

    <section>
      <h2>Seeds</h2>
      <table>
        <thead><tr><th>seed</th><th>player</th><th>srv</th><th>ret</th><th>bl</th><th>men</th></tr></thead>
        <tbody>
          {#each draw as p}
            <tr>
              <td class="dim">{p.seed + 1}</td>
              <td>{p.name}</td>
              <td>{p.serve}</td>
              <td>{p.return_game}</td>
              <td>{p.baseline}</td>
              <td>{p.mental}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    background: #0f1115;
    color: #d7dae0;
    font: 13px/1.4 ui-sans-serif, system-ui, sans-serif;
  }
  .wrap {
    padding: 16px 20px;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-wrap: wrap;
    border-bottom: 1px solid #232733;
    padding-bottom: 12px;
    margin-bottom: 14px;
  }
  h1 {
    margin: 0;
    font-size: 20px;
  }
  h2 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #6b7280;
    margin: 0 0 8px;
  }
  .meta {
    color: #9aa0ab;
    font-size: 12px;
  }
  .champ {
    color: #7bd88f;
    font-weight: 500;
  }
  button {
    background: #4f8cff;
    border: 1px solid #4f8cff;
    color: #fff;
    border-radius: 6px;
    padding: 6px 14px;
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 360px;
    gap: 24px;
  }
  .round {
    margin-bottom: 14px;
  }
  .rname {
    font-size: 11px;
    text-transform: uppercase;
    color: #6b7280;
    margin-bottom: 4px;
  }
  .match {
    padding: 4px 0;
    border-bottom: 1px solid #181b22;
    color: #9aa0ab;
  }
  .match .w {
    color: #d7dae0;
    font-weight: 500;
  }
  .match .sc {
    color: #6b7280;
    font-variant-numeric: tabular-nums;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-variant-numeric: tabular-nums;
  }
  thead th {
    text-align: left;
    font-size: 11px;
    text-transform: uppercase;
    color: #6b7280;
    border-bottom: 1px solid #232733;
    padding: 6px 8px;
  }
  tbody td {
    padding: 5px 8px;
    border-bottom: 1px solid #181b22;
  }
  td.dim {
    color: #6b7280;
  }
  .empty {
    color: #6b7280;
  }
</style>
