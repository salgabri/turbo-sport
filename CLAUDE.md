# CLAUDE.md

Operational guidance for Claude Code. Read fully before acting. The "why" behind
every decision here lives in `docs/ARCHITECTURE.md` — read that when working on the
relevant subsystem.

## What this is

`[name TBD]` — a sport-management simulation in the vein of Football Manager. The
defining thesis: simulate a far larger, more detailed world (100k+ players/staff,
many concurrent competitions, deep per-entity detail) and do it **dramatically faster**
than existing games. The product is "Excel with moments" — a data-dense management UI
is the primary experience; 2D / simple-3D match (and cycling-stage, etc.)
visualizations are secondary polish.

The performance edge is **not** "load data into RAM" — every game in this genre,
FM included, already does that. The edge is the combination of: data-oriented memory
layout + full multicore parallelism + no database in the simulation hot path.

## Locked decisions

- **Language:** Rust for the entire simulation core.
- **ECS:** `bevy_ecs` used standalone (the ECS crate only — not the full Bevy
  app/renderer, unless we later adopt Bevy for the UI). Archetype storage + parallel
  system scheduling.
- **Parallelism:** `rayon` for embarrassingly-parallel match simulation (matches
  within a matchday are independent of one another).
- **Persistence:** the in-memory ECS world is the source of truth during play. Saves
  serialized to disk via `rkyv` (zero-copy load) or `bincode`. The save format **must**
  carry a version header from the first commit — save migration across patches is a
  hard requirement, not a later concern.
- **Optional on-disk store:** `redb` (pure-Rust embedded KV) if/when historical data
  outgrows RAM. **Not** SQLite — stay in-process and out of the hot path.
- **Target:** standalone desktop app, Windows primary, cross-platform via the
  toolchain. No server, no network dependency for core play.

## Open decisions — do NOT resolve implicitly

- **UI layer:** undecided between
  - **(a) Tauri + Svelte** — best-in-class dense data tables; requires deliberately
    managing the Rust↔webview IPC boundary (never ship 100k rows across it —
    paginate / virtualize / summarize), and
  - **(b) all-Bevy** — sim and match view in one engine, no glue, but weaker tooling
    for dense tabular UI.
  Build the sim core UI-agnostic so this stays deferrable. If a UI decision becomes
  necessary, **surface it — do not pick silently.**

## Workspace layout (Cargo workspace)

```
sim-core/            sport-agnostic engine
  - ECS world, tick / calendar / time
  - entity lifecycle (aging, contracts, morale, injuries)
  - economy (finances, wages, transfer market)
  - persistence (versioned save/load)
  - traits every sport must implement (MatchEngine, EntitySchema, CompetitionFormat, …)
sports/
  football/          implements the sport-agnostic traits for football
  cycling/           (later) implements them for cycling
app/                 (later) desktop UI shell, once the UI decision is made
docs/
  ARCHITECTURE.md    full design rationale
```

Dependency direction: sport crates depend on `sim-core`; `sim-core` never depends on
a sport crate or on any UI crate.

## Hard constraints (do not violate)

1. **No OOP entity objects.** Do not model players/staff as heap objects with vtables.
   Use packed component arrays (ECS) for cache locality. This is the core of the
   performance thesis.
2. **No database in the hot loop.** State lives in memory. Disk is touched only for
   load-at-start and save.
3. **Sim core stays UI-agnostic.** No rendering or UI assumptions leak into `sim-core`
   or the sport crates.
4. **Design the framework from real duplication, not speculation.** Build football
   fully first. Extract shared abstractions only once a second sport reveals where they
   actually belong. Premature generalization here produces the wrong traits.
5. **Versioned saves from day one.**

## Build order (sequential — do not jump ahead)

1. Cargo workspace skeleton + `sim-core` crate.
2. `sim-core`: `bevy_ecs` world + tick / calendar / time system — a world that can
   advance simulated time (days → seasons).
3. Entity schema + lifecycle systems (aging, contracts) operating over components.
4. Economy basics (finances, wages).
5. `sports/football`: the match engine implementing the core traits, parallelized
   with `rayon`.
6. Persistence: versioned save/load via `rkyv` / `bincode`.
7. `sports/cycling`: second sport — use it to validate and refactor the trait
   boundaries.
8. `app/`: UI shell (after the UI decision).

## Start here, then stop

Set up the Cargo workspace and the `sim-core` crate with a `bevy_ecs` world and a
working tick/calendar system that can advance simulated time (days → seasons). Add a
small throwaway example that spawns N entities and advances the clock, to prove the
loop runs. **Do not** build the match engine, persistence, or any UI yet. Report back
before moving past step 2.

## Conventions

- Rust 2021+ edition, stable toolchain.
- Keep `cargo clippy` clean; treat warnings seriously.
- Keep `sim-core` dependency-light.
