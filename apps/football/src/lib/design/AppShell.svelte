<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { SportTheme } from "./theme";
  import type { Screen } from "./dto";

  type NavItem = { id: Screen; label: string; icon: string; badge?: number };

  let {
    theme,
    gameName,
    clubName,
    clubTag,
    clubCrest,
    leagueName,
    managerName = "You",
    dateStr,
    notif = 0,
    screen,
    title,
    sub,
    busy = false,
    onNav,
    onAdvance,
    toast = null,
    actions,
    children,
  }: {
    theme: SportTheme;
    gameName: string;
    clubName: string;
    clubTag: string;
    clubCrest: string;
    leagueName: string;
    managerName?: string;
    dateStr: string;
    notif?: number;
    screen: Screen;
    title: string;
    sub: string;
    busy?: boolean;
    onNav: (s: Screen) => void;
    onAdvance: () => void;
    toast?: string | null;
    actions?: import("svelte").Snippet;
    children: import("svelte").Snippet;
  } = $props();

  const nav: NavItem[] = [
    { id: "home", label: "Home", icon: "home" },
    { id: "squad", label: "Squad", icon: "squad" },
    { id: "profile", label: "Player", icon: "user" },
    { id: "table", label: "Table", icon: "trophy" },
    { id: "transfers", label: "Transfers", icon: "transfer" },
    { id: "match", label: "Match", icon: "play" },
  ];

  const accentVars = $derived(
    `--accent:${theme.accent};--accent-soft:${theme.accent}22;--accent-dim:${theme.accent}14;--accent-line:${theme.accent}55`,
  );
</script>

<div class="shell" style={accentVars}>
  <!-- TITLE BAR -->
  <div class="titlebar">
    <div class="tb-left">
      <div class="logo"><div class="logo-slash"></div></div>
      <span class="gamename">{gameName}</span>
      <span class="gamesub">— {clubName} · {leagueName}</span>
    </div>
    <div class="tb-win">
      <div class="wbtn"><Icon name="min" size={15} /></div>
      <div class="wbtn"><Icon name="sq" size={13} /></div>
      <div class="wbtn close"><Icon name="x" size={15} /></div>
    </div>
  </div>

  <!-- BODY -->
  <div class="body">
    <!-- SIDEBAR -->
    <aside class="sidebar">
      <div class="clubcard">
        <div class="crest">{clubCrest}</div>
        <div class="clubmeta">
          <div class="clubname">{clubName}</div>
          <div class="clubtag mono">{clubTag}</div>
        </div>
      </div>

      <div class="navwrap">
        <div class="navlabel mono">MENU</div>
        <div class="navlist">
          {#each nav as it (it.id)}
            <button
              class="navitem"
              class:on={screen === it.id}
              onclick={() => onNav(it.id)}>
              <span class="navicon"><Icon name={it.icon} size={18} /></span>
              <span class="navtext">{it.label}</span>
              {#if it.badge}<span class="navbadge mono">{it.badge}</span>{/if}
            </button>
          {/each}
        </div>
      </div>

      <div class="managermini">
        <div class="mavatar">YM</div>
        <div class="minfo">
          <div class="mname">{managerName}</div>
          <div class="mrole mono">Manager · Season 1</div>
        </div>
        <div class="mgear"><Icon name="gear" size={17} /></div>
      </div>
    </aside>

    <!-- MAIN -->
    <main class="main">
      <div class="topbar">
        <div class="tb-title">
          <div class="tt-main">{title}</div>
          <div class="tt-sub mono">{sub}</div>
        </div>

        <div class="search">
          <Icon name="search" size={16} />
          <span class="ph">Search players, clubs, staff…</span>
          <span class="kbd mono">⌘K</span>
        </div>

        <div class="tb-right">
          {#if actions}<div class="actions">{@render actions()}</div>{/if}
          <div class="datechip">
            <Icon name="calendar" size={15} />
            <span class="mono">{dateStr}</span>
          </div>
          <div class="bell">
            <Icon name="bell" size={18} />
            {#if notif}<span class="bellbadge mono">{notif}</span>{/if}
          </div>
          <button class="advance" disabled={busy} onclick={onAdvance}>
            <Icon name="play" size={15} />
            <span>Advance</span>
          </button>
        </div>
      </div>

      <div class="screen">
        {@render children()}
      </div>
    </main>
  </div>

  {#if toast}
    <div class="toast">
      <span style="color:var(--accent);display:flex"><Icon name="check" size={16} /></span>
      {toast}
    </div>
  {/if}
</div>

<style>
  .shell {
    height: 100vh;
    width: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    color: var(--text);
    overflow: hidden;
    position: relative;
    font-family: var(--font-sans);
  }

  /* title bar */
  .titlebar {
    height: 34px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background: var(--bg-titlebar);
    border-bottom: 1px solid var(--line-soft);
    user-select: none;
  }
  .tb-left {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .logo {
    width: 15px;
    height: 15px;
    border-radius: 4px;
    background: var(--accent);
    position: relative;
    box-shadow: 0 0 10px var(--accent-soft);
  }
  .logo-slash {
    position: absolute;
    inset: 4.5px 2px 4.5px 6.5px;
    background: var(--bg-titlebar);
    border-radius: 1px;
    transform: skewX(-14deg);
  }
  .gamename {
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.22em;
    font-weight: 600;
    color: var(--text-3);
  }
  .gamesub {
    font-size: 11px;
    color: var(--faint);
  }
  .tb-win {
    display: flex;
    align-items: center;
    gap: 2px;
    color: #5a636e;
  }
  .wbtn {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 5px;
    cursor: pointer;
  }
  .wbtn:hover {
    background: var(--hover);
  }
  .wbtn.close:hover {
    background: #7a2530;
    color: #fff;
  }

  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  /* sidebar */
  .sidebar {
    width: 252px;
    flex: none;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--line-soft);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .clubcard {
    padding: 16px 16px 14px;
    display: flex;
    align-items: center;
    gap: 12px;
    border-bottom: 1px solid #171c22;
  }
  .crest {
    width: 46px;
    height: 46px;
    flex: none;
    border-radius: 11px;
    background: var(--accent-soft);
    border: 1px solid var(--accent-line);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 16px;
    color: var(--accent);
  }
  .clubmeta {
    min-width: 0;
  }
  .clubname {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .clubtag {
    font-size: 11px;
    color: var(--muted-3);
    margin-top: 2px;
  }
  .navwrap {
    padding: 16px 12px 6px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  .navlabel {
    font-size: 10px;
    letter-spacing: 0.16em;
    font-weight: 600;
    color: #565f6a;
    padding: 0 4px 8px;
  }
  .navlist {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .navitem {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 11px;
    border-radius: 9px;
    cursor: pointer;
    background: none;
    border: none;
    color: var(--muted-2);
    font-size: 13.5px;
    font-weight: 600;
    font-family: inherit;
    text-align: left;
  }
  .navitem:hover {
    background: #12161c;
  }
  .navitem.on {
    background: var(--accent-soft);
    color: var(--text);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .navicon {
    display: flex;
    color: var(--muted-3);
  }
  .navitem.on .navicon {
    color: var(--accent);
  }
  .navtext {
    flex: 1;
  }
  .navbadge {
    font-size: 11px;
    font-weight: 700;
    background: var(--accent);
    color: var(--bg-titlebar);
    min-width: 19px;
    height: 19px;
    padding: 0 5px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .managermini {
    padding: 12px 14px;
    border-top: 1px solid #171c22;
    display: flex;
    align-items: center;
    gap: 11px;
  }
  .mavatar {
    width: 34px;
    height: 34px;
    flex: none;
    border-radius: 9px;
    background: var(--raised);
    border: 1px solid #272e37;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 13px;
    color: var(--text-3);
  }
  .minfo {
    flex: 1;
    min-width: 0;
  }
  .mname {
    font-size: 13px;
    font-weight: 600;
  }
  .mrole {
    font-size: 10.5px;
    color: var(--dim);
  }
  .mgear {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    color: var(--dim);
    cursor: pointer;
  }
  .mgear:hover {
    background: #161b22;
    color: var(--text-3);
  }

  /* main */
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-main);
  }
  .topbar {
    height: 60px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 0 22px;
    border-bottom: 1px solid var(--line-soft);
    background: var(--bg-topbar);
  }
  .tb-title {
    min-width: 0;
  }
  .tt-main {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.015em;
    line-height: 1.1;
  }
  .tt-sub {
    font-size: 11.5px;
    color: var(--muted-3);
    margin-top: 2px;
  }
  .search {
    flex: 1;
    max-width: 340px;
    display: flex;
    align-items: center;
    gap: 9px;
    background: #14181e;
    border: 1px solid var(--line-3);
    border-radius: 9px;
    padding: 8px 11px;
    color: var(--dim);
  }
  .search .ph {
    flex: 1;
    font-size: 12.5px;
    color: #5c6570;
  }
  .kbd {
    font-size: 10.5px;
    background: #1c222a;
    border: 1px solid #2a323c;
    padding: 2px 6px;
    border-radius: 5px;
    color: var(--muted-3);
  }
  .tb-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .actions :global(button) {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: #14181e;
    border: 1px solid var(--line-3);
    color: var(--text-3);
    border-radius: 9px;
    font-size: 12.5px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
  }
  .actions :global(button:hover:not(:disabled)) {
    border-color: var(--accent-line);
    color: var(--accent);
  }
  .actions :global(button:disabled) {
    opacity: 0.5;
    cursor: default;
  }
  .datechip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    background: #14181e;
    border: 1px solid var(--line-3);
    border-radius: 9px;
    color: var(--muted-3);
  }
  .datechip .mono {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-2);
    letter-spacing: 0.02em;
  }
  .bell {
    position: relative;
    width: 38px;
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #14181e;
    border: 1px solid var(--line-3);
    border-radius: 9px;
    color: var(--muted);
    cursor: pointer;
  }
  .bell:hover {
    color: var(--text);
    border-color: #333c47;
  }
  .bellbadge {
    position: absolute;
    top: -5px;
    right: -5px;
    min-width: 17px;
    height: 17px;
    padding: 0 4px;
    background: var(--accent);
    color: var(--bg-titlebar);
    font-size: 10px;
    font-weight: 700;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--bg-topbar);
  }
  .advance {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 17px;
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
  .advance:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .advance:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .screen {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .toast {
    position: absolute;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--raised);
    border: 1px solid #2f3742;
    color: var(--text);
    padding: 11px 18px;
    border-radius: 10px;
    font-size: 13px;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 9px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
    animation: tsToast 0.2s ease;
    z-index: 50;
  }
</style>
