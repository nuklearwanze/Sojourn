# Feature Specification: Simulation Core & Time (FA-01)

**Feature Branch**: `002-sim-core`
**Created**: 2026-06-12
**Status**: Draft
**Input**: User description: Build the simulation core for Sojourn — the deterministic, headless, pausable real-time engine that every other system plugs into: fixed-timestep advance with a single authoritative world state; seeded PRNG streams; event-driven time-warp with interrupt-and-pause; an in-order event/decision log sufficient for full replay; exact save/load round-trip; a headless API (step, query, decide, register interrupts) with no presentation logic; module boundaries and data-ownership rules for later slices; and a deterministic test harness with a double-run determinism check. No tech stack.

> **Position in the programme.** This is the child specification for feature area **FA-01** of the
> umbrella spec (`specs/001-sojourn-solar-4x/spec.md`). It refines umbrella requirements
> FR-SIM-001…010 and implements the cross-cutting guarantees FR-XCU-003/004/006/011 for the rest
> of the project. Authoritative sources: the umbrella spec (including its Clarifications),
> `.specify/memory/constitution.md` v1.0.0 (Principles II, III, IV, V),
> `design/00-OVERVIEW.md` §4–5 (loops, time model) and `design/04-SPACEFLIGHT.md` §8
> (implementer notes). Where this spec is silent, those documents govern.

---

## Scope Boundary

**This slice delivers** the simulation kernel: the clock, the stepping loop, seeded randomness,
the command (decision) pipeline, the event system and warp/interrupt scheduler, watch conditions,
the journal/replay machinery, save/load, the module-contract layer (boundaries, registration,
state ownership, deterministic ordering), the headless API, and the deterministic test harness.

**This slice does NOT deliver** any game system that plugs into the kernel: no astrodynamics
(FA-02), world content (FA-03), research (FA-05), economy (FA-06), politics/event *content*
(FA-09), or UI (FA-10). Where game behaviour is needed to exercise the kernel (tests,
benchmarks), synthetic stand-in modules are used. The core defines event *classes* and the pause
machinery; the game meaning of those events (what a "budget vote" is) arrives with later slices.
Score computation (FR-SIM-008) belongs to FA-09; this slice provides only the horizon clock,
end-of-run signal and run-mode states the scorer will consume.

## User Scenarios & Testing *(mandatory)*

Two actors use this slice directly: **the player** (time control, interrupts, saves — initially
via the test harness, later via FA-10) and **the integrator** (developers/test authors building
the other nine feature areas on the kernel's contracts). Player value is real but indirect until
FA-02/FA-03 land; integrator value is immediate.

### User Story 1 - Reproduce History Exactly (Priority: P1)

An integrator creates a world from seed S, steps it headlessly for years of simulated time while
submitting a scripted sequence of decisions, and records a state fingerprint and the event log.
Running the identical seed and script again — on the same platform and build — produces a
bit-identical state fingerprint and an identical event log. All randomness anywhere in the run
came from the seed.

**Why this priority**: Determinism is the constitutional foundation (Principle III) and the
reason this slice exists. Every other capability in this spec is meaningless if two identical
runs can diverge.

**Independent Test**: Double-run check in the harness: same seed + same command script, run
twice, assert identical state hash at every checkpoint and identical event logs. Inject a
deliberately unseeded perturbation in a synthetic module and assert the harness *catches* the
divergence.

**Acceptance Scenarios**:

1. **Given** seed S and command script C, **When** the core runs the script twice on the same platform and build, **Then** the state hashes at every declared checkpoint and the complete event logs are identical.
2. **Given** a synthetic module that requests randomness, **When** it draws from its core-provided stream, **Then** the values depend only on the seed and the draw sequence — and a test module that uses any other entropy source is detected by the double-run check.
3. **Given** two runs whose command scripts differ by one decision, **When** both complete, **Then** their histories diverge only from the tick at which the differing decision applied.
4. **Given** the same seed and script executed with different wall-clock pacing (run flat-out vs with arbitrary stalls), **Then** the simulated history is identical — wall-clock time never influences outcomes.

---

### User Story 2 - Fast-Forward Until Something Matters (Priority: P1)

The player sets time-warp to a high rate and the world rushes through quiet months; the instant
anything pause-worthy occurs — a manoeuvre node comes due, an anomaly fires, a budget vote opens,
a player-defined watch condition ("alert me when the depot drops below 20 t") becomes true — the
core halts *before* the event's consequences need a human, presents the cause, and waits. While
paused, the player can still query everything and submit decisions. Resuming continues exactly
where the world stopped.

**Why this priority**: Interrupt-and-pause is the game's heartbeat (design 00-OVERVIEW §4–5) and
the second half of the slice's reason to exist; it is what makes a century playable.

**Independent Test**: Headless: schedule synthetic events of each class at known times, register
watch conditions, run at varying warp rates, and assert every pause-enabled occurrence interrupts
at its exact simulated time with the world state not yet advanced past it; assert pause-disabled
classes log without halting.

**Acceptance Scenarios**:

1. **Given** warp at any rate and a pause-enabled event due at simulated time T, **When** the simulation reaches T, **Then** it pauses with the world at T, the event surfaced and none of its player-facing consequences applied without input.
2. **Given** several events landing at the same simulated instant, **When** the core pauses, **Then** all simultaneous events are presented, in the documented deterministic order, and none is lost.
3. **Given** a registered watch condition over queryable state, **When** the condition first becomes true, **Then** an interrupt fires at the tick of first truth, including when it becomes true mid-warp at the highest rate.
4. **Given** a paused world, **When** the player queries state and submits decisions, **Then** all queries and submissions work, decisions are timestamped to the paused tick, and resuming applies them deterministically.
5. **Given** an event class configured not to pause, **When** such an event fires, **Then** it is logged and queryable but warp continues uninterrupted.
6. **Given** warp rate changes (including pausing and resuming) at arbitrary moments, **Then** the resulting simulated history is unaffected by *when* the player changed rates (FR-CORE-204: warp is pure playback speed).

---

### User Story 3 - Leave and Come Back to the Identical World (Priority: P1)

The player saves — including mid-burn, mid-anomaly, with interrupts queued — quits, and later
loads. The restored world is exactly the world they left: same state, same pending events, same
random future. The save remembers which content-data version the run started with and continues
on it even if the installed game has newer data.

**Why this priority**: Exact persistence is a constitutional gate (FR-XCU-006) and the
load-bearing promise behind a 100-hour campaign; it must be proven at the kernel level before
any content exists.

**Independent Test**: Harness: run seed+script to arbitrary points (including fine-timestep
phases of a synthetic module), save, load, continue to the horizon; assert final hash equals an
unbroken reference run. Load a save whose pinned data version differs from installed data and
assert it runs on the pinned version.

**Acceptance Scenarios**:

1. **Given** a run paused at tick T, **When** it is saved, the process restarted, and the save loaded, **Then** the state hash at load equals the hash at save, and continuing to any later tick produces the same hash as an uninterrupted run.
2. **Given** a save taken while a synthetic fine-timestep activity is in progress, **When** loaded, **Then** the activity resumes mid-flight with no discontinuity.
3. **Given** a save created under content-data version D1 and an installation now carrying D2, **When** the save loads, **Then** the run continues on D1, the pinned version is visible to the player, and a missing D1 produces a clear, actionable error rather than silent substitution.
4. **Given** a save file from an older v1.x save *format*, **When** loaded, **Then** format migration produces an identical-behaviour world (structure migrates; pinned content values never change).

---

### User Story 4 - Survive a Crash, Replay a Run (Priority: P2)

The player's machine loses power mid-session at high warp. On relaunch, the game recovers the
run — by replaying seed plus the journaled decision/event record — to within moments of the
interruption, in ironman as well as save-anywhere mode. Separately, a developer takes a bug
report consisting of a seed and journal, replays it, and lands on the exact tick where the
problem occurred.

**Why this priority**: Crash recovery (umbrella FR-SIM-010) and journal replay are what make
determinism *useful* in the field; they depend on US1–US3 being in place.

**Independent Test**: Kill-test: terminate the core process abruptly at random points across 100
trials; assert recovery succeeds, recovered state hash matches a reference run at the recovery
tick, and lost progress is within the durability bound. Replay test: journal from any completed
run reproduces the final state hash.

**Acceptance Scenarios**:

1. **Given** a run in progress, **When** the process is killed without warning at an arbitrary moment, **Then** relaunch recovers the run to a tick within the declared durability bound, and the recovered state is identical to a reference run at that tick.
2. **Given** ironman mode, **When** a crash and recovery occur, **Then** the player can neither lose the campaign nor obtain an earlier state than the durability bound allows (no free reroll).
3. **Given** a complete journal and its seed, **When** replayed headlessly, **Then** the final state hash and full event log match the original run.
4. **Given** a journal whose tail was corrupted by the crash, **When** recovery runs, **Then** the intact prefix is recovered and the player is told precisely what was lost.

---

### User Story 5 - Build a Game System on the Kernel (Priority: P2)

An integrator building a later slice (say, research) registers a module with the core: it
declares the state slice it owns, subscribes to the events and read-only views it needs from
other modules, receives its own seeded random sub-stream, and gets stepped in a deterministic
order. It cannot write another module's state, cannot bypass the event system, and anything it
does is captured by the journal. The contracts are documented well enough that FA-02…FA-09 teams
can build against them without reading kernel internals.

**Why this priority**: The module boundary and data-ownership rules are this slice's deliverable
*to the programme*; every later feature area consumes them. They must exist before any second
slice starts.

**Independent Test**: Build two synthetic modules with a declared dependency; assert
deterministic update ordering, single-writer enforcement (an attempted foreign write fails
loudly), event-mediated interaction, per-module stream isolation (adding a draw in module A does
not shift module B's stream), and that the contract documentation covers every public surface.

**Acceptance Scenarios**:

1. **Given** two registered modules where B declares a dependency on A's published view, **When** the core steps a tick, **Then** A and B execute in the documented deterministic order, identical on every run.
2. **Given** a module attempting to mutate state outside its declared ownership, **Then** the attempt is rejected as a defect (loud failure in development builds), never silently allowed.
3. **Given** modules A and B drawing randomness, **When** A's draw count changes, **Then** B's drawn values are unaffected (independent sub-streams).
4. **Given** a module raising a gameplay event, **Then** the event carries its class, simulated timestamp and payload, enters the deterministic queue, is journaled, and respects the pause policy for its class.
5. **Given** the published module contract documentation, **When** an integrator implements a toy module using only the documentation, **Then** it passes the conformance test suite without kernel-internal knowledge.

---

### User Story 6 - Gate Every Change on Determinism (Priority: P3)

A developer changes kernel or module code and pushes. The continuous-integration pipeline runs
the headless harness: scripted scenarios advance simulated decades, the double-run check compares
state hashes and event logs, save/load round-trips and journal replays are verified, and any
nondeterminism fails the build with enough diagnostics (first divergent tick, divergent
subsystem) to find the cause quickly.

**Why this priority**: The constitution makes nondeterminism a release-blocking bug; automated
enforcement is how that stays true for a decade of development. It packages US1–US4 into CI.

**Independent Test**: The harness itself is exercised: seeded mutation tests (deliberately
nondeterministic builds) MUST fail the gate; clean builds MUST pass; divergence diagnostics MUST
identify the first divergent tick.

**Acceptance Scenarios**:

1. **Given** a clean build, **When** CI runs the determinism suite, **Then** all double-run, round-trip and replay checks pass within the CI time budget.
2. **Given** a build with an injected nondeterminism (unseeded draw, wall-clock read, order-dependent accumulation), **When** CI runs, **Then** the suite fails and reports the first divergent tick and the divergent state region.
3. **Given** a scenario script, **When** run on the harness, **Then** no UI, renderer or input layer is loaded (verified by the architecture audit).

---

### Edge Cases

- **Simultaneity**: multiple events, watch-condition firings and decision applications landing on one tick must resolve in one documented, stable order — including ties between a player decision and an event that would invalidate it.
- **Watch condition already true at registration**: must fire immediately (at the registration tick), not wait for a false→true edge — and this behaviour must be documented and consistent.
- **Watch condition oscillation**: a condition flapping true/false at tick granularity must fire per documented edge semantics (first truth, then re-arm rules), not spam unbounded interrupts.
- **Decisions submitted during warp**: applied at the earliest deterministic boundary (next tick), timestamped, journaled; never applied retroactively.
- **Decision invalidated before application**: a decision journaled at tick T whose target no longer exists at T must resolve deterministically (rejection event), not crash or silently half-apply.
- **Save at an interrupt**: saving while paused on an un-acknowledged interrupt must restore with the interrupt still pending.
- **End of horizon mid-activity**: the 2126 (or configured) horizon arriving while activities are in progress must signal end-of-run deterministically; continued sandbox play remains deterministic and flagged unscored.
- **Journal tail corruption**: recovery uses the longest valid prefix and reports the loss precisely (US4 scenario 4).
- **Pinned content-data version missing on load**: clear, actionable failure; never silent substitution of different values.
- **Stream exhaustion / new consumers**: adding a new randomness consumer in a patch must not shift existing consumers' streams within a pinned run (sub-stream identity is stable, not positional).
- **Module registration conflicts**: two modules claiming the same state slice, or circular declared dependencies, must fail loudly at registration, not produce order-dependent behaviour.
- **Hash collisions**: state-fingerprint comparisons are for equality evidence; the fingerprint scheme must make accidental collision probability negligible and the harness must support full-state comparison when a hash mismatch needs diagnosis.
- **Clock arithmetic at scale**: a century of fine timesteps must not accumulate time-representation error (tick count × step must remain exact; calendar mapping stays correct through leap years to 2126).
- **Ironman tampering**: a hand-edited ironman journal/save must be detectable (integrity check), with the response being refusal-with-explanation, not crash.

## Requirements *(mandatory)*

IDs are FR-CORE-###, grouped by concern. Traceability to the umbrella spec is noted inline
(FR-SIM-### / FR-XCU-###). Constitution principles II–V bind throughout.

### Time & Stepping

- **FR-CORE-101**: The core MUST maintain a single authoritative world state advanced by a fixed-timestep loop; each tick has an integer index and an exact simulated timestamp; no state outside the world state may influence simulation outcomes. *(FR-SIM-001, Constitution III)*
- **FR-CORE-102**: Simulated time MUST span 1 January 2026 through a configurable horizon (25/50/100 years, default 100) with an end-of-run signal at the horizon and deterministic, explicitly-flagged unscored continuation beyond it. Score computation itself is FA-09's concern. *(FR-SIM-001, FR-SIM-008)*
- **FR-CORE-103**: Time representation MUST be exact over the full span: a century of ticks accumulates zero time-representation drift, and the civil-calendar mapping (dates, leap years, fiscal-year and recurring-cycle boundaries) is correct throughout. The core MUST expose calendar arithmetic as a service; domain seasons (eclipse, dust-storm) are computed by their owning modules on top of it. *(FR-SIM-006)*
- **FR-CORE-104**: The core MUST support multi-rate resolution within the deterministic loop, and the scheduling MUST be **kernel-managed**: each module declares its update cadence and the state conditions that escalate it to fine timestep (e.g., burns, EDL, docking in progress), and the kernel derives the resolution schedule **only from simulation state** — never from wall-clock or rendering concerns — so quiet spans advance efficiently in one tested place rather than via per-module decimation (clarified 2026-06-12). *(FR-SIM-002)*
- **FR-CORE-105**: Wall-clock pacing (real-time 1 s/s up to ≈1 year/min warp, and paused) MUST be a presentation-side concern layered on the headless stepping API; the kernel itself steps as fast as the caller drives it, and wall-clock time MUST NOT be readable from inside the simulation. *(FR-SIM-002, FR-XCU-003)*

### Determinism & Randomness

- **FR-CORE-201**: A run MUST be fully determined by: seed + content-data version + ordered command log. Two executions with identical determinants on the same platform and build MUST produce bit-identical state at every tick and identical event logs. *(FR-XCU-003; umbrella clarification 2026-06-12: per-platform/build guarantee)*
- **FR-CORE-202**: All randomness MUST flow from a single per-run master seed through named, hierarchically derived sub-streams (per module, and per stable entity/purpose within a module). Sub-stream identity MUST be stable across patches and independent: one consumer's draw count never shifts another's values. Unseeded entropy, wall-clock reads and platform-varying behaviour are defects in the core. *(FR-XCU-003)*
- **FR-CORE-203**: The core MUST provide a canonical state fingerprint (hash) covering the entire authoritative state, computable on demand at tick boundaries, with accidental-collision probability negligible for CI use, plus a full-state structural comparison mode for diagnosing divergence.
- **FR-CORE-204**: Simulated outcomes MUST be invariant to execution pacing **and to the player's warp-rate choices**: warp is pure playback speed. Identical seed + decisions produce identical history regardless of how the player warped or paused; warp-rate selections are never journaled and never reach the simulation; all multi-rate resolution (FR-CORE-104) is driven by simulation state alone. The double-run determinism check MUST deliberately vary stepping patterns between runs to prove this (clarified 2026-06-12).
- **FR-CORE-205**: Within a tick, all orderings that could affect outcomes — module execution, event delivery, decision application, watch-condition evaluation — MUST follow documented, stable, total orderings that contain no platform-, hash- or insertion-order-dependence.

### Commands (Decisions) & the Journal

- **FR-CORE-301**: All external influence on the simulation MUST enter as commands (decisions) submitted through the API; each command is validated, timestamped to the tick at which it applies, and recorded in order. There MUST be no side channel that mutates simulation state. *(FR-XCU-011)*
- **FR-CORE-302**: The core MUST maintain an append-only journal of the run — seed, configuration, content-data version reference, and the in-order record of commands and events — sufficient to replay the run from the beginning to any tick with bit-identical results. *(FR-XCU-011, FR-SIM-007)*
- **FR-CORE-303**: The journal MUST be persisted continuously with a declared durability bound so that abrupt process termination loses no more than that bound; recovery MUST replay seed + journal (optionally from the latest checkpoint/autosave for speed) to the recovered tick, identically in ironman and save-anywhere modes, and MUST handle a corrupted journal tail by recovering the longest valid prefix and reporting exactly what was lost. *(FR-SIM-010; umbrella clarification: journaled crash recovery)*
- **FR-CORE-304**: Commands that are invalid at their application tick (stale target, unaffordable, superseded) MUST resolve deterministically into a recorded rejection outcome — never a crash, partial application, or silent drop.
- **FR-CORE-305**: The journal and saves MUST carry integrity verification sufficient to detect tampering or corruption; in ironman mode, failed verification MUST refuse load with an explanation.

### Events, Warp Scheduling & Interrupts

- **FR-CORE-401**: The core MUST provide the event system all slices share: events carry a class, simulated timestamp, source and payload; they enter a deterministic queue; they are journaled; and they are queryable as the chronological feed FA-10 will render. The **full run history MUST remain queryable** (filterable by class, time range and source) for the life of the run — the kernel may serve older events from disk-backed storage while keeping a recent window fast, within the SC-006 memory/performance budget (clarified 2026-06-12). *(FR-SIM-003, FR-SIM-005)*
- **FR-CORE-402**: The core MUST support scheduled future events (known-time occurrences such as manoeuvre nodes or review dates) and condition-triggered events, and MUST advance efficiently through spans with no due events ("increment until something matters") without skipping any due occurrence. *(FR-SIM-002, FR-SIM-003)*
- **FR-CORE-403**: Each event class MUST carry a pause policy (interrupt vs log-only), configurable per class by the player at runtime, with defaults from data; pause-policy changes are commands (journaled). At minimum the classes named by the umbrella exist from this slice: manoeuvre nodes, mission milestones, anomalies, design/program reviews, budget votes, discoveries, plus player watch conditions. Their gameplay *content* arrives with later slices. *(FR-SIM-003)*
- **FR-CORE-404**: On a pause-enabled occurrence, the core MUST halt with the world at the event's simulated time, before any consequence requiring player input is applied, surface all simultaneous occurrences in deterministic order, and lose none — including when many events coincide. *(FR-SIM-003, FR-SIM-005, umbrella SC-014)*
- **FR-CORE-405**: While paused, the full API MUST remain available: state queries, decision submission, interrupt acknowledgement, save. The world is fully inspectable and plannable at zero warp. *(FR-SIM-004)*
- **FR-CORE-406**: Players MUST be able to register, modify and remove watch conditions — predicates over queryable state that raise an interrupt at the tick the condition first becomes true. Registration, modification and removal are commands (journaled). A condition already true at registration fires at the registration tick; re-arm semantics are documented and deterministic. Expressiveness in v1.0 is a **curated catalogue of parameterised condition templates defined in data** (e.g., "resource at location < X", "craft state = Y", "date reached", "value crosses threshold"), composable with AND/OR; the template catalogue grows in data without kernel changes, and no free-form expression language is exposed (clarified 2026-06-12).
- **FR-CORE-407**: Watch-condition evaluation MUST be deterministic and efficient: conditions are evaluated at a documented cadence (at least every tick boundary at which their referenced state can change), and their cost MUST scale to hundreds of registered conditions within the kernel's tick budget.

### State, Modules & Ownership

- **FR-CORE-501**: The core MUST define the module contract every later slice builds on: a module declares (a) the state slice it exclusively owns, (b) the read-only views it consumes from others, (c) the event classes it emits and subscribes to, (d) its update-phase placement, (e) its randomness sub-streams, and (f) its update cadence plus fine-step escalation conditions (FR-CORE-104). Registration MUST reject ownership overlaps and unsatisfiable/circular orderings loudly.
- **FR-CORE-502**: Single-writer ownership MUST be enforced: only the owning module mutates its slice; all cross-module influence flows through published views (read-only) and events/commands. Violations are defects surfaced loudly in development builds. *(Constitution IV, V)*
- **FR-CORE-503**: Module update order within a tick MUST be explicit, documented, derived from declared dependencies, and identical on every run (FR-CORE-205); adding a module MUST NOT silently reorder existing modules.
- **FR-CORE-504**: The full authoritative state — every module's slice plus kernel state (clock, queues, streams, pending commands, watch conditions) — MUST be serialisable, hashable (FR-CORE-203) and round-trippable as one unit; modules conform to a serialisation contract the kernel drives.
- **FR-CORE-505**: The kernel MUST contain no game-domain logic (no physics, economics, research rules) and no content values; synthetic test modules live with the harness, not the kernel. Game constants consumed by modules come from schema-validated, sourced data files per FR-XCU-001/002 — the kernel provides the loading/validation hooks and the content-data version identity that saves pin. *(FR-XCU-002, FR-XCU-006)*

### Persistence

- **FR-CORE-601**: Save MUST capture the complete run — authoritative state, kernel machinery (queues, streams, watch conditions, pending interrupts and commands), journal position, configuration, run mode, and the pinned content-data version reference — such that load reproduces bit-identical state and identical subsequent evolution, including saves taken during fine-timestep activities and while paused on un-acknowledged interrupts. *(FR-XCU-006)*
- **FR-CORE-602**: Saves MUST be versioned and forward-migratable across v1.x save-*format* changes; migration alters structure only and never substitutes a pinned run's content values. A missing pinned content-data version on load MUST produce a clear, actionable error. *(FR-XCU-006; umbrella clarification: saves pin content data)*
- **FR-CORE-603**: Autosaves MUST be written at key events and configurable intervals in both modes; ironman MUST restrict the player to the rolling autosave/journal (no manual save-scumming) while save-anywhere allows manual saves at any pause. *(FR-SIM-009, FR-SIM-010)*
- **FR-CORE-604**: Save, autosave and journal persistence MUST not corrupt on interrupted writes: the previous good save/journal prefix always survives a crash during writing.

### Headless API & Test Harness

- **FR-CORE-701**: The core MUST expose a complete headless API: create run (seed, configuration, content-data version), step (by ticks/until simulated time/until next interrupt), query state (full coverage of all published views and kernel status), submit commands, manage watch conditions and pause policies, acknowledge interrupts, save/load, compute fingerprint, and export/replay journals. Everything the game can do MUST be achievable through this API with no UI, rendering or input dependency. State queries are answered **between step calls only**, always reflecting a completed tick boundary — callers can never observe a half-applied tick; drivers (warp loop, harness) interleave stepping and reading (clarified 2026-06-12). *(FR-XCU-004, FR-SIM-007)*
- **FR-CORE-702**: The API boundary MUST be the only doorway: an architecture audit MUST verify the kernel and modules load and run with no presentation-layer dependency present, and that no caller can mutate state except via commands. *(FR-XCU-004)*
- **FR-CORE-703**: A deterministic test harness MUST ship with this slice: scenario scripts (seed + configuration + command script + expected checkpoints) run headlessly; the double-run determinism check (identical state hash at every checkpoint and identical event log) is a standing CI gate; save/load round-trip, journal replay, and kill-test crash recovery are automated; divergence diagnostics report the first divergent tick and state region. *(Constitution: Development Workflow / Testing; FR-XCU-012)*
- **FR-CORE-704**: The harness MUST include synthetic load modules approximating the full game's kernel load (entity counts, event rates, watch conditions per umbrella scale: 3,000+ propagated entities, hundreds of active craft, thousands of yearly events) so kernel performance (FR-XCU-009) is measurable before real slices exist.
- **FR-CORE-705**: The harness MUST prove its own teeth: builds with injected nondeterminism (unseeded draws, wall-clock reads, order-dependent accumulation) MUST fail the gate.

### Key Entities

- **Run**: one game instance — seed, configuration (horizon, mode, difficulty inputs), pinned content-data version, journal, current state.
- **World State**: the single authoritative state: all module slices + kernel state; hashable, serialisable as a unit.
- **Tick / Sim Clock**: integer tick index and exact simulated timestamp; calendar mapping service.
- **Random Stream**: named, hierarchical, seed-derived sub-stream with stable identity; per module/entity/purpose.
- **Command (Decision)**: validated, tick-stamped external input; the only mutation pathway; journaled; deterministic rejection outcome when invalid.
- **Event**: classed, timestamped, journaled occurrence with payload and pause policy; scheduled (known-time) or condition-triggered.
- **Event Class & Pause Policy**: taxonomy with per-class interrupt configuration (player-adjustable, journaled).
- **Watch Condition**: player-registered predicate over queryable state with documented edge/re-arm semantics.
- **Interrupt**: a pause demand raised by an event occurrence; pending until acknowledged; survives save/load.
- **Module Registration**: a slice's declared ownership, dependencies/views, event classes, update phase, streams.
- **Published View**: a module's read-only state surface consumed by other modules and the API.
- **Journal**: append-only seed+configuration+commands+events record (this is the umbrella's FR-XCU-011 "event/decision log"); durability-bounded; integrity-verified; replayable.
- **Save / Checkpoint**: versioned full-state capture with pinned content-data version; migratable structure.
- **State Fingerprint**: canonical hash over the full state at a tick boundary, plus structural diff mode.
- **Scenario Script**: harness artefact — seed, configuration, command script, expected checkpoints/hashes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (Double-run identity)**: Across a CI suite of ≥ 30 scenario scripts spanning simulated decades, varied event densities and all command types, 100% of double runs produce identical state hashes at every checkpoint and identical event logs.
- **SC-002 (Round-trip identity)**: 100% of save/load round-trip tests — including saves mid-fine-timestep activity, while paused on pending interrupts, and across a simulated format migration — produce state identical to an unbroken reference run at every subsequent checkpoint.
- **SC-003 (Replay fidelity)**: 100% of journal replays of completed runs reproduce the original final state hash and full event log; replaying to an arbitrary interior tick matches the reference hash at that tick.
- **SC-004 (Crash recovery)**: In ≥ 100 randomized kill-tests, 100% of recoveries succeed, recovered state matches the reference run at the recovered tick, and lost progress never exceeds 5 seconds of wall-clock activity preceding the kill.
- **SC-005 (Interrupt exactness)**: In 100% of interrupt tests, pause-enabled occurrences halt the world at exactly their simulated time with no post-event consequence applied, simultaneous occurrences are all surfaced in the documented order, and zero configured events are dropped — at every warp rate tested, including the maximum.
- **SC-006 (Kernel performance)**: Under the synthetic full-game load profile (FR-CORE-704) on the reference machine (high-end consumer desktop: 8+ performance cores, discrete GPU, 32 GB RAM), the kernel sustains ≥ 1 simulated year per wall-clock minute, supporting the umbrella's century-in-≤2.5-hours target, with kernel overhead (scheduling, eventing, journaling, hashing cadence) consuming ≤ 20% of the tick budget.
- **SC-007 (Watch-condition fidelity & cost)**: With 500 registered watch conditions under the synthetic load, 100% of condition firings occur at the tick of first truth, and total evaluation cost stays within the FR-CORE-407 budget.
- **SC-008 (Headless completeness)**: Every player-meaningful action defined by this slice (time control, decisions, watch conditions, pause policies, saves, interrupt acknowledgement) is exercised by at least one headless scenario script; the architecture audit finds zero presentation dependencies and zero mutation pathways outside commands.
- **SC-009 (Gate sensitivity)**: 100% of mutation builds with injected nondeterminism (≥ 10 distinct injection types) fail the CI determinism gate, with diagnostics identifying the first divergent tick.
- **SC-010 (Contract usability)**: An integrator following only the published module-contract documentation implements the reference toy module passing the conformance suite without kernel-internal knowledge, demonstrated at least once before any FA-02+ slice begins implementation.
- **SC-011 (Time integrity)**: After a full simulated century at fine timestep, accumulated time-representation drift is exactly zero and all calendar boundary checks (leap years, fiscal years, cycle boundaries through 2126) pass.

## Assumptions

- **Per-platform determinism** (umbrella clarification 2026-06-12): bit-identical guarantees hold per platform and build; cross-platform replay identity is out of scope for v1.0. Saves remain loadable across supported platforms.
- **Determinism across game versions is not guaranteed** (clarified 2026-06-12): replays and journals bind to the build that produced them (plus the pinned content-data version). Recovery and migration move a run forward; a code patch may change how an existing run's future unfolds relative to the old build. Within any single build the guarantee is absolute. Bug reports are reproduced on the build that produced the journal, so released builds MUST remain archived/retrievable for reproduction (a release-process obligation, not a runtime feature).
- **Saves pin content-data versions** (umbrella clarification): this slice owns the pinning machinery; producing/installing multiple data versions side-by-side is a packaging concern resolved at plan time.
- **Numeric values are data, not spec**: the base timestep, fine-timestep ratios, warp ladder, autosave cadence, journal durability flush cadence and hash cadence are defined in schema-validated configuration data (with sources where plausibility-bearing) and tuned at plan/implementation time within the budgets set by the success criteria.
- **The kernel is game-agnostic by design** but is not a general-purpose engine deliverable: no public/third-party API stability is promised in v1.0 (modding deferred per umbrella clarification); the module contract is an internal programme contract.
- **Difficulty plumbing only**: difficulty inputs pass through run configuration to modules (per FR-XCU-010 they may never alter physics); no difficulty logic lives in this slice.
- **Event-class taxonomy starts minimal**: the umbrella's named classes are registered by this slice with pause-policy defaults in data; later slices add classes through the same registration mechanism without kernel changes.
- **Observer/rebuild and soft-fail flows** (FR-SIM-008) are FA-09 behaviours; this slice only guarantees the kernel can keep simulating when the player faction loses agency.
- **Fully offline** (umbrella clarification): no telemetry or phone-home in any of this slice's machinery; journals/saves are shared manually for bug reports.

## Out of Scope (this slice)

- All game-domain systems and content: astrodynamics, world data, research, economy, bases, crew, politics, milestones, AI factions (FA-02…FA-09).
- All presentation: UI shell, screens, rendering, input handling, alert display (FA-10) — including the warp *controls*; this slice provides the stepping/pacing API they will drive.
- Score computation and Grand Goals (FA-09); this slice provides the horizon clock and end-of-run signal only.
- Player-facing save management UX (browsers, naming, cloud sync) — FA-10 concern; this slice provides the persistence operations.
- Cross-platform bit-identical replay; cross-version replay (pending the open clarification above); modding/public API stability.
- Localisation of event/log text: the kernel carries identifiers and payloads; human-readable rendering is FA-10's.

## Clarifications

### Session 2026-06-12

- Q: Is time-warp pure playback speed or a journaled input? → A: Pure playback speed — identical seed+decisions produce identical history regardless of warp/pause patterns; warp selections never enter the journal; multi-rate resolution is driven by simulation state alone; the double-run check varies stepping patterns to prove it (FR-CORE-204).
- Q: May game patches change in-run behaviour for existing campaigns? → A: Yes — journals/replays bind to their build (plus pinned content data); determinism is absolute within a build; bug reproduction uses the originating build, so released builds stay archived (Assumptions).
- Q: How expressive are player watch conditions in v1.0? → A: Curated catalogue of parameterised condition templates defined in data, composable with AND/OR; no free-form expression language (FR-CORE-406).
- Q: One global tick for all modules, or kernel-managed cadences? → A: Kernel-managed — each module declares its update cadence and fine-step escalation conditions; the kernel schedules deterministically and owns the quiet-span efficiency logic (FR-CORE-104, FR-CORE-501).
- Q: What do state readers get while warp is stepping? → A: Queries are answered only between step calls, always at a completed tick boundary; the driving loop interleaves stepping and reading; no concurrent snapshot machinery in the kernel (FR-CORE-701).
- Q: How much event history stays queryable over a century run? → A: The full run history is queryable — the kernel may serve older events from disk-backed storage while keeping a recent window fast; memory stays within the SC-006 budget (FR-CORE-401).

No open [NEEDS CLARIFICATION] markers remain.
