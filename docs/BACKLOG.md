# Turbo Sport — genre gap analysis & prioritized backlog

Requirements collected from the sport-management-sim genre (Football Manager, Out of the
Park Baseball, Franchise/Eastside Hockey Manager, Pro Cycling Manager, Tennis Manager,
Motorsport/F1 Manager) and cross-checked against our current build. Items we **already have**
are excluded (see "Already built" at the bottom). Each item: **what** (scope for us) · **why**
(genre expectation) · **status** · **effort** (S/M/L/XL) · **fit** (how it sits in our locked
ECS / deterministic / versioned-save / sim-core-vs-sport architecture).

> **Strategic lens.** Our differentiator is **scale + speed + determinism**, not feature
> parity. So two things get weighted up regardless of tier: (a) proving the 100k-entity / fast
> world thesis, and (b) systems whose value *compounds* with world size (scouting a huge player
> pool, a deep transfer market, records across decades). Feature-for-feature we will never
> out-FM FM; we win by making a bigger, faster world feel alive.

---

## P0 — the core management loop (what makes it feel like a real sim)

These are the biggest gaps between "four sport demos" and "a management sim." Without most of
P0 there is no *game* to play between matches.

### 1. Player development & training  · status: **missing** · effort: **L**
- **What:** attributes actually change over time. A deterministic growth model moving `overall`
  toward `potential`, gated by age curve (rise → peak → decline), match minutes, and a training
  focus the manager sets (per-player attribute/role focus + a squad training intensity).
- **Why:** the genre's central progression loop (FM's CA/PA-toward-potential is its spine). Our
  `Rating{overall,potential}` exists but is frozen — the whole "raise a wonderkid" fantasy is
  absent.
- **Fit:** deterministic system in `sim-core` (age curve, minutes→XP) with the per-sport growth
  weighting supplied by the sport (same hook pattern as OVR). No new save fields beyond a small
  training-focus component. Enables Youth, Scouting payoff, and Staff later.

### 2. Tactics & in-match management (team sports)  · status: **stub** · effort: **XL**
- **What:** selectable formation + per-position **roles/duties**, a mentality dial and a handful
  of team instructions (tempo, press, line height, width), pre-match/half-time **team talks**,
  and live **subs / role changes** during the 2D match. Basketball gets its analogue (rotation,
  pace, defensive scheme).
- **Why:** tactics are football's soul and the main skill-expression surface; right now our
  formation is cosmetic (viz only) and the engine reads only aggregate lineup ratings.
- **Fit:** the engine already consumes a `Lineup`; extend it to weight by role/instruction and
  make the live `Match.svelte` send in-match commands. Sport-specific (football/basketball),
  deterministic (instructions are inputs, not RNG). Largest single item — split into
  formation+roles first, instructions+talks second, live changes third.

### 3. Scouting, player search & recruitment  · status: **missing** · effort: **L**
- **What:** (a) a **global player search** with rich filters + saved filters over the whole
  world (age/pos/nation/value/wage/contract/attributes); (b) a **scouting** layer with
  attribute **fog-of-war** — unscouted players show ranged/masked ratings that sharpen as scouts
  watch them; (c) shortlists + scout reports (star ratings, potential estimate, fit).
- **Why:** recruitment is the core loop *and* the purest expression of our "Excel over a huge
  world" thesis — searching 100k players is where scale becomes fun.
- **Fit:** search/filters are a `sim-core::view` read-layer feature (bounded, paginated —
  already the contract). Fog-of-war = a per-club `Knowledge` component + a view that returns
  ranges. Deterministic. **High strategic value** (compounds with world size).

### 4. Contract & transfer negotiation  · status: **partial** · effort: **L**
- **What:** replace instant free-agent signing with real negotiation: bids/counter-bids on fee
  and wage, contract length, and clauses (signing/loyalty bonus, release clause, sell-on,
  appearance/goal bonuses); AI clubs value players and haggle; agents take a cut; loan deals.
- **Why:** the transfer window is the genre's headline drama; we have only a one-shot AI signer.
- **Fit:** extend the economy + a small negotiation state machine (multi-day offers resolved on
  the clock — deterministic, no RNG needed). New optional save fields (`#[serde(default)]`).
  Depends on scouting/value being trustworthy.

### 5. Board, objectives, expectations & manager career  · status: **missing** · effort: **M/L**
- **What:** a board that sets season objectives (league finish, cup run, budget discipline),
  tracks confidence, and can sack you; a manager profile with reputation that gates job offers;
  hiring/firing across clubs.
- **Why:** gives the game *stakes and purpose* — without objectives, advancing time is aimless.
- **Fit:** `sim-core` resources (Board expectation, ManagerReputation) + deterministic
  evaluation each matchday/season. UI-agnostic (surfaced via view DTOs). Feeds the inbox.

### 6. Multiple & knockout competitions + fixture calendar  · status: **partial** · effort: **L**
- **What:** domestic **cups** (single-elim/two-leg) and a **continental** competition running
  concurrently with the league; internationals as a stretch; a real congested fixture calendar
  a club plays across all of them.
- **Why:** clubs play many competitions at once; we run one league per sport. Also unlocks
  squad-rotation tension (ties to fitness/training).
- **Fit:** `sim-core::competition` already has round-robin + brackets (tennis) + a pyramid —
  harvest a competition-scheduler that places multiple comps on one calendar. Deterministic.

### 7. Cycling & tennis season + persistence layer  · status: **missing** · effort: **L/XL**
- **What:** the individual sports are event apps with **no save, no season, no career** — bring
  them to team-sport parity: a calendar of races/tournaments, rider/player rosters with
  contracts/value/OVR that persist, save/load, and standings/rankings that carry over.
- **Why:** two of our four apps can't actually be *played* as a career yet.
- **Fit:** reuse the `sim-core` spine (contracts, economy, saves, view layer) already proving out
  in football/basketball; each needs a new versioned save (MAGIC+VERSION) + a season driver.
  Largest for tennis/cycling because it's net-new surface, not a reskin.

---

## P1 — depth that rounds out realism

### 8. Injuries & suspensions  · status: **partial** · effort: **M**
- **What:** injury *types/severity/recovery time* (we only have a boolean flag + fitness) with a
  medical/physio influence on risk & recovery; cards accumulating into suspensions.
- **Why:** injuries drive squad-depth decisions; a flag isn't enough. **Fit:** extend
  `Condition` + a deterministic injury roll seeded per player-day; sport tracks cards.

### 9. Finances depth  · status: **partial** · effort: **M/L**
- **What:** revenue streams (sponsors, TV/prize money, gate/attendance tied to stadium &
  form), split **transfer vs wage** budgets, and a spending constraint (FFP / salary cap /
  luxury tax — sport-configurable). **Why:** turns money from one number into a system with
  trade-offs. **Fit:** economy module + board sets budgets; deterministic.

### 10. Staff  · status: **missing** · effort: **M/L**
- **What:** assistant manager, coaches (quality gates training), scouts (gate scouting range),
  physios (gate injury/recovery), Director of Football; hireable, with delegation of tasks.
- **Why:** staff are how you *scale yourself* — and they gate items 1/3/8. **Fit:** staff are
  ECS people too (reuse the person model) with a role component; deterministic effects.

### 11. Youth academy & intake + loans  · status: **partial** · effort: **M**
- **What:** annual youth **intake** with a quality/"golden generation" range driven by academy
  rating; loan-out to develop youngsters. **Why:** the long-term club-building fantasy.
  **Fit:** football already regenerates youth — add academy quality + intake-day event + a loan
  contract variant. Basketball/others get a draft instead (see 12).

### 12. Drafts (basketball) & history/records  · status: **missing** · effort: **M**
- **What:** a rookie **draft** with a prospect class for the league sports that use one
  (basketball); and a **history/records** layer — past champions, season-by-season tables,
  player career stats, awards, a hall of fame. **Why:** dynasty/franchise depth (OOTP's core
  appeal) — the reward for playing many seasons. **Fit:** history is *bounded, cold* data →
  the deferred `redb` on-disk store is the right home (keeps RAM/hot-loop clean); draft is a
  deterministic pre-season event.

### 13. Real inbox / news / notifications  · status: **stub** · effort: **M**
- **What:** an event-driven inbox (results, injuries, transfer news, board messages, contract
  expiries, milestones) with unread/filter and a "continue until something needs you" flow.
- **Why:** the genre's connective tissue; ours is a placeholder. **Fit:** `sim-core::Inbox`
  resource (was scoped, deferred) populated deterministically off the clock; view fn.

### 14. Season stats, player comparison & data hub  · status: **missing** · effort: **M**
- **What:** per-player season tallies (apps, goals/assists, avg rating; PPG/RPG/APG; stage wins;
  titles) surfaced on Profile/Squad, a **compare two players** view, and league leaders.
- **Why:** the "data-dense" identity we're chasing; also the payoff of the match engine.
  **Fit:** a `SeasonTally` component (previously scoped, deferred) updated post-match; view fns.

### 15. Individual-sport in-event tactics  · status: **missing** · effort: **L (per sport)**
- **What:** cycling — team orders, leadouts, breakaway/GC roles, feed the stage playback;
  tennis — surface specialization + fatigue/scheduling + ATP-style ranking points; motorsport
  (if added) — pit/tyre strategy + car R&D. **Why:** management *meaning* for individual
  sports. **Fit:** extend each sport's engine inputs; deterministic.

### 16. Save management  · status: **partial** · effort: **S/M**
- **What:** multiple named saves, autosave, a save browser, save metadata (club/date/size).
  **Why:** table-stakes UX; we have raw save/load only. **Fit:** frontend + a saves directory
  convention; format already versioned.

---

## P2 — proof, polish & differentiation

### 17. Scale & speed proof (THE thesis)  · status: **missing** · effort: **M/L** · *strategically P0*
- **What:** (a) generate & simulate a **100k+ entity** world and benchmark season-advance speed
  vs the genre's known slowness; (b) a **virtualized** data grid so squad/search/finance tables
  render 100k rows over the view DTOs at frame budget. **Why:** this is the entire product
  premise and it is currently **unproven** — everything above is more valuable once this holds.
  **Fit:** virtualization is a UI concern over the existing bounded-DTO contract; the sim is
  already ECS+rayon+deterministic. Do a spike early even though it's "P2 polish" in feature
  terms — it de-risks the whole roadmap.

### 18. Editor & data content  · status: **partial** · effort: **M**
- **What:** grow the standalone DB editor into a full pre-game editor (edit any entity, create
  leagues/clubs), plus importable **real-name / data packs** and asset packs (kits/logos).
  **Why:** the genre lives on custom databases + community "real name fixes." **Fit:** editor
  already exists (Tauri+Svelte over the DB); extend it + a data-pack import format.

### 19. Onboarding & assistant delegation  · status: **missing** · effort: **M**
- **What:** first-run tutorial, difficulty/assistant delegation (let the assistant run
  training/opposition instructions/team talks), tooltips. **Why:** the genre is famously opaque;
  onboarding widens the audience. **Fit:** frontend + delegation flags routing to AI defaults.

### 20. Set pieces, opposition instructions, traits & mentoring  · status: **missing** · effort: **M**
- **What:** the tactical depth layer that sits *on top of* item 2 once it exists. **Why:**
  edge-gaining subsystems veterans expect. **Fit:** engine inputs; deterministic. Sequenced
  after 2.

### 21. Localization & accessibility  · status: **missing** · effort: **M**
- **What:** i18n framework, keyboard nav, scalable text, colorblind-safe palettes. **Why:**
  reach & table-stakes for a shipped product. **Fit:** frontend; the design system centralizes
  tokens already.

---

## P3 — online / live-service (respecting "no network dependency for core play")

Our locked constraint: **core play is fully offline**. Online is a *separate mode*, not a
dependency — so it sits last, but note it's where the genre's longevity/community lives.

- **22. Online multiplayer leagues** (multiple human managers, host/commissioner, deadline-based
  processing) · **XL**. Netcode + conflict resolution + a headless sim server; our determinism
  is a real asset here (all clients can reproduce the same world). Biggest single online lift.
- **23. Cloud / portable saves & cross-device** · **M**. Our versioned self-contained saves make
  this mostly a sync/storage problem.
- **24. Community & content sharing** (share saves/databases/tactics; a Workshop-style hub) ·
  **M**.
- **25. Live real-world data feeds** (track the real season, roster updates) · **L**. Licensing +
  an ingestion pipeline; likely post-launch.

---

## Recommended sequencing

1. **De-risk the thesis first (spike #17)** — a 100k world + virtualized grid + a speed
   benchmark. If this holds, it justifies everything; if it doesn't, it changes the roadmap.
2. **Make one season worth playing (football):** Training (#1) → Objectives/Board (#5) →
   Inbox/Stats (#13/#14) → Scouting+Search (#3) → Contracts/Negotiation (#4). That sequence
   turns "advance the clock" into a loop with progression, purpose, feedback and a market.
3. **Deepen the match:** Tactics (#2) then its depth layer (#20).
4. **Broaden competitions (#6)** and **bring cycling/tennis to career parity (#7)**.
5. **Round out realism:** Injuries (#8), Finances (#9), Staff (#10), Youth/Draft (#11/#12),
   History (#12) — many of these gate on 1/3/5 already being in.
6. **Polish & reach:** Save management (#16), Editor/data packs (#18), Onboarding (#19), i18n
   (#21).
7. **Online (P3)** as a distinct later track, leveraging determinism.

Do football fully through a tier before generalizing to the other sports (CLAUDE.md constraint
#4: extract shared systems from real duplication, not speculation).

---

## Already built (excluded from the backlog)

Deterministic ECS core (bevy_ecs + rayon, per-match seeded RNG); calendar/lifecycle (aging,
contracts, morale, fitness, injury flag, retirement, free agents, football youth regen); basic
economy (balance/income/payroll) + a simple free-agent transfer AI; per-sport attributes +
OVR/potential/position/nationality/market value that feed the engines; competitions (football
league + promotion/relegation pyramid + multi-season, basketball league, cycling GC, tennis
bracket); deterministic match engines + **live 2D match experiences for all four sports**
(season-consistent for the team sports); versioned bincode saves with a real migration seam;
a standalone database editor; a summarized view/read layer; a full reskinned data-dense UI
(Home / Squad / Profile+radar / Table / Transfers / Match).

---

*Sources: Football Manager 2024/26 manual & feature pages, FM wiki, Passion4FM, FMScout; OOTP
27 manual/wiki & Baseball Prospectus; Franchise & Eastside Hockey Manager; Pro Cycling Manager
2024, Tennis Manager 2024, F1/Motorsport Manager guides; genre multiplayer/editor/community
docs. Full URL list in the research task output.*
