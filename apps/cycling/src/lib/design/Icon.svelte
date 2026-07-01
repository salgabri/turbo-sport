<script lang="ts" module>
  // Icon set from the design comp's <symbol> sprite. Inner markup per name;
  // rendered inline so no global <defs>/<use> plumbing is needed.
  const STROKE =
    'fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"';
  export const ICONS: Record<string, string> = {
    home: `<g ${STROKE}><path d="M3 11l9-7 9 7"/><path d="M5 10v9h5v-6h4v6h5v-9"/></g>`,
    squad: `<g ${STROKE}><path d="M4 6h16"/><path d="M4 12h16"/><path d="M4 18h16"/></g>`,
    user: `<g ${STROKE}><circle cx="12" cy="8" r="3.6"/><path d="M4.5 20c0-4 3.5-6 7.5-6s7.5 2 7.5 6"/></g>`,
    trophy: `<g ${STROKE}><path d="M7 4h10v4a5 5 0 01-10 0z"/><path d="M7 6H4v1.5a3.5 3.5 0 003.5 3.5"/><path d="M17 6h3v1.5a3.5 3.5 0 01-3.5 3.5"/><path d="M9.5 20h5M12 13v3.5"/></g>`,
    transfer: `<g ${STROKE}><path d="M5 8h13l-3.2-3.2"/><path d="M19 16H6l3.2 3.2"/></g>`,
    search: `<g ${STROKE}><circle cx="11" cy="11" r="6.5"/><path d="M16 16l4 4"/></g>`,
    calendar: `<g ${STROKE}><rect x="4" y="5" width="16" height="15" rx="2.5"/><path d="M4 9.5h16M8.5 3.5v3M15.5 3.5v3"/></g>`,
    play: `<path fill="currentColor" d="M8 5.5v13l10.5-6.5z"/>`,
    pause: `<g fill="currentColor"><rect x="7" y="6" width="3.6" height="12" rx="1"/><rect x="13.4" y="6" width="3.6" height="12" rx="1"/></g>`,
    chart: `<g ${STROKE}><path d="M5 20V11M10 20V5M15 20V13M20 20V8"/></g>`,
    gear: `<g ${STROKE}><circle cx="12" cy="12" r="3"/><path d="M12 4v2M12 18v2M4 12h2M18 12h2M6.3 6.3l1.4 1.4M16.3 16.3l1.4 1.4M17.7 6.3l-1.4 1.4M7.7 16.3l-1.4 1.4"/></g>`,
    bell: `<g ${STROKE}><path d="M6 16h12l-1.6-3.2V9.5a4.4 4.4 0 00-8.8 0v3.3z"/><path d="M10 19a2 2 0 004 0"/></g>`,
    chevron: `<g fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6-6 6"/></g>`,
    up: `<g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 19V6M6 12l6-6 6 6"/></g>`,
    down: `<g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v13M6 12l6 6 6-6"/></g>`,
    injury: `<path fill="currentColor" d="M10 4h4v6h6v4h-6v6h-4v-6H4v-4h6z"/>`,
    filter: `<g ${STROKE}><path d="M4 5h16l-6.2 7.2V19l-3.6 1.8v-8.6z"/></g>`,
    check: `<g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12.5l4.2 4.2L19 7"/></g>`,
    x: `<g fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M6 6l12 12M18 6L6 18"/></g>`,
    clock: `<g ${STROKE}><circle cx="12" cy="12" r="8"/><path d="M12 8v4.5l3 1.8"/></g>`,
    dot: `<circle fill="currentColor" cx="12" cy="12" r="5"/>`,
    min: `<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M6 12h12"/></g>`,
    sq: `<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="6" y="6" width="12" height="12" rx="1.5"/></g>`,
  };
</script>

<script lang="ts">
  let { name, size = 18 }: { name: string; size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  style="flex:none;display:block"
  aria-hidden="true">
  {@html ICONS[name] ?? ""}
</svg>
