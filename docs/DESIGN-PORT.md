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
| 3 | Football backend: widen 4→8 attrs feeding the engine, OVR/position/value/nationality, save v3 | **DONE** |
| 4 | Football Tauri rich DTO + wire Squad/Profile to real attrs/OVR/value | **DONE** |
| 5 | Reskin basketball / cycling / tennis apps to the design (visual parity) | **DONE** |
| 6 | Backend attribute parity: **basketball done**; cycling/tennis already show their real attrs; extract `packages/ui` | in progress |

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

## Phase 5 — what landed (basketball / cycling / tennis reskins)

The football design foundation (`tokens.css`, `color.ts`, `Icon.svelte`, `AppShell.svelte`)
was **duplicated** into each app (per-app `src/lib/design/`) with a sport `theme.ts` and
adapted `dto.ts`:

- **basketball** — full six-screen shell, theme `#f2913d`, positions G/F/C, court match
  variant. Same command set as football; `StandingRow` adapted to basketball's win/loss
  shape (W/L/Pct/PF/PA/Diff). Attributes tab/OVR are empty-state until Phase 6 authors
  basketball attributes.
- **cycling** — event app (`roster()` / `run_tour()`): shell + Home, Roster (derived
  POS/OVR + CLM/SPR/TT/END heat), Race (GC race-standings variant). Nav Home/Roster/Race.
- **tennis** — event app (`draw()` / `run_tournament()`): shell + Home, Draw (seeded, SRV/
  RET/BAS/MEN heat), Bracket (rounds as columns, champion banner). Nav Home/Draw/Bracket.

Each verified independently: `svelte-check` 0 errors / 0 warnings and a clean `npm run build`.
Shared UI is duplicated for now; extracting to `packages/ui` is Phase 6 cleanup.

## Phase 6 — basketball backend attribute parity (done)

Same widen-and-migrate pattern as football Phase 3/4, applied to basketball:
`Baller` widened from four aggregate ratings to the six design attributes
(ins/out/pm/reb/def/ath), folded into the engine's offense/defense/three_point/
rebounding aggregates on the same 0..99 scale (tuning preserved); position-weighted
`overall()`; `PositionGroup`/`Rating`/`MarketValue`/`Nationality` authored on load;
`BasketballAbility` + sample regenerated with G/F/C positions and nationalities.
Save bumped **TSBB v1 → v2** with a frozen `GameSaveV1` mirror (core `SaveDataV2`,
four-rating column mapped onto the six attrs). A `view::team_squad`/`free_agents`
detailed DTO carries the attrs + position label; the Tauri `team_squad`/`market`
commands return it, and the (generic) reskinned Squad/Profile screens render the
radar + heat cells + OVR automatically. Verified: cargo test (basketball 12,
workspace green), clippy clean, app crate `cargo check` green, svelte-check 0/0.

Cycling and tennis already surface their real attributes (climbing/sprint/TT/
endurance; serve/return/baseline/mental) directly in the reskinned Roster/Draw
screens, so no backend widening was needed for them to "fit the design".

## Runtime verification (mock-IPC)

All four apps were driven in a dev preview with a stubbed Tauri `invoke` and
confirmed to render real-shaped data with no console errors:
- football & basketball — Squad Attributes tab (OVR + per-attr heat cells),
  position pills, footer aggregates, Profile attribute radar (8 / 6 axes),
  correct per-sport currency (£ / $).
- cycling — GC race standings populate after "Run the Tour" (gaps formatted).
- tennis — bracket populates after "Play Tournament" (champion banner, rounds,
  scores), accent `#ef5a5a`.

## Deliberately not done (documented follow-ups)

- **Extract `packages/ui`**: the design system is duplicated per app. Extraction
  to a shared workspace package is deferred (cross-project Vite/SvelteKit
  resolution is the risk; duplication builds reliably today).
- **Cycling/tennis OVR/position/value + persistence**: not needed for design
  parity (their real 4 attributes already show); a full management layer is a
  larger future effort.
- Self-hosted fonts (still `@import` from Google Fonts) — see below.

## Live 2D match (football)

The Match screen is now a real live experience, not a shell. The engine stays a
pure `(lineups, rng) -> result`; a new `football::playback` module reads two real
squads, runs that engine, and dresses the deterministic result into a
`MatchPlayback` DTO: a 4-3-3 of pitch dots drawn from each side's best XI, a
minute-ordered event feed with named scorers (weighted by shooting) plus a few
bookings, and the six stat totals. All non-engine flavour (scorer, cards,
corners/fouls) is drawn from a **separate** seeded stream keyed off the match
seed, so it never perturbs the goal/xG sequence — same seed, same match.

Tauri `next_match(team_id)` returns the managed club's next fixture (in-season) or
a friendly, seeded deterministically. The front-end `Match.svelte` replays it
against a client-side clock with play/pause and 1×/2×/5× speed: the pitch shows
the dots + a drifting ball, the scoreboard/score update as goals pass, the stat
bars ramp to their totals, and the feed fills minute-by-minute. Verified: football
27 tests (incl. playback determinism + goals-match-score), clippy clean, app crate
`cargo check` green, svelte-check 0/0, and a mock-IPC render (dots/stats/controls
present, clock + ball animating; real-time playback needs a foreground window as
the headless preview freezes background timers).

**Basketball live court** (done): the possession engine returns only a final score,
so `basketball::playback` synthesises a deterministic scoring timeline (baskets of
1/2/3 spread across 48 minutes, named scorers, 3-point rate scaling with outside
shooting) that sums exactly to the engine's result — again on a separate seeded
stream, engine untouched. It emits the **same `MatchPlayback` shape** as football,
so the shared `Match.svelte` replays it after one change: the pitch/court is chosen
by `theme.matchVariant`, and the scoreboard sums event `points` (a football goal is
1; a basket is 2/3). Basketball uses a 5-man court formation, `#f2913d` accent, and
basketball stat rows. Verified: basketball 14 tests (incl. scores-sum-to-final +
determinism), clippy clean, app crate `cargo check` green, svelte-check 0/0, court
render confirmed (5+5 dots, key/hoops, stats).

**Cycling stage + tennis tie (done):** the individual sports got their own live
views (event-shaped apps, so custom screens, not the pitch/court component).
- Cycling `playback`: runs one Mountain `simulate_stage` for the roster, ranks by
  time, gaps to the leader; `Stage.svelte` counts KM-to-go down over a climb-profile
  SVG while each rider's gap grows from 0 to its final value (play/pause + speed).
  Command `next_stage`.
- Tennis `playback`: runs `simulate_match` for the top two seeds, then synthesises
  per-set game scores + a per-game feed consistent with the sets won (separate
  seeded stream). `Match.svelte` reveals set boxes + the game feed as the clock
  advances. Command `featured_match`.
Verified: cycling 9 + tennis 11 tests (determinism + consistency), clippy clean,
both app crates `cargo check` green, svelte-check 0/0, and mock renders confirm the
climb profile + rider gaps and the tennis scoreboard/set-grid/feed.

**All four sports now have live 2D experiences.**

**Season-consistent live match (football + basketball):** when a season is running,
`next_match` seeds the watched game with the *exact* per-fixture stream the season
uses (`derive_seed(derive_seed(world_seed, [season_id, matchday]), [k])`) and
aggregates the lineup/roster identically to the season driver — so the scoreline you
watch is the one that gets recorded in the table when you advance past that matchday.
Tested with `watched_next_match_matches_the_season_result` on both sports. (Discovered
+ fixed a matching subtlety: basketball's `gather_rosters` counts retired players, so
the playback roster must too.)

Later: actually advancing the season *from* the match screen (play → record → next);
engine-driven ball/positions instead of procedural drift.

### Known follow-ups
- Fonts are pulled from Google Fonts via `@import` in `tokens.css`. Self-host
  woff2 so core play has **no network dependency** (CLAUDE.md target) — polish.
- `Match` and `Profile` radar/attrs are empty-state shells until phase 3/4.
