<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  import "$lib/design/tokens.css";
  import AppShell from "$lib/design/AppShell.svelte";
  import { TENNIS } from "$lib/design/theme";
  import type { PlayerRow, Tourney, Screen } from "$lib/design/dto";

  import Home from "$lib/screens/Home.svelte";
  import Draw from "$lib/screens/Draw.svelte";
  import Bracket from "$lib/screens/Bracket.svelte";

  const theme = TENNIS;

  let draw = $state<PlayerRow[]>([]);
  let result = $state<Tourney | null>(null);
  let busy = $state(false);
  let toast = $state<string | null>(null);

  let screen = $state<Screen>("home");

  const SCREEN_META: Record<Screen, { title: string; sub: string }> = {
    home: { title: "Home", sub: "Overview" },
    draw: { title: "The Draw", sub: "Seeded field" },
    bracket: { title: "Bracket", sub: "Single elimination" },
  };

  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  function flash(msg: string) {
    toast = msg;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 2200);
  }

  async function loadDraw() {
    try {
      draw = await invoke<PlayerRow[]>("draw");
    } catch (e) {
      flash(`${e}`);
    }
  }

  async function play() {
    busy = true;
    try {
      result = await invoke<Tourney>("run_tournament");
      flash(`Champion — ${result.champion}`);
    } catch (e) {
      flash(`${e}`);
    } finally {
      busy = false;
    }
  }

  function onNav(s: Screen) {
    screen = s;
  }

  const meta = $derived(SCREEN_META[screen]);
  const topSeedName = $derived([...draw].sort((a, b) => a.seed - b.seed)[0]?.name ?? "—");

  onMount(loadDraw);
</script>

<AppShell
  {theme}
  gameName="TURBO TENNIS"
  clubName={topSeedName}
  clubTag="Academy"
  clubCrest="TEN"
  leagueName="ATP Tour"
  managerName="You"
  {screen}
  title={meta.title}
  sub={meta.sub}
  {onNav}
  {toast}>
  {#snippet actions()}
    {#if screen === "bracket"}
      <button disabled={busy} onclick={play}>{busy ? "Playing…" : "Play Tournament"}</button>
    {/if}
  {/snippet}

  {#if screen === "home"}
    <Home {theme} {draw} {result} {onNav} />
  {:else if screen === "draw"}
    <Draw {theme} {draw} />
  {:else if screen === "bracket"}
    <Bracket {theme} {result} {busy} onPlay={play} />
  {/if}
</AppShell>
