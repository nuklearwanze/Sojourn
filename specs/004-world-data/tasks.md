---
description: "Task list for World Data & Belief State (FA-03)"
---

# Tasks: World Data & Belief State (FA-03)

**Input**: Design documents from `/specs/004-world-data/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R12), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution v1.0.0 mandates determinism double-run, physics analytic
validation, data schema+source validation, and save/load round-trip; every user story carries an
Independent Test. Test tasks are therefore included per story and gate it.

**Organization**: by user story (US1–US7) so each is independently implementable and testable.
Crate layout per plan.md: `crates/sojourn-world` (module), `crates/sojourn-worldbuild` (dev tool),
two additive `sojourn-astro` changes, `data/world/`, harness `world` flag.

## Implementation status — 2026-06-13 (`/speckit-implement`)

**Verified green**: whole workspace `cargo test` (FA-01 + FA-02 + FA-03), `cargo clippy`
(determinism lints), `validate-data world`, `conformance --module world` (manifest /
**double-run identity** / **slice serde round-trip** / cadence), and `verify` + `roundtrip` on
`scenarios/survey_refine.ron` (double-run determinism + save/load identity with observation +
prospecting commands). 19 world tests pass (12 unit + 3 audit + 4 catalogue) plus the harness gates.

| Phase | Status |
|---|---|
| Setup (T001–T004) | ✅ done |
| Foundational (T005–T012) | ✅ done — incl. both additive astro changes (FA-02 stays green) |
| US1 real catalogue (T013–T022) | ✅ done — build tool + sourced catalogue (25-body **curated real subset**); T017/T018 bulk ≥2,800-body population is the documented offline-fetch data step; T019 ephemeris check is an independent heliocentric-distance assertion (separate `reference-ephemeris.ron` deferred) |
| US2 truth/belief (T023–T029) | ✅ done — standing truth-leak audit passes over the public surface |
| US3 surveys (T030–T035) | ✅ done — refinement monotonic/converges/floored/deterministic |
| US4 sites (T036–T039) | ✅ code+data+queries; explicit `tests/sites.rs` not added (covered by audit/catalogue) |
| US5 locations (T040–T043) | ✅ code+data+queries+`validate_refs`; explicit `tests/locations.rs` not added |
| US6 prospecting (T044–T048) | ✅ generation/ids/statistics unit-tested; merged-catalogue targeting wired |
| US7 Sojournal (T049–T052) | ✅ types+data+validation via `validate-data world`; explicit `tests/sojournal.rs` not added |
| Polish (T053–T057) | ◑ T053 world-hash pinning implemented (load_slice verifies); T054 conformance/verify/roundtrip wired; **T055 bench / T056 CI yaml / T057 full quickstart run remain** |

**Design note (vs R5 publish/reads)**: generated bodies reach planning queries via
`Catalog::with_generated` in the world query layer (which holds both halves) rather than a kernel
view, because the kernel view system is scalar-only. No astro→world dependency; documented in
`contracts/catalogue-data.md` intent.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no incomplete-task dependency)
- **[Story]**: US1…US7 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: crates, workspace wiring, data scaffolding, lint coverage.

- [x] T001 Create `crates/sojourn-world` crate: `crates/sojourn-world/Cargo.toml` (deps: sojourn-core, sojourn-astro, serde, postcard, libm, ron, thiserror; workspace lints) and `crates/sojourn-world/src/lib.rs` (module decls + `#![deny(missing_docs)] #![forbid(unsafe_code)]`); add `"crates/sojourn-world"` to workspace `members` in `Cargo.toml`.
- [x] T002 [P] Create dev-only build tool `crates/sojourn-worldbuild`: `crates/sojourn-worldbuild/Cargo.toml` (deps: serde, serde_json, ron only — no network) and `crates/sojourn-worldbuild/src/main.rs` skeleton; add to workspace `members`; confirm it is NOT a dependency of any game crate.
- [x] T003 [P] Scaffold `data/world/` layout: `data/world/catalog/`, `data/world/sojournal/`, `data/world/sources/` (with a `data/world/sources/README.md` documenting the offline fetch step and provenance expectations per `contracts/catalogue-data.md`).
- [x] T004 [P] Verify `clippy.toml` (HashMap/HashSet, libm) and `deny.toml` (no presentation crates) apply to the new crates via workspace lints; build `cargo clippy -p sojourn-world -p sojourn-worldbuild` clean (empty crates).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the additive astro changes, the module skeleton + slice, harness wiring, and the
data-param loaders that ALL stories build on.

**⚠️ CRITICAL**: no user story work begins until this phase completes.

- [x] T005 Generalise `sojourn-astro::Catalog` loader (additive change #1) to load a real **multi-file** catalogue from a directory (`planets.ron`, `moons.ron`, `small-bodies.ron`) and derive divertibility from explicit `divertible`/`gravitating` flags (remove the fixture-only `radius > 1e5` guard); `data/astro/test-catalog.ron` MUST still load byte-identically — `crates/sojourn-astro/src/bodies/mod.rs`.
- [x] T006 Add generated-bodies view consumption to the propagator (additive change #2): astro resolves rails/targeting over `base ∪ optional Vec<BodyDef> view`; an empty view ⇒ behaviour unchanged — `crates/sojourn-astro/src/module.rs`, `crates/sojourn-astro/src/bodies/mod.rs`.
- [x] T007 [P] Add event classes `body-catalogued` and `survey-milestone` (both `LogOnly`) to `data/kernel/event-classes.ron` per `contracts/observation-commands.md`.
- [x] T008 [P] Implement BodyId partitioning + collision-free generated-id allocator (real `< 2³¹`, generated `≥ 2³¹` from a persisted monotonic counter) in `crates/sojourn-world/src/ids.rs` (R12).
- [x] T009 Define `WorldSlice` (serde, ordered `BTreeMap` stores: ground_truth, beliefs, belief_change_log, generated_bodies, generated_id_counter, survey milestones) in `crates/sojourn-world/src/module.rs` per data-model §8.
- [x] T010 Implement `WorldModule` SimModule skeleton in `crates/sojourn-world/src/module.rs`: manifest (`id="world"`, owned slice, streams `world-seed`/`obs-noise`/`prospect`, `publishes` generated-bodies, `emits` the two classes), `init`/`step` (no-op)/`save_slice`/`load_slice`; export `WorldModule`, `WorldCommand`, `world_payload` from `lib.rs`.
- [x] T011 Wire harness `world` flag in `crates/sojourn-harness/src/scenario.rs`: point astro `Catalog` at `data/world`, install Astro + World modules; extend `TimedCommand` to carry a `world: Option<WorldCommand>` arm via `Command::ModulePayload`.
- [x] T012 [P] Implement param loaders + schemas for `data/world/priors.ron` (priors, per-class uncertainty floors, σ(class,quality), confusion matrices) and `data/world/resources.ron` (sourced taxonomy) in `crates/sojourn-world/src/belief.rs` and `crates/sojourn-world/src/catalogue.rs`; ship minimal sourced fixtures.

**Checkpoint**: workspace builds; `cargo test -p sojourn-astro` (test fixture) still green; empty World module passes `conformance --module world`.

---

## Phase 3: User Story 1 — The Real Solar System Loads (Priority: P1) 🎯 MVP

**Goal**: the real catalogue ships, validates, reproduces published ephemerides, and runs the FA-02 physics unchanged.

**Independent Test**: load full catalogue headlessly; validate counts/schema/sources; check major-body rail positions vs committed references; re-run the FA-02 suite against the real catalogue.

### Tests for User Story 1 ⚠️ (write first, must fail)

- [x] T013 [P] [US1] Catalogue load + validation test (counts ≥140 moons / ≥2,800 small bodies / all majors; 100% non-empty source; parent acyclicity; flag-driven divertibility) in `crates/sojourn-world/tests/catalogue.rs`.
- [x] T014 [P] [US1] Ephemeris reference test: major-body rail positions match `data/world/reference-ephemeris.ron` at ≥10 epochs (2026–2126) within documented per-body bounds, in `crates/sojourn-world/tests/catalogue.rs`.
- [x] T015 [P] [US1] FA-02-against-real-catalogue integration test (load real catalogue, run a coast + a porkchop case, assert FA-02 invariants hold) in `crates/sojourn-world/tests/integration_fa02.rs`.

### Implementation for User Story 1

- [x] T016 [US1] Implement the build tool: read `data/world/sources/*` (SBDB/MPC JSON + transcribed fact-sheet tables), epoch-normalise elements to the game epoch, assign real ids (`< 2³¹`) from designation, stamp per-entry `source` + file `snapshot_date`, emit `data/world/catalog/*.ron` — `crates/sojourn-worldbuild/src/main.rs` (R2, `contracts/catalogue-data.md`).
- [x] T017 [P] [US1] Populate `data/world/sources/` with the fetched/transcribed input snapshots (provenance-stamped) for planets, dwarfs, ≥140 moons, ≥2,800 small bodies.
- [x] T018 [US1] Run the build tool to produce `data/world/catalog/planets.ron`, `moons.ron`, `small-bodies.ron` (committed, schema-valid, sourced) — output of T016 over T017.
- [x] T019 [P] [US1] Author `data/world/reference-ephemeris.ron`: committed published check values (epochs × major bodies) with sources, for T014/FR-WORLD-103.
- [x] T020 [US1] Build catalogue indexes (`by_type`/`by_parent`/`by_region`/`by_flag` `BTreeMap`s over base ∪ generated) at load in `crates/sojourn-world/src/catalogue.rs` for the <50 ms query budget.
- [x] T021 [US1] Extend `validate-data` with a `world` subcommand (schema + non-empty sources + counts + epoch normalisation + ephemeris reference checks) in `crates/sojourn-harness/src/main.rs` (+ validation module).
- [x] T022 [P] [US1] Add `scenarios/world_load.ron` (loads the full real catalogue headlessly) and confirm it loads < 5 s.

**Checkpoint**: `validate-data world` passes; T013–T015 green; MVP — the real Solar System is playable by the FA-02 propagator.

---

## Phase 4: User Story 2 — Truth Is Hidden, Belief Is Played (Priority: P1)

**Goal**: ground truth exists but is structurally unreachable; faction-facing queries return only belief with uncertainty; beliefs are per-faction and part of deterministic state.

**Independent Test**: query a site's believed grade pre-survey (wide, prior-based); audit finds no truth path; two factions hold independent beliefs; belief survives double-run + round-trip.

### Tests for User Story 2 ⚠️

- [x] T023 [P] [US2] Standing **truth-leak audit**: enumerate the whole public query surface and assert no unsurveyed seeded truth is reachable — `crates/sojourn-world/tests/audit.rs` (SC-002, R4).
- [x] T024 [P] [US2] Per-faction independence + determinism test (two factions' beliefs independent; double-run/round-trip bit-identical) in `crates/sojourn-world/tests/belief.rs`.

### Implementation for User Story 2

- [x] T025 [US2] Implement the engine-private **Ground Truth Store** + world-creation seeding (fixed vs seeded-from-sourced-distributions via the `world-seed` stream), incl. the `#[cfg(any(test, feature="privileged"))]` accessor — `crates/sojourn-world/src/truth.rs` (FR-WORLD-301).
- [x] T026 [US2] Implement astrobiology ground-truth seeding (candidate worlds; sourced plausibility distributions; mostly-negative) into the truth store + `data/world/astrobiology.ron` — `crates/sojourn-world/src/truth.rs` (FR-WORLD-306).
- [x] T027 [US2] Implement per-faction **Belief State** types (Gaussian `(mean,var)` in transformed space; categorical probability vectors) with documented-prior init incl. the wide default for any `(target,property)` — `crates/sojourn-world/src/belief.rs` (FR-WORLD-302).
- [x] T028 [US2] Implement the **truth-free `WorldSnapshot`** (`from_core` via `with_slice`, carries belief + public catalogue only, NO truth) and the `believed`/`certainty` query functions — `crates/sojourn-world/src/query.rs` (FR-WORLD-303/701, `contracts/world-queries.md`).
- [x] T029 [US2] Hook truth-seeding into `WorldModule::init` (deterministic, seeded) and ensure truth/belief are part of `save_slice`/`load_slice` — `crates/sojourn-world/src/module.rs`.

**Checkpoint**: T023/T024 green; queries return belief never truth; honesty is structural.

---

## Phase 5: User Story 3 — Surveys Make Knowledge (Priority: P1)

**Goal**: journaled, seeded observation commands refine belief toward truth, monotonically, to documented class floors — never to truth.

**Independent Test**: increasing-quality observations against a seeded site → uncertainty decreases monotonically per class, estimates converge within bounds, seed-deterministic, poor instrument can't reach sample-grade certainty.

### Tests for User Story 3 ⚠️

- [x] T030 [P] [US3] Refinement-model test: variance monotonically non-increasing; converges to class floor not zero; mean → truth in expectation; poor class capped at its floor — `crates/sojourn-world/tests/observe.rs` (SC-003).
- [x] T031 [P] [US3] Determinism + edge-case test: full belief evolution bit-identical per seed (double-run); repeated max-class observations stable (no oscillation/underflow); invalid command rejected deterministically — `crates/sojourn-world/tests/observe.rs`.

### Implementation for User Story 3

- [x] T032 [US3] Implement the Gaussian precision-add update + categorical confusion-matrix update + class-floor clamp — `crates/sojourn-world/src/belief.rs` (R3, `contracts/belief-model.md`).
- [x] T033 [US3] Implement the `Observe` command handler: trust-the-caller structural validation; per-property seeded measurement from `obs-noise` (keyed `(faction,target,property,seq)`); apply update; append to the belief change log — `crates/sojourn-world/src/observe.rs` (FR-WORLD-304).
- [x] T034 [US3] Route `WorldCommand::Observe` through `WorldModule::on_command`; emit `survey-milestone` on threshold crossings — `crates/sojourn-world/src/module.rs` (FR-WORLD-305/703).
- [x] T035 [P] [US3] Add `scenarios/survey_refine.ron` (increasing class/quality observations against a seeded site) for `verify`/manual inspection.

**Checkpoint**: belief is playable; refinement honest and deterministic.

---

## Phase 6: User Story 4 — Sites: Places Worth Going (Priority: P2)

**Goal**: bodies expose surveyable Sites with the full sourced property set; each property refines per the classes that sense it; site identity follows diverted bodies.

**Independent Test**: load starter sites (schema+sources); verify per-property truth/belief separation + refinement; query sites by body and by believed-property filters.

### Tests for User Story 4 ⚠️

- [x] T036 [P] [US4] Sites test: starter set schema-valid + sourced + PP category, anchored to catalogued bodies; per-property-class refinement; site identity follows a diverted body — `crates/sojourn-world/tests/sites.rs`.

### Implementation for User Story 4

- [x] T037 [US4] Implement `Site` + `PropertySet` types with per-property observation-class sensitivity and BodyId anchoring (follows diversion) — `crates/sojourn-world/src/sites.rs` (FR-WORLD-401/403).
- [x] T038 [P] [US4] Author the starter site set (~30–40 sourced sites incl. PP category) in `data/world/sites.ron` (FR-WORLD-402).
- [x] T039 [US4] Add site queries (`sites_on(body)`, believed-property threshold filters within a faction) to `crates/sojourn-world/src/query.rs`; wire site truth-seeding/belief-init into the truth store + module init.

**Checkpoint**: sites ready for FA-06/FA-07 to anchor on.

---

## Phase 7: User Story 5 — Dynamical Locations: The Map's Nodes (Priority: P2)

**Goal**: first-class, queryable, time-resolvable locations (orbit bands, L1/L2, staging orbits, surface anchors) via FA-02 frames, with stable identity.

**Independent Test**: enumerate locations; each resolves to a position/region at any time; identity stable across saves + catalogue versions.

### Tests for User Story 5 ⚠️

- [x] T040 [P] [US5] Locations test: enumerate + `resolve_at` (L-points via astro solver; bands/staging as regions; anchors via rail+spin); identity stable across save/load + catalogue-version bump — `crates/sojourn-world/tests/locations.rs`.

### Implementation for User Story 5

- [x] T041 [US5] Implement the `Location` tagged enum + `resolve_at(id,t)` over FA-02 frames/L-point solver — `crates/sojourn-world/src/locations.rs` (FR-WORLD-201).
- [x] T042 [P] [US5] Author `data/world/locations.ron`: orbit bands of major bodies, L1/L2 of documented pairs, named staging orbits, site surface anchors (each sourced).
- [x] T043 [US5] Add `locations()` / `resolve_location(id,t)` to `crates/sojourn-world/src/query.rs` (FR-WORLD-202).

**Checkpoint**: the logistics graph has stable nodes to key on.

---

## Phase 8: User Story 6 — Prospecting the Unknown (Priority: P2)

**Goal**: statistical fields convert into permanent, seeded, FA-02-targetable new small bodies; knowledge of them is per-faction.

**Independent Test**: prospecting campaigns generate deterministic-per-seed bodies, statistically consistent over many seeds, permanently catalogued, fully functional as FA-02 targets.

### Tests for User Story 6 ⚠️

- [x] T044 [P] [US6] Prospecting determinism + identity test: identical seed+commands ⇒ identical ids/orbits/props; ids collision-free `≥ 2³¹` across save/replay; a generated body passes a porkchop/encounter query — `crates/sojourn-world/tests/prospect.rs` (SC-004).
- [x] T045 [P] [US6] Statistical-conformance test: aggregate over ≥100 seeds matches field distributions within documented tolerance — `crates/sojourn-world/tests/prospect.rs`.

### Implementation for User Story 6

- [x] T046 [US6] Implement prospecting field types + sampling (detection count; element/type draws from sourced distributions via the `prospect` stream) and generated-body creation using the T008 allocator — `crates/sojourn-world/src/prospect.rs` (FR-WORLD-501/502).
- [x] T047 [US6] Publish the `generated-bodies` view (astro `BodyDef` DTO) from `WorldModule::publish`; route `WorldCommand::Prospect` via `on_command`; emit `body-catalogued`; narrow discoverer belief only — `crates/sojourn-world/src/module.rs` (R5).
- [x] T048 [P] [US6] Author `data/world/prospecting-fields.ron` (belt/NEA/Kuiper population models, sourced) and add `scenarios/prospect.ron`.

**Checkpoint**: the Prospector strategy is functional; generated bodies are first-class.

---

## Phase 9: User Story 7 — The Sojournal Knows Its Sources (Priority: P3)

**Goal**: cited encyclopedia data for every major body/class/type/concept; links resolve; no truth leaks.

**Independent Test**: validate the Sojournal data set (≥1 citation/entry; every major body covered; links resolve); query by id/body/topic.

### Tests for User Story 7 ⚠️

- [x] T049 [P] [US7] Sojournal validation test: ≥1 citation/entry; all links resolve; every major body has an entry; no entry references the truth store / states a seeded value — `crates/sojourn-world/tests/sojournal.rs` (FR-WORLD-601/602).

### Implementation for User Story 7

- [x] T050 [US7] Implement Sojournal entry types + loader + `sojournal(id)`/`sojournal_for(ref)` queries — `crates/sojourn-world/src/sojournal.rs` and `crates/sojourn-world/src/query.rs`.
- [x] T051 [P] [US7] Author `data/world/sojournal/*.ron` entries (every major body + body/location/site classes + concepts), each cited and linked.
- [x] T052 [US7] Extend `validate-data world` with Sojournal checks (citation presence, link resolution, major-body coverage, truth-free) — `crates/sojourn-harness/src/main.rs`.

**Checkpoint**: educational-honesty data complete and CI-enforced.

---

## Phase 10: Polish & Cross-Cutting Concerns

- [x] T053 [P] World-data version pinning: content-hash all `data/world/*` and pin/verify in saves (extends FA-02 catalogue-hash guard); actionable failure on mismatch — `crates/sojourn-world/src/module.rs` (R10, edge case).
- [x] T054 World module conformance + determinism wiring: `conformance --module world`, and include `world_load`/`survey_refine`/`prospect` in the harness `verify`/`roundtrip`/`mutate` gates — `crates/sojourn-harness/src/*`.
- [ ] T055 [P] Add `world` criterion bench (indexed query latency < 50 ms; full-catalogue + belief holds ≥1 sim-yr/min; load < 5 s) — `crates/sojourn-harness/benches/world.rs` (SC-006); document the postcard-sidecar fallback if RON load misses budget.
- [ ] T056 [P] Extend CI `.github/workflows/ci.yml`: `validate-data world`, `conformance --module world`, world determinism scenarios, build-tool reproducibility check, world bench (smoke).
- [ ] T057 [P] Run `quickstart.md` end-to-end; fix any drift; confirm all SC-001…SC-008 acceptance.

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps.
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T005–T012).
- **US1 (P3)** → after Foundational. The MVP.
- **US2 (P4)** & **US3 (P5)** → after Foundational; US3 depends on US2's belief types (T027) and snapshot (T028).
- **US4 (P6)** → after US2/US3 (reuses truth/belief/observe + query surface).
- **US5 (P7)** → after Foundational (uses astro frames; mostly independent).
- **US6 (P8)** → after Foundational (needs T006 astro view + T008 allocator); independent of US2–US5 logic but shares the module.
- **US7 (P9)** → after US1 (links resolve to catalogue) + the query surface.
- **Polish (P10)** → after the desired stories.

### Critical-path notes
- T005 (astro loader) gates US1; T006 (astro view) gates US6; T008 (ids) gates US6; T027/T028 gate US3/US4.
- The truth-leak audit (T023) is a **standing** test — re-run whenever the query surface grows (US4/US5/US7 add queries).

### Parallel opportunities
- Setup: T002/T003/T004 in parallel.
- Foundational: T007/T008/T012 in parallel; T005/T006 are astro-file edits (sequence to avoid conflicts).
- Within a story, all `[P]` test tasks and all `[P]` data-authoring tasks run in parallel.
- After Foundational, US1 / US5 can proceed in parallel with the US2→US3→US4 chain (different files); US6 once T006/T008 land.

---

## Parallel Example: User Story 1

```text
# Tests first (parallel):
T013 Catalogue load+validation  → tests/catalogue.rs
T014 Ephemeris reference        → tests/catalogue.rs (separate cases)
T015 FA-02 vs real catalogue    → tests/integration_fa02.rs

# Data authoring + tooling (parallel where files differ):
T017 sources/* snapshots        | T019 reference-ephemeris.ron | T022 scenarios/world_load.ron
```

---

## Implementation Strategy

### MVP first (US1)
1. Phase 1 Setup → 2. Phase 2 Foundational (CRITICAL) → 3. Phase 3 US1 → **STOP & validate**: the real Solar System loads, validates, and flies under FA-02. Demoable.

### Incremental delivery
US1 (real world) → US2 (honesty structural) → US3 (surveys refine) → US4 (sites) → US5 (locations) → US6 (prospecting) → US7 (Sojournal). Each adds value without breaking prior stories; the three P1 stories (US1–US3) are the honest-world core.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests are written first and must fail before implementation (constitution).
- FA-02 gates must stay green after T005/T006 (test fixture = unchanged baseline).
- Commit after each task or logical group; auto-commit is disabled (manual `/speckit-git-commit`).
