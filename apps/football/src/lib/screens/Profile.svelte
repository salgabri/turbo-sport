<script lang="ts">
  import { tierColor, moraleWord, money, posPill } from "../design/color";
  import { FOOTBALL, posColor, type SportTheme } from "../design/theme";
  import type { PlayerView } from "../design/dto";

  let { theme, player }: { theme: SportTheme; player: PlayerView | null } = $props();

  const cur = $derived(theme.currency ?? FOOTBALL.currency);

  // --- Header derived values (REAL fields) ---
  const num = $derived(player?.shirt ?? "—");
  const posGroup = $derived(player?.position_group ?? null);
  const posText = $derived(posGroup ?? "—");
  const posStyle = $derived(posPill(posColor(theme, posGroup), "md"));

  const clubLine = $derived(
    player?.nationality ??
      (player?.contract_until ? `Contracted to ${player.contract_until}` : "Contracted"),
  );

  const facts = $derived.by(() => {
    if (!player) return [] as { l: string; v: string }[];
    const m = player.morale != null ? moraleWord(player.morale) : null;
    return [
      { l: "Age", v: player.age != null ? String(player.age) : "—" },
      { l: "Fitness", v: player.fitness != null ? `${Math.round(player.fitness)}%` : "—" },
      { l: "Morale", v: m ? m.t : "—" },
      { l: "Value", v: player.market_value != null ? money(player.market_value, cur) : "—" },
      { l: "Wage", v: player.wage != null ? `${money(player.wage, cur)}/wk` : "—" },
      { l: "Contract", v: player.contract_until ? `to ${player.contract_until}` : "—" },
    ];
  });

  const hasOvr = $derived(player?.overall != null);
  const hasPot = $derived(player?.potential != null);
  const ovrColor = $derived(hasOvr ? tierColor(player!.overall as number) : "#616b77");
  const potColor = $derived(hasPot ? tierColor(player!.potential as number) : "#616b77");
  const ovrVal = $derived(hasOvr ? String(player!.overall) : "—");
  const potVal = $derived(hasPot ? String(player!.potential) : "—");

  const hasAttrs = $derived(
    player?.attrs != null && Object.keys(player!.attrs as object).length > 0,
  );

  // --- Radar geometry over the sport's outfield attributes ---
  const RAD = { cx: 112, cy: 100, r: 78 };
  function pt(i: number, frac: number, n: number): [number, number] {
    const ang = -Math.PI / 2 + (i * 2 * Math.PI) / n;
    return [RAD.cx + RAD.r * frac * Math.cos(ang), RAD.cy + RAD.r * frac * Math.sin(ang)];
  }
  const radar = $derived.by(() => {
    const a = player?.attrs as Record<string, number> | null | undefined;
    if (!a) return null;
    const defs = theme.attributes;
    const n = defs.length;
    const vals = defs.map((d) => a[d.key] ?? 0);
    const poly = vals.map((v, i) => pt(i, Math.max(v, 1) / 99, n).join(",")).join(" ");
    const rings = [0.33, 0.66, 1].map((f) => defs.map((_, i) => pt(i, f, n).join(",")).join(" "));
    const axes = defs.map((_, i) => {
      const [x2, y2] = pt(i, 1, n);
      return { x1: RAD.cx, y1: RAD.cy, x2, y2 };
    });
    const labels = defs.map((d, i) => {
      const [x, y] = pt(i, 1.17, n);
      return { x, y, short: d.short };
    });
    return { poly, rings, axes, labels };
  });

  const attrGroups = $derived.by(() => {
    const a = player?.attrs as Record<string, number> | null | undefined;
    if (!a) return [];
    return theme.attrGroups.map((g) => ({
      name: g,
      items: theme.attributes
        .filter((d) => d.group === g)
        .map((d) => {
          const v = a[d.key] ?? 0;
          return { label: d.label, val: v, color: tierColor(v) };
        }),
    }));
  });

  const potLabel = $derived.by(() => {
    if (!hasOvr || !hasPot) return "";
    const gap = (player!.potential as number) - (player!.overall as number);
    return gap >= 10 ? "Very high" : gap >= 5 ? "High" : gap >= 1 ? "Moderate" : "Reached";
  });
  const devPct = $derived(
    hasOvr && hasPot
      ? Math.round(((player!.overall as number) / Math.max(player!.potential as number, 1)) * 100)
      : 0,
  );
</script>

{#if !player}
  <div style="padding:22px;display:flex;flex-direction:column;gap:16px;max-width:1180px">
    <div class="empty" style="padding:60px 40px">Select a player from the Squad screen.</div>
  </div>
{:else}
  <div style="padding:22px;display:flex;flex-direction:column;gap:16px;max-width:1180px">
    <!-- ===== Header card ===== -->
    <div style="display:flex;gap:20px;align-items:stretch;background:#14181e;border:1px solid #232a33;border-radius:16px;padding:18px 20px">
      <div style="width:112px;height:148px;flex:none;border-radius:12px;background:repeating-linear-gradient(135deg,#1a1f27,#1a1f27 8px,#171b22 8px,#171b22 16px);border:1px solid #2a323c;display:flex;align-items:flex-end;justify-content:center;padding-bottom:10px;position:relative">
        <span style="position:absolute;top:9px;left:9px;font-size:8.5px;color:#4a535e;font-family:var(--font-mono);letter-spacing:.08em">RENDER</span>
        <span style="font-family:var(--font-mono);font-size:30px;font-weight:600;color:#39424c">{num}</span>
      </div>

      <div style="flex:1;display:flex;flex-direction:column;justify-content:center;gap:11px">
        <div style="display:flex;align-items:center;gap:10px">
          <span style={posStyle}>{posText}</span>
          <span style="font-size:12px;color:#8b95a1;font-family:var(--font-mono)">{clubLine}</span>
        </div>
        <div style="font-size:31px;font-weight:800;letter-spacing:-.02em;line-height:1">{player.name ?? "—"}</div>
        <div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:2px">
          {#each facts as f}
            <div style="display:flex;flex-direction:column;gap:1px;background:#0f1319;border:1px solid #232a33;border-radius:8px;padding:6px 12px">
              <span style="font-size:9px;color:#616b77;font-family:var(--font-mono);text-transform:uppercase;letter-spacing:.05em">{f.l}</span>
              <span style="font-size:13px;font-weight:600;color:#d4dae1">{f.v}</span>
            </div>
          {/each}
        </div>
      </div>

      <div style="flex:none;display:flex;gap:14px;align-items:center;padding-left:6px">
        <div style="text-align:center">
          <div style="font-size:9.5px;color:#616b77;font-family:var(--font-mono);letter-spacing:.08em">CURRENT</div>
          <div style="font-size:42px;font-weight:800;line-height:1.05;color:{ovrColor}">{ovrVal}</div>
        </div>
        <div style="width:1px;height:46px;background:#2a323c"></div>
        <div style="text-align:center">
          <div style="font-size:9.5px;color:#616b77;font-family:var(--font-mono);letter-spacing:.08em">POTENTIAL</div>
          <div style="font-size:42px;font-weight:800;line-height:1.05;color:{potColor}">{potVal}</div>
        </div>
      </div>
    </div>

    <!-- ===== Grid ===== -->
    <div style="display:grid;grid-template-columns:1.35fr 1fr;gap:16px;align-items:start">
      <!-- LEFT: Attributes -->
      <section style="background:#14181e;border:1px solid #232a33;border-radius:14px;overflow:hidden">
        <div style="padding:13px 16px;border-bottom:1px solid #232a33;display:flex;align-items:center;justify-content:space-between">
          <span style="font-size:14px;font-weight:700">Attributes</span>
          <span style="font-size:11px;color:#616b77;font-family:var(--font-mono)">out of 99</span>
        </div>
        {#if hasAttrs && radar}
          <div style="display:flex;gap:6px;padding:16px 16px 18px">
            <div style="flex:none;width:224px;display:flex;align-items:center;justify-content:center">
              <svg viewBox="0 0 224 214" style="width:224px;height:214px">
                {#each radar.rings as ring}
                  <polygon points={ring} fill="none" stroke="#222932" stroke-width="1" />
                {/each}
                {#each radar.axes as ax}
                  <line x1={ax.x1} y1={ax.y1} x2={ax.x2} y2={ax.y2} stroke="#222932" stroke-width="1" />
                {/each}
                <polygon points={radar.poly} fill="var(--accent-soft)" stroke="var(--accent)" stroke-width="2" stroke-linejoin="round" />
                {#each radar.labels as lb}
                  <text x={lb.x} y={lb.y} fill="#7a828d" font-size="9" font-weight="600" text-anchor="middle" dominant-baseline="middle" font-family="var(--font-mono)">{lb.short}</text>
                {/each}
              </svg>
            </div>
            <div style="flex:1;display:flex;flex-direction:column;gap:15px;padding-top:2px">
              {#each attrGroups as g}
                <div>
                  <div style="font-size:9.5px;color:#616b77;font-family:var(--font-mono);text-transform:uppercase;letter-spacing:.08em;margin-bottom:8px">{g.name}</div>
                  <div style="display:flex;flex-direction:column;gap:8px">
                    {#each g.items as it}
                      <div style="display:flex;align-items:center;gap:10px">
                        <span style="width:78px;font-size:12px;color:#9aa4b0;flex:none">{it.label}</span>
                        <div style="flex:1;height:5px;background:#232a33;border-radius:3px;overflow:hidden">
                          <div style="height:100%;border-radius:3px;width:{it.val}%;background:{it.color}"></div>
                        </div>
                        <span style="width:22px;text-align:right;font-family:var(--font-mono);font-size:12.5px;font-weight:700;color:{it.color}">{it.val}</span>
                      </div>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="empty" style="padding:56px 24px">No attribute data for this player.</div>
        {/if}
      </section>

      <!-- RIGHT column -->
      <div style="display:flex;flex-direction:column;gap:16px">
        <!-- Development -->
        <section style="background:#14181e;border:1px solid #232a33;border-radius:14px;overflow:hidden">
          <div style="padding:12px 16px;border-bottom:1px solid #232a33"><span style="font-size:14px;font-weight:700">Development</span></div>
          {#if hasOvr && hasPot}
            <div style="padding:15px 16px">
              <div style="display:flex;align-items:flex-end;justify-content:space-between;margin-bottom:11px">
                <div>
                  <div style="font-size:9.5px;color:#616b77;font-family:var(--font-mono);letter-spacing:.06em">CURRENT</div>
                  <div style="font-size:24px;font-weight:800;color:{ovrColor}">{ovrVal}</div>
                </div>
                <span style="font-size:11px;font-weight:600;color:var(--accent);background:var(--accent-soft);padding:4px 10px;border-radius:20px;margin-bottom:4px">{potLabel} potential</span>
                <div style="text-align:right">
                  <div style="font-size:9.5px;color:#616b77;font-family:var(--font-mono);letter-spacing:.06em">POTENTIAL</div>
                  <div style="font-size:24px;font-weight:800;color:{potColor}">{potVal}</div>
                </div>
              </div>
              <div style="height:7px;background:#232a33;border-radius:4px;overflow:hidden">
                <div style="height:100%;border-radius:4px;width:{devPct}%;background:var(--accent)"></div>
              </div>
            </div>
          {:else}
            <div class="empty" style="padding:44px 16px">No rating yet.</div>
          {/if}
        </section>

        <!-- This Season -->
        <section style="background:#14181e;border:1px solid #232a33;border-radius:14px;overflow:hidden">
          <div style="padding:12px 16px;border-bottom:1px solid #232a33;display:flex;align-items:center;justify-content:space-between">
            <span style="font-size:14px;font-weight:700">This Season</span>
            <div style="text-align:right">
              <div style="font-size:9px;color:#616b77;font-family:var(--font-mono);letter-spacing:.06em">AVG RATING</div>
              <div style="font-size:20px;font-weight:800;color:#616b77;line-height:1.1">—</div>
            </div>
          </div>
          <div class="empty" style="padding:44px 16px">Per-match stats arrive with match tracking.</div>
        </section>

        <!-- Scout Report -->
        <section style="background:#14181e;border:1px solid #232a33;border-radius:14px;overflow:hidden">
          <div style="padding:12px 16px;border-bottom:1px solid #232a33"><span style="font-size:14px;font-weight:700">Scout Report</span></div>
          <div class="empty" style="padding:44px 16px">No scout report filed.</div>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--faint);
    line-height: 1.5;
  }
</style>
