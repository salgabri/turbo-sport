<script lang="ts">
  import { onDestroy } from "svelte";
  import Icon from "../design/Icon.svelte";
  import type { SportTheme } from "../design/theme";
  import type { StagePlayback } from "../design/dto";

  let {
    theme,
    playback,
    onReload,
    loading = false,
  }: {
    theme: SportTheme;
    playback: StagePlayback | null;
    onReload: () => void;
    loading?: boolean;
  } = $props();

  // ---- playback clock ----
  // `frac` runs 0 → 1 across the climb; the UI derives "km to go" counting down from it.
  let frac = $state(0);
  let playing = $state(false);
  let speed = $state(1);
  const speeds = [1, 2, 5];
  let timer: ReturnType<typeof setInterval> | undefined;

  const kmTotal = $derived(playback?.km_total ?? 12);
  const ended = $derived(frac >= 1);

  function tick() {
    if (!playing || !playback) return;
    frac = Math.min(frac + speed * 0.01, 1);
    if (frac >= 1) playing = false;
  }
  timer = setInterval(tick, 100);
  onDestroy(() => clearInterval(timer));

  // Reset the clock whenever a new playback arrives.
  let seenRef = $state<StagePlayback | null>(null);
  $effect(() => {
    if (playback !== seenRef) {
      seenRef = playback;
      frac = 0;
      playing = !!playback;
    }
  });

  function togglePlay() {
    if (ended) {
      frac = 0;
      playing = true;
    } else {
      playing = !playing;
    }
  }

  // ---- derived stage state at the current point on the climb ----
  // km to go counts down 12.0 → 0.0 as the clock runs.
  const kmToGo = $derived(kmTotal * (1 - frac));
  const kmLabel = $derived(ended ? "SUMMIT" : `${kmToGo.toFixed(1)}`);
  // progress = how far up the climb we are (0 at the foot, 1 at the summit).
  const progress = $derived(kmTotal > 0 ? 1 - kmToGo / kmTotal : 0);

  const profilePoints = $derived(
    (playback?.profile ?? []).map(([x, y]) => `${x},${y}`).join(" "),
  );

  // Riders reveal their live gap: it starts at 0 and grows to the final gap by the summit.
  const liveRiders = $derived(
    (playback?.riders ?? []).map((r) => ({
      rank: r.rank,
      name: r.name,
      liveGap: r.gap_secs * progress,
    })),
  );

  // "+M:SS" from gap seconds; the leader shows an em dash.
  function gapStr(rank: number, secs: number): string {
    if (rank === 1) return "—";
    const s = Math.max(0, Math.round(secs));
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `+${m}:${String(sec).padStart(2, "0")}`;
  }

  const LEADER = $derived(theme.accent);
  function rankColor(rank: number): string {
    if (rank === 1) return LEADER;
    if (rank <= 3) return "#5aa9e6";
    return "#3a424c";
  }
</script>

<div class="stage">
  {#if !playback}
    <div class="hero">
      <div class="hero-inner">
        <div class="hero-title">Stage Centre</div>
        <div class="hero-sub">Watch the queen stage climb to its summit finish, live.</div>
        <button class="load" disabled={loading} onclick={onReload}>
          <Icon name="play" size={15} />
          <span>{loading ? "Loading…" : "Simulate the stage"}</span>
        </button>
      </div>
    </div>
  {:else}
    <!-- scoreboard -->
    <div class="scoreboard">
      <div class="stage-name">
        <div class="sn-main">{playback.stage_name}</div>
        <div class="sn-sub mono">Summit finish · {liveRiders.length} riders</div>
      </div>
      <div class="km">
        <div class="km-num mono">{kmLabel}</div>
        <div class="km-cap">
          {#if playing && !ended}<span class="live-dot"></span>{/if}
          <span class="mono">{ended ? "FINISH" : "KM TO GO"}</span>
        </div>
      </div>
      <div class="grad">
        <div class="grad-num mono">{playback.gradient}</div>
        <div class="grad-cap mono">GRADIENT</div>
      </div>
    </div>

    <!-- body: profile + right rail -->
    <div class="body">
      <div class="profile-wrap">
        <section class="panel profile-card">
          <div class="panel-h">Stage Profile</div>
          <div class="profile-svg">
            <svg viewBox="0 0 100 46" preserveAspectRatio="none" style="width:100%;height:100%;display:block">
              <polygon
                points={profilePoints}
                fill="var(--accent-soft)"
                stroke="var(--accent)"
                stroke-width="0.8" />
              <!-- peloton marker riding up the climb -->
              <line
                x1={progress * 100}
                x2={progress * 100}
                y1="0"
                y2="46"
                stroke="var(--accent)"
                stroke-width="0.5"
                stroke-dasharray="1.5 1.5"
                opacity="0.6" />
            </svg>
          </div>
          <div class="profile-foot mono">
            <span>Foot of the climb</span>
            <span>Summit · {kmTotal.toFixed(1)} km</span>
          </div>
        </section>
      </div>

      <div class="rail">
        <section class="panel feed">
          <div class="panel-h">On the Road</div>
          <div class="feed-list">
            {#each liveRiders as r (r.rank)}
              <div class="row" class:leader={r.rank === 1}>
                <span class="pos-wrap">
                  <span class="dot" style="background:{rankColor(r.rank)}"></span>
                  <span class="pos mono">{r.rank}</span>
                </span>
                <span
                  class="rider"
                  style="font-weight:{r.rank === 1 ? 700 : 500};color:{r.rank === 1
                    ? LEADER
                    : 'var(--text)'}">{r.name}</span>
                <span
                  class="gap mono"
                  style="color:{r.rank === 1 ? LEADER : 'var(--text-3)'}">
                  {gapStr(r.rank, r.liveGap)}
                </span>
              </div>
            {/each}
          </div>
        </section>
      </div>
    </div>

    <!-- controls -->
    <div class="controls">
      <button class="play" onclick={togglePlay}>
        <Icon name={playing && !ended ? "pause" : "play"} size={15} />
        <span>{ended ? "Replay" : playing ? "Pause" : "Play"}</span>
      </button>
      <div class="speeds">
        {#each speeds as sp}
          <div class="sp" class:on={speed === sp} role="button" tabindex="0"
            onclick={() => (speed = sp)}
            onkeydown={(ev) => (ev.key === "Enter" || ev.key === " ") && (speed = sp)}>{sp}×</div>
        {/each}
      </div>
      <div class="spacer"></div>
      <button class="ghost" disabled={loading} onclick={onReload}>New stage</button>
    </div>
  {/if}
</div>

<style>
  .stage {
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 13px;
    height: 100%;
    box-sizing: border-box;
  }

  /* hero (no stage loaded) */
  .hero {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .hero-inner {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
  .hero-title {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .hero-sub {
    font-size: 13px;
    color: var(--muted-3);
    font-family: var(--font-mono);
  }
  .load,
  .play,
  .ghost {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: inherit;
    cursor: pointer;
    border-radius: 0;
    font-weight: 700;
    font-size: 13px;
  }
  .load,
  .play {
    padding: 9px 17px;
    background: var(--accent);
    color: #1a1405;
    border: none;
    box-shadow: 0 0 20px var(--accent-soft);
  }
  .load {
    margin-top: 6px;
  }
  .load:hover:not(:disabled),
  .play:hover {
    filter: brightness(1.08);
  }
  .ghost {
    padding: 8px 14px;
    background: transparent;
    color: var(--text-3);
    border: 1px solid var(--line-3);
    font-weight: 600;
  }
  .ghost:hover:not(:disabled) {
    border-color: var(--accent-line);
    color: var(--accent);
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  /* scoreboard */
  .scoreboard {
    flex: none;
    display: flex;
    align-items: center;
    gap: 24px;
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: 0;
    padding: 13px 22px;
  }
  .stage-name {
    flex: 1;
    min-width: 0;
  }
  .sn-main {
    font-size: 15px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sn-sub {
    font-size: 11.5px;
    color: var(--muted-3);
    margin-top: 2px;
  }
  .km {
    text-align: center;
    flex: none;
  }
  .km-num {
    font-size: 32px;
    font-weight: 700;
    line-height: 1;
    color: var(--accent);
  }
  .km-cap {
    display: flex;
    align-items: center;
    gap: 6px;
    justify-content: center;
    margin-top: 6px;
    font-size: 11px;
    color: var(--muted-3);
    font-weight: 600;
    letter-spacing: 0.04em;
  }
  .live-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #ef5a5a;
    animation: tsPulse 1.4s infinite;
  }
  .grad {
    text-align: center;
    flex: none;
  }
  .grad-num {
    font-size: 20px;
    font-weight: 700;
    line-height: 1;
    color: var(--text);
  }
  .grad-cap {
    margin-top: 7px;
    font-size: 11px;
    color: var(--muted-3);
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  /* body */
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 14px;
  }
  .profile-wrap {
    flex: 1.5;
    min-width: 0;
    display: flex;
  }
  .profile-card {
    flex: 1;
  }
  .profile-svg {
    flex: 1;
    min-height: 0;
    padding: 14px 16px;
  }
  .profile-foot {
    display: flex;
    justify-content: space-between;
    padding: 0 16px 14px;
    font-size: 10.5px;
    color: var(--muted-3);
    letter-spacing: 0.03em;
  }

  /* right rail */
  .rail {
    width: 296px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
  }
  .panel {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .panel-h {
    padding: 11px 15px;
    border-bottom: 1px solid var(--line);
    font-size: 13px;
    font-weight: 700;
    flex: none;
  }
  .feed {
    flex: 1;
    min-height: 0;
  }
  .feed-list {
    flex: 1;
    overflow: auto;
    padding: 4px 0;
  }
  .row {
    display: flex;
    align-items: center;
    padding: 9px 15px;
    border-bottom: 1px solid var(--line);
  }
  .row:hover {
    background: var(--hover, #161b22);
  }
  .row.leader {
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .pos-wrap {
    width: 40px;
    display: flex;
    align-items: center;
    gap: 9px;
    flex: none;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
  }
  .pos {
    font-size: 12px;
    color: var(--muted-2);
  }
  .rider {
    flex: 1;
    min-width: 0;
    font-size: 12.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .gap {
    width: 66px;
    flex: none;
    text-align: right;
    font-size: 12px;
    font-weight: 600;
  }

  /* controls */
  .controls {
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: 0;
    padding: 9px 14px;
  }
  .speeds {
    display: flex;
    align-items: center;
    gap: 3px;
    background: #0f1319;
    border: 1px solid var(--line);
    border-radius: 0;
    padding: 3px;
  }
  .sp {
    padding: 5px 11px;
    border-radius: 0;
    font-size: 12px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--muted-2);
    cursor: pointer;
    user-select: none;
  }
  .sp.on {
    background: var(--accent);
    color: #1a1405;
  }
  .spacer {
    flex: 1;
  }
</style>
