<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
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

  const FILTER = [{ name: "Database", extensions: ["json"] }];

  let db = $state<Database | null>(null);
  let selected = $state<number | "free" | null>(null);
  let currentPath = $state<string | null>(null);
  let status = $state("");

  const club = $derived(
    db && typeof selected === "number" ? db.clubs.find((c) => c.id === selected) : undefined,
  );
  const players = $derived.by((): PlayerRecord[] => {
    if (!db) return [];
    if (selected === "free") return db.players.filter((p) => p.club_id === null);
    return db.players.filter((p) => p.club_id === selected);
  });
  const clubsIn = (d: DivisionRecord): ClubRecord[] =>
    d.club_ids.map((id) => db!.clubs.find((c) => c.id === id)).filter((c): c is ClubRecord => !!c);

  // ---- file actions -------------------------------------------------------
  async function loadSample() {
    db = await invoke<Database>("load_sample");
    selected = db.clubs[0]?.id ?? null;
    currentPath = null;
    status = "loaded sample";
  }
  async function openFile() {
    const path = await openDialog({ filters: FILTER, multiple: false });
    if (typeof path !== "string") return;
    try {
      db = await invoke<Database>("open", { path });
      selected = db.clubs[0]?.id ?? null;
      currentPath = path;
      status = `opened ${path}`;
    } catch (e) {
      status = `open error: ${e}`;
    }
  }
  async function saveTo(path: string) {
    try {
      await invoke("save", { path, db });
      currentPath = path;
      status = `saved to ${path}`;
    } catch (e) {
      status = `save error: ${e}`;
    }
  }
  async function saveAs() {
    const path = await saveDialog({ filters: FILTER, defaultPath: "database.json" });
    if (typeof path === "string") await saveTo(path);
  }
  async function save() {
    if (currentPath) await saveTo(currentPath);
    else await saveAs();
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

  // ---- club / division edits ---------------------------------------------
  function addClub(divIdx: number) {
    if (!db) return;
    const id = db.clubs.reduce((m, c) => Math.max(m, c.id), -1) + 1;
    db.clubs.push({ id, name: `New Club ${id}`, balance: 10_000_000, weekly_income: 200_000, squad_target: 16 });
    db.divisions[divIdx].club_ids.push(id);
    selected = id;
  }
  function removeClub(id: number) {
    if (!db) return;
    db.clubs = db.clubs.filter((c) => c.id !== id);
    for (const d of db.divisions) d.club_ids = d.club_ids.filter((x) => x !== id);
    for (const p of db.players) if (p.club_id === id) p.club_id = null;
    if (selected === id) selected = db.clubs[0]?.id ?? "free";
  }
  function moveClub(id: number, toDiv: number) {
    if (!db) return;
    for (const d of db.divisions) d.club_ids = d.club_ids.filter((x) => x !== id);
    db.divisions[toDiv].club_ids.push(id);
  }

  // ---- player edits -------------------------------------------------------
  function addPlayer() {
    if (!db) return;
    db.players.push({
      name: "New Player",
      club_id: selected === "free" ? null : (selected as number),
      birth: { year: 2005, month: 1, day: 1 },
      attacking: 50,
      defending: 50,
      finishing: 50,
      goalkeeping: 50,
      wage: 1000,
      contract_until: { year: 2028, month: 6, day: 30 },
    });
  }
  function removePlayer(p: PlayerRecord) {
    if (!db) return;
    const i = db.players.indexOf(p);
    if (i >= 0) db.players.splice(i, 1);
  }

  function divIndexOf(id: number): number {
    return db ? db.divisions.findIndex((d) => d.club_ids.includes(id)) : -1;
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
  <button onclick={openFile}>Open…</button>
  <button onclick={loadSample}>Sample</button>
  <button class="primary" onclick={save}>Save</button>
  <button onclick={saveAs}>Save As…</button>
  <button onclick={check}>Validate</button>
  <span class="status">{status}</span>
</header>

{#if db}
  <main>
    <aside>
      {#each db.divisions as div, di (di)}
        <div class="divhdr">
          <input class="divname" bind:value={div.name} />
          <input class="divtier" type="number" min="1" bind:value={div.tier} title="tier" />
          <button class="mini" title="add club" onclick={() => addClub(di)}>+</button>
        </div>
        {#each clubsIn(div) as c (c.id)}
          <div class="clubrow">
            <button class="clubsel" class:active={selected === c.id} onclick={() => (selected = c.id)}>
              {c.name}
            </button>
            <select
              class="movesel"
              value={divIndexOf(c.id)}
              title="move to division"
              onchange={(e) => moveClub(c.id, +e.currentTarget.value)}
            >
              {#each db.divisions as d, j (j)}
                <option value={j}>{d.name}</option>
              {/each}
            </select>
          </div>
        {/each}
      {/each}
      <div class="divhdr"><span class="divname pool">Pool</span></div>
      <button class="clubsel" class:active={selected === "free"} onclick={() => (selected = "free")}>
        Free agents
      </button>
    </aside>

    <section>
      {#if club}
        <div class="clubfields">
          <label>Name <input bind:value={club.name} /></label>
          <label>Balance <input type="number" bind:value={club.balance} /></label>
          <label>Income/wk <input type="number" bind:value={club.weekly_income} /></label>
          <label>Squad target <input type="number" bind:value={club.squad_target} /></label>
          <button class="danger" onclick={() => removeClub(club.id)}>Remove club</button>
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
  .dbname {
    font-weight: 600;
    width: 200px;
  }
  .status {
    color: #9aa0ab;
    font-size: 12px;
    margin-left: 8px;
    max-width: 360px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  input,
  select {
    background: #0d0f14;
    border: 1px solid #2a2f3a;
    color: #d7dae0;
    border-radius: 4px;
    padding: 4px 6px;
    font: inherit;
  }
  input:focus,
  select:focus {
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
  button.danger {
    color: #ff6b6b;
    border-color: #3a2730;
  }
  main {
    display: grid;
    grid-template-columns: 250px 1fr;
    height: calc(100vh - 49px);
  }
  aside {
    border-right: 1px solid #232733;
    overflow-y: auto;
    background: #12141a;
    padding-bottom: 20px;
  }
  .divhdr {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 10px 10px 4px;
  }
  .divname {
    flex: 1;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #9aa0ab;
    font-weight: 600;
  }
  .divname.pool {
    padding: 4px 6px;
  }
  .divtier {
    width: 38px;
  }
  .mini {
    padding: 2px 8px;
  }
  .clubrow {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px;
  }
  .clubsel {
    flex: 1;
    text-align: left;
    background: none;
    border: none;
    border-left: 2px solid transparent;
    border-radius: 0;
    padding: 6px 6px;
  }
  .clubsel:hover {
    background: #1a1d26;
  }
  .clubsel.active {
    background: #1a1d26;
    border-left-color: #4f8cff;
  }
  .movesel {
    width: 26px;
    padding: 3px 2px;
    font-size: 11px;
  }
  section {
    overflow-y: auto;
    padding: 14px 18px;
  }
  .clubfields {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
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
