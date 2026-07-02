<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  import "$lib/design/tokens.css";
  import AppShell from "$lib/design/AppShell.svelte";
  import { BASKETBALL } from "$lib/design/theme";
  import type { ClubView, PlayerView, StandingRow, Screen, MatchPlayback, ScorerRow } from "$lib/design/dto";

  import Home from "$lib/screens/Home.svelte";
  import Squad from "$lib/screens/Squad.svelte";
  import Profile from "$lib/screens/Profile.svelte";
  import Table from "$lib/screens/Table.svelte";
  import Transfers from "$lib/screens/Transfers.svelte";
  import Match from "$lib/screens/Match.svelte";

  const theme = BASKETBALL;
  const SAVE_FILTER = [{ name: "Save game", extensions: ["tsbb", "sav"] }];

  let clubs = $state<ClubView[]>([]);
  let myClubId = $state<number | null>(null);
  let squad = $state<PlayerView[]>([]);
  let market = $state<PlayerView[]>([]);
  let standings = $state<StandingRow[]>([]);
  let scorers = $state<ScorerRow[]>([]);
  let date = $state("");
  let seasonActive = $state(false);
  let busy = $state(false);
  let toast = $state<string | null>(null);

  let screen = $state<Screen>("home");
  let selectedPlayer = $state<PlayerView | null>(null);
  let matchPlayback = $state<MatchPlayback | null>(null);
  let matchLoading = $state(false);

  const myClub = $derived(clubs.find((c) => c.team_id === myClubId) ?? null);
  const teamName = (id: number): string => clubs.find((c) => c.team_id === id)?.name ?? `Team ${id}`;

  const SCREEN_META: Record<Screen, { title: string; sub: string }> = {
    home: { title: "Home", sub: "Overview" },
    squad: { title: "Roster", sub: "First team" },
    profile: { title: "Player Profile", sub: "Attributes & development" },
    table: { title: "Conference", sub: "Standings" },
    transfers: { title: "Transfers", sub: "Market & shortlist" },
    match: { title: "Match", sub: "Live centre" },
  };

  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  function flash(msg: string) {
    toast = msg;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 2200);
  }

  async function refresh() {
    date = await invoke<string>("current_date");
    seasonActive = await invoke<boolean>("season_active");
    clubs = await invoke<ClubView[]>("clubs");
    if (myClubId === null) myClubId = clubs[0]?.team_id ?? null;
    if (myClubId !== null) squad = await invoke<PlayerView[]>("team_squad", { teamId: myClubId });
    market = await invoke<PlayerView[]>("market", { limit: 50 });
    if (seasonActive) {
      standings = await invoke<StandingRow[]>("standings");
      scorers = await invoke<ScorerRow[]>("top_scorers", { limit: 10 });
    } else {
      standings = [];
      scorers = [];
    }
  }

  async function withBusy(label: string, fn: () => Promise<void>) {
    busy = true;
    try {
      await fn();
    } catch (e) {
      flash(`${e}`);
    } finally {
      busy = false;
    }
  }

  const advance = () =>
    withBusy("advancing…", async () => {
      await invoke<string>("advance", { days: 1 });
      await refresh();
      flash(`Time advanced — ${date}`);
    });

  const transferWindow = () =>
    withBusy("transfer window…", async () => {
      const n = await invoke<number>("transfer_window");
      await refresh();
      flash(`Transfer window: ${n} signings`);
    });

  const startSeason = () =>
    withBusy("starting season…", async () => {
      await invoke("start_season");
      await refresh();
      flash("Season started");
    });

  const saveGame = () =>
    withBusy("saving…", async () => {
      const path = await saveDialog({ filters: SAVE_FILTER, defaultPath: "game.tsbb" });
      if (typeof path === "string") {
        await invoke("save_game", { path });
        flash("Game saved");
      }
    });

  const loadGame = () =>
    withBusy("loading…", async () => {
      const path = await openDialog({ filters: SAVE_FILTER, multiple: false });
      if (typeof path === "string") {
        await invoke<string>("load_game", { path });
        myClubId = null;
        await refresh();
        flash("Game loaded");
      }
    });

  async function loadMatch() {
    if (myClubId === null) return;
    matchLoading = true;
    try {
      matchPlayback = await invoke<MatchPlayback | null>("next_match", { teamId: myClubId });
    } catch (e) {
      flash(`${e}`);
    } finally {
      matchLoading = false;
    }
  }

  function onNav(s: Screen) {
    screen = s;
    if (s === "match" && !matchPlayback && !matchLoading) loadMatch();
  }
  function onSelectPlayer(p: PlayerView) {
    selectedPlayer = p;
    screen = "profile";
  }

  const meta = $derived(SCREEN_META[screen]);
  const clubName = $derived(myClub?.name ?? "—");

  onMount(refresh);
</script>

<AppShell
  {theme}
  gameName="TURBO BASKETBALL"
  clubName={clubName}
  clubTag="Roster"
  clubCrest={(clubName || "?").slice(0, 2).toUpperCase()}
  leagueName="Conference"
  managerName="You"
  dateStr={date || "—"}
  notif={0}
  {screen}
  title={meta.title}
  sub={meta.sub}
  {busy}
  {onNav}
  onAdvance={advance}
  {toast}>
  {#snippet actions()}
    {#if !seasonActive}
      <button disabled={busy} onclick={startSeason}>Start season</button>
    {/if}
    <button disabled={busy} onclick={transferWindow}>Transfer window</button>
    <button disabled={busy} onclick={saveGame}>Save</button>
    <button disabled={busy} onclick={loadGame}>Load</button>
  {/snippet}

  {#if screen === "home"}
    <Home {theme} club={myClub} {squad} {standings} myTeamId={myClubId} {teamName} {seasonActive} {onNav} />
  {:else if screen === "squad"}
    <Squad {theme} {squad} count={squad.length} selectedName={selectedPlayer?.name ?? "—"} {onSelectPlayer} />
  {:else if screen === "profile"}
    <Profile {theme} player={selectedPlayer} />
  {:else if screen === "table"}
    <Table {theme} {standings} {teamName} myTeamId={myClubId} {scorers} />
  {:else if screen === "transfers"}
    <Transfers {theme} {market} club={myClub} />
  {:else if screen === "match"}
    <Match {theme} playback={matchPlayback} onReload={loadMatch} loading={matchLoading} />
  {/if}
</AppShell>
