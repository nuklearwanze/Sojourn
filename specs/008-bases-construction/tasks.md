---
description: "Task list for Bases & Construction (FA-07)"
---

# Tasks: Bases & Construction (FA-07)

**Input**: Design documents from `/specs/008-bases-construction/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R14), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution v1.0.0 mandates determinism double-run, **analytic validation against
known cases** (power-margin additivity, shielding exp-attenuation, limiting-factor index, embargo
rate+buffer), data schema+source validation and save/load round-trip; every user story carries an
Independent Test.

**Organization**: by user story (US1–US6). Crate layout per plan.md: `crates/sojourn-base` (module;
**dep `sojourn-core` only** — Site facts, tech maturity, delivery/ISRU status flow in as composed
values, the FA-04/FA-06 decoupling), `data/base/`, harness `base` flag. **No kernel change; no
world/research/economy change.** Emergent properties are pure query-time derivations.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1…US6 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/sojourn-base` crate: `Cargo.toml` (deps: sojourn-core, serde, postcard, libm, ron, thiserror; workspace lints) and `src/lib.rs` (`#![forbid(unsafe_code)]`, module decls); add `"crates/sojourn-base"` to workspace `members` in `Cargo.toml`.
- [x] T002 [P] Scaffold `data/base/` directory (placeholder headers noting sourcing per Principle I).
- [x] T003 [P] Confirm `clippy.toml`/`deny.toml` apply via workspace lints; `cargo clippy -p sojourn-base` clean (empty crate).

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: no user story work begins until this phase completes.

- [x] T004 Implement id types (`FactionId`, `BaseId`, `ModuleId`, `ModuleTypeId`, `SiteId`, `ProjectId`) in `crates/sojourn-base/src/ids.rs` (data-model §1).
- [x] T005 Implement the module catalogue loader & schema in `crates/sojourn-base/src/catalogue.rs` (data-model §2): `ModuleParams` enum (Habitat/Power/Eclss/**Greenhouse**/IsruHost/Science/Storage/Manufacturing/Shielding — the Greenhouse module is the food-loop's local-supply source, G1), `ModuleType`, non-empty-source validation, `closure_fraction` ∈ [0,1], `Shielding.material` resolution against params, and the **no-combat-module** screen (`contracts/base-data.md`).
- [x] T006 Implement the composed-value input shapes (`SiteFacts`, `TechMaturity`, `DeliveryStatus`, `IsruOutput`, `BaseInputs`, `PpCategory`) in `crates/sojourn-base/src/inputs.rs` (`contracts/integration-seams.md`, R2).
- [x] T007 [P] Add the base event classes (`module-commissioned`, `base-operational`, `pp-violation`, `embargo-result`, `settlement-milestone`) to `data/kernel/event-classes.ron` per `contracts/base-commands.md`.
- [x] T008 [P] Implement the `trace.rs` traceability-tree types (recursive nodes; sourced leaves; `all_leaves_sourced`) in `crates/sojourn-base/src/trace.rs` (R13) — reused for every emergent-property trace.
- [x] T009 Define `Base`/`ModuleInstance`/commissioning state + `BaseSlice` + class templates in `crates/sojourn-base/src/base.rs` + `module.rs` (data-model §4/§11).
- [x] T010 Implement `BaseModule` SimModule skeleton in `crates/sojourn-base/src/module.rs`: manifest (`id="base"`, owned slice, **zero streams**, emits the T007 events, `cadence_ticks=86_400`), `init`, daily no-op-ish `step`, `save_slice`/`load_slice` (verify `data_hash`); export `BaseModule`, `BaseCommand`, `base_payload` from `lib.rs`.
- [x] T011 Implement `BaseModule::load(dir)` in `module.rs`: load `modules.ron`/`params.ron`/`classes.ron` + content-hash; export loaded defs; reference resolution.
- [x] T012 Wire harness `base` flag in `crates/sojourn-harness/src/scenario.rs` (install `BaseModule::load("data/base")`; extend `TimedCommand` with a `base: Option<BaseCommand>` arm via `Command::ModulePayload`) and `crates/sojourn-harness/src/main.rs` (validate-data `base` branch; `conformance "base"` factory); add `sojourn-base` to the harness `Cargo.toml`.

**Checkpoint**: workspace builds; FA-01…06 suites still green; empty Base module passes `conformance --module base`.

---

## Phase 3: User Story 1 — Compose a base from modules with emergent properties (Priority: P1) 🎯 MVP

**Goal**: compose a base from modules at a site; power margin, ECLSS closure, population capacity, shielding/dose, hazard exposure emerge from physics, fully traceable; partial base ⇒ partial properties.

**Independent Test**: compose from sourced modules → properties match the analytic composition; adding a power module raises the margin by its sourced generation; every property traces to sourced leaves; double-run bit-identical.

### Tests for User Story 1 ⚠️

- [x] T013 [P] [US1] Power-margin + traceability test: power margin = Σ generation (solar-scaled) − Σ demand; adding/removing a power module changes it by that module's generation; `trace()` resolves to sourced leaves — in `crates/sojourn-base/tests/compose.rs`.
- [x] T014 [P] [US1] Shielding/closure/population test: shielding dose-attenuation = `exp(−Σᵢ ρxᵢ/λᵢ)` (mixed materials compose); ECLSS closure = best-module; population = min(Σ accommodation, Σ ECLSS crew_support) with negative power a separate viability flag — in `crates/sojourn-base/tests/compose.rs`.

### Implementation for User Story 1

- [x] T015 [US1] Implement the power balance (gen incl. solar-distance PV scaling vs demand, over commissioned modules) in `crates/sojourn-base/src/power.rs` (R5).
- [x] T016 [US1] Implement the mass-attenuation exponential shielding (`exp(−Σᵢ ρxᵢ/λᵢ)` — per-material sum in the exponent so mixed materials compose; transmitted dose; shortfall flag) in `crates/sojourn-base/src/shielding.rs` (R7).
- [x] T017 [US1] Implement static life-support derivation (best-module ECLSS closure; population = min(Σ accommodation, Σ ECLSS crew_support), with power a separate viability flag — no per-crew power divisor, U1; consumables) in `crates/sojourn-base/src/lifesupport.rs` (R6).
- [x] T018 [US1] Implement `FoundBase`/`AddModule` command handling (trust-the-caller; tech gating surfaced at query) in `crates/sojourn-base/src/module.rs` (R8/R9, `contracts/base-commands.md`).
- [x] T019 [US1] Implement `BaseSnapshot::from_core`/`from_parts` (composing the base slice + `BaseInputs`) and the `emergent`/`power`/`shielding`/`life_support`/`trace` queries in `crates/sojourn-base/src/query.rs` (FR-BC-601, R3/R13).
- [x] T020 [P] [US1] Author `data/base/modules.ron` (sourced module catalogue incl. power/habitat/ECLSS/**greenhouse**/shielding so a minimal base composes and the food loop has a supply source), `data/base/params.ron` (shielding λ per material, dose limit, PV ref distance, closure loops), `data/base/classes.ron`, and a base fixture.

**Checkpoint**: the compose→derive→trace loop works and is deterministic — MVP.

---

## Phase 4: User Story 2 — Construction projects routed through logistics (Priority: P1)

**Goal**: a base is built by a construction project; modules commission as delivered-mass + crew-time land; partial base ⇒ partial properties; crew-time gates commissioning.

**Independent Test**: open a project; deliver one module's mass + crew-time → it commissions and contributes; others don't; insufficient crew-time slows/blocks; completing deliveries finishes the base.

### Tests for User Story 2 ⚠️

- [x] T021 [P] [US2] Commissioning + partial-base test: a module commissions only when its delivered-mass **and** crew-time meet demand; a partial base's properties reflect only commissioned modules — in `crates/sojourn-base/tests/construction.rs`.
- [x] T022 [P] [US2] Crew-time gating test: insufficient crew-time/construction capacity slows or blocks commissioning (no silent completion) — in `crates/sojourn-base/tests/construction.rs`.

### Implementation for User Story 2

- [x] T023 [US2] Implement the construction project (`ConstructionProject`, `ModuleDemand` derivation from module mass + `crew_hr_per_kg`, commissioning) in `crates/sojourn-base/src/construction.rs` (R8, data-model §5).
- [x] T024 [US2] Implement `OpenConstruction`/`DeliverToBase` handlers + delivery-driven commissioning + `module-commissioned`/`base-operational` emission in `crates/sojourn-base/src/module.rs` (R8).
- [x] T025 [US2] Implement the `construction` progress query (delivered vs remaining, % complete) in `crates/sojourn-base/src/query.rs` (FR-BC-205).
- [x] T026 [P] [US2] Author `scenarios/base_construction.ron` (found → open → deliver → commission → derive → embargo).

**Checkpoint**: building far from Earth is a logistics problem; partial bases are partial.

---

## Phase 5: User Story 3 — Siting respects planetary protection & suitability (Priority: P2)

**Goal**: siting/operating respects PP categories + physical suitability; violations red-flagged with the specific rule; forward-contamination consequence representable; never silently permitted.

**Independent Test**: site in a PP Special Region without containment → hard PP red-flag; solar base in permanent shadow → power/illumination flag; high-radiation site without shielding → shielding flag.

### Tests for User Story 3 ⚠️

- [x] T027 [P] [US3] Siting-guards test: PP Special-Region-without-containment (Hard), permanently-shadowed solar base (Hard), shielding shortfall vs site dose (Hard), unbuildable slope / no comms (Soft) each red-flag with the violated constraint; a forward-contamination consequence is representable — in `crates/sojourn-base/tests/siting.rs`.

### Implementation for User Story 3

- [x] T028 [US3] Implement the siting guards (PP category + suitability: illumination/thermal/slope/comms/hazard; shielding/power shortfalls; sub-maturity module) producing Hard/Soft red-flags + a forward-contamination consequence in `crates/sojourn-base/src/siting.rs` (R9, data-model §10).
- [x] T029 [US3] Implement the `siting_flags` query + `pp-violation` emission in `crates/sojourn-base/src/query.rs`/`module.rs` (FR-BC-301…303).

**Checkpoint**: the constraint binds; cutting corners has consequences, never silent.

---

## Phase 6: User Story 4 — On-site ISRU & fabrication reduce imports (Priority: P2)

**Goal**: a base hosts on-site production (FA-06 ISRU output + manufacturing + regolith construction) that yields local materials/shielding, reducing imported mass; regolith shielding is mass not launched.

**Independent Test**: `BuildLocal` a shielding module from local regolith → its mass is satisfied without imported mass; `local_production` reports import mass avoided; future delivered-mass demand falls.

### Tests for User Story 4 ⚠️

- [x] T030 [P] [US4] Import-reduction test: regolith-built shielding adds areal density without a corresponding imported (launched) mass; the project's remaining imported-mass demand falls; local production rates derive from sourced params + composed ISRU output — in `crates/sojourn-base/tests/production.rs`.

### Implementation for User Story 4

- [x] T031 [US4] Implement on-site production (regolith construction → local material substitution, base manufacturing rates, import-mass-avoided accounting) in `crates/sojourn-base/src/production.rs` (R10, data-model §8).
- [x] T032 [US4] Implement `BuildLocal`/`RecordIsruHost` handlers (mark `built_local`; reduce import demand) in `crates/sojourn-base/src/module.rs` (R10).
- [x] T033 [US4] Implement the `local_production` + `production_consumption` queries (composing `IsruOutput`) in `crates/sojourn-base/src/query.rs` (FR-BC-401…404, FR-BC-603).
- [x] T034 [P] [US4] Add regolith-construction/manufacturing conversion rates to `data/base/params.ron` (sourced D6 params).

**Checkpoint**: local production relaxes the mass constraint honestly — the path to self-sufficiency.

---

## Phase 7: User Story 5 — Sustainability, self-sufficiency & embargo (Priority: P2)

**Goal**: a base computes a limiting-factor self-sufficiency index and an analytic resupply-embargo survival test (the Homestead condition); progress is continuous.

**Independent Test**: the self-sufficiency index = min loop ratio and rises with the binding loop; a base above threshold survives a 5-year embargo, one below fails — both deterministic.

### Tests for User Story 5 ⚠️

- [x] T035 [P] [US5] Self-sufficiency test: index = minimum over per-loop closure ratios; improving the binding loop raises it (monotonic) — in `crates/sojourn-base/tests/sustainability.rs`.
- [x] T036 [P] [US5] Embargo test: per loop, survive iff production ≥ demand OR buffer ≥ deficit×span; a base above threshold survives a 5-year embargo, one below fails — in `crates/sojourn-base/tests/sustainability.rs`.

### Implementation for User Story 5

- [x] T037 [US5] Implement the limiting-factor self-sufficiency index + the analytic rate+buffer embargo test in `crates/sojourn-base/src/sustainability.rs` (R11/R12, data-model §9).
- [x] T038 [US5] Implement the `EvaluateEmbargo` handler + `embargo-result`/`settlement-milestone` emission in `crates/sojourn-base/src/module.rs` (R12).
- [x] T039 [US5] Implement the `self_sufficiency`/`embargo` queries in `crates/sojourn-base/src/query.rs` (FR-BC-501/502/503).
- [x] T040 [P] [US5] Add the closure-loop definitions + embargo/dose params to `data/base/params.ron` (sourced).

**Checkpoint**: self-sufficiency is an honest, continuous measure; the Homestead test tells the truth.

---

## Phase 8: User Story 6 — Base state exposed to economy, life support & politics (Priority: P3)

**Goal**: the base publishes production/consumption (economy), habitat/closure/shielding/population (life support, Slice 8) and settlement milestones (politics, Slice 9); versioned, comparable.

**Independent Test**: query production/consumption, habitat state and milestones; a base integrates with economy stocks at its location; `compare()` diffs two bases.

### Tests for User Story 6 ⚠️

- [x] T041 [P] [US6] Exposure test: `production_consumption` reports inputs/outputs at the base's location; `life_support` reports habitat capacity/closure/shielding/population; `milestones` lists settlement milestones; `compare()` diffs two bases — in `crates/sojourn-base/tests/exposure.rs`.

### Implementation for User Story 6

- [x] T042 [US6] Implement the `production_consumption`/`life_support` exposure, `milestones`, and `compare` queries in `crates/sojourn-base/src/query.rs` (FR-BC-601/602/603).

**Checkpoint**: the integration seam other slices consume is live.

---

## Phase 9: Polish & Cross-Cutting Concerns

- [x] T043 [P] Base-data version pinning: content-hash all `data/base/*`, pin/verify in saves (extends the FA-02…06 hash guard); actionable mismatch error — `crates/sojourn-base/src/module.rs` (FR-BC-703, R14).
- [x] T044 Conformance + determinism wiring: `conformance --module base`; include the base scenario in the harness `verify`/`roundtrip`/`mutate` gates — `crates/sojourn-harness/src/*`.
- [x] T045 [P] `validate-data base` (schema + sources + module/shielding ref resolution + no-combat) **and the analytic gates** (power-margin additivity, shielding per-material exp-attenuation `exp(−Σᵢ ρxᵢ/λᵢ)`, limiting-factor index min, embargo rate+buffer) in `crates/sojourn-harness/src/main.rs` + `data/base/validation.ron` (FR-BC-701, `contracts/base-data.md`).
- [ ] T046 [P] Add a `base` criterion bench (sub-ms emergent derivations over dozens of bases × hundreds of modules) — `crates/sojourn-harness/benches/base.rs` (SC-010). **Deferred** (consistent with FA-03/04/05/06 benches; perf SC verified informally by sub-ms test timings).
- [x] T047 [P] Extend CI `.github/workflows/ci.yml`: `validate-data data/base`, `conformance --module base`, the base determinism scenario (verify + roundtrip), base bench (smoke).
- [x] T048 [P] Run `quickstart.md` end-to-end; confirm SC-001…SC-010.

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps.
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T004–T012). The catalogue (T005), inputs (T006) and snapshot (T019) gate everything.
- **US1 (P3)** → after Foundational. The MVP (compose/derive/trace).
- **US2 (P4)** → after US1 (commissioning gates which modules contribute to US1's properties).
- **US3 (P5)** → after US1 (guards check derived power/shielding + composed site facts).
- **US4 (P6)** → after US1/US2 (on-site production substitutes for construction-delivery mass).
- **US5 (P7)** → after US1/US4 (self-sufficiency composes closure + local production; embargo uses buffers).
- **US6 (P8)** → after US1–US5 (exposes the full derived state).
- **Polish (P10)** → after the desired stories.

### Critical-path notes
- T005 (catalogue) and T019 (snapshot) gate everything; T006 (composed-value inputs) is the integration seam every story consumes; T024 (delivery-driven commissioning) gates US2/US4; T008 (trace) is woven through (build first, thread it as each derivation lands).
- The analytic gates (T045) verify the physics (Principles II/VII) — keep them in sync with each derivation (power with US1, shielding with US1, index/embargo with US5).

### Parallel opportunities
- Setup: T002/T003 parallel.
- Foundational: T007/T008 parallel; T004–T006 sequential-ish (ids → catalogue/inputs).
- Within a story, `[P]` test tasks and `[P]` data-authoring tasks run in parallel.
- After Foundational + US1, US3 can proceed alongside US2 (different files); US4 once US2 lands; US5 once US4 lands.

---

## Parallel Example: User Story 1

```text
# Tests first (parallel):
T013 power-margin + traceability   → tests/compose.rs
T014 shielding/closure/population   → tests/compose.rs
# Data + impl (different files):
T020 modules.ron + params.ron + classes.ron  |  T015 power.rs  |  T016 shielding.rs  |  T017 lifesupport.rs  |  T019 query.rs
```

---

## Implementation Strategy

### MVP first (US1)
Setup → Foundational → US1 → **STOP & validate**: compose a base and derive traceable, analytic-correct
emergent properties (power margin, closure, population, shielding) deterministically — a base as the sum
of physical truths. Demoable.

### Incremental delivery
US1 (compose/derive) → US2 (construction via logistics) → US3 (siting/PP) → US4 (on-site production) →
US5 (self-sufficiency/embargo) → US6 (exposure). The two P1 stories (US1–US2) are the honest base core;
US4 opens self-sufficiency; US5 closes the Homestead loop; US6 is the FA-08/09 seam.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests written first, must fail before implementation (Constitution); the **analytic gates** are the
  Principle-II/VII enforcement (power additivity, shielding exp, index min, embargo).
- FA-01…06 suites must stay green; **no upstream-crate change** (composed-value decoupling) and **no
  kernel change**.
- Commit after each task or logical group; **run `cargo fmt --all` before committing** (CI enforces
  `fmt --check`); auto-commit is disabled (manual `/speckit-git-commit`).

### Implementation deviations (recorded for honesty)
- **Commissioning is command-driven (T024)**: a module commissions inside `DeliverToBase`/`BuildLocal`
  (the host bridges an FA-06 delivery), not inside the daily `step` — the step only stamps the tick. This
  keeps the commissioning trigger explicit and the step deterministic (the FA-06 `OperateIsru` pattern).
- **Embargo result via command (T038)**: `EvaluateEmbargo` carries the `survives` verdict the host
  computes from the `BaseSnapshot` (the analytic check is a pure query, `embargo()`); the command records
  the result + emits the `settlement-milestone`. The pure embargo computation is tested directly.
- **PP containment proxy (T028)**: "has containment" is proxied by the base carrying any shielding module
  until FA-09 models the full COSPAR containment/sterilisation chain; the `forward_contamination`
  consequence is representable now.
- **Power is illumination-independent (R5)**: PV generation scales with solar **distance** only; a
  permanently-shadowed solar base is caught by the **illumination red-flag** (siting guard) rather than by
  zeroed generation — the honest gameplay signal per FR-BC-302.
- **T046 bench**: deferred (see above).
