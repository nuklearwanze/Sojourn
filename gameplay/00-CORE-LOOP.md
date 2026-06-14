# Sojourn — Core Gameplay Loop & Playability Plan

**Status:** design (the playability programme, FA-11 … FA-20)
**Audience:** developers driving the next phase with `/speckit.*` in Claude Code
**Scope:** turn the implemented-but-inert simulation (FA-01 … FA-10) into a *playable game* as ESA, one testable vertical slice at a time.

This is the keystone document. It defines (1) the core loop the player actually performs, (2) the **orchestration architecture** that connects the decoupled slices into that loop, (3) the **ESA bootstrap** start-state, and (4) the **sequenced, independently-testable increments** (GP-00 … GP-09) that each get their own design doc and `/speckit` command set. Read this first; then the per-increment docs (`01-…` … `10-…`) and `UI-UX-CONVENTIONS.md`.

---

## 1. Where we are, and what "playable" is missing

The workspace already contains the full simulation as decoupled, headlessly-tested crates: `sojourn-core` (deterministic kernel: fixed-step scheduler, seeded streams, event log, save/migrate, state hash, `SimModule` contract, `Command`/`ModulePayload` routing), and the nine gameplay slices `sojourn-astro / -world / -research / -vehicle / -economy / -base / -crew / -polity`, plus the presentation pair `sojourn-ui` (headless view-model + `UiHost`) and `sojourn-ui-desktop` (egui renderer). Each slice exposes a rich typed command surface — e.g. `EconomyCommand::{ApplyAppropriation, BuyLaunch, Transact, DispatchShipment}`, `ResearchCommand::{SetAllocation, StartProgram, Hire, Train}`, `VehicleCommand::{ComposeDesign, RegisterProduction}`, `AstroCommand::{SpawnCraft, CreateNode, CommitPlan}`, `BaseCommand::{FoundBase, AddModule}`, `CrewCommand::{OccupyAsset, AssignCrew, Shelter}`, `WorldCommand::{Observe, Prospect}`, `PolityCommand::{InitWorld, RecordAchievement, CollectEvidence, SelectGrandGoal}`.

Three things stand between that and a game:

1. **No coherent start-state.** `UiHost::new_game()` builds a core with every module loaded but *empty*: every Understanding Level is 0, no faction is seeded (factions appear only after `PolityCommand::InitWorld`), there is no funding, no fleet, no bases, no belief-state observations. The game boots into a blank slate, not into ESA in January 2026.

2. **No cross-system causality.** The slices are deliberately decoupled: the economy doesn't know that buying a launch should make a craft exist in astro; astro doesn't know a craft must be paid for. A single player action ("launch the Castor tug to LEO") is *inherently* a fan-out across `EconomyCommand::BuyLaunch` (debit funds + book mass-to-orbit) **and** `AstroCommand::SpawnCraft` (create the craft) **and**, later, `CrewCommand::OccupyAsset`. Nothing above the slices composes that fan-out, validates it as one decision, and previews it as one consequence.

3. **The screens are read-only placeholders.** The twelve screens render view-models but the gameplay verbs — allocate, design, buy, plan, commit, found, assign — are mostly not wired, there is no plan→preview→commit flow against real commands, and the depth each system carries needs *many subscreens* the current shell does not have.

The plan below fixes all three, in a dependency order where **every increment is independently playable and testable**.

---

## 2. The core gameplay loop

The player is **ESA's programme director**. The game is the Aurora-style "increment time until something matters, then decide" rhythm, layered over three nested horizons (already named in the README):

- **Operational (days–weeks):** watch missions, answer interrupts (anomalies, solar storms, budget news, discoveries), approve manoeuvres at nodes, allocate crew-time and DSN passes, manage launch manifests.
- **Programme (months–years):** run research programmes through TRL gates, design and produce vehicles, bid contracts, build infrastructure, plan transfer-window campaigns. *Mars windows every ~26 months are the heartbeat.*
- **Strategic (years–decades):** set the research-portfolio strategy, make architecture bets, choose sites, pursue settlement and the Grand Goal.

### 2.1 The session beat

One "beat" of play, repeated:

```
        ┌─────────────────────────────────────────────────────────┐
        │  1. OBSERVE   paused at an interrupt; read the event      │
        │               queue, the map, the fleet, the ledgers      │
        │                          ↓                                │
        │  2. DECIDE    across horizons: allocate budget & research,│
        │               design/produce vehicles, buy launches,      │
        │               plan transfers, survey targets, found/grow  │
        │               bases, assign & care for crew, set policy    │
        │                          ↓                                │
        │  3. COMMIT    irreversible actions are GATED: the core     │
        │               computes the consequence preview; the player │
        │               confirms (plan → preview → commit)           │
        │                          ↓                                │
        │  4. ADVANCE   choose time-warp; the deterministic core runs│
        │               until the next interrupt fires               │
        │                          ↓                                │
        │  5. CONSEQUENCE events resolve, state updates, prestige/   │
        │               mood/budget shift, milestones get claimed →  │
        └───────────────────────────── loop ───────────────────────┘
```

Steps 1, 4 and 5 already exist in the kernel (event store, interrupt-and-pause, time-warp, watches). Steps 2 and 3 are what this programme builds, screen by screen, on top of the **intent layer** (§3).

### 2.2 The mission thread — the loop's spine

What turns a sandbox into a game is that **a mission threads every system**:

```
fund (economy) → research enables (research/tech) → design vehicle (vehicle)
   → produce (vehicle/economy) → buy launch (economy) → craft exists (astro)
   → plan transfer (astro) → survey / deliver (world / base) → operate crew (crew)
   → return science / claim a first (polity) → prestige → budget (economy)
```

The earliest increments build the **shortest end-to-end thread** (fund → buy a launch → a craft exists → fly it → observe a target), then later increments deepen each link. After GP-04 the player can already play a minimal but real game; GP-08 adds the win condition.

---

## 3. Orchestration architecture — the "director" layer

The decoupling is sacred (Constitution IV/V): slices must not learn about each other, the kernel must stay domain-free, and the UI must hold **no game logic and no authoritative state**. So the cross-system causality cannot live in a slice, in the kernel, or in `sojourn-ui`. It needs a new home **above the slices and below the renderer**.

### 3.1 New crate: `sojourn-game` (the intent/orchestration layer)

A new library crate, a peer of `sojourn-ui`, depending on `sojourn-core` + every slice. It is **stateless** (holds no authoritative state) and **headlessly testable** in `sojourn-harness`. Its three jobs:

1. **Start-state scenarios.** Build the ESA-default initial command script (§4) deterministically from sourced data. This is the "new game" without a configurator.

2. **Intents → command batches.** Translate one player **Intent** (e.g. `Intent::BuyLaunchAndSpawn { design, orbit_class, … }`) into an ordered, validated `Vec<Command>` against the slices (here: `EconomyCommand::BuyLaunch` then `AstroCommand::SpawnCraft`). Validation is read-only against current snapshots (affordability, prerequisites, gates).

3. **Composed previews.** Build the single **`Preview`** the UI's plan→preview→commit flow shows, by asking the core to compute each leg's consequence and composing them into one traceable consequence set (funds delta, mass-to-orbit delta, "a craft will exist", what becomes unrecoverable). The previews are *core-computed*, never invented here (Principle II/VIII).

`sojourn-ui`'s `UiHost` stops wiring slices ad-hoc and instead depends on `sojourn-game`: the renderer collects a draft Intent from the screen, asks `sojourn-game` for the Preview, shows the gate, and on confirm submits the batch. The renderer keeps **zero** game logic.

### 3.2 Durable cross-cutting state: a journalled `sojourn-mission` module

Some gameplay state belongs to no existing slice and must **persist deterministically** across saves and replays — most importantly the **Mission/Programme thread** object (which design, which craft, which crew, which science goal, what stage) and **standing orders**. Ephemeral things (which screen is open, a draft plan not yet committed) stay UI-local and out of the save (as today). But durable orchestration state must be journalled **core** state.

Therefore, when the first increment that needs durable threads arrives (GP-04, the fleet/missions), introduce **`sojourn-mission`** as a proper `SimModule` registered in the core like any slice: it owns Mission records, advances them deterministically, emits mission events into the event store, and is saved/migrated by the kernel. `sojourn-game` *composes* intents that create/advance missions; `sojourn-mission` *stores* them. This keeps determinism, replay and round-trip saves intact (Principle III) while still respecting decoupling (the mission module depends only on `sojourn-core` and consumes other slices' outputs as opaque composed values, exactly like the existing slices do).

### 3.3 The boundary in one picture

```
 sojourn-ui-desktop (egui renderer)         ← no game logic, no state
        │  draft Intent ↑        ↓ Preview / CommitOutcome
 sojourn-ui (UiHost, view-models)            ← presentation only
        │  Intent ↑              ↓ Vec<Command> + composed Preview
 sojourn-game (intents, ESA bootstrap)       ← orchestration, STATELESS, headless-testable
        │  Command batches ↓     ↑ read-only snapshots
 sojourn-core  ⟵ registers ⟶  slices  +  sojourn-mission (durable threads)
   (authoritative, deterministic, journalled, saved)
```

The audit that greps `sojourn-core` for UI/slice deps still passes; the new crates sit *above* the core's dependency tree.

---

## 4. The ESA bootstrap (default new game)

No configurator. `new_game()` gains an ESA-default path that runs the `sojourn-game` start-state script at tick 0, producing a coherent January-2026 ESA. Every starting number is **sourced data** in `data/` (Principle I/V), not hard-coded — GP-00 adds a `data/scenario/esa_2026.ron` (with `source` fields) the script reads. The start-state must establish, at minimum:

- **The world.** `PolityCommand::InitWorld` seeds the factions (ESA = player faction 0; the always-AI agencies + private archetypes as competitors), the astrobiology candidate priors (bridged from `data/world/astrobiology.ron`), site PP categories, and difficulty. This is the prerequisite that turns the political/mood/milestone/astrobiology systems on.
- **Funding.** ESA's funding profile + the first appropriation (`EconomyCommand::RegisterFunding` + `ApplyAppropriation`), opening balances for the six currencies, and the fiscal calendar (next budget vote).
- **Knowledge.** Plausible 2026 starting Understanding Levels and tech maturities (`ResearchCommand::InjectUnderstanding` / seeded baseline) — chemical propulsion mature, electric mid-TRL, NTP/NEP low, closed-ecology low, astrodynamics high — plus a small starting personnel roster (scientists/engineers/PMs/astronauts).
- **Assets.** The real January-2026 starting position: ISS participation (Columbus), a couple of operational satellites, **no off-Earth bases**. Existing craft spawned via `AstroCommand::SpawnCraft` at their real locations; the belief-state initialised (well-known bodies known, prospecting fields unknown).
- **Goals.** No Grand Goal selected yet (the player picks one in GP-08); the milestone ledger seeded with the firsts already claimed in reality (e.g. crewed lunar landing → historical), the rest open.

Acceptance: booting ESA shows non-zero, plausible numbers on every screen; advancing one year delivers the next appropriation; save→load→continue is bit-identical.

---

## 5. The increment sequence

Ten increments, each a **vertical slice** (a bit of orchestration + one system's depth + its interactive screen) that is independently playable and testable. Each has its own design doc and `/speckit` command set. The mapping to the repo's spec numbering continues the existing series (sim-core = FA-01 = `specs/002-…`; UI = FA-10 = `specs/011-…`):

| Increment | Design doc | Feature area | Spec dir | Playable capability added |
|---|---|---|---|---|
| **GP-00** Session & ESA bootstrap | `01-SESSION-AND-BOOTSTRAP.md` | FA-11 | `specs/012-session-bootstrap` | Boots ESA 2026; live shell (clock, currencies, run-state, event queue, interrupts); save/load; the `sojourn-game` layer + start-state data exist |
| **GP-01** Economy & budget spine | `02-ECONOMY-BUDGET.md` | FA-12 | `specs/013-economy-budget` | Appropriations arrive; the conserved ledger is live; buy a launch (debits funds + books mass-to-orbit); contracts/RFP board |
| **GP-02** Research & personnel | `03-RESEARCH-PERSONNEL.md` | FA-13 | `specs/014-research-personnel` | Allocate the research portfolio; start programmes; TRL gates & test campaigns; hire/train; ULs rise; breakthroughs/dead-ends surface |
| **GP-03** Vehicle designer | `04-VEHICLE-DESIGNER.md` | FA-14 | `specs/015-vehicle-designer` | Compose/edit/derive designs from researched components; derived performance + trace + realism flags; register production |
| **GP-04** Flight & fleet | `05-FLIGHT-AND-FLEET.md` | FA-15 | `specs/016-flight-fleet` | Launched designs become craft; plan transfers (porkchop→nodes→commit); low-thrust arcs; live fleet; interrupt-at-node; `sojourn-mission` introduced |
| **GP-05** World survey & belief-state | `06-WORLD-SURVEY.md` | FA-16 | `specs/017-world-survey` | Observe/prospect to refine belief-state; map resource/science layers; targets resolve grades & uncertainty |
| **GP-06** Bases & construction | `07-BASES-CONSTRUCTION.md` | FA-17 | `specs/018-bases-construction` | Found a base at a surveyed site; add modules; build queue via logistics; emergent power/closure/sustainability; embargo eval |
| **GP-07** Crew & life support | `08-CREW-LIFE-SUPPORT.md` | FA-18 | `specs/019-crew-life-support` | Crew assets; consumables/radiation/physiology/psych; maintenance/shelter/resupply; storm response; EDL eval; loss-of-crew |
| **GP-08** Politics, astrobiology & scoring | `09-POLITICS-ASTROBIO-SCORING.md` | FA-19 | `specs/020-politics-astrobiology` | Milestone race; mood→budget/valuation; policy/treaties & lobbying; PP contamination; staged evidence; AI factions; Grand Goal + composite score + horizon (the win condition) |
| **GP-09** Sojournal, onboarding & UX polish | `10-JOURNAL-ONBOARDING-POLISH.md` | FA-20 | `specs/021-journal-onboarding` | Source-cited encyclopedia deep-linked from every number; event-log filtering + pause-policy config; onboarding layers; Continue/New entry |

### 5.1 Dependency graph

```
GP-00 (session + bootstrap + sojourn-game)
  ├── GP-01 economy ──┐
  ├── GP-02 research ─┤
  │        │          │
  │        ▼          ▼
  │     GP-03 vehicle (needs research maturity + economy cost)
  │        │
  │        ▼
  └──►  GP-04 flight & fleet (needs a produced+launched design; introduces sojourn-mission)
           │
           ├── GP-05 world survey (fly probes to observe)
           │       │
           │       ▼
           ├── GP-06 bases (found at a surveyed site; needs economy logistics)
           │       │
           │       ▼
           └── GP-07 crew (crew the craft/bases from GP-04/06)
                   │
                   ▼
               GP-08 politics / astrobiology / scoring (consumes all of the above as facts)
                   │
                   ▼
               GP-09 journal / onboarding / polish (cross-cuts everything)
```

A minimal **end-to-end playable game** exists after **GP-04** (fund → research → design → launch → fly). **GP-05–07** add the reasons to fly and the difficulty of crewing. **GP-08** adds winning. **GP-09** makes it legible and learnable.

### 5.2 The contract every increment honours

- **Independently testable.** Each ends with a scripted headless scenario in `sojourn-harness` (the playable thread for that increment) plus determinism double-run + save round-trip, and a set of `sojourn-ui` view-model unit tests (no renderer). "Playable" = a human can perform the new verb in `sojourn-ui-desktop` and observe the consequence.
- **Decoupling preserved.** New causality lives in `sojourn-game` (stateless) or `sojourn-mission` (journalled module); slices and kernel stay as they are.
- **Plausibility & traceability.** Every new number is sourced data with a `source` field; every derived figure the UI shows drills to its inputs; previews are core-computed.
- **Determinism.** Any durable state goes through the journal; UI/session ephemera stay out of the save.
- **Scope discipline.** No combat, no aliens; discovered life is a science object (Principle IX).

---

## 6. How to use these documents

For each increment, in order:

1. Read its design doc (`0N-….md`) and the relevant section of `UI-UX-CONVENTIONS.md`.
2. Run the increment's `/speckit` set from `speckit/gameplay/gp-0N-….md`: `specify → clarify → checklist → plan → tasks → analyze → implement`, each on its own feature branch (`specs/0NN-…`).
3. Land the increment only when its harness scenario, determinism/round-trip tests, and view-model tests are green, and a human can perform the verb in the desktop app.

The constitution at `.specify/memory/constitution.md` remains authoritative; nothing here overrides it.
