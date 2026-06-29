# Architecture & Design Rationale

This document captures the reasoning behind the stack and structure. `CLAUDE.md` is the
operational summary; this is the "why." Where the two ever disagree, `CLAUDE.md` wins
for what to *do* — this file explains the thinking so decisions can be revisited
intelligently.

## Performance thesis

Goal: simulate a world an order of magnitude larger and more detailed than Football
Manager (100k+ players/staff, many concurrent competitions, deep per-entity detail)
while running dramatically faster.

The naive assumption is that the win comes from "keeping everything in RAM." It does
not. Virtually every game in this genre, FM included, loads its active world into
memory — you cannot hit disk per attribute access in a real-time-ish sim. Being in
memory is table stakes, not an edge.

The actual edge is three things **together**:

1. **Data-oriented memory layout.** Entities as packed component arrays, not OOP
   objects. Iterating 100k players' attributes becomes a linear scan over contiguous
   memory — cache-friendly and branch-predictable. OOP objects with vtables scattered
   across the heap destroy cache locality at this scale.
2. **Full multicore parallelism.** Matches within a matchday are independent —
   embarrassingly parallel. Saturating every core with `rayon` is the multiplier.
3. **No database in the hot path.** The in-memory world is the source of truth. Disk is
   for load-at-start and save-to-disk only.

## Why this beats Football Manager specifically

A calibration note, with explicit honesty about certainty levels.

**Reasonably established about FM:** it loads the active game world into memory; its
"database size / leagues to load" setting is fundamentally a memory + CPU tradeoff
(more leagues loaded = more entities resident and simulated = slower, hence the
in-game warnings); it ships a large research database loaded at start and serializes
saves to disk. So FM follows a load-from-disk → simulate-in-RAM → serialize-to-save
cycle.

**Widely reported by players (and the key opening):** FM's match engine does **not**
scale well across many cores — single-thread performance matters more than core count,
and adding cores yields diminishing returns. That is the gap we exploit.

**Inference, not verified fact:** FM's exact internal memory layout, whether it uses
strict data-oriented design / ECS, and its save format. Treat any specific claim about
FM's *implementation* as educated reasoning from genre norms, not documentation.

**Conclusion:** FM already keeps the world in RAM (so that is not our advantage) and
plausibly has a largely single-threaded hot path over OOP-ish data (so *that* is our
advantage). If we merely replicate "load into RAM" with OOP objects and a
single-threaded loop, we *match* FM — we do not beat it. The edge is layout +
parallelism, not residency.

## Why Rust

C++-level performance with memory safety; fearless concurrency (essential for parallel
match simulation without data-race footguns); a strong ecosystem for this exact problem
(`bevy_ecs`, `rayon`, `rkyv`, `redb`). For a CPU-bound sim throwing every core at 100k
entities, a garbage-collected language is the wrong tool.

## ECS / data-oriented design

Use `bevy_ecs` standalone — the ECS crate without committing to the full Bevy renderer
(unless Bevy is later chosen for the UI). Chosen over `hecs` mainly for its mature
archetype storage and built-in parallel system scheduling, which we want for thousands
of concurrent matches. `hecs` remains a lighter fallback if dependency minimalism later
becomes a priority.

Components are plain data in contiguous arrays. Systems are functions that query and
transform them. Aging, finances, morale, injuries, AI manager decisions — all operate
generically over components and live in `sim-core`. The actual match/stage simulation
lives in the sport crates.

## Parallelism

`rayon`'s work-stealing `par_iter` over a matchday's fixtures. Each match is
self-contained, so this is near-linear scaling across cores with minimal boilerplate.
This is the single biggest lever over FM — and the reason the standalone-desktop target
matters: on a server you would ration cores across many users; on the desktop, one
player's simulation gets all of them.

## The "shared greatness" framework

The hardest design problem here, and the one most likely to be done wrong if rushed.

Structure: a Cargo workspace where `sim-core` is sport-agnostic and defines **traits**
each sport must implement. Shared systems (lifecycle, economy, transfer market, AI)
live in core and operate over generic components. Sport-specific simulation lives in
sport crates implementing the traits.

Illustrative starting shapes — a hypothesis to refine against real code, **not** a
contract to lock in:

```rust
/// Simulates a single fixture/event and produces a result.
pub trait MatchEngine {
    type Lineup;
    type Result;
    fn simulate(&self, home: &Self::Lineup, away: &Self::Lineup, ctx: &SimContext)
        -> Self::Result;
}

/// Declares the attribute/component schema for a sport's entities.
pub trait EntitySchema {
    fn register_components(world: &mut World);
    // football: pace, passing, tackling, …; cycling: climbing, sprinting, time-trial, …
}

/// Describes how a competition is structured and progresses.
pub trait CompetitionFormat {
    fn generate_fixtures(&self, participants: &[EntityId]) -> Schedule;
    fn standings(&self, results: &[MatchResult]) -> Table;
}
```

**Critical methodology:** do *not* finalize these traits up front. Build football
completely first. Then implement a second sport (cycling) and let the *actual* friction
— what genuinely duplicates vs. what cycling refuses to express in a football-shaped
trait — reveal where the boundaries belong. Designing the framework before two sports
exist reliably produces the wrong abstraction.

The central tension to hold throughout: rich enough to share real logic, generic enough
that cycling and football are never forced into the same mold.

## Persistence

The in-memory ECS world is the source of truth during play. The world *is* the game
state.

Saves: serialize with `rkyv` (zero-copy deserialization — memory-map a save and access
it without parsing; dramatically faster for large saves) or `bincode` (simpler, slower
for big saves).

**Save versioning is non-negotiable from the first commit.** A player's multi-season
save must survive future patches; schema migration of save files is a recurring,
unavoidable desktop-game problem. Put a version header in the format on day one and
design with migration in mind from the start.

If historical data (e.g. full match history) ever outgrows RAM, add `redb` — a
pure-Rust embedded KV store — to keep it on disk while staying in-process and out of
the hot path. Explicitly **not** SQLite: SQLite is the right tool for a networked
webapp, the wrong tool for a desktop sim's hot loop.

## Standalone desktop context

Why this target reinforces every choice above:

- You own the whole machine: all cores to one player's sim (justifies the `rayon`
  strategy), all RAM for the resident world (100k entities is a few hundred MB —
  trivial on a 16–32 GB PC).
- The only serialization boundaries are save-to-disk (occasional) and, under Tauri,
  the Rust↔webview IPC. No per-interaction network or database round-trips.
- Distribution and updates become your responsibility: Tauri produces platform
  installers, has a built-in auto-updater, and yields small (~10 MB, OS-webview)
  binaries; pure Bevy bundles larger and you handle packaging yourself (e.g.
  `cargo-bundle`). Windows is the priority market for this genre.

## UI — the open decision

Deliberately unresolved. The sim core is built UI-agnostic so this can wait.

**Option A — Tauri + Svelte.** A web frontend running locally, with native filesystem
access, native menus, an auto-updater, and small binaries — "web tech, desktop
citizen." The web platform is unmatched for dense, virtualized tables and grids, which
*is* the primary experience here. Cost: the Rust↔JS IPC boundary — you must paginate,
virtualize, and summarize, and never ship 100k rows across it. For 2D/3D match views
inside the webview: PixiJS (2D) or Three.js (light 3D).

**Option B — all-Bevy.** Sim and match visualization in one engine, zero glue, direct
ECS access. Architecturally purest. Cost: building dense tabular UI in immediate-mode
tooling that is less mature than HTML for that specific job.

Given that visualization is explicitly secondary and the data experience primary,
Tauri's table strength is a strong argument; Bevy's unification (the match view living
right next to the data it animates, with no IPC) is the counter-argument. Decide before
building `app/`, or keep the core UI-agnostic until the decision is forced.
