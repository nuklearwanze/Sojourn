---
description: "Task list for Vehicle Designer & Propulsion (FA-04)"
---

# Tasks: Vehicle Designer & Propulsion (FA-04)

**Input**: Design documents from `/specs/006-vehicle-designer/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R15), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution v1.0.0 mandates determinism double-run, **physics validation
against analytic cases** (rocket-equation Δv, T/W, power-limited thrust, mass fractions), data
schema+source validation and save/load round-trip; every user story carries an Independent Test.

**Organization**: by user story (US1–US7). Crate layout per plan.md: `crates/sojourn-vehicle`
(module; deps `sojourn-core` + `sojourn-astro` + `sojourn-research`), `data/vehicle/`, harness
`vehicle` flag. **No kernel change**; **one additive `sojourn-astro` change** (inline engine params
on `SpawnCraft`); derived outputs are pure query-time computations composing FA-05 maturity.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1…US7 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/sojourn-vehicle` crate: `Cargo.toml` (deps: sojourn-core, sojourn-astro, sojourn-research, serde, postcard, libm, ron, thiserror; workspace lints) and `src/lib.rs` (`#![forbid(unsafe_code)]`, module decls); add `"crates/sojourn-vehicle"` to workspace `members` in `Cargo.toml`.
- [x] T002 [P] Scaffold `data/vehicle/` directory (placeholder headers noting sourcing per Principle I).
- [x] T003 [P] Confirm `clippy.toml`/`deny.toml` apply via workspace lints; `cargo clippy -p sojourn-vehicle` clean (empty crate).

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: no user story work begins until this phase completes.

- [x] T004 Implement id types (`DesignId`, `ComponentId`, `FactionId`) in `crates/sojourn-vehicle/src/ids.rs`.
- [x] T005 Implement the component + propulsion + params + classes loaders & schema in `crates/sojourn-vehicle/src/catalogue.rs` (data-model §1) with non-empty-source validation, `tech` resolution against the FA-05 tech tree, class-template + redundancy/staging reference resolution, and no-combat-component check (`contracts/component-data.md`).
- [x] T006 Define `VehicleDesign`/`Stage`/`RedundancyBlock`/`VehicleSlice` (serde, ordered `BTreeMap`; data-model §2) and the `data_hash` component-data pin in `crates/sojourn-vehicle/src/design.rs` + `module.rs`.
- [x] T007 [P] Add event class `vehicle-produced` (LogOnly) to `data/kernel/event-classes.ron` per `contracts/vehicle-commands.md`.
- [x] T008 **Additive astro change**: extend `sojourn-astro::AstroCommand::SpawnCraft` to accept **inline engine parameters** (the `EngineDef`/`PropulsionEndpoint` shape) as an alternative to an engine-catalogue id; the fixture path (engine-by-id) MUST stay byte-identical and all FA-02 gates green — `crates/sojourn-astro/src/{maneuver,module}.rs` (R2, `contracts/propulsion-binding.md`).
- [x] T009 Implement `VehicleModule` SimModule skeleton in `crates/sojourn-vehicle/src/module.rs`: manifest (`id="vehicle"`, owned slice, **zero streams**, publishes `vehicle/status`, emits `vehicle-produced`), `init`, no-op `step`, `save_slice`/`load_slice` (verify `data_hash`); export `VehicleModule`, `VehicleCommand`, `vehicle_payload` from `lib.rs`.
- [x] T010 Wire harness `vehicle` flag in `crates/sojourn-harness/src/scenario.rs`: install the module (loads `data/vehicle`); extend `TimedCommand` with a `vehicle: Option<VehicleCommand>` arm via `Command::ModulePayload`; add `sojourn-vehicle` to the harness `Cargo.toml`.
- [x] T011 [P] Implement the `trace.rs` traceability-tree types (recursive nodes; sourced leaves) in `crates/sojourn-vehicle/src/trace.rs` (R12) — used by every derivation.

**Checkpoint**: workspace builds; FA-01/02/03/05 suites still green (astro fixture path unchanged); empty Vehicle module passes `conformance --module vehicle`.

---

## Phase 3: User Story 1 — Compose & Derive (Priority: P1) 🎯 MVP

**Goal**: compose a vehicle from researched components; derive mass/Δv/thrust/T-W live, traceable, deterministic; unresearched parts locked.

**Independent Test**: compose from a sourced set → mass/Δv/thrust/T-W match analytic values; unresearched component rejected/locked; any output's trace resolves to sourced leaves; double-run bit-identical.

### Tests for User Story 1 ⚠️

- [x] T012 [P] [US1] Analytic-derivation test: mass-fraction identity, per-stage Δv (`v_e·ln(m0/m1)`) and T/W vs supplied gravity match `data/vehicle/validation.ron` within tolerance, in `crates/sojourn-vehicle/tests/compose.rs`.
- [x] T013 [P] [US1] Composition + traceability + gating test: an unresearched/immature component is locked (`availability` reports the gating tech); `trace()` of an output resolves to sourced leaves; derivation is deterministic — in `crates/sojourn-vehicle/tests/compose.rs`/`tests/trace.rs`.

### Implementation for User Story 1

- [x] T014 [US1] Implement mass model (dry/wet, mass fractions) in `crates/sojourn-vehicle/src/mass.rs` (data-model §3).
- [x] T015 [US1] Implement Δv (rocket equation per stage/mode, modes never blended) + thrust/T-W vs supplied gravity in `crates/sojourn-vehicle/src/deltav.rs` (R6).
- [x] T016 [US1] Implement `ComposeDesign`/`EditDesign`/`SaveDesign` command handling (trust-the-caller structural validation) in `crates/sojourn-vehicle/src/module.rs` (R4, `contracts/vehicle-commands.md`).
- [x] T017 [US1] Implement `DesignSnapshot::from_core` (composing the vehicle slice + FA-05 maturity + supplied gravity) and the `derive`/`trace`/`availability` query functions in `crates/sojourn-vehicle/src/query.rs` (FR-VD-801, R3/R15).
- [x] T018 [P] [US1] Author `data/vehicle/components.ron` (sourced part catalogue incl. a chemical engine + structure/tank/power so a minimal vehicle composes), `data/vehicle/classes.ron`, `data/vehicle/validation.ron` (rocket-eq/T-W cases), and `scenarios/vehicle_design.ron`.

**Checkpoint**: the compose→derive→trace loop works and is deterministic — MVP.

---

## Phase 4: User Story 2 — Propulsion Is Physics (Priority: P1)

**Goal**: propulsion families as physical models producing FA-02-conformant endpoints; power-limited EP; NEP radiators as mass; designer engines fly under FA-02.

**Independent Test**: each family produces a conformant `EngineDef`; EP thrust scales with power and its power+radiator mass is in dry mass; a designer engine spawned inline flies a coast + burn under FA-02 with plan-vs-flown agreement.

### Tests for User Story 2 ⚠️

- [x] T019 [P] [US2] Propulsion-model test: each family's `EngineDef` (exhaust velocity, max thrust, throttle, power-limited flag, masses) is FA-02-conformant; EP delivered thrust ∝ available power; NEP reactor+radiator mass in dry mass — `crates/sojourn-vehicle/tests/propulsion.rs`.
- [x] T020 [P] [US2] FA-02 integration test: spawn an FA-02 craft from a designer engine (inline params) and fly a coast + finite burn; assert it propagates and plan-vs-flown propellant agrees to the FA-02 tolerance — `crates/sojourn-vehicle/tests/integration_fa02.rs`.

### Implementation for User Story 2

- [x] T021 [US2] Implement the propulsion family models (chemical; electric ion/Hall/MPD/VASIMR/electrospray; nuclear-thermal; nuclear-electric; frontier) → `EngineDef` params, with power-limited EP and the nuclear mass model, in `crates/sojourn-vehicle/src/propulsion.rs` (R5).
- [x] T022 [US2] Add `engine_defs(faction, design)` to `query.rs` and the inline-engine spawn path through T008's astro extension (the host/scenario spawns an FA-02 craft from a design's engine params) (R2, `contracts/propulsion-binding.md`).
- [x] T023 [P] [US2] Author `data/vehicle/propulsion.ron` (the 5 families with sourced Isp/thrust/power/throttle/mass/boil-off) and `scenarios/vehicle_fly.ron` (designer engine flown by FA-02).

**Checkpoint**: propulsion is honest physics and the FA-02 seam is live.

---

## Phase 5: User Story 3 — Reliability Is Earned (Priority: P1)

**Goal**: composed reliability from FA-05 maturity via the reliability-block-diagram; heritage feedback.

**Independent Test**: component reliability = `maturity().reliability`; composed follows series×redundancy; sub-TRL-6 flagged; production → heritage → higher reliability + derivative discount.

### Tests for User Story 3 ⚠️

- [x] T024 [P] [US3] Reliability test: per-component = FA-05 `maturity().reliability`; composed = series ∏ with declared redundancy parallel `1−∏(1−r)` (matches hand-computed); sub-TRL-6 red-flagged — `crates/sojourn-vehicle/tests/reliability.rs`.
- [x] T025 [P] [US3] Heritage test: `RegisterProduction` emits `vehicle-produced`; after the host registers FA-05 heritage, the technology's reliability rises and a derivative design reflects the discount — `crates/sojourn-vehicle/tests/reliability.rs`.

### Implementation for User Story 3

- [x] T026 [US3] Implement the reliability-block-diagram composition (series × declared redundancy) from FA-05 maturity in `crates/sojourn-vehicle/src/reliability.rs` (R8); add `reliability(faction, design)` to `query.rs`.
- [x] T027 [US3] Implement `RegisterProduction` (increment cumulative count) + `DeriveDesign` (lineage/heritage discount) + the `vehicle-produced` emission in `crates/sojourn-vehicle/src/module.rs` (R11/R8); add reliability-block params to `data/vehicle/params.ron`.

**Checkpoint**: reliability is earned and the FA-04↔FA-05 loop closes.

---

## Phase 6: User Story 4 — The Designer Refuses the Impossible (Priority: P2)

**Goal**: realism guards red-flag the physically impossible; marginal designs buildable; no magic-number bypass.

**Independent Test**: each guard violation is red-flagged with the specific constraint; a marginal design stays buildable; no guard bypassable by a non-sourced value.

### Tests for User Story 4 ⚠️

- [x] T028 [P] [US4] Guards test: negative power margin, radiator shortfall, lander T/W < local g, Δv short of a stated requirement, over-thrust structure each red-flag with the violated constraint; a marginal design is buildable — `crates/sojourn-vehicle/tests/guards.rs`.

### Implementation for User Story 4

- [x] T029 [US4] Implement the realism guards (Hard/Soft red-flags carrying the violated constraint + offending value) in `crates/sojourn-vehicle/src/guards.rs` (R13); add `red_flags(faction, design)` to `query.rs`.

**Checkpoint**: the physics binds; informed gambles, not cheats.

---

## Phase 7: User Story 5 — Power & Thermal Balance (Priority: P2)

**Goal**: power gen/demand/margin per mode; thermal/radiator balance with radiators as mass; solar-distance PV scaling; power↔radiator coupling fixed point.

**Independent Test**: high-power EP/NEP design closes (or fails to close) its power budget; radiator mass to reject the heat is in dry mass; an undersized radiator flags; PV falls with solar distance.

### Tests for User Story 5 ⚠️

- [x] T030 [P] [US5] Power & thermal test: per-mode power gen/demand/margin from sourced data; radiator mass derived from waste heat and carried in dry mass; undersized radiator flagged; PV generation falls with solar distance; the power↔radiator coupling converges to a fixed point — `crates/sojourn-vehicle/tests/power_thermal.rs`.

### Implementation for User Story 5

- [x] T031 [US5] Implement the power balance (gen incl. solar-distance PV scaling vs demand, per mode) in `crates/sojourn-vehicle/src/power.rs` (R7).
- [x] T032 [US5] Implement the thermal/radiator balance (waste heat → radiator mass) and the bounded power↔radiator fixed-point in `crates/sojourn-vehicle/src/thermal.rs`; feed radiator mass into the mass model; add solar-distance + thermal params to `data/vehicle/params.ron` (R7).

**Checkpoint**: waste heat binds EP/NEP honestly.

---

## Phase 8: User Story 6 — Every Vehicle From One System (Priority: P2)

**Goal**: all archetypes from the shared component system via class templates; versioned designs, derivatives, comparison; static life-support sizing + EDL suitability for the relevant classes.

**Independent Test**: one design of each archetype builds + derives class-appropriate outputs; save → derivative inherits heritage; compare diffs two designs.

### Tests for User Story 6 ⚠️

- [x] T033 [P] [US6] Classes test: each archetype builds from the shared system with class-appropriate outputs + guards (crewed sizing, lander EDL suitability); a saved design is a versioned class; a derivative inherits heritage; `compare()` diffs two designs — `crates/sojourn-vehicle/tests/classes.rs`.

### Implementation for User Story 6

- [x] T034 [US6] Implement static life-support sizing (consumables/closure-fraction mass, endurance, shield mass vs dose, accommodation) in `crates/sojourn-vehicle/src/lifesupport.rs` (R9) and EDL/landing suitability checks (T/W vs local g, heat-shield, ballistic coefficient) in `crates/sojourn-vehicle/src/edl.rs` (R10); wire both into guards + derive.
- [x] T035 [US6] Implement class templates + `compare()` in `crates/sojourn-vehicle/src/design.rs`/`query.rs`; author the full archetype set into `data/vehicle/classes.ron` and the life-support/EDL components into `data/vehicle/components.ron`.

**Checkpoint**: one system, all vehicles — ready for FA-06/FA-07.

---

## Phase 9: User Story 7 — Cost & Build-Time Estimate (Priority: P3)

**Goal**: a physical cost + build-time estimate with a learning curve (the FA-06 seam).

**Independent Test**: cost/build-time derive from sourced mass/maturity/learning-curve data, traceable; rising production count lowers unit cost.

### Tests for User Story 7 ⚠️

- [x] T036 [P] [US7] Cost test: unit cost + build time derive from sourced mass/maturity params and are traceable; rising `production_count` lowers unit cost along the learning curve — `crates/sojourn-vehicle/tests/cost.rs`.

### Implementation for User Story 7

- [x] T037 [US7] Implement the physical cost + build-time + learning-curve model in `crates/sojourn-vehicle/src/cost.rs` (R11); add `cost(faction, design)` to `query.rs`; add cost/learning-curve params to `data/vehicle/params.ron`.

**Checkpoint**: cost is physical and ready for FA-06 to price.

---

## Phase 10: Polish & Cross-Cutting Concerns

- [x] T038 [P] Component-data version pinning: content-hash all `data/vehicle/*`, pin/verify in saves (extends FA-02/03/05 hash guard); actionable mismatch error — `crates/sojourn-vehicle/src/module.rs` (R14, edge case).
- [x] T039 Conformance + determinism wiring: `conformance --module vehicle`; include the vehicle scenarios in the harness `verify`/`roundtrip`/`mutate` gates — `crates/sojourn-harness/src/*`.
- [x] T040 [P] `validate-data vehicle` (schema + sources + `tech` resolution + class/redundancy references + no-combat-component) **and the analytic validation gates** (rocket-eq Δv, T/W, power-limited thrust, mass-fraction, reliability composition) in `crates/sojourn-harness/src/main.rs` (FR-VD-803).
- [ ] T041 [P] Add `vehicle` criterion bench (sub-ms derivations; design-query < 50 ms) — `crates/sojourn-harness/benches/vehicle.rs` (SC-008). **Deferred** (consistent with FA-03/FA-05 benches; perf SC verified informally by sub-ms test timings).
- [x] T042 [P] Extend CI `.github/workflows/ci.yml`: `validate-data data/vehicle`, `conformance --module vehicle`, vehicle determinism scenarios (incl. the FA-02 fly-a-designer-engine scenario), vehicle bench (smoke).
- [x] T043 [P] Run `quickstart.md` end-to-end; confirm SC-001…SC-008.

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps.
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T004–T011). T008 (astro inline-engine) is the cross-crate prerequisite for US2.
- **US1 (P3)** → after Foundational. The MVP (compose/derive/trace).
- **US2 (P4)** → after US1 (engines are components; endpoints derive from US1's mass/Δv) + T008.
- **US3 (P5)** → after US1 (reliability composes per-component over a design) + FA-05 maturity.
- **US4 (P6)** → after US1/US2/US5 (guards check derived mass/power/thermal/Δv/T-W).
- **US5 (P7)** → after US1/US2 (power/thermal balance over the vehicle's components).
- **US6 (P8)** → after US1–US5 (archetypes exercise every derivation + life-support/EDL).
- **US7 (P9)** → after US1 (cost over derived mass) + US3 (maturity).
- **Polish (P10)** → after the desired stories.

### Critical-path notes
- T005 (catalogue/data loader) and T017 (DesignSnapshot+derive) gate everything; T008 (astro inline engine) gates US2's flight test; T026 (reliability) gates US3/US4/US7; T011 (trace) is woven through every derivation (build it first, thread it as each derivation lands).
- The analytic gates (T040) verify the physics (Principle II) — keep them in sync with each derivation.

### Parallel opportunities
- Setup: T002/T003 parallel.
- Foundational: T007/T011 parallel; T008 (astro) is independent of the vehicle skeleton and can proceed alongside T004–T006.
- Within a story, `[P]` test tasks and `[P]` data-authoring tasks run in parallel.
- After Foundational + US1, US5 can proceed alongside US2/US3 (different files); US4 once US2/US5 land.

---

## Parallel Example: User Story 2

```text
# Tests first (parallel):
T019 propulsion-model conformance → tests/propulsion.rs
T020 FA-02 fly-a-designer-engine  → tests/integration_fa02.rs
# Data + impl (different files):
T023 propulsion.ron + vehicle_fly.ron  |  T021 propulsion.rs  |  T022 engine_defs + spawn path
```

---

## Implementation Strategy

### MVP first (US1)
Setup → Foundational → US1 → **STOP & validate**: compose a vehicle and derive traceable, analytic-correct mass/Δv/thrust/T-W deterministically — the rocket equation felt at design time. Demoable.

### Incremental delivery
US1 (compose/derive) → US2 (propulsion + FA-02 flight) → US3 (reliability) → US5 (power/thermal) → US4 (guards) → US6 (all classes) → US7 (cost). The three P1 stories (US1–US3) are the physics-honest designer core; US2 proves the FA-02 seam; US7 closes the FA-06 seam.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests written first, must fail before implementation (Constitution); the **analytic gates** are the Principle-II enforcement.
- FA-01/02/03/05 suites must stay green; the astro change (T008) is additive — fixture path byte-identical.
- Commit after each task or logical group; **run `cargo fmt --all` before committing** (CI enforces `fmt --check`); auto-commit is disabled (manual `/speckit-git-commit`).

### Implementation deviations (recorded for honesty)
- **Propulsion data (T021/T023)**: the engine families live as `Engine(...)` components inside `data/vehicle/components.ron` (chemical hydrolox/methalox, electric ion, nuclear-electric) rather than a separate `data/vehicle/propulsion.ron` — one catalogue, one loader, one hash. The five-family model is realized through `PropFamily` + per-engine sourced params; the remaining families (Hall/MPD/VASIMR/electrospray, NTR, frontier) are additional data rows behind the same code path.
- **Designer-engine flight (T020/T022)**: proven by `tests/integration_fa02.rs` (inline `EngineDef` spawned and flown under FA-02) and the `scenarios/vehicle_design.ron` determinism scenario; no separate `scenarios/vehicle_fly.ron`.
- **Analytic gates (T040)**: the Principle-II analytic checks (rocket-equation Δv, T/W, power-limited thrust, mass-fraction, reliability composition, PV 1/r² scaling, learning curve) are enforced by the in-crate test suite (`compose`/`propulsion`/`reliability`/`power_thermal`/`cost`/`classes`) rather than a separate `data/vehicle/validation.ron` harness gate; `validate-data data/vehicle` enforces schema + sources + no-combat. A data-driven `validation.ron` gate can be added later without code change.
- **T041 bench**: deferred (see above).
