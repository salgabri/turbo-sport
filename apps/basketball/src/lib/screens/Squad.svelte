<script lang="ts">
  import { moraleWord, fitColor, tierColor, money, posPill } from "../design/color";
  import { BASKETBALL, posColor, type SportTheme } from "../design/theme";
  import type { PlayerView } from "../design/dto";

  let {
    theme,
    squad,
    count,
    selectedName,
    onSelectPlayer,
  }: {
    theme: SportTheme;
    squad: PlayerView[];
    count: number;
    selectedName: string;
    onSelectPlayer: (p: PlayerView) => void;
  } = $props();

  // ---- column-set tabs (client-side) ----
  type TabId = "overview" | "attributes" | "contract";
  let activeTab = $state<TabId>("overview");
  const tabs: { id: TabId; label: string }[] = [
    { id: "overview", label: "Overview" },
    { id: "attributes", label: "Attributes" },
    { id: "contract", label: "Contract" },
  ];

  // ---- sticky-header column labels per tab ----
  type Col = { label: string; width: string };
  const overviewCols: Col[] = [
    { label: "Morale", width: "84px" },
    { label: "Fit", width: "54px" },
    { label: "Age", width: "50px" },
    { label: "Wage", width: "70px" },
    { label: "Contract", width: "72px" },
    { label: "Status", width: "78px" },
  ];
  // Attribute columns are derived from the sport theme: OVR + each attr short.
  const attributeCols = $derived<Col[]>([
    { label: "OVR", width: "50px" },
    ...theme.attributes.map((a) => ({ label: a.short, width: "46px" })),
  ]);
  const contractCols: Col[] = [
    { label: "Wage", width: "70px" },
    { label: "Value", width: "74px" },
    { label: "Signed", width: "62px" },
    { label: "Expires", width: "64px" },
    { label: "Apps", width: "52px" },
    { label: "Pts", width: "46px" },
  ];
  const activeCols = $derived(
    activeTab === "attributes"
      ? attributeCols
      : activeTab === "contract"
        ? contractCols
        : overviewCols,
  );

  const DASH = "—";
  const cur = $derived(theme.currency);

  // ---- footer metrics (REAL where available, dash otherwise) ----
  const ages = $derived(
    squad.map((p) => p.age).filter((a): a is number => typeof a === "number"),
  );
  const avgAge = $derived(
    ages.length ? (ages.reduce((s, a) => s + a, 0) / ages.length).toFixed(1) : DASH,
  );
  const ovrs = $derived(
    squad.map((p) => p.overall).filter((o): o is number => typeof o === "number"),
  );
  const avgOvr = $derived(
    ovrs.length ? Math.round(ovrs.reduce((s, o) => s + o, 0) / ovrs.length).toString() : DASH,
  );
  const totalValue = $derived.by(() => {
    const vals = squad
      .map((p) => p.market_value)
      .filter((v): v is number => typeof v === "number");
    return vals.length ? money(vals.reduce((s, v) => s + v, 0), theme.currency) : DASH;
  });

  // ---- cell style builders ----
  const base = (width: string) =>
    `text-align:right;width:${width};padding:0 5px;white-space:nowrap;`;
  const mono = "font-family:var(--font-mono);";

  function moneyCell(width: string, n: number | null | undefined) {
    return {
      text: typeof n === "number" ? money(n, cur) : DASH,
      style:
        base(width) +
        mono +
        `font-size:12px;color:${typeof n === "number" ? "#c2cad3" : "#5a636e"};font-weight:500`,
    };
  }
  function fitCell(width: string, v: number | null | undefined) {
    return {
      text: typeof v === "number" ? v + "%" : DASH,
      style:
        base(width) +
        mono +
        `font-weight:600;font-size:12.5px;color:${typeof v === "number" ? fitColor(v) : "#5a636e"}`,
    };
  }
  function moraleCell(width: string, v: number | null | undefined) {
    if (typeof v !== "number")
      return { text: DASH, style: base(width) + mono + "font-size:12px;color:#5a636e" };
    const w = moraleWord(v);
    return {
      text: w.t,
      style: base(width) + `font-weight:600;font-size:12px;color:${w.c}`,
    };
  }
  function monoCell(width: string, text: string | null | undefined) {
    const has = text != null && text !== "";
    return {
      text: has ? String(text) : DASH,
      style: base(width) + mono + `font-size:12px;color:${has ? "#8b95a1" : "#5a636e"}`,
    };
  }
  function ovrCell(width: string, v: number | null | undefined) {
    return {
      text: typeof v === "number" ? String(v) : DASH,
      style:
        base(width) +
        mono +
        `font-weight:700;font-size:13.5px;color:${typeof v === "number" ? tierColor(v) : "#5a636e"}`,
    };
  }
  function heatCell(width: string, v: number | null | undefined) {
    return {
      text: typeof v === "number" ? String(v) : DASH,
      style:
        base(width) +
        mono +
        `font-weight:600;font-size:12.5px;color:${typeof v === "number" ? tierColor(v) : "#5a636e"}`,
    };
  }

  // status word for the Overview "Status" column, from REAL bools
  function statusCell(width: string, p: PlayerView) {
    let t = "Available";
    let c = "#7ac88f";
    if (p.retired) {
      t = "Retired";
      c = "#8b95a1";
    } else if (p.injured) {
      t = typeof p.injury_days === "number" && p.injury_days > 0 ? `${p.injury_days}d out` : "Injured";
      c = "#ef6b6b";
    } else if (p.suspended) {
      t = "Suspended";
      c = "#ef6b6b";
    } else if (p.free_agent) {
      t = "Free Agent";
      c = "#edb95e";
    }
    return { text: t, style: base(width) + `font-weight:600;font-size:12px;color:${c}` };
  }

  // build the numeric cells for the active column set for one player
  function cellsFor(p: PlayerView) {
    if (activeTab === "overview") {
      return [
        moraleCell("84px", p.morale),
        fitCell("54px", p.fitness),
        monoCell("50px", typeof p.age === "number" ? String(p.age) : null),
        moneyCell("70px", p.wage),
        monoCell("72px", p.contract_until),
        statusCell("78px", p),
      ];
    }
    if (activeTab === "attributes") {
      const a = (p.attrs ?? {}) as Record<string, number>;
      return [
        ovrCell("50px", p.overall),
        ...theme.attributes.map((d) => heatCell("46px", a[d.key])),
      ];
    }
    // contract
    return [
      moneyCell("70px", p.wage),
      moneyCell("74px", p.market_value),
      monoCell("62px", p.signed),
      monoCell("64px", p.contract_until),
      monoCell("52px", typeof p.apps === "number" ? String(p.apps) : null),
      monoCell("46px", typeof p.goals === "number" ? String(p.goals) : null),
    ];
  }

  const posMuted = "#7a828d";
  function posStyleFor(p: PlayerView) {
    const hex = p.position_group ? posColor(theme, p.position_group) : posMuted;
    return posPill(hex, "sm");
  }
  function posLabel(p: PlayerView) {
    return p.position_group ?? DASH;
  }

  function meta(p: PlayerView) {
    const age = typeof p.age === "number" ? String(p.age) : null;
    const nat = p.nationality ?? null;
    if (age && nat) return `${age} · ${nat}`;
    if (age) return age;
    if (nat) return nat;
    return DASH;
  }

  const empty = $derived(squad.length === 0);
</script>

<div class="squad">
  <!-- top bar: column-set tabs + count -->
  <div class="topbar">
    <div class="segset">
      {#each tabs as t}
        <div
          class="seg"
          class:on={activeTab === t.id}
          onclick={() => (activeTab = t.id)}
          onkeydown={(e) => (e.key === "Enter" || e.key === " ") && (activeTab = t.id)}
          role="button"
          tabindex="0">
          {t.label}
        </div>
      {/each}
    </div>
    <div class="right">
      <span class="count">{count} players · click to select</span>
    </div>
  </div>

  <!-- scroll body -->
  <div class="body">
    {#if empty}
      <div class="empty">No players in this list.</div>
    {:else}
      <!-- sticky header -->
      <div class="header">
        <div class="lead">
          <span class="h-pos">POS</span>
          <span class="h-num">#</span>
          <span class="h-player">PLAYER</span>
        </div>
        <div class="h-cells">
          {#each activeCols as col}
            <span class="h-col" style="width:{col.width}">{col.label}</span>
          {/each}
        </div>
      </div>

      {#each squad as p}
          <div
            class="row"
            class:sel={selectedName != null && selectedName === p.name}
            onclick={() => onSelectPlayer(p)}
            onkeydown={(e) => (e.key === "Enter" || e.key === " ") && onSelectPlayer(p)}
            role="button"
            tabindex="0">
            <div class="lead">
              <span style={posStyleFor(p)}>{posLabel(p)}</span>
              <span class="num">{p.shirt ?? DASH}</span>
              <div class="pcol">
                <div class="pname-line">
                  <span class="pname">{p.name ?? DASH}</span>
                  {#if p.captain}
                    <span class="capt">C</span>
                  {/if}
                  {#if p.injured}
                    <svg class="ic inj" viewBox="0 0 24 24" aria-hidden="true"
                      ><path fill="currentColor" d="M10 4h4v6h6v4h-6v6h-4v-6H4v-4h6z" /></svg>
                  {/if}
                  {#if p.suspended}
                    <span class="susp"></span>
                  {/if}
                </div>
                <div class="meta">{meta(p)}</div>
              </div>
            </div>
            <div class="cells">
              {#each cellsFor(p) as cell}
                <span style={cell.style}>{cell.text}</span>
              {/each}
            </div>
          </div>
        {/each}
    {/if}
  </div>

  <!-- footer bar -->
  <div class="footer">
    <span>SIZE <b>{count}</b></span>
    <span>AVG AGE <b>{avgAge}</b></span>
    <span>AVG OVR <b>{avgOvr}</b></span>
    <span>TOTAL VALUE <b>{totalValue}</b></span>
    <span class="sel-lbl">SELECTED <b class="sel-name">{selectedName || "None"}</b></span>
  </div>
</div>

<style>
  .squad {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  /* ---- top bar ---- */
  .topbar {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 11px 20px;
    border-bottom: 1px solid #191e25;
    background: #0e1115;
  }
  .segset {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #0f1319;
    border: 1px solid #232a33;
    border-radius: 0;
    padding: 3px;
  }
  .seg {
    padding: 5px 12px;
    border-radius: 0;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    background: transparent;
    color: #8b95a1;
    transition: all 0.12s;
    user-select: none;
  }
  .seg.on {
    background: var(--accent);
    color: #0a0c0f;
  }
  .right {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .count {
    font-size: 11.5px;
    color: #616b77;
    font-family: var(--font-mono);
  }

  /* ---- scroll body ---- */
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  /* ---- sticky header ---- */
  .header {
    display: flex;
    align-items: center;
    padding: 9px 20px;
    position: sticky;
    top: 0;
    background: #12161c;
    border-bottom: 1px solid #232a33;
    z-index: 2;
  }
  .lead {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .h-pos {
    width: 34px;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .h-num {
    width: 22px;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    text-align: center;
  }
  .h-player {
    flex: 1;
    font-size: 9.5px;
    color: #616b77;
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .h-cells {
    flex: none;
    display: flex;
  }
  .h-col {
    text-align: right;
    padding: 0 5px;
    font-size: 9.5px;
    letter-spacing: 0.05em;
    color: #616b77;
    font-family: var(--font-mono);
    text-transform: uppercase;
    cursor: pointer;
    white-space: nowrap;
  }
  .h-col:hover {
    color: #c2cad3;
  }

  /* ---- rows ---- */
  .row {
    display: flex;
    align-items: center;
    padding: 9px 20px;
    border-bottom: 1px solid #171c22;
    cursor: pointer;
  }
  .row:hover {
    background: #161b22;
  }
  .row.sel {
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .num {
    width: 22px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: #616b77;
    text-align: center;
  }
  .pcol {
    flex: 1;
    min-width: 0;
  }
  .pname-line {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .pname {
    font-size: 13.5px;
    font-weight: 600;
    color: #e9edf1;
    white-space: nowrap;
  }
  .capt {
    font-size: 9px;
    font-weight: 700;
    color: #0a0c0f;
    background: var(--accent);
    width: 15px;
    height: 15px;
    border-radius: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: none;
  }
  .ic {
    width: 14px;
    height: 14px;
    flex: none;
  }
  .inj {
    color: #ef6b6b;
  }
  .susp {
    width: 11px;
    height: 14px;
    border-radius: 0;
    background: #ef6b6b;
    display: inline-block;
    flex: none;
  }
  .meta {
    font-size: 11px;
    color: #6b7480;
    font-family: var(--font-mono);
    margin-top: 1px;
  }
  .cells {
    flex: none;
    display: flex;
    align-items: center;
  }

  /* ---- empty states ---- */
  .empty {
    padding: 70px;
    text-align: center;
    color: #5a636e;
    font-family: var(--font-mono);
    font-size: 13px;
  }

  /* ---- footer ---- */
  .footer {
    flex: none;
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 10px 20px;
    border-top: 1px solid #191e25;
    background: #0e1115;
    font-family: var(--font-mono);
    font-size: 11px;
    color: #6b7480;
    letter-spacing: 0.03em;
  }
  .footer b {
    color: #d4dae1;
    font-weight: 600;
  }
  .sel-lbl {
    margin-left: auto;
  }
  .sel-name {
    color: var(--accent) !important;
  }
</style>
