<script lang="ts">
  import Icon from "../design/Icon.svelte";
  import { BASKETBALL, type SportTheme } from "../design/theme";

  let { theme = BASKETBALL }: { theme?: SportTheme } = $props();

  // No live-match data yet — this renders the designed match frame as a
  // not-yet-live shell awaiting the match-engine hookup.
  const speeds = ["1×", "2×", "5×"];
</script>

<div class="match">
  <!-- scoreboard: neutral, not in progress -->
  <div class="scoreboard">
    <div class="side side-home">
      <span class="team-name">Home</span>
      <div class="crest crest-home">H</div>
    </div>
    <div class="score">
      <div class="score-line">
        <span class="score-num">–</span>
        <span class="score-sep">–</span>
        <span class="score-num">–</span>
      </div>
      <div class="status-pill">Not in progress</div>
    </div>
    <div class="side side-away">
      <div class="crest crest-away">A</div>
      <span class="team-name">Away</span>
    </div>
  </div>

  <!-- body -->
  <div class="body">
    <div class="col-main">
      <!-- court -->
      <div class="court">
        <div class="court-outline"></div>
        <div class="court-halfline"></div>
        <div class="court-circle"></div>
        <!-- left paint + free-throw arc + hoop -->
        <div class="paint paint-left"></div>
        <div class="ftarc ftarc-left"></div>
        <div class="hoop hoop-left"></div>
        <!-- right paint + free-throw arc + hoop -->
        <div class="paint paint-right"></div>
        <div class="ftarc ftarc-right"></div>
        <div class="hoop hoop-right"></div>
      </div>
    </div>

    <!-- right column -->
    <div class="col-side">
      <!-- Match Stats -->
      <div class="panel panel-stats">
        <div class="panel-head"><span class="panel-title">Match Stats</span></div>
        <div class="panel-empty">Match statistics appear once the game tips off.</div>
      </div>

      <!-- Match Feed -->
      <div class="panel panel-feed">
        <div class="panel-head"><span class="panel-title">Play-by-Play</span></div>
        <div class="panel-empty panel-empty-fill">
          Live game playback arrives with the match-engine hookup.
        </div>
      </div>
    </div>
  </div>

  <!-- controls (disabled) -->
  <div class="controls">
    <div class="play-btn" aria-disabled="true">
      <Icon name="play" size={15} />
      <span>Play</span>
    </div>
    <div class="speeds" aria-disabled="true">
      {#each speeds as sp, i}
        <div class="speed" class:active={i === 0}>{sp}</div>
      {/each}
    </div>
    <div class="spacer"></div>
    <div class="ghost-btn" aria-disabled="true">Timeout</div>
    <div class="ghost-btn" aria-disabled="true">Substitution</div>
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

  /* ---- scoreboard ---- */
  .scoreboard {
    flex: none;
    display: flex;
    align-items: center;
    gap: 24px;
    background: #12161c;
    border: 1px solid #232a33;
    border-radius: 14px;
    padding: 13px 22px;
  }
  .side {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 13px;
  }
  .side-home {
    justify-content: flex-end;
  }
  .team-name {
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
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
  .crest-home {
    background: var(--accent-soft);
    border: 1px solid var(--accent-line);
    color: var(--accent);
  }
  .crest-away {
    background: #1b212a;
    border: 1px solid #2a323c;
    color: #c2cad3;
  }
  .score {
    text-align: center;
    flex: none;
  }
  .score-line {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .score-num {
    font-size: 32px;
    font-weight: 800;
    font-family: var(--font-mono);
    line-height: 1;
    color: #4a535e;
  }
  .score-sep {
    font-size: 18px;
    color: #4a535e;
  }
  .status-pill {
    margin-top: 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 3px 10px;
    border-radius: 7px;
    background: #171c22;
    border: 1px solid #232a33;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--muted-3);
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  /* ---- body ---- */
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 14px;
  }
  .col-main {
    flex: 1.5;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .col-side {
    width: 296px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
  }

  /* ---- court (neutral hardwood) ---- */
  .court {
    flex: 1;
    min-height: 0;
    position: relative;
    border: 1px solid #4a3418;
    border-radius: 12px;
    overflow: hidden;
    background: repeating-linear-gradient(
      90deg,
      #3a2a17 0,
      #3a2a17 8.33%,
      #35271500 8.33%,
      #352715 16.66%
    );
  }
  .court-outline {
    position: absolute;
    inset: 4%;
    border: 2px solid rgba(255, 255, 255, 0.18);
    border-radius: 3px;
  }
  .court-halfline {
    position: absolute;
    left: 50%;
    top: 4%;
    bottom: 4%;
    width: 2px;
    background: rgba(255, 255, 255, 0.18);
    transform: translateX(-50%);
  }
  .court-circle {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 16%;
    padding-bottom: 16%;
    border: 2px solid rgba(255, 255, 255, 0.18);
    border-radius: 50%;
    transform: translate(-50%, -50%);
  }
  /* the paint / key box at each end */
  .paint {
    position: absolute;
    top: 34%;
    bottom: 34%;
    width: 15%;
    border: 2px solid rgba(255, 255, 255, 0.16);
  }
  .paint-left {
    left: 4%;
    border-left: none;
  }
  .paint-right {
    right: 4%;
    border-right: none;
  }
  /* free-throw semicircle at the top of each key */
  .ftarc {
    position: absolute;
    top: 50%;
    width: 9%;
    padding-bottom: 18%;
    border: 2px solid rgba(255, 255, 255, 0.16);
    border-radius: 50%;
    transform: translateY(-50%);
  }
  .ftarc-left {
    left: calc(4% + 15% - 4.5%);
  }
  .ftarc-right {
    right: calc(4% + 15% - 4.5%);
  }
  /* hoop marker at each baseline */
  .hoop {
    position: absolute;
    top: 50%;
    width: 12px;
    height: 12px;
    border: 2px solid var(--accent);
    border-radius: 50%;
    transform: translateY(-50%);
    box-shadow: 0 0 8px var(--accent-soft);
  }
  .hoop-left {
    left: 6.5%;
  }
  .hoop-right {
    right: 6.5%;
  }

  /* ---- panels ---- */
  .panel {
    background: #14181e;
    border: 1px solid #232a33;
    border-radius: 12px;
    overflow: hidden;
  }
  .panel-stats {
    flex: none;
  }
  .panel-feed {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .panel-head {
    padding: 11px 15px;
    border-bottom: 1px solid #232a33;
  }
  .panel-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text);
  }
  .panel-empty {
    padding: 44px 22px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.5;
    color: var(--faint);
  }
  .panel-empty-fill {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* ---- controls ---- */
  .controls {
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
    background: #12161c;
    border: 1px solid #232a33;
    border-radius: 12px;
    padding: 9px 14px;
  }
  .play-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--accent);
    color: #0a0c0f;
    font-weight: 700;
    font-size: 13px;
    border-radius: 8px;
    opacity: 0.5;
    cursor: default;
  }
  .speeds {
    display: flex;
    align-items: center;
    gap: 3px;
    background: #0f1319;
    border: 1px solid #232a33;
    border-radius: 8px;
    padding: 3px;
    opacity: 0.5;
  }
  .speed {
    padding: 5px 11px;
    font-size: 12px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--muted-3);
    border-radius: 6px;
    cursor: default;
  }
  .speed.active {
    background: #1b212a;
    color: var(--text-3);
  }
  .spacer {
    flex: 1;
  }
  .ghost-btn {
    padding: 8px 14px;
    border: 1px solid #2a323c;
    color: #c2cad3;
    border-radius: 8px;
    font-size: 12.5px;
    font-weight: 600;
    opacity: 0.5;
    cursor: default;
  }
</style>
