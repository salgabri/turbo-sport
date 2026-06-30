<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  type ClubView = {
    team_id: number;
    name: string | null;
    balance: number;
    weekly_income: number;
    wage_bill: number;
    squad_size: number;
  };
  type PlayerView = {
    name: string | null;
    team_id: number | null;
    age: number | null;
    wage: number | null;
    contract_until: string | null;
    fitness: number | null;
    injured: boolean;
    morale: number | null;
    free_agent: boolean;
    retired: boolean;
  };
  type StandingRow = {
    team_id: number;
    played: number;
    won: number;
    drawn: number;
    lost: number;
    goals_for: number;
    goals_against: number;
    goal_difference: number;
    points: number;
  };

  const SAVE_FILTER = [{ name: "Save game", extensions: ["tsfb", "sav"] }];

  let clubs = $state<ClubView[]>([]);
  let selected = $state<number | null>(null);
  let squad = $state<PlayerView[]>([]);
  let market = $state<PlayerView[]>([]);
  let standings = $state<StandingRow[]>([]);
  let tab = $state<"squad" | "market" | "standings">("squad");
  let date = $state("");
  let seasonActive = $state(false);
  let status = $state("");
  let busy = $state(false);

  const money = (n: number): string => {
    const a = Math.abs(n);
    const s = n < 0 ? "-" : "";
    if (a >= 1e6) return `${s}£${(a / 1e6).toFixed(1)}M`;
    if (a >= 1e3) return `${s}£${(a / 1e3).toFixed(0)}k`;
    return `${s}£${a}`;
  };
  const clubName = (id: number): string => clubs.find((c) => c.team_id === id)?.name ?? `Team ${id}`;

  async function refresh() {
    date = await invoke<string>("current_date");
    seasonActive = await invoke<boolean>("season_active");
    clubs = await invoke<ClubView[]>("clubs");
    if (selected !== null) squad = await invoke<PlayerView[]>("team_squad", { teamId: selected });
    if (tab === "market") market = await invoke<PlayerView[]>("market", { limit: 50 });
    if (seasonActive) standings = await invoke<StandingRow[]>("standings");
  }

  async function selectClub(t: number) {
    selected = t;
    tab = "squad";
    squad = await invoke<PlayerView[]>("team_squad", { teamId: t });
  }
  async function showMarket() {
    tab = "market";
    market = await invoke<PlayerView[]>("market", { limit: 50 });
  }
  async function showStandings() {
    tab = "standings";
    standings = await invoke<StandingRow[]>("standings");
  }

  async function withBusy(label: string, fn: () => Promise<void>) {
    busy = true;
    status = label;
    try {
      await fn();
    } catch (e) {
      status = `${e}`;
    } finally {
      busy = false;
    }
  }

  const advance = (days: number) =>
    withBusy(`advancing ${days}d…`, async () => {
      await invoke<string>("advance", { days });
      await refresh();
      status = `now ${date}`;
    });

  const transferWindow = () =>
    withBusy("running transfer window…", async () => {
      const n = await invoke<number>("transfer_window");
      await refresh();
      status = `transfer window: ${n} signings`;
    });

  const startSeason = () =>
    withBusy("starting season…", async () => {
      await invoke("start_season");
      await showStandings();
      await refresh();
      status = "season started";
    });

  const saveGame = () =>
    withBusy("saving…", async () => {
      const path = await saveDialog({ filters: SAVE_FILTER, defaultPath: "game.tsfb" });
      if (typeof path === "string") {
        await invoke("save_game", { path });
        status = `saved ${path}`;
      } else status = "";
    });

  const loadGame = () =>
    withBusy("loading…", async () => {
      const path = await openDialog({ filters: SAVE_FILTER, multiple: false });
      if (typeof path === "string") {
        await invoke<string>("load_game", { path });
        selected = clubs[0]?.team_id ?? null;
        await refresh();
        status = `loaded ${path}`;
      } else status = "";
    });

  onMount(async () => {
    await refresh();
    if (clubs.length) await selectClub(clubs[0].team_id);
  });
</script>

<div class="controls">
  <span class="date">{date || "—"}</span>
  <span class="sep"></span>
  <button disabled={busy} onclick={() => advance(1)}>+1 day</button>
  <button disabled={busy} onclick={() => advance(7)}>+1 week</button>
  <button disabled={busy} onclick={() => advance(30)}>+1 month</button>
  <span class="sep"></span>
  <button disabled={busy} onclick={transferWindow}>Transfer window</button>
  {#if !seasonActive}
    <button disabled={busy} onclick={startSeason}>Start season</button>
  {:else}
    <span class="badge">season running</span>
  {/if}
  <span class="spacer"></span>
  <button disabled={busy} onclick={saveGame}>Save</button>
  <button disabled={busy} onclick={loadGame}>Load</button>
  <span class="status">{status}</span>
</div>

<main>
  <aside>
    <h2>Clubs</h2>
    <ul>
      {#each clubs as c (c.team_id)}
        <li>
          <button class:active={c.team_id === selected} onclick={() => selectClub(c.team_id)}>
            <span class="name">{clubName(c.team_id)}</span>
            <span class="bal" class:neg={c.balance < 0}>{money(c.balance)}</span>
            <span class="sub">{c.squad_size} players · wages {money(c.wage_bill)}/wk</span>
          </button>
        </li>
      {/each}
    </ul>
  </aside>

  <section>
    <nav>
      <button class:on={tab === "squad"} onclick={() => selected !== null && selectClub(selected)}>Squad</button>
      <button class:on={tab === "market"} onclick={showMarket}>Transfer market</button>
      <button class:on={tab === "standings"} onclick={showStandings}>Standings</button>
    </nav>

    {#if tab === "squad"}
      {@const club = clubs.find((c) => c.team_id === selected)}
      <header>
        <h1>{club ? clubName(club.team_id) : "—"}</h1>
        {#if club}
          <div class="meta">
            Balance <b class:neg={club.balance < 0}>{money(club.balance)}</b> · Income
            {money(club.weekly_income)}/wk · Wage bill {money(club.wage_bill)}/wk · Squad
            {club.squad_size}
          </div>
        {/if}
      </header>
      <table>
        <thead>
          <tr><th>#</th><th>Name</th><th>Age</th><th>Wage</th><th>Contract until</th><th>Fitness</th><th>Morale</th><th>Status</th></tr>
        </thead>
        <tbody>
          {#each squad as p, i}
            <tr>
              <td class="dim">{i + 1}</td>
              <td>{p.name ?? "—"}</td>
              <td>{p.age ?? "—"}</td>
              <td>{p.wage != null ? money(p.wage) : "—"}</td>
              <td>{p.contract_until ?? "—"}</td>
              <td class:inj={p.injured}>{p.fitness ?? "—"}{p.injured ? " ⚠" : ""}</td>
              <td>{p.morale ?? "—"}</td>
              <td>{p.retired ? "retired" : p.free_agent ? "free agent" : "contracted"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else if tab === "market"}
      <header><h1>Transfer market</h1><div class="meta">Free agents available to sign</div></header>
      <table>
        <thead><tr><th>#</th><th>Name</th><th>Age</th><th>Fitness</th></tr></thead>
        <tbody>
          {#each market as p, i}
            <tr><td class="dim">{i + 1}</td><td>{p.name ?? "—"}</td><td>{p.age ?? "—"}</td><td>{p.fitness ?? "—"}</td></tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <header><h1>League table</h1></header>
      {#if standings.length === 0}
        <p class="empty">No season running — press “Start season”, then advance time.</p>
      {:else}
        <table>
          <thead>
            <tr><th>#</th><th>Club</th><th>P</th><th>W</th><th>D</th><th>L</th><th>GF</th><th>GA</th><th>GD</th><th>Pts</th></tr>
          </thead>
          <tbody>
            {#each standings as r, i}
              <tr>
                <td class="dim">{i + 1}</td>
                <td>{clubName(r.team_id)}</td>
                <td>{r.played}</td>
                <td>{r.won}</td>
                <td>{r.drawn}</td>
                <td>{r.lost}</td>
                <td>{r.goals_for}</td>
                <td>{r.goals_against}</td>
                <td>{r.goal_difference > 0 ? "+" : ""}{r.goal_difference}</td>
                <td class="pts">{r.points}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    background: #0f1115;
    color: #d7dae0;
    font: 13px/1.4 ui-sans-serif, system-ui, sans-serif;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    border-bottom: 1px solid #232733;
    background: #12141a;
    height: 34px;
  }
  .controls .date {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    min-width: 84px;
  }
  .controls .sep {
    width: 1px;
    height: 18px;
    background: #232733;
    margin: 0 4px;
  }
  .controls .spacer {
    flex: 1;
  }
  .controls .badge {
    color: #7bd88f;
    font-size: 11px;
    border: 1px solid #244;
    border-radius: 10px;
    padding: 2px 8px;
  }
  .controls .status {
    color: #9aa0ab;
    font-size: 12px;
    margin-left: 8px;
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  button {
    background: #1b1f2a;
    border: 1px solid #2a2f3a;
    color: #d7dae0;
    border-radius: 5px;
    padding: 5px 10px;
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    border-color: #4f8cff;
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  main {
    display: grid;
    grid-template-columns: 260px 1fr;
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
  aside li button {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 8px;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    border-left: 2px solid transparent;
    border-radius: 0;
    padding: 8px 14px;
  }
  aside li button:hover {
    background: #1a1d26;
  }
  aside li button.active {
    background: #1a1d26;
    border-left-color: #4f8cff;
  }
  aside .name {
    font-weight: 600;
  }
  aside .bal {
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: #7bd88f;
  }
  aside .sub {
    grid-column: 1 / -1;
    font-size: 11px;
    color: #6b7280;
  }
  .neg {
    color: #ff6b6b !important;
  }
  section {
    overflow-y: auto;
    padding: 0 20px 20px;
  }
  nav {
    position: sticky;
    top: 0;
    background: #0f1115;
    display: flex;
    gap: 4px;
    padding: 12px 0;
    border-bottom: 1px solid #232733;
  }
  nav button {
    color: #9aa0ab;
  }
  nav button.on {
    background: #4f8cff;
    border-color: #4f8cff;
    color: #fff;
  }
  header {
    padding: 16px 0 8px;
  }
  header h1 {
    margin: 0 0 4px;
    font-size: 20px;
  }
  .meta {
    color: #9aa0ab;
    font-size: 12px;
  }
  .meta b {
    color: #7bd88f;
  }
  .empty {
    color: #6b7280;
    padding: 20px 0;
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
    letter-spacing: 0.05em;
    color: #6b7280;
    border-bottom: 1px solid #232733;
    padding: 8px 10px;
    position: sticky;
    top: 49px;
    background: #0f1115;
  }
  tbody td {
    padding: 7px 10px;
    border-bottom: 1px solid #181b22;
  }
  tbody tr:hover {
    background: #14171f;
  }
  td.dim {
    color: #6b7280;
  }
  td.inj {
    color: #ffb454;
  }
  td.pts {
    font-weight: 700;
  }
</style>
