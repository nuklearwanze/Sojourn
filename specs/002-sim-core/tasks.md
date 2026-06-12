# Tasks: Simulation Core & Time (FA-01)

**Input**: Design documents from `/specs/002-sim-core/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: INCLUDED — for this slice the test scaffolding is itself specified scope
(FR-CORE-703/705, US6): the determinism suite, round-trip, replay, kill-test and mutation gates
are deliverables, so test tasks appear inside their owning stories rather than as optional TDD.

**Organization**: Tasks grouped by the spec's user stories. US1–US3 are P1, US4–US5 P2, US6 P3.
Because this is a kernel, Phase 2 (Foundational) is deliberately substantial: it builds the
spine (clock, RNG, state, data, registry, events, commands, step loop, harness skeleton) that
every story then extends independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependency on incomplete tasks)
- **[Story]**: US1…US6 from spec.md (user-story phases only)

## Path Conventions

Cargo workspace per plan.md: kernel at `crates/sojourn-core/`, harness at
`crates/sojourn-harness/`, kernel data at `data/kernel/`, scenario fixtures at `scenarios/`,
CI at `.github/workflows/ci.yml`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Workspace, toolchain pinning, lint/licence gates, crate scaffolds, CI skeleton

- [ ] T001 Create Cargo workspace root: `Cargo.toml` (members `crates/*`, workspace-level lints/profile, no fast-math overrides), `.gitignore` (`target/`, `runs/`)
- [ ] T002 Pin toolchain in `rust-toolchain.toml` (exact stable version, edition 2024 in crate manifests)
- [ ] T003 [P] Determinism lint config in `clippy.toml` (disallowed-types: `HashMap`/`HashSet` in core; disallowed-methods: `std::time::Instant::now`, `SystemTime::now`, `f64::sin` & co. per research R7/R8) and wire `-D warnings` into workspace lint settings in `Cargo.toml`
- [ ] T004 [P] Licence/dependency gate in `deny.toml` (allow MIT/Apache-2.0/CC0/Zlib; ban Tauri/webview/render crates from `sojourn-core` dependency tree)
- [ ] T005 Scaffold `crates/sojourn-core/` (`Cargo.toml` with serde+derive, postcard, rand_core, rand_chacha, blake3, slotmap, libm, thiserror; `src/lib.rs` with module stubs per plan structure: api, clock, rng, state, module, sched, command, event, watch, journal, save, hash, data, error)
- [ ] T006 Scaffold `crates/sojourn-harness/` (`Cargo.toml` with clap, ron, serde_json, anyhow, criterion dev-dep; `src/main.rs` clap skeleton with stub subcommands run|verify|roundtrip|replay|killtest|bench|mutate|validate-data|conformance; `benches/` dir)
- [ ] T007 [P] Initial kernel data files in `data/kernel/`: `event-classes.ron` (registry incl. maneuver-node, mission-milestone, anomaly, program-review, budget-vote, discovery, watch-fired, command-rejected, end-of-horizon, kernel-diagnostic with default pause policies), `watch-templates.ron` (starter template catalogue; binds kernel/status views only), `config.ron` (base tick, cadence/autosave/flush/hash cadences) — with `source` field support
- [ ] T008 [P] CI skeleton in `.github/workflows/ci.yml`: Windows+Linux matrix; jobs fmt → clippy → cargo-deny → test (determinism-suite jobs land in US6/T066)

**Checkpoint**: `cargo build --workspace` and all lint gates pass on an empty-but-structured workspace

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The kernel spine every story extends — no story work before this completes

**⚠️ CRITICAL**: Determinism rules (integer clock, named streams, ordered iteration, single-writer) are laid down here; every later task inherits them.

- [ ] T009 Error taxonomy in `crates/sojourn-core/src/error.rs` (`CoreError` per contracts/core-api.md §Errors, thiserror, all variants serializable)
- [ ] T010 [P] Integer tick clock in `crates/sojourn-core/src/clock/mod.rs` (`Tick`, `SimTimeNs` i128, base-tick from config, zero-drift arithmetic, horizon detection) with unit tests for century-scale exactness (SC-011)
- [ ] T011 [P] Civil calendar in `crates/sojourn-core/src/clock/calendar.rs` (proleptic Gregorian 2026–2126, leap years, fiscal-year/recurring-cycle boundaries, tick↔date conversion; exhaustive boundary unit tests)
- [ ] T012 [P] RNG registry in `crates/sojourn-core/src/rng/mod.rs` (master seed, `StreamPath`, BLAKE3 keyed name-derivation, ChaCha12 streams, serde state round-trip; unit tests: name-stability, stream isolation — A's draw count never shifts B)
- [ ] T013 [P] State foundation in `crates/sojourn-core/src/state/mod.rs` (`StateSlice` trait with serde + canonical-encoding obligations, stable 64-bit handles over slotmap, `WorldState` shell with deterministic-container rule)
- [ ] T014 [P] Data layer in `crates/sojourn-core/src/data/mod.rs` (`DataSet` RON loading with `deny_unknown_fields`, `DataVersionId` = BLAKE3 of canonical content, kernel tunables, `DataResolver`, validation API with semantic checks + source-field hook)
- [ ] T015 Module registry in `crates/sojourn-core/src/module/mod.rs` (`ModuleManifest` with all six declarations, registration validation: ownership overlap / unknown views / circular deps rejected loudly; deterministic topological ordering with documented tiebreak; insertion of new module never silently reorders — test)
- [ ] T016 Module contexts in `crates/sojourn-core/src/module/ctx.rs` (`SimModule` trait, `InitCtx`/`StepCtx` exposing exactly: clock/calendar, declared views, declared streams, declared emits, schedule, defer_command — undeclared access = loud defect)
- [ ] T017 Event spine in `crates/sojourn-core/src/event/mod.rs` (`EventRecord`, `EventClassId` from data registry, deterministic `EventQueue` ordered by (tick, tiebreak, seq), in-memory `EventStore` with class/time/source filter query; disk-tier interface stubbed for US4)
- [ ] T018 Command pipeline in `crates/sojourn-core/src/command/mod.rs` (`Command` enum incl. cfg-gated `Synthetic`, `CommandEnvelope`, validation, pending queue, apply-at-tick-boundary, deterministic `Rejected` outcomes emitted as `command-rejected` events — never partial, FR-CORE-304)
- [ ] T019 Step loop in `crates/sojourn-core/src/sched/mod.rs` (fixed-timestep advance: apply pending commands at boundary → step modules in derived order → deliver events (`on_event`, deterministic order) → publish views; single-threaded; allocation-light steady state)
- [ ] T020 API facade in `crates/sojourn-core/src/api/mod.rs` (`SimCore::create(config, data, modules)`, `step(Ticks|UntilSimTime)` returning `StepResult`, `status()`, `views()/view()`, `events(filter)`, `submit()`; owned serde DTOs only; queries answered between step calls at completed tick boundaries)
- [ ] T021 Public surface in `crates/sojourn-core/src/lib.rs` (re-export exactly the contracts/core-api.md surface; `#![deny(missing_docs)]` on public items; no internal leakage)
- [ ] T022 [P] Scenario scripts in `crates/sojourn-harness/src/scenario.rs` (RON `ScenarioScript` per data-model §10: seed, config, tick-stamped commands, checkpoints, optional golden fingerprints, optional load profile) and wire `run` subcommand in `crates/sojourn-harness/src/main.rs` (headless stepping, `--until-interrupt`, `--print-status`)
- [ ] T023 [P] Basic synthetic module in `crates/sojourn-harness/src/synthetic/mod.rs` (implements `SimModule`: entity churn in own slice, event emission, declared streams/cadence — the workhorse for all suite tests)
- [ ] T024 Spine integration test in `crates/sojourn-core/tests/spine.rs` (create run from `data/kernel/`, register synthetic module, step a simulated year, submit/apply/reject command paths, event query filters, horizon signal + `ContinueSandbox`)

**Checkpoint**: headless run works end-to-end (`cargo run -p sojourn-harness -- run scenarios/...`); all user stories can now proceed

---

## Phase 3: User Story 1 — Reproduce History Exactly (Priority: P1) 🎯 MVP

**Goal**: Identical seed + identical decisions ⇒ bit-identical state fingerprints and identical event logs, with divergence diagnostics — the constitutional foundation.

**Independent Test**: `sojourn-harness verify` runs a scenario twice with varied stepping patterns and proves hash + event-log identity; an injected-nondeterminism control is *caught*.

### Implementation for User Story 1

- [ ] T025 [P] [US1] Canonical fingerprint in `crates/sojourn-core/src/hash/mod.rs` (BLAKE3 over canonical postcard(WorldState) at tick boundaries; `StateFingerprint`; hash cadence from config)
- [ ] T026 [P] [US1] Structural diff in `crates/sojourn-core/src/hash/diff.rs` (compare two serialized states, report first divergent module/field for SC-009 diagnostics)
- [ ] T027 [US1] Wire `fingerprint()`, `fingerprint_diff()`, event-log export into `crates/sojourn-core/src/api/mod.rs`
- [ ] T028 [US1] Double-run check in `crates/sojourn-harness/src/doublerun.rs` (`verify` subcommand: run scenario twice with deliberately varied stepping patterns per FR-CORE-204, compare checkpoint fingerprints + complete event logs, report first divergent tick + diff)
- [ ] T029 [P] [US1] Scenario fixtures in `scenarios/`: `smoke_decade.ron` (10 simulated years, mixed commands, checkpoints) and `divergence_pair.ron` (two scripts differing by one decision)
- [ ] T030 [US1] Determinism integration tests in `crates/sojourn-core/tests/determinism.rs` (double-run identity; histories diverge only from the differing decision's tick; pacing invariance — flat-out vs stalled stepping identical)
- [ ] T031 [US1] Negative control in `crates/sojourn-harness/src/mutation/unseeded.rs` (cfg-gated synthetic-module variant drawing unseeded entropy; test asserts `verify` CATCHES it — first teeth for FR-CORE-705)

**Checkpoint**: MVP — the kernel provably reproduces history; CI can already gate on `verify`

---

## Phase 4: User Story 2 — Fast-Forward Until Something Matters (Priority: P1)

**Goal**: Kernel-managed cadences, event-driven advance, interrupt-and-pause at exact ticks, configurable pause policies, data-driven composable watch conditions.

**Independent Test**: Scheduled synthetic events of each class + registered watches across warp patterns: every pause-enabled occurrence halts at its exact tick before consequences, simultaneous events all surface in documented order, log-only classes never halt.

### Implementation for User Story 2

- [ ] T032 [US2] Cadence scheduler in `crates/sojourn-core/src/sched/cadence.rs` (per-manifest base cadence as tick multiples + state-condition fine-step escalation; efficient due-event advance — "increment until something matters" — skipping no due occurrence)
- [ ] T033 [US2] Interrupts in `crates/sojourn-core/src/event/interrupt.rs` (`Interrupt` store; raise on pause-enabled occurrence; `step` refuses to advance past un-acknowledged interrupts returning `InterruptPending`; `AcknowledgeInterrupt` command; pending interrupts survive in WorldState)
- [ ] T034 [US2] Pause policies in `crates/sojourn-core/src/event/policy.rs` (per-class `PausePolicy` map, defaults from `data/kernel/event-classes.ron`, `SetPausePolicy` command journal-bound, log-only path)
- [ ] T035 [US2] `StepRequest::UntilInterrupt` + `StopReason` + `new_events` in `crates/sojourn-core/src/api/mod.rs` (returns at first interrupting tick with world AT that tick, consequences unapplied)
- [ ] T036 [P] [US2] Watch catalogue in `crates/sojourn-core/src/watch/mod.rs` (template catalogue from `data/kernel/watch-templates.ron`, `WatchSpec` AND/OR expression tree, parameter/type validation against view bindings, Register/Modify/Remove commands; view bindings validated at run creation against the registered module set — unknown view ⇒ loud CoreError at create, not at first evaluation)
- [ ] T037 [US2] Watch evaluation in `crates/sojourn-core/src/watch/eval.rs` (dirty-tracked tick-boundary evaluation in WatchId order; fires at tick of first truth incl. true-at-registration; documented re-arm semantics; `watch-fired` events; budget-conscious for 500 conditions)
- [ ] T038 [US2] Intra-tick total order in `crates/sojourn-core/src/event/order.rs` (documented deterministic ordering across module events, command applications and watch firings landing on one tick; tiebreak unit tests incl. decision-vs-invalidating-event)
- [ ] T039 [P] [US2] Scenario fixtures in `scenarios/`: `interrupts.ron` (simultaneous multi-class event storm at one tick) and `watches.ron` (composed AND/OR conditions, oscillating condition, true-at-registration case)
- [ ] T040 [US2] Interrupt integration tests in `crates/sojourn-core/tests/interrupts.rs` (exact-tick halt before consequences at every stepping pattern; zero events lost under storm; log-only continues; full API usable while interrupt pending; SC-005/SC-007 assertions)

**Checkpoint**: the interrupt-and-pause heartbeat works; US1 verify still green with warp variation

---

## Phase 5: User Story 3 — Leave and Come Back to the Identical World (Priority: P1)

**Goal**: Versioned, integrity-checked, atomic save/load that round-trips the entire world bit-identically and pins the content-data version.

**Independent Test**: `sojourn-harness roundtrip` saves at arbitrary ticks (incl. mid-fine-timestep, pending interrupts), reloads, continues, and matches an unbroken reference run; loading with missing pinned data errors actionably.

### Implementation for User Story 3

- [ ] T041 [US3] Save container in `crates/sojourn-core/src/save/mod.rs` (header per contracts/persistence-format.md §1: magic, format version, build id, data version, run id, tick, BLAKE3 checksum; postcard payload; atomic write-temp→fsync→rename, prior save preserved)
- [ ] T042 [US3] Full-state serde coverage in `crates/sojourn-core/src/state/serde.rs` (WorldState complete: all slices, RNG stream states, event queue, pending interrupts/commands, watch states, pause policies, event-store index; coverage audit test asserting fingerprint(loaded) == fingerprint(saved))
- [ ] T043 [US3] Content-data pinning in `crates/sojourn-core/src/save/pin.rs` (`DataResolver` lookup by `DataVersionId`; `DataVersionUnavailable {pinned, hint}` error; pinned version in `status()`)
- [ ] T044 [US3] Migration framework in `crates/sojourn-core/src/save/migrate.rs` (save_format_version-keyed sequential structural migrations, pinned content never substituted) plus golden-fixture saves in `crates/sojourn-core/tests/fixtures/saves/`
- [ ] T045 [US3] `SimCore::save()/load()` + run-mode rules in `crates/sojourn-core/src/api/mod.rs` (ironman vs save-anywhere surface; integrity verification on load, ironman refusal with explanation)
- [ ] T046 [US3] Round-trip check in `crates/sojourn-harness/src/roundtrip.rs` (`roundtrip` subcommand: `--save-at-ticks`, reload, continue to horizon, compare every subsequent checkpoint vs unbroken run)
- [ ] T047 [US3] Persistence integration tests in `crates/sojourn-core/tests/persistence.rs` (mid-fine-timestep save, save with pending interrupt restores pending, missing pinned data error, migration golden test, corrupted save refused; SC-002 assertions)

**Checkpoint**: all three P1 stories done — deterministic, interruptible, persistent kernel

---

## Phase 6: User Story 4 — Survive a Crash, Replay a Run (Priority: P2)

**Goal**: Append-only integrity-checked journal with bounded durability, crash recovery via checkpoint+tail-replay, and full-run replay with verification diagnostics.

**Independent Test**: `killtest` (100 random-point process kills) always recovers to a reference-identical state with ≤5 s loss; `replay --verify` reproduces any completed run and pinpoints injected divergence.

**Dependency note**: builds on US3 (checkpoints reference save containers).

### Implementation for User Story 4

- [ ] T048 [US4] Journal framing in `crates/sojourn-core/src/journal/mod.rs` (frame kinds HEADER/COMMAND/OUTCOME/EVENT/CHECKPOINT/HASHMARK per contracts/persistence-format.md §2; length-prefix + per-frame BLAKE3; ironman chained checksums; strictly monotonic seq; append-only writer)
- [ ] T048b [US4] Disk-backed event-history tiering in `crates/sojourn-core/src/event/store.rs` (implement the tier interface stubbed in T017: recent window in memory, older events spilled to per-run on-disk tiers; `events(filter)` pages transparently across tiers; tier index included in save container; century-scale memory test against the SC-006 budget using the full-load scenario)
- [ ] T049 [US4] Durability policy in `crates/sojourn-core/src/journal/durability.rs` (group-fsync per COMMAND, per interrupt-raising EVENT, ≤5 s background cadence — wall-clock confined to persistence layer as the documented exception; autosave triggers + CHECKPOINT frames, and retention policy per contracts/persistence-format.md §3: ironman keeps rolling latest + 1 backup generation, save-anywhere keeps player-named saves)
- [ ] T050 [US4] Recovery in `crates/sojourn-core/src/journal/recover.rs` (newest valid checkpoint + intact-tail command replay; torn-tail detection via frame validation; `RecoveryReport` with precise loss: frame seq + tick range)
- [ ] T051 [US4] Replay in `crates/sojourn-core/src/journal/replay.rs` (`SimCore::replay(journal, resolver, until, verify)`: reconstruct from HEADER+COMMANDs; verify mode compares OUTCOME/EVENT/HASHMARK frames, reports first divergence; `BuildMismatch` graceful refusal on foreign builds)
- [ ] T052 [US4] Integrity & tamper handling in `crates/sojourn-core/src/journal/integrity.rs` (verification on open; ironman: interior edit/truncation detection via chained checksums ⇒ refuse with explanation)
- [ ] T053 [P] [US4] Replay CLI in `crates/sojourn-harness/src/replay.rs` (`replay <journal> [--verify] [--until-tick]` with divergence reporting)
- [ ] T054 [US4] Kill-test in `crates/sojourn-harness/src/killtest.rs` (spawn child harness process, terminate abruptly at random points, recover, compare vs reference at recovered tick, assert ≤5 s wall-clock loss; `--trials 100` mode for SC-004)
- [ ] T055 [US4] Journal integration tests in `crates/sojourn-core/tests/journal.rs` (replay-to-arbitrary-tick fidelity, corrupted-tail recovery + precise report, ironman crash-is-not-a-reroll, no warp data anywhere in journal, full-history queries spanning disk tiers after reload; SC-003/SC-004 assertions)

**Checkpoint**: determinism is field-usable — crashes recover, bug reports (seed+journal) reproduce exactly

---

## Phase 7: User Story 5 — Build a Game System on the Kernel (Priority: P2)

**Goal**: The module contract hardened, documented and proven usable from documentation alone — the deliverable FA-02…FA-09 consume.

**Independent Test**: The reference toy module, written against `contracts/module-contract.md` only, passes the full conformance suite (SC-010); contract-violation attempts fail loudly.

### Implementation for User Story 5

- [ ] T056 [US5] Enforcement hardening in `crates/sojourn-core/src/module/enforce.rs` (foreign-write structurally impossible via own-slice-only `&mut`; undeclared view/stream/event access = loud dev-build defect with module id + violation detail)
- [ ] T057 [P] [US5] Conformance suite as reusable library in `crates/sojourn-core/src/module/conformance.rs` (manifest validation, double-run-with-module, stream isolation, slice serde round-trip, cadence/escalation behaviour checks — the table in contracts/module-contract.md)
- [ ] T058 [P] [US5] Reference toy module in `crates/sojourn-harness/src/synthetic/toy.rs` (implemented strictly from contracts/module-contract.md; passes conformance suite; doubles as documentation example)
- [ ] T059 [US5] Registration-failure tests in `crates/sojourn-core/tests/registry.rs` (ownership overlap rejection, circular dependency rejection, unknown view rejection, no-silent-reorder on module addition)
- [ ] T060 [P] [US5] Doc-sync pass on `specs/002-sim-core/contracts/module-contract.md` (reconcile signatures/semantics with implementation; record any deltas; contract doc remains the arbiter for future slices)
- [ ] T061 [US5] Conformance CLI in `crates/sojourn-harness/src/main.rs` (`conformance` subcommand running the suite against a named module — the gate every future module crate runs in its CI)

**Checkpoint**: FA-02+ teams can start module crates against the contract with a mechanical acceptance gate

---

## Phase 8: User Story 6 — Gate Every Change on Determinism (Priority: P3)

**Goal**: The full CI quality wall: mutation tests proving the gates have teeth, synthetic full-game load, performance budgets, data validation — wired as required checks.

**Independent Test**: Clean build passes all CI jobs; each of ≥10 injected-nondeterminism builds fails `verify` with first-divergent-tick diagnostics; budgets hold on the reference machine.

### Implementation for User Story 6

- [ ] T062 [US6] Mutation injection set in `crates/sojourn-harness/src/mutation/` (cfg-gated: unseeded draw, wall-clock read, hash-ordered iteration, platform-libm transcendental, float-accumulated time, unordered event tiebreak, undeclared stream, foreign-write bypass attempt, fsync skip, hash-coverage gap — ≥10 types per SC-009)
- [ ] T063 [US6] `mutate --all` in `crates/sojourn-harness/src/mutation/mod.rs` (builds each injection variant, runs `verify`/`roundtrip`/`replay`, asserts the gate FAILS and diagnostics identify the divergence)
- [ ] T064 [P] [US6] Synthetic full-game load profile in `crates/sojourn-harness/src/synthetic/load.rs` (FR-CORE-704: 3,000+ propagated-entity stand-ins, ~200 craft stand-ins, ~10k events/sim-year, 500 watch conditions; scenario `scenarios/full_load_century.ron`)
- [ ] T065 [P] [US6] Criterion benches with budget assertions in `crates/sojourn-harness/benches/` (step loop, cadence scheduler, journal append/fsync batching, fingerprint cadence; allocation-counter test allocator asserting zero steady-state per-tick allocation)
- [ ] T066 [US6] CI completion in `.github/workflows/ci.yml` (add jobs: determinism suite — verify/roundtrip/replay/killtest on smoke scenarios; `mutate --all`; `validate-data data/kernel/`; dependency audit `cargo tree` no-UI-crates check; bench budget job; all required checks on both OS legs)
- [ ] T067 [US6] `validate-data` implementation in `crates/sojourn-harness/src/main.rs` + validation API in `crates/sojourn-core/src/data/mod.rs` (strict schema, semantic checks, source-field presence hook for plausibility-bearing values — the Principle V pipeline content slices inherit)
- [ ] T068 [US6] Performance acceptance on reference machine: run `full_load_century` scenario, record ≥1 sim-year/min and ≤20% kernel overhead + SC-007 watch budget results in `specs/002-sim-core/perf-results.md`

**Checkpoint**: nondeterminism is now mechanically unshippable; budgets tracked

---

## Phase 9: Polish & Cross-Cutting Concerns

- [ ] T069 [P] Rustdoc completeness pass on `crates/sojourn-core/` (public API fully documented, examples compile via doctests, `#![deny(missing_docs)]` clean)
- [ ] T070 [P] Quickstart validation: execute every command in `specs/002-sim-core/quickstart.md` exactly as written on a clean checkout; fix doc/reality drift
- [ ] T071 Architecture audit automation in `.github/workflows/ci.yml` (assert `sojourn-core` tree has no Tauri/webview/render deps and no presentation logic — SC-008/FR-CORE-702 made mechanical)
- [ ] T072 Cleanup/refactor pass across `crates/` (clippy pedantic triage, dead-code removal, naming consistency with data-model.md terms)
- [ ] T073 Verification traceability in `specs/002-sim-core/verification.md` (map SC-001…SC-011 to the specific passing tests/benches/CI jobs that prove each; build the SC-008 scenario coverage matrix over the full command/API surface — every Command variant, StepRequest mode, watch op, pause-policy change, ironman + save-anywhere flows — and ADD scenario fixtures in `scenarios/` for any action not yet exercised; flag remaining gaps)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Ph1)** → **Foundational (Ph2)** → user stories.
- **US1 (Ph3)**, **US2 (Ph4)**, **US3 (Ph5)**: independent of each other after Ph2 (different files/modules; all three touch `api/mod.rs` — sequence those single tasks T027/T035/T045 or rebase carefully).
- **US4 (Ph6)**: depends on **US3** (CHECKPOINT frames reference save containers) and benefits from US1 (fingerprints for verification anchors).
- **US5 (Ph7)**: depends only on Ph2 (registry/ctx) + US1 (double-run used by conformance suite).
- **US6 (Ph8)**: packages gates from US1–US4; T064/T065 can start right after Ph2.
- **Polish (Ph9)**: after all desired stories.

### Story completion order (single developer)

Ph1 → Ph2 → US1 (MVP) → US2 → US3 → US4 → US5 → US6 → Polish.

### Parallel Opportunities

- Ph1: T003, T004, T007, T008 in parallel after T001–T002.
- Ph2: T010–T014 all parallel after T009; T022/T023 parallel once T020 exists.
- US1: T025, T026, T029 parallel; T031 after T028.
- US2: T036, T039 parallel with T032–T035 track.
- US4: T053 parallel with T050–T052.
- US5: T057, T058, T060 parallel after T056.
- US6: T064, T065 parallel any time after Ph2; T062–T063 after US1–US4 gates exist.
- Cross-story (multi-developer): after Ph2, Dev A=US1, Dev B=US2, Dev C=US3 — coordinate only on `crates/sojourn-core/src/api/mod.rs` (T027/T035/T045).

## Parallel Example: User Story 1

```text
# After Ph2, launch together:
Task T025: "Canonical fingerprint in crates/sojourn-core/src/hash/mod.rs"
Task T026: "Structural diff in crates/sojourn-core/src/hash/diff.rs"
Task T029: "Scenario fixtures scenarios/smoke_decade.ron + divergence_pair.ron"
# Then sequentially: T027 (api wiring) → T028 (verify subcommand) → T030, T031
```

## Implementation Strategy

**MVP first**: Ph1 + Ph2 + US1 = a headless kernel that provably reproduces history — the
smallest demonstrable embodiment of the constitution's Principle III, and the gate (`verify`)
that protects every subsequent task. Stop, validate, then layer US2 (interrupts) and US3
(persistence) to complete the P1 promise; US4–US6 convert determinism from a property into an
operational capability (recovery, contracts, CI wall).

Each checkpoint leaves the workspace green (`cargo test --workspace` + active CI jobs) — no
story merges in a state that breaks a previous story's independent test.

---

## Notes

- Task count: 74 (Setup 8, Foundational 16, US1 7, US2 9, US3 7, US4 9, US5 6, US6 7, Polish 5).
- Spec/plan traceability: FR-CORE-1xx → T010–T011/T032; 2xx → T012/T025–T031/T038; 3xx → T018/T048–T052; 4xx → T017/T032–T040/T048b; 5xx → T013–T016/T056–T061; 6xx → T041–T047; 7xx → T020–T024/T053–T055/T062–T068.
- Commit after each task or logical group; never bypass the clippy/deny gates (they ARE the feature).
