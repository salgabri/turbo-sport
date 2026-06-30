<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type RiderRow = { name: string; climbing: number; sprinting: number; time_trial: number; endurance: number };
  type GcRow = { rank: number; name: string; gap_secs: number };

  let roster = $state<RiderRow[]>([]);
  let gc = $state<GcRow[]>([]);
  let busy = $state(false);
  let status = $state("");

  const gap = (s: number): string => {
    if (s === 0) return "leader";
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return "+" + (m > 0 ? m + "' " : "") + sec + '"';
  };

  async function loadRoster() {
    roster = await invoke<RiderRow[]>("roster");
  }
  async function runTour() {
    busy = true;
    status = "racing…";
    try {
      gc = await invoke<GcRow[]>("run_tour");
      status = "grand tour complete — winner: " + (gc[0]?.name ?? "—");
    } finally {
      busy = false;
    }
  }

  onMount(loadRoster);
</script>

<div class="wrap">
  <header>
    <h1>Turbo Cycling</h1>
    <div class="meta">{roster.length} riders · 7-stage grand tour</div>
    <button disabled={busy} onclick={runTour}>{busy ? "racing…" : "run grand tour"}</button>
    <span class="status">{status}</span>
  </header>

  <div class="grid">
    <section>
      <h2>General classification</h2>
      {#if gc.length === 0}
        <p class="empty">Press “run grand tour” to race.</p>
      {:else}
        <table>
          <thead><tr><th>#</th><th>rider</th><th>gap</th></tr></thead>
          <tbody>
            {#each gc as r}
              <tr>
                <td class="dim">{r.rank}</td>
                <td>{r.name}</td>
                <td class:lead={r.gap_secs === 0}>{gap(r.gap_secs)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section>
      <h2>Peloton</h2>
      <table>
        <thead><tr><th>rider</th><th>clb</th><th>spr</th><th>tt</th><th>end</th></tr></thead>
        <tbody>
          {#each roster as r}
            <tr>
              <td>{r.name}</td>
              <td>{r.climbing}</td>
              <td>{r.sprinting}</td>
              <td>{r.time_trial}</td>
              <td>{r.endurance}</td>
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
  .status {
    color: #9aa0ab;
    font-size: 12px;
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
    grid-template-columns: 1fr 1fr;
    gap: 24px;
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
    padding: 6px 8px;
    border-bottom: 1px solid #181b22;
  }
  td.dim {
    color: #6b7280;
  }
  td.lead {
    color: #7bd88f;
  }
  .empty {
    color: #6b7280;
  }
</style>
