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

  const hasAttrs = $derived(player?.attrs != null && Object.keys(player!.attrs as object).length > 0);
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
        {#if hasAttrs}
          <div class="empty" style="padding:48px 16px">Attribute detail rendering pending.</div>
        {:else}
          <div class="empty" style="padding:56px 24px">Attribute ratings arrive with the match-engine update.</div>
        {/if}
      </section>

      <!-- RIGHT column -->
      <div style="display:flex;flex-direction:column;gap:16px">
        <!-- Development -->
        <section style="background:#14181e;border:1px solid #232a33;border-radius:14px;overflow:hidden">
          <div style="padding:12px 16px;border-bottom:1px solid #232a33"><span style="font-size:14px;font-weight:700">Development</span></div>
          <div class="empty" style="padding:44px 16px">Growth tracking arrives with the match-engine update.</div>
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
          <div class="empty" style="padding:44px 16px">No match data yet.</div>
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
