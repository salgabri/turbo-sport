<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  import "$lib/design/tokens.css";
  import AppShell from "$lib/design/AppShell.svelte";
  import { CYCLING } from "$lib/design/theme";
  import type { RiderRow, GcRow, Screen, StagePlayback } from "$lib/design/dto";

  import Home from "$lib/screens/Home.svelte";
  import Roster from "$lib/screens/Roster.svelte";
  import Race from "$lib/screens/Race.svelte";
  import Stage from "$lib/screens/Stage.svelte";

  const theme = CYCLING;

  let roster = $state<RiderRow[]>([]);
  let gc = $state<GcRow[]>([]);
  let busy = $state(false);
  let toast = $state<string | null>(null);

  let screen = $state<Screen>("home");
  let stagePlayback = $state<StagePlayback | null>(null);
  let stageLoading = $state(false);

  const hasRaced = $derived(gc.length > 0);

  const SCREEN_META: Record<Screen, { title: string; sub: string }> = {
    home: { title: "Home", sub: "Overview" },
    roster: { title: "Roster", sub: "Team riders" },
    race: { title: "Race", sub: "General classification" },
    stage: { title: "Stage", sub: "Live climb" },
  };

  let toastTimer: ReturnType<typeof setTimeout> | undefined;
  function flash(msg: string) {
    toast = msg;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), 2200);
  }

  async function loadRoster() {
    try {
      roster = await invoke<RiderRow[]>("roster");
    } catch (e) {
      flash(`${e}`);
    }
  }

  async function runTour() {
    busy = true;
    try {
      gc = await invoke<GcRow[]>("run_tour");
      flash(`Grand tour complete — winner ${gc[0]?.name ?? "—"}`);
    } catch (e) {
      flash(`${e}`);
    } finally {
      busy = false;
    }
  }

  // Topbar "Run Tour": race, then jump to the classification.
  async function runAndShow() {
    await runTour();
    screen = "race";
  }

  async function loadStage() {
    stageLoading = true;
    try {
      stagePlayback = await invoke<StagePlayback>("next_stage");
    } catch (e) {
      flash(`${e}`);
    } finally {
      stageLoading = false;
    }
  }

  function onNav(s: Screen) {
    screen = s;
    if (s === "stage" && !stagePlayback && !stageLoading) loadStage();
  }

  const meta = $derived(SCREEN_META[screen]);

  onMount(loadRoster);
</script>

<AppShell
  {theme}
  gameName="TURBO CYCLING"
  clubName="Turbo Cycling"
  clubTag="Pro Team"
  clubCrest="TC"
  leagueName="World Tour"
  managerName="You"
  dateStr={`${roster.length} riders`}
  notif={0}
  {screen}
  title={meta.title}
  sub={meta.sub}
  {busy}
  {onNav}
  onAdvance={runAndShow}
  {toast}>
  {#if screen === "home"}
    <Home {theme} {roster} {hasRaced} {onNav} />
  {:else if screen === "roster"}
    <Roster {theme} {roster} count={roster.length} />
  {:else if screen === "race"}
    <Race {theme} {gc} {busy} onRun={runTour} />
  {:else if screen === "stage"}
    <Stage {theme} playback={stagePlayback} onReload={loadStage} loading={stageLoading} />
  {/if}
</AppShell>
