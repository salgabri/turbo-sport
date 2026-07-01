<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import type { SportTheme } from "../design/theme";
  import type { Tourney } from "../design/dto";

  let {
    theme,
    result,
    busy,
    onPlay,
  }: {
    theme: SportTheme;
    result: Tourney | null;
    busy: boolean;
    onPlay: () => void;
  } = $props();

  // Backend round names -> display labels. Fallback: title-case the raw name.
  function roundLabel(name: string): string {
    const map: Record<string, string> = {
      final: "Final",
      semifinals: "Semi-finals",
      quarterfinals: "Quarter-finals",
    };
    if (map[name]) return map[name];
    // "round of 16" -> "Round of 16"
    return name.replace(/\b\w/g, (c) => c.toUpperCase());
  }
</script>

<div class="match">
  <!-- scoreboard: champion banner / prompt -->
  <div class="scoreboard" class:won={!!result}>
    {#if result}
      <div class="champ-block">
        <span class="champ-icon"><Icon name="trophy" size={22} /></span>
        <div class="champ-meta">
          <span class="champ-label">Champion</span>
          <span class="champ-name">{result.champion}</span>
        </div>
      </div>
      <button class="replay-btn" disabled={busy} onclick={onPlay}>
        <Icon name="play" size={15} />
        <span>{busy ? "Playing…" : "Replay"}</span>
      </button>
    {:else}
      <div class="prompt-block">
        <span class="prompt-title">Bracket not played</span>
        <span class="prompt-sub mono">Single elimination · winner takes the title</span>
      </div>
      <button class="play-btn" disabled={busy} onclick={onPlay}>
        <Icon name="play" size={15} />
        <span>{busy ? "Playing…" : "Play Tournament"}</span>
      </button>
    {/if}
  </div>

  <!-- body -->
  <div class="body">
    {#if !result}
      <div class="empty-frame">
        <div class="empty-inner">
          Press <b>Play Tournament</b> to run the draw and build the bracket.
        </div>
      </div>
    {:else}
      <div class="rounds">
        {#each result.rounds as round (round.name)}
          <div class="round">
            <div class="round-head">
              <span class="round-title">{roundLabel(round.name)}</span>
              <span class="round-count mono">{round.matches.length}</span>
            </div>
            <div class="round-stack">
              {#each round.matches as m}
                <div class="mcard">
                  <div class="mline win">
                    <span class="pname">{m.winner}</span>
                  </div>
                  <div class="mline lose">
                    <span class="pname">{m.loser}</span>
                  </div>
                  <div class="mscore mono">{m.score}</div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
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

  /* ---- scoreboard / banner ---- */
  .scoreboard {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    background: #12161c;
    border: 1px solid #232a33;
    border-radius: 14px;
    padding: 15px 22px;
  }
  .scoreboard.won {
    border-color: var(--accent-line);
    background: var(--accent-dim);
  }
  .champ-block {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .champ-icon {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent-soft);
    border: 1px solid var(--accent-line);
    color: var(--accent);
  }
  .champ-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .champ-label {
    font-size: 10px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent);
  }
  .champ-name {
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.02em;
    color: var(--text);
  }
  .prompt-block {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .prompt-title {
    font-size: 17px;
    font-weight: 700;
    color: var(--text);
  }
  .prompt-sub {
    font-size: 11.5px;
    color: var(--muted-3);
    letter-spacing: 0.02em;
  }
  .play-btn,
  .replay-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    background: var(--accent);
    color: #08120c;
    font-weight: 700;
    font-size: 13.5px;
    border: none;
    border-radius: 9px;
    cursor: pointer;
    box-shadow: 0 0 20px var(--accent-soft);
    font-family: inherit;
  }
  .play-btn:hover:not(:disabled),
  .replay-btn:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .replay-btn {
    background: none;
    color: var(--accent);
    border: 1px solid var(--accent-line);
    box-shadow: none;
  }
  .replay-btn:hover:not(:disabled) {
    background: var(--accent-soft);
    filter: none;
  }
  .play-btn:disabled,
  .replay-btn:disabled {
    opacity: 0.6;
    cursor: default;
  }

  /* ---- body ---- */
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  /* ---- empty frame (panel chrome) ---- */
  .empty-frame {
    flex: 1;
    background: #14181e;
    border: 1px solid #232a33;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .empty-inner {
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.6;
    color: var(--faint);
  }
  .empty-inner b {
    color: var(--muted);
  }

  /* ---- rounds (horizontal columns) ---- */
  .rounds {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 14px;
    overflow: auto;
    padding-bottom: 4px;
  }
  .round {
    flex: none;
    width: 220px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .round-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 2px;
  }
  .round-title {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--text-2);
  }
  .round-count {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--dim);
    background: #171c22;
    border: 1px solid #232a33;
    padding: 1px 6px;
    border-radius: 5px;
  }
  .round-stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .mcard {
    background: #14181e;
    border: 1px solid #232a33;
    border-radius: 10px;
    padding: 9px 11px;
    position: relative;
  }
  .mline {
    display: flex;
    align-items: center;
    padding: 2px 0;
  }
  .mline .pname {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 12.5px;
  }
  .mline.win .pname {
    font-weight: 700;
    color: var(--accent);
  }
  .mline.lose .pname {
    font-weight: 500;
    color: var(--muted-2);
  }
  .mscore {
    position: absolute;
    top: 9px;
    right: 11px;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted-3);
    background: #0f1319;
    border: 1px solid #232a33;
    padding: 1px 7px;
    border-radius: 5px;
  }
</style>
