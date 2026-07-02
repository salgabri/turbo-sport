<script lang="ts">
  import { onDestroy } from "svelte";
  import Icon from "../design/Icon.svelte";
  import type { SportTheme } from "../design/theme";
  import type { TiePlayback, TieEvent } from "../design/dto";

  let {
    theme,
    playback,
    onReload,
    loading = false,
  }: {
    theme: SportTheme;
    playback: TiePlayback | null;
    onReload: () => void;
    loading?: boolean;
  } = $props();

  // ---- playback clock (measured in games across the whole tie) ----
  let clock = $state(0); // current game index (float)
  let playing = $state(false);
  let speed = $state(1);
  const speeds = [1, 2, 5];
  let timer: ReturnType<typeof setInterval> | undefined;

  // Total games = sum of every set's games; the clock plays out over this range.
  const totalGames = $derived(
    (playback?.sets ?? []).reduce((n, s) => n + s.home_games + s.away_games, 0),
  );
  const ended = $derived(totalGames > 0 && clock >= totalGames);

  function tick() {
    if (!playing || !playback) return;
    clock = Math.min(clock + speed * 0.15, totalGames);
    if (clock >= totalGames) playing = false;
  }
  timer = setInterval(tick, 100);
  onDestroy(() => clearInterval(timer));

  // Reset the clock whenever a new playback arrives.
  let seenRef = $state<TiePlayback | null>(null);
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

  // ---- derived tie state at the current game index ----
  const shown = $derived(Math.floor(clock));
  // Cumulative game index at which each set *ends* (1-indexed, inclusive).
  const setEnds = $derived.by(() => {
    const ends: number[] = [];
    let acc = 0;
    for (const s of playback?.sets ?? []) {
      acc += s.home_games + s.away_games;
      ends.push(acc);
    }
    return ends;
  });
  // Events revealed so far (game index <= clock), newest first for the feed.
  const passed = $derived<TieEvent[]>((playback?.feed ?? []).filter((e) => e.game <= clock));
  const feed = $derived([...passed].reverse());

  // Per-set boxes: a set's box "reveals" (shows its final score) once the clock has passed
  // the last game of that set. Otherwise we show the live running games within that set.
  type SetBox = { set: number; home: number; away: number; revealed: boolean; live: boolean };
  const setBoxes = $derived.by<SetBox[]>(() => {
    const sets = playback?.sets ?? [];
    return sets.map((s, i) => {
      const start = i === 0 ? 0 : setEnds[i - 1];
      const end = setEnds[i];
      const revealed = shown >= end;
      if (revealed) return { set: i + 1, home: s.home_games, away: s.away_games, revealed: true, live: false };
      // Still playing this set: count revealed games within its window.
      const inSet = passed.filter((e) => e.set === i + 1);
      const h = inSet.filter((e) => e.side === 0).length;
      const a = inSet.filter((e) => e.side === 1).length;
      const live = shown >= start && shown < end;
      return { set: i + 1, home: h, away: a, revealed: false, live };
    });
  });

  // Current games line: the live set's running score, or FT if ended.
  const liveBox = $derived(setBoxes.find((b) => b.live));
  const currentLine = $derived.by(() => {
    if (ended) return "Match complete";
    if (!liveBox) return "Ready to play";
    return `Set ${liveBox.set} · ${liveBox.home}–${liveBox.away}`;
  });

  // Sets won so far (for the big scoreboard number), only counting revealed sets.
  const homeSets = $derived(setBoxes.filter((b) => b.revealed && b.home > b.away).length);
  const awaySets = $derived(setBoxes.filter((b) => b.revealed && b.away > b.home).length);

  const clockLabel = $derived(ended ? "FT" : `${shown}/${totalGames} games`);
  const winnerName = $derived(
    playback ? (playback.winner_side === 0 ? playback.home_name : playback.away_name) : "",
  );

  function crest(name: string): string {
    return name
      .split(/\s+/)
      .filter(Boolean)
      .map((w) => w[0])
      .slice(0, 2)
      .join("")
      .toUpperCase();
  }
</script>

<div class="match">
  {#if !playback}
    <div class="hero">
      <div class="hero-inner">
        <div class="hero-title">Match Centre</div>
        <div class="hero-sub">Watch the featured tie play out game by game.</div>
        <button class="load" disabled={loading} onclick={onReload}>
          <Icon name="play" size={15} />
          <span>{loading ? "Loading…" : "Play the featured match"}</span>
        </button>
      </div>
    </div>
  {:else}
    <!-- scoreboard -->
    <div class="scoreboard">
      <div class="team home">
        <div class="tinfo home-t">
          <span class="tname">{playback.home_name}</span>
          <span class="seed mono">#{playback.home_seed}</span>
        </div>
        <div class="crest home-crest">{crest(playback.home_name)}</div>
      </div>
      <div class="score">
        <div class="score-nums">
          <span class="mono" class:won={ended && playback.winner_side === 0}>{homeSets}</span>
          <span class="dash">–</span>
          <span class="mono" class:won={ended && playback.winner_side === 1}>{awaySets}</span>
        </div>
        <div class="clock">
          {#if playing && !ended}<span class="live-dot"></span>{/if}
          <span class="mono">{clockLabel}</span>
        </div>
      </div>
      <div class="team away">
        <div class="crest away-crest">{crest(playback.away_name)}</div>
        <div class="tinfo away-t">
          <span class="tname">{playback.away_name}</span>
          <span class="seed mono">#{playback.away_seed}</span>
        </div>
      </div>
    </div>

    <!-- winner banner -->
    {#if ended}
      <div class="banner">
        <Icon name="trophy" size={16} />
        <span>Match: {winnerName}</span>
      </div>
    {/if}

    <!-- body: set grid + feed -->
    <div class="body">
      <div class="left">
        <section class="panel setgrid">
          <div class="panel-h">Sets</div>
          <div class="boxes">
            {#each setBoxes as b (b.set)}
              <div class="setbox" class:revealed={b.revealed} class:live={b.live}>
                <div class="set-label mono">Set {b.set}</div>
                {#if b.revealed || b.live}
                  <div class="set-games">
                    <span class="mono" class:hi={b.home > b.away}>{b.home}</span>
                    <span class="set-dash">–</span>
                    <span class="mono" class:hi={b.away > b.home}>{b.away}</span>
                  </div>
                {:else}
                  <div class="set-games pending mono">·–·</div>
                {/if}
                {#if b.live}<span class="set-live mono">LIVE</span>{/if}
              </div>
            {/each}
          </div>
        </section>

        <section class="panel current">
          <div class="panel-h">Current</div>
          <div class="current-line mono">{currentLine}</div>
        </section>
      </div>

      <div class="rail">
        <section class="panel feed">
          <div class="panel-h">Point &amp; Game Feed</div>
          <div class="feed-list">
            {#if feed.length === 0}
              <div class="feed-empty">First serve. Games will appear here.</div>
            {:else}
              {#each feed as e (e.game)}
                <div class="fev">
                  <div class="fmin mono">G{e.game}</div>
                  <div class="fbar" style="background:{e.side === 0 ? 'var(--accent)' : '#5aa9e6'}"></div>
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
    color: #180a0a;
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
  .team {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 13px;
  }
  .team.home {
    justify-content: flex-end;
  }
  .tinfo {
    display: flex;
    flex-direction: column;
  }
  .home-t {
    align-items: flex-end;
  }
  .tname {
    font-size: 15px;
    font-weight: 700;
  }
  .seed {
    font-size: 11px;
    color: var(--accent);
    margin-top: 2px;
  }
  .crest {
    width: 36px;
    height: 36px;
    flex: none;
    border-radius: 0;
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
    font-weight: 700;
    line-height: 1;
  }
  .score-nums .won {
    color: var(--accent);
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

  /* winner banner */
  .banner {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    padding: 11px 16px;
    background: var(--accent-soft);
    border: 1px solid var(--accent-line);
    border-radius: 0;
    color: var(--accent);
    font-weight: 700;
    font-size: 14px;
  }

  /* body */
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 14px;
  }
  .left {
    flex: 1.5;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .rail {
    width: 320px;
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

  /* set grid */
  .setgrid {
    flex: none;
  }
  .boxes {
    padding: 15px;
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
  .setbox {
    flex: 1;
    min-width: 110px;
    background: var(--raised);
    border: 1px solid #232a33;
    border-radius: 0;
    padding: 14px 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    opacity: 0.45;
    transition:
      opacity 0.3s ease,
      border-color 0.3s ease;
  }
  .setbox.live,
  .setbox.revealed {
    opacity: 1;
  }
  .setbox.live {
    border-color: var(--accent-line);
    box-shadow: 0 0 16px var(--accent-soft);
  }
  .set-label {
    font-size: 10.5px;
    color: var(--muted-3);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .set-games {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 26px;
    font-weight: 700;
    line-height: 1;
  }
  .set-games .hi {
    color: var(--accent);
  }
  .set-games.pending {
    color: var(--faint);
    font-size: 22px;
    font-weight: 600;
  }
  .set-dash {
    font-size: 15px;
    color: #4a535e;
  }
  .set-live {
    font-size: 9.5px;
    letter-spacing: 0.12em;
    color: var(--accent);
    font-weight: 700;
  }

  /* current line */
  .current {
    flex: none;
  }
  .current-line {
    padding: 15px;
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
  }

  /* feed */
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
    border-radius: 0;
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
    color: #180a0a;
  }
  .spacer {
    flex: 1;
  }
</style>
