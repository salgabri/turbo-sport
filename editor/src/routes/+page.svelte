<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type DbDate = { year: number; month: number; day: number };
  type ClubRecord = { id: number; name: string; balance: number; weekly_income: number; squad_target: number };
  type PlayerRecord = {
    name: string;
    club_id: number | null;
    birth: DbDate;
    attacking: number;
    defending: number;
    finishing: number;
    goalkeeping: number;
    wage: number;
    contract_until: DbDate;
  };
  type DivisionRecord = { name: string; tier: number; club_ids: number[] };
  type Database = {
    name: string;
    start_date: DbDate;
    seed: number;
    divisions: DivisionRecord[];
    clubs: ClubRecord[];
    players: PlayerRecord[];
  };

  let db = $state<Database | null>(null);
  let selected = $state<number | "free" | null>(null);
  let savePath = $state("database.json");
  let openPath = $state("database.json");
  let status = $state("");

  const club = $derived(
    db && typeof selected === "number" ? db.clubs.find((c) => c.id === selected) : undefined,
  );
  const players = $derived.by((): PlayerRecord[] => {
    if (!db) return [];
    if (selected === "free") return db.players.filter((p) => p.club_id === null);
    return db.players.filter((p) => p.club_id === selected);
  });

  function newPlayer(clubId: number | null): PlayerRecord {
    return {
      name: "New Player",
      club_id: clubId,
      birth: { year: 2005, month: 1, day: 1 },
      attacking: 50,
      defending: 50,
      finishing: 50,
      goalkeeping: 50,
      wage: 1000,
      contract_until: { year: 2028, month: 6, day: 30 },
    };
  }
  function addPlayer() {
    if (!db) return;
    db.players.push(newPlayer(selected === "free" ? null : (selected as number)));
  }
  function removePlayer(p: PlayerRecord) {
    if (!db) return;
    const i = db.players.indexOf(p);
    if (i >= 0) db.players.splice(i, 1);
  }

  async function loadSample() {
    db = await invoke<Database>("load_sample");
    selected = db.clubs[0]?.id ?? null;
    status = "loaded sample";
  }
  async function open() {
    try {
      db = await invoke<Database>("open", { path: openPath });
      selected = db.clubs[0]?.id ?? null;
      status = `opened ${openPath}`;
    } catch (e) {
      status = `open error: ${e}`;
    }
  }
  async function save() {
    if (!db) return;
    try {
      await invoke("save", { path: savePath, db });
      status = `saved to ${savePath}`;
    } catch (e) {
      status = `save error: ${e}`;
    }
  }
  async function check() {
    if (!db) return;
    try {
      await invoke("validate", { db });
      status = "valid ✓";
    } catch (e) {
      status = `invalid: ${e}`;
    }
  }

  onMount(loadSample);
</script>

<header>
  {#if db}
    <input class="dbname" bind:value={db.name} placeholder="Database name" />
  {:else}
    <span class="dbname">…</span>
  {/if}
  <div class="spacer"></div>
  <input class="path" bind:value={openPath} /><button onclick={open}>Open</button>
  <button onclick={loadSample}>Sample</button>
  <span class="gap"></span>
  <input class="path" bind:value={savePath} /><button class="primary" onclick={save}>Save</button>
  <button onclick={check}>Validate</button>
  <span class="status">{status}</span>
</header>

{#if db}
  <main>
    <aside>
      <h2>Clubs</h2>
      <ul>
        {#each db.clubs as c (c.id)}
          <li>
            <button class:active={selected === c.id} onclick={() => (selected = c.id)}>{c.name}</button>
          </li>
        {/each}
        <li class="div">Pool</li>
        <li>
          <button class:active={selected === "free"} onclick={() => (selected = "free")}>Free agents</button>
        </li>
      </ul>
    </aside>

    <section>
      {#if club}
        <div class="clubfields">
          <label>Name <input bind:value={club.name} /></label>
          <label>Balance <input type="number" bind:value={club.balance} /></label>
          <label>Income/wk <input type="number" bind:value={club.weekly_income} /></label>
          <label>Squad target <input type="number" bind:value={club.squad_target} /></label>
        </div>
      {:else}
        <h1>Free agents</h1>
      {/if}

      <div class="toolbar">
        <span>{players.length} players</span>
        <button onclick={addPlayer}>+ Add player</button>
      </div>

      <table>
        <thead>
          <tr><th>Name</th><th>Born</th><th>ATT</th><th>DEF</th><th>FIN</th><th>GK</th><th>Wage</th><th></th></tr>
        </thead>
        <tbody>
          {#each players as p}
            <tr>
              <td><input class="wide" bind:value={p.name} /></td>
              <td><input class="yr" type="number" bind:value={p.birth.year} /></td>
              <td><input class="num" type="number" min="1" max="99" bind:value={p.attacking} /></td>
              <td><input class="num" type="number" min="1" max="99" bind:value={p.defending} /></td>
              <td><input class="num" type="number" min="1" max="99" bind:value={p.finishing} /></td>
              <td><input class="num" type="number" min="1" max="99" bind:value={p.goalkeeping} /></td>
              <td><input class="num" type="number" bind:value={p.wage} /></td>
              <td><button class="del" onclick={() => removePlayer(p)}>✕</button></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  </main>
{:else}
  <p class="loading">Loading…</p>
{/if}

<style>
  :global(body) {
    margin: 0;
    background: #0f1115;
    color: #d7dae0;
    font: 13px/1.4 ui-sans-serif, system-ui, sans-serif;
  }
  header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border-bottom: 1px solid #232733;
    background: #12141a;
  }
  header .spacer {
    flex: 1;
  }
  header .gap {
    width: 12px;
  }
  .dbname {
    font-weight: 600;
    width: 200px;
  }
  .path {
    width: 150px;
  }
  .status {
    color: #9aa0ab;
    font-size: 12px;
    margin-left: 8px;
  }
  input {
    background: #0d0f14;
    border: 1px solid #2a2f3a;
    color: #d7dae0;
    border-radius: 4px;
    padding: 4px 6px;
    font: inherit;
  }
  input:focus {
    outline: none;
    border-color: #4f8cff;
  }
  button {
    background: #1b1f2a;
    border: 1px solid #2a2f3a;
    color: #d7dae0;
    border-radius: 5px;
    padding: 5px 10px;
    cursor: pointer;
  }
  button:hover {
    border-color: #4f8cff;
  }
  button.primary {
    background: #4f8cff;
    border-color: #4f8cff;
    color: #fff;
  }
  main {
    display: grid;
    grid-template-columns: 220px 1fr;
    height: calc(100vh - 49px);
  }
  aside {
    border-right: 1px solid #232733;
    overflow-y: auto;
    background: #12141a;
  }
  aside h2 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #6b7280;
    padding: 12px 14px 6px;
    margin: 0;
  }
  aside ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  aside li.div {
    font-size: 11px;
    color: #6b7280;
    padding: 10px 14px 2px;
  }
  aside li button {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    border-left: 2px solid transparent;
    border-radius: 0;
    padding: 7px 14px;
  }
  aside li button:hover {
    background: #1a1d26;
  }
  aside li button.active {
    background: #1a1d26;
    border-left-color: #4f8cff;
  }
  section {
    overflow-y: auto;
    padding: 14px 18px;
  }
  .clubfields {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-bottom: 12px;
  }
  .clubfields label {
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 11px;
    color: #9aa0ab;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 8px 0;
    color: #9aa0ab;
  }
  table {
    width: 100%;
    border-collapse: collapse;
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
    padding: 3px 6px;
    border-bottom: 1px solid #181b22;
  }
  .wide {
    width: 150px;
  }
  .num {
    width: 48px;
  }
  .yr {
    width: 60px;
  }
  .del {
    padding: 2px 7px;
    color: #ff6b6b;
  }
  .loading {
    padding: 40px;
    color: #6b7280;
  }
</style>
