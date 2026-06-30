<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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

  let clubs = $state<ClubView[]>([]);
  let selected = $state<number | null>(null);
  let squad = $state<PlayerView[]>([]);
  let market = $state<PlayerView[]>([]);
  let tab = $state<"squad" | "market">("squad");

  const money = (n: number): string => {
    const a = Math.abs(n);
    const s = n < 0 ? "-" : "";
    if (a >= 1e6) return `${s}£${(a / 1e6).toFixed(1)}M`;
    if (a >= 1e3) return `${s}£${(a / 1e3).toFixed(0)}k`;
    return `${s}£${a}`;
  };

  async function loadClubs() {
    clubs = await invoke<ClubView[]>("clubs");
    if (clubs.length && selected === null) await selectClub(clubs[0].team_id);
  }
  async function selectClub(t: number) {
    selected = t;
    tab = "squad";
    squad = await invoke<PlayerView[]>("team_squad", { teamId: t });
  }
  async function loadMarket() {
    tab = "market";
    market = await invoke<PlayerView[]>("market", { limit: 50 });
  }

  onMount(loadClubs);
</script>

<main>
  <aside>
    <h2>Clubs</h2>
    <ul>
      {#each clubs as c (c.team_id)}
        <li>
          <button class:active={c.team_id === selected} onclick={() => selectClub(c.team_id)}>
            <span class="name">{c.name ?? `Team ${c.team_id}`}</span>
            <span class="bal" class:neg={c.balance < 0}>{money(c.balance)}</span>
            <span class="sub">{c.squad_size} players · wages {money(c.wage_bill)}/wk</span>
          </button>
        </li>
      {/each}
    </ul>
  </aside>

  <section>
    <nav>
      <button class:on={tab === "squad"} onclick={() => selected !== null && selectClub(selected)}>
        Squad
      </button>
      <button class:on={tab === "market"} onclick={loadMarket}>Transfer market</button>
    </nav>

    {#if tab === "squad"}
      {@const club = clubs.find((c) => c.team_id === selected)}
      <header>
        <h1>{club?.name ?? `Team ${selected}`}</h1>
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
    {:else}
      <header><h1>Transfer market</h1><div class="meta">Free agents available to sign</div></header>
      <table>
        <thead><tr><th>#</th><th>Name</th><th>Age</th><th>Fitness</th><th>Status</th></tr></thead>
        <tbody>
          {#each market as p, i}
            <tr>
              <td class="dim">{i + 1}</td>
              <td>{p.name ?? "—"}</td>
              <td>{p.age ?? "—"}</td>
              <td>{p.fitness ?? "—"}</td>
              <td>free agent</td>
            </tr>
          {/each}
        </tbody>
      </table>
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
  main {
    display: grid;
    grid-template-columns: 260px 1fr;
    height: 100vh;
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
  aside button {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 8px;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: inherit;
    padding: 8px 14px;
    cursor: pointer;
    border-left: 2px solid transparent;
  }
  aside button:hover {
    background: #1a1d26;
  }
  aside button.active {
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
    background: none;
    border: 1px solid #232733;
    color: #9aa0ab;
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
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
</style>
