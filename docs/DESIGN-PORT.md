# Design port — Claude Design → Turbo Sport apps

Tracks the multi-phase port of the "Claude Design" handoff (HTML/CSS/JS design
comps for Football / Basketball / Cycling / Motorsport) onto the real
Tauri + Svelte apps, extending the backend so the rich screens have real data.

Source design comps live outside the repo (handoff bundle:
`Turbo sports-handoff.zip` → `turbo-sports/project/*.dc.html`). They are
prototypes; we recreate the *visual output* in Svelte, not their structure.

## Locked scope decisions

- **Go full**: extend the deterministic ECS backend to actually produce what the
  design shows (player attributes, OVR/POT, ratings, positions, nationality,
  market value, season stats, form/history, inbox, live-match events). Attributes
  **feed the engines** (not cosmetic) — a stronger card genuinely wins more.
- **All four apps**: football, basketball, cycling, tennis.
- **Motorsport → Tennis**: the design's 4th comp is Motorsport but the repo ships
  Tennis. We keep Tennis and adapt the motorsport *information architecture* onto
  it (points/ranking standings, 6-attr card, bracket variant). No 5th app.
- **Shared design system**: extract to a shared location — but only **after**
  football proves the components (CLAUDE.md: extract from real duplication). Phase
  1 builds them locally under `apps/football/src/lib/`; a later phase lifts the
  proven set to `packages/ui` and the other apps consume it.

## Design system (source of truth: the .dc.html inline styles)

- Tokens: bg `#0b0d10`, main `#0f1216`, panels `#14181e`, borders `#232a33`;
  fonts **Archivo** (display) + **IBM Plex Mono** (numbers/labels); per-sport
  `--accent` (football `#4fd08a`, basketball `#f2913d`/`#5ec98a`*, cycling
  `#f2c14e`, motorsport→tennis `#ef5a5a`). *(basketball accent TBD on port.)*
- Colour ramps (in `color.ts`): `tierColor` (OVR/attr), `fitColor`, `moraleWord`,
  `TONE`, per-position group colours, `formChip`.
- Six screens on one themed shell (title bar · sidebar · topbar · toast):
  Home / Squad / Profile / Table / Transfers / Match. Match has pitch/court/
  climb/track variants; Table has league/race variants.

## Phases

| # | Phase | Status |
|---|-------|--------|
| 1 | Shared design system + football reskin on current data | **DONE (football)** |
| 2 | sim-core data model + save 2→3 (Nationality, PositionGroup, MarketValue, Rating; extend DTOs) | **DONE** |
| 3 | Football backend: widen 4→8 attrs, engine reads them, capture timeline + form + history | todo |
| 4 | Football Tauri DTOs + all 6 screens fully live | todo |
| 5 | Basketball to parity | todo |
| 6 | Cycling + Tennis management/persistence layer (they have none today) + extract `packages/ui` | todo |

## Phase 1 — what landed (football)

Foundation in `apps/football/src/lib/design/`:
`tokens.css`, `color.ts`, `theme.ts` (FOOTBALL SportTheme), `dto.ts` (real +
optional future fields), `Icon.svelte` (design icon set), `AppShell.svelte`
(title bar / sidebar / nav / topbar / toast, `--accent` themed, optional
`actions` snippet for Save/Load/Start-season/Transfer-window).

Screens in `apps/football/src/lib/screens/`: `Home`, `Squad`, `Profile`,
`Table`, `Transfers`, `Match`. Wired in `routes/+page.svelte` to the **existing**
Tauri commands (`clubs`, `team_squad`, `market`, `standings`, `current_date`,
`season_active`, `start_season`, `advance`, `transfer_window`, `save_game`,
`load_game`) — no backend change yet.

Data-gap handling: screens render the **full designed shape** using optional DTO
fields, and show a tasteful mono/`--faint` empty state (inside the real panel
chrome) wherever the backend does not yet produce the field (attributes, radar,
OVR/POT, ratings, inbox, fixtures, live match). These fill in during phases 2–4.

Verified: `svelte-check` 0 errors / 0 warnings; `npm run build` succeeds; all six
screens mount with no runtime errors (checked via in-page nav drive). Full
Tauri-runtime data path is unchanged from the previous working app, so real data
flows when run under `npm run tauri dev`.

## Phase 2 — what landed (sim-core)

New sport-neutral components: `Nationality`, `PositionGroup(u8)` (opaque — the sport/UI
name the meanings), `Rating{overall,potential}`, `MarketValue` + a deterministic
`value_from(overall, age)`. `PlayerView` gained nationality / position_group / overall /
potential / market_value; `ClubView` gained avg_overall / best_overall / squad_value.

Save bumped **FORMAT_VERSION 2 → 3**. Because bincode is positional (not self-describing),
the migration decodes old bytes with a **frozen `persistence::legacy` mirror**
(`SaveDataV2`/`EntitySaveV2`) then converts forward — `#[serde(default)]` does NOT work for
bincode. `read_save` now dispatches by version. `SaveDataV2` is re-exported so a sport's own
save (which embeds `SaveData`) can compose the same migration. Tested: v3 round-trip + a
hand-built v2 payload migrating forward. Whole workspace `cargo test` green.

Deferred out of Phase 2 to keep it bounded (do in a later slice): `SeasonTally` and the
`Inbox` resource (they belong with form/history/inbox, Phase 3+).

> ⚠️ Phase 3 must-fix: football's `GameSave` still declares `VERSION = 2` but now embeds a v3
> `SaveData`. Old football saves won't load until football bumps to v3 with a `GameSaveV2`
> mirror (`core: sim_core::SaveDataV2`) — do this in the same bump that widens `Footballer`
> 4→8, so football migrates once.

### Known follow-ups
- Fonts are pulled from Google Fonts via `@import` in `tokens.css`. Self-host
  woff2 so core play has **no network dependency** (CLAUDE.md target) — polish.
- `Match` and `Profile` radar/attrs are empty-state shells until phase 3/4.
