<script lang="ts">
  import { onDestroy } from "svelte";
  import Icon from "../design/Icon.svelte";
  import type { SportTheme } from "../design/theme";
  import type { MatchPlayback, PlayEvent } from "../design/dto";

  let {
    theme,
    playback,
    onReload,
    loading = false,
  }: {
    theme: SportTheme;
    playback: MatchPlayback | null;
    onReload: () => void;
    loading?: boolean;
  } = $props();

  // ---- playback clock ----
  let clock = $state(0); // current match minute (float)
  let playing = $state(false);
  let speed = $state(1);
  const speeds = [1, 2, 5];
  let timer: ReturnType<typeof setInterval> | undefined;

  const minutes = $derived(playback?.minutes ?? 90);
  const ended = $derived(clock >= minutes);
  const isCourt = $derived(theme.matchVariant === "court");

  function tick() {
    if (!playing || !playback) return;
    clock = Math.min(clock + speed * 0.15, minutes);
    if (clock >= minutes) playing = false;
  }
  timer = setInterval(tick, 100);
  onDestroy(() => clearInterval(timer));

  // Reset the clock whenever a new playback arrives.
  let seenRef = $state<MatchPlayback | null>(null);
  $effect(() => {
    if (playback !== seenRef) {
      seenRef = playback;
      clock = 0;
      playing = !!playback;
    }
  });

  function togglePlay() {
    if (ended) {
      clock = 0;
      playing = true;
    } else {
      playing = !playing;
    }
  }

  // ---- derived match state at the current minute ----
  const shown = $derived(Math.floor(clock));
  const clockLabel = $derived(ended ? "FT" : `${shown}'`);
  const passed = $derived<PlayEvent[]>((playback?.events ?? []).filter((e) => e.minute <= clock));
  const sideScore = (s: number) =>
    passed.filter((e) => e.side === s).reduce((n, e) => n + (e.points ?? 0), 0);
  const homeScore = $derived(sideScore(0));
  const awayScore = $derived(sideScore(1));
  const feed = $derived([...passed].reverse());

  // Stats interpolate from 0 → final across the match.
  const frac = $derived(minutes > 0 ? Math.min(clock / minutes, 1) : 0);
  const statNow = $derived(
    (playback?.stats ?? []).map((s) => {
      if (s.label === "Possession") {
        return { label: s.label, h: Math.round(s.home), a: Math.round(s.away), rawH: s.home, rawA: s.away, poss: true };
      }
      const round1 = s.label === "Expected Goals";
      const rawH = s.home * frac;
      const rawA = s.away * frac;
      const f = (v: number) => (round1 ? v.toFixed(1) : Math.round(v).toString());
      return { label: s.label, h: f(rawH), a: f(rawA), rawH, rawA, poss: false };
    }),
  );

  // Ball: lively procedural drift across the pitch (percent coords).
  const ball = $derived({
    x: 50 + 34 * Math.sin(clock * 0.5),
    y: 50 + 16 * Math.sin(clock * 0.9 + 1),
  });

  function dotStyle(x: number, y: number, home: boolean): string {
    const bg = home ? "var(--accent)" : "#c2cad3";
    const fg = home ? "#08120c" : "#0a0c0f";
    return `position:absolute;left:${x}%;top:${y}%;transform:translate(-50%,-50%);width:19px;height:19px;border-radius:50%;background:${bg};color:${fg};display:flex;align-items:center;justify-content:center;font-family:var(--font-mono);font-size:10px;font-weight:700;box-shadow:0 1px 4px rgba(0,0,0,.4)`;
  }
  function barPct(v: number, total: number): number {
    return total <= 0 ? 0 : Math.min((v / total) * 100, 100);
  }
</script>

<div class="match">
  {#if !playback}
    <div class="hero">
      <div class="hero-inner">
        <div class="hero-title">Match Centre</div>
        <div class="hero-sub">Watch your next fixture play out live in 2D.</div>
        <button class="load" disabled={loading} onclick={onReload}>
          <Icon name="play" size={15} />
          <span>{loading ? "Loading…" : "Load next match"}</span>
        </button>
      </div>
    </div>
  {:else}
    <!-- scoreboard -->
    <div class="scoreboard">
      <div class="team home">
        <span class="tname">{playback.home.name}</span>
        <div class="crest home-crest">{playback.home.crest}</div>
      </div>
      <div class="score">
        <div class="score-nums">
          <span class="mono">{homeScore}</span><span class="dash">–</span><span class="mono">{awayScore}</span>
        </div>
        <div class="clock">
          {#if playing && !ended}<span class="live-dot"></span>{/if}
          <span class="mono">{clockLabel}</span>
        </div>
      </div>
      <div class="team away">
        <div class="crest away-crest">{playback.away.crest}</div>
        <span class="tname">{playback.away.name}</span>
      </div>
    </div>

    <!-- body: pitch + right rail -->
    <div class="body">
      <div class="pitch-wrap">
        <div class="field" class:court={isCourt}>
          <div class="f-box"></div>
          <div class="f-mid"></div>
          <div class="f-circle"></div>
          {#if isCourt}
            <div class="key left"></div>
            <div class="key right"></div>
            <div class="hoop left"></div>
            <div class="hoop right"></div>
          {:else}
            <div class="f-goal left"></div>
            <div class="f-goal right"></div>
          {/if}
          {#each playback.home.dots as d}
            <div style={dotStyle(d.x, d.y, true)}>{d.n}</div>
          {/each}
          {#each playback.away.dots as d}
            <div style={dotStyle(d.x, d.y, false)}>{d.n}</div>
          {/each}
          <div class="ball" style="left:{ball.x}%;top:{ball.y}%"></div>
        </div>
      </div>

      <div class="rail">
        <section class="panel stats">
          <div class="panel-h">Match Stats</div>
          <div class="stat-list">
            {#each statNow as s}
              <div class="stat">
                <div class="stat-top">
                  <span class="mono">{s.h}</span>
                  <span class="stat-label">{s.label}</span>
                  <span class="mono">{s.a}</span>
                </div>
                {#if s.poss}
                  <div class="stat-bars">
                    <div class="bar l"><div class="fill" style="width:{s.rawH}%;background:var(--accent)"></div></div>
                    <div class="bar r"><div class="fill" style="width:{s.rawA}%;background:#39424e"></div></div>
                  </div>
                {:else}
                  {@const tot = Math.max(s.rawH, s.rawA, 0.001)}
                  <div class="stat-bars">
                    <div class="bar l"><div class="fill" style="width:{barPct(s.rawH, tot)}%;background:var(--accent)"></div></div>
                    <div class="bar r"><div class="fill" style="width:{barPct(s.rawA, tot)}%;background:#39424e"></div></div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </section>

        <section class="panel feed">
          <div class="panel-h">Match Feed</div>
          <div class="feed-list">
            {#if feed.length === 0}
              <div class="feed-empty">Kick-off. Events will appear here.</div>
            {:else}
              {#each feed as e (e.minute + e.title)}
                <div class="fev">
                  <div class="fmin mono">{e.minute}'</div>
                  <div class="fbar" style="background:{e.kind === 'goal' ? 'var(--accent)' : '#edb95e'}"></div>
                  <div class="fbody">
                    <div class="ftitle">{e.title}</div>
                    <div class="fsub">{e.sub}</div>
                  </div>
                </div>
              {/each}
            {/if}
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
      <button class="ghost" disabled={loading} onclick={onReload}>New match</button>
    </div>
  {/if}
</div>

<style>
  .match {
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 13px;
    height: 100%;
    box-sizing: border-box;
  }

  /* hero (no match loaded) */
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
    font-weight: 800;
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
    border-radius: 9px;
    font-weight: 700;
    font-size: 13px;
  }
  .load,
  .play {
    padding: 9px 17px;
    background: var(--accent);
    color: #08120c;
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
    border-radius: 14px;
    padding: 13px 22px;
  }
  .team {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 13px;
  }
  .team.home {
    justify-content: flex-end;
  }
  .tname {
    font-size: 15px;
    font-weight: 700;
  }
  .crest {
    width: 36px;
    height: 36px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 13px;
  }
  .home-crest {
    background: var(--accent-soft);
    border: 1px solid var(--accent-line);
    color: var(--accent);
  }
  .away-crest {
    background: var(--raised);
    border: 1px solid #2a323c;
    color: var(--text-3);
  }
  .score {
    text-align: center;
    flex: none;
  }
  .score-nums {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 32px;
    font-weight: 800;
    line-height: 1;
  }
  .dash {
    font-size: 18px;
    color: #4a535e;
  }
  .clock {
    display: flex;
    align-items: center;
    gap: 6px;
    justify-content: center;
    margin-top: 6px;
    font-size: 12px;
    color: var(--accent);
    font-weight: 600;
  }
  .live-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #ef5a5a;
    animation: tsPulse 1.4s infinite;
  }

  /* body */
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 14px;
  }
  .pitch-wrap {
    flex: 1.5;
    min-width: 0;
    display: flex;
  }
  .field {
    flex: 1;
    position: relative;
    border: 1px solid #234a30;
    border-radius: 12px;
    overflow: hidden;
    background: repeating-linear-gradient(
      90deg,
      #163a22 0,
      #163a22 9.09%,
      #14351f 9.09%,
      #14351f 18.18%
    );
  }
  .field.court {
    border-color: #3a2f22;
    background: #241c14;
  }
  .f-box {
    position: absolute;
    inset: 4%;
    border: 2px solid rgba(255, 255, 255, 0.16);
    border-radius: 3px;
  }
  .court .f-box {
    border-color: rgba(242, 145, 61, 0.28);
  }
  .f-mid {
    position: absolute;
    left: 50%;
    top: 4%;
    bottom: 4%;
    width: 2px;
    background: rgba(255, 255, 255, 0.16);
    transform: translateX(-50%);
  }
  .court .f-mid {
    background: rgba(242, 145, 61, 0.28);
  }
  .f-circle {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 20%;
    padding-bottom: 20%;
    border: 2px solid rgba(255, 255, 255, 0.16);
    border-radius: 50%;
    transform: translate(-50%, -50%);
  }
  .court .f-circle {
    width: 16%;
    padding-bottom: 16%;
    border-color: rgba(242, 145, 61, 0.28);
  }
  .f-goal {
    position: absolute;
    top: 28%;
    bottom: 28%;
    width: 13%;
    border: 2px solid rgba(255, 255, 255, 0.14);
  }
  .f-goal.left {
    left: 4%;
    border-left: none;
  }
  .f-goal.right {
    right: 4%;
    border-right: none;
  }
  /* basketball court key + hoops */
  .key {
    position: absolute;
    top: 33%;
    bottom: 33%;
    width: 15%;
    border: 2px solid rgba(242, 145, 61, 0.22);
  }
  .key.left {
    left: 4%;
    border-left: none;
  }
  .key.right {
    right: 4%;
    border-right: none;
  }
  .hoop {
    position: absolute;
    top: 50%;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(242, 145, 61, 0.5);
    border-radius: 50%;
  }
  .hoop.left {
    left: 5.5%;
    transform: translate(-50%, -50%);
  }
  .hoop.right {
    right: 5.5%;
    transform: translate(50%, -50%);
  }
  .ball {
    position: absolute;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 0 8px rgba(255, 255, 255, 0.7);
    transform: translate(-50%, -50%);
    transition:
      left 0.28s linear,
      top 0.28s linear;
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
    border-radius: 12px;
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
  .stats {
    flex: none;
  }
  .stat-list {
    padding: 13px 15px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .stat-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
    font-size: 12.5px;
    font-weight: 700;
  }
  .stat-label {
    font-size: 10.5px;
    color: var(--muted-3);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 500;
  }
  .stat-bars {
    display: flex;
    gap: 4px;
    height: 5px;
  }
  .bar {
    flex: 1;
    background: #20262f;
    border-radius: 3px;
    overflow: hidden;
    display: flex;
  }
  .bar.l {
    justify-content: flex-end;
  }
  .fill {
    height: 100%;
    border-radius: 3px;
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
  .feed-empty {
    padding: 30px 15px;
    text-align: center;
    color: var(--faint);
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .fev {
    display: flex;
    gap: 11px;
    padding: 9px 15px;
  }
  .fmin {
    width: 34px;
    flex: none;
    font-size: 11px;
    color: var(--muted-3);
    padding-top: 1px;
  }
  .fbar {
    width: 3px;
    flex: none;
    border-radius: 2px;
  }
  .fbody {
    flex: 1;
    min-width: 0;
  }
  .ftitle {
    font-size: 12.5px;
    font-weight: 600;
  }
  .fsub {
    font-size: 11.5px;
    color: var(--muted-2);
    margin-top: 1px;
  }

  /* controls */
  .controls {
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 9px 14px;
  }
  .speeds {
    display: flex;
    align-items: center;
    gap: 3px;
    background: #0f1319;
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 3px;
  }
  .sp {
    padding: 5px 11px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--muted-2);
    cursor: pointer;
    user-select: none;
  }
  .sp.on {
    background: var(--accent);
    color: #0a0c0f;
  }
  .spacer {
    flex: 1;
  }
</style>
