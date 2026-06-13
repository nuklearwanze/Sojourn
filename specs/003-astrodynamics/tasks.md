# Tasks: Astrodynamics & Flight (FA-02)

**Input**: Design documents from `/specs/003-astrodynamics/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: INCLUDED — the analytic validation suite is itself specified scope (FR-ASTRO-601,
constitution physics gate); per-story tests are deliverables, not optional TDD scaffolding.

**Organization**: Tasks grouped by the spec's user stories (US1/US2 = P1; US3/US4/US5/US8 = P2;
US6/US7 = P3). Phase order runs P1 → P2 → P3, so US8 (reconciliation/TCM) lands before the P3
stories. The Foundational phase is substantial — it builds the physics spine (math, rails,
frames, SOI, forces, integrator, module wiring, kernel command amendment) every story extends.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different files, no dependency on incomplete tasks)
- **[Story]**: US1…US8 from spec.md (user-story phases only)

## Path Conventions

Workspace per plan.md: module crate at `crates/sojourn-astro/`, kernel amendment in
`crates/sojourn-core/`, astro data at `data/astro/`, scenarios at `scenarios/astro_*.ron`.

---

## Phase 1: Setup

**Purpose**: Crate scaffold, data-file skeletons, event classes, CI inclusion

- [x] T001 Scaffold `crates/sojourn-astro/` (`Cargo.toml`: sojourn-core, serde, postcard, libm, slotmap, ron, thiserror, workspace lints; `src/lib.rs` with module stubs per plan structure: math, bodies, frames, soi, forces, integrator, propulsion, craft, maneuver, planner, module) and add the member to the workspace `Cargo.toml`
- [x] T002 [P] Create `data/astro/` skeletons with `source` fields throughout: `test-catalog.ron` (star, Earth-like planet w/ J2+atmosphere, Moon-like satellite, Mars-like planet, divertible asteroid — sourced textbook/standards values per contracts/body-catalog.md), `engines.ron` (chem-hydrolox, ep-ion fixtures), `config.ron` (step tiers, encounter radius factor, divergence threshold, diversion budget 16, exec-error sigmas, Lambert caps), `validation.ron` (placeholder; cases land in T020)
- [x] T003 [P] Add astro event classes to `data/kernel/event-classes.ron`: soi-crossing (LogOnly), impact (Interrupt), atmosphere-entry (Interrupt), plan-invalidated (Interrupt), propellant-exhausted (Interrupt), aero-violation (Interrupt) — each with a source field per contracts/astro-commands-events.md
- [x] T004 [P] CI inclusion in `.github/workflows/ci.yml`: `validate-data data/astro` step in the determinism job; sojourn-astro implicitly covered by workspace test/clippy jobs (verify globs)

**Checkpoint**: workspace builds with the empty astro crate; data files validate

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Kernel command amendment + the physics spine every story uses

**⚠️ CRITICAL**: T005 amends the FA-01 kernel (additive); all astro commands depend on it.

- [x] T005 Kernel amendment in `crates/sojourn-core`: add `Command::ModulePayload { module, kind, payload: Vec<u8> }` (`src/command/mod.rs`), default-rejecting `SimModule::on_command(&self, slice, kind, payload, ctx) -> CommandOutcome` (`src/module/mod.rs`), submit-time module-exists validation + application routing with journaled outcome (`src/api/mod.rs`), and a kernel unit/integration test proving routing + deterministic rejection of malformed payloads (`tests/spine.rs` addition)
- [x] T006 [P] Update FA-01 contract docs for the amendment: `specs/002-sim-core/contracts/core-api.md` (command set) and `specs/002-sim-core/contracts/module-contract.md` (on_command hook, obligations)
- [x] T007 [P] Math primitives in `crates/sojourn-astro/src/math.rs`: `Vec3` ops (dot/cross/norm/rotations), Keplerian elements ↔ state-vector conversions, PRN (prograde/radial/normal) frame from state — all transcendentals via libm; unit tests for round-trips and known conversions
- [x] T008 [P] Body catalogue in `crates/sojourn-astro/src/bodies/mod.rs`: `BodyDef`/`AtmosphereDef` RON loading (strict serde), semantic validation (sources present, divertible only on small bodies, parent tree acyclic), SOI radius derivation (Laplace formula); fill `data/astro/test-catalog.ron` with the sourced values
- [x] T009 Rails in `crates/sojourn-astro/src/bodies/rails.rs`: Kepler-equation solver (Newton + universal-variable safety, libm), `state_at(body, t)` pure function, per-step (body, time) cache; unit tests against analytic positions at quarter-period marks and full-period closure
- [x] T010 [P] Frames in `crates/sojourn-astro/src/frames.rs`: heliocentric/body-centred/rotating conversions, dominant-frame rebasing helpers; invertibility unit tests at f64 precision bounds
- [x] T011 SOI machinery in `crates/sojourn-astro/src/soi.rs`: hierarchy from catalogue, dominant-body determination, crossing detection during integration, continuity-exact rebasing (research R6); unit tests for handoff continuity
- [x] T012 [P] Propulsion interface in `crates/sojourn-astro/src/propulsion.rs`: `PropulsionEndpoint` trait per contracts/propulsion-interface.md + fixture engines loaded from `data/astro/engines.ron`; unit tests for mass-flow arithmetic and power-limited capping
- [x] T013 Force models in `crates/sojourn-astro/src/forces/mod.rs`: point gravity over the gravitating set (flag rule + dominant always + own-SOI rule), J2 (dominant body), cannonball SRP with cylindrical-shadow occultation, exponential drag below interface altitude, thrust with mass-flow coupling — each independently enableable; per-term unit tests against hand-computed accelerations
- [x] T014 Integrator in `crates/sojourn-astro/src/integrator.rs`: fixed-step RK4 over (r, v, m), three state-driven tiers from `config.ron` (coast/encounter/burn with fixed substep subdivision), tier selection as a pure function of state; convergence unit test (halving steps reduces error ~16×)
- [x] T015 Craft store in `crates/sojourn-astro/src/craft.rs`: slotmap-backed `Craft` (dominant frame state, `MassState`, engine ref, throttle, guidance slot, `FlightStatus`), invariants (no negative mass, no craft gravity/collision per research R12)
- [x] T016 Module integration in `crates/sojourn-astro/src/module.rs`: `SimModule` impl — manifest (id "astro", slice, publishes astro/status flat view incl. any_fine_tier escalation field, reads kernel/status, emits the six new classes + maneuver-node, stream astro/exec-error, cadence = coast tier, escalations on any_fine_tier), `step` propagating craft + diverted bodies with impact/atmosphere-entry/soi-crossing events, `publish`, postcard slice serde, `on_command` decoding `AstroCommand` (postcard) with SpawnCraft/DespawnCraft/SetThrottle/SetResearchGate + deterministic rejections per contracts/astro-commands-events.md
- [x] T017 Divertible bodies in `crates/sojourn-astro/src/bodies/divert.rs`: `BodyMotion` lifecycle (Railed → Diverted → ReRailed), divert-body/re-rail-body commands (divertible-only, budget-checked from config, continuity-exact transitions, journaled), diverted-body propagation alongside craft
- [x] T017b [P] Divert lifecycle tests in `crates/sojourn-astro/tests/divert.rs` (state continuity exact at Railed→Diverted and Diverted→ReRailed transitions; divert-body rejected for non-divertible bodies and beyond the config budget (16); re-rail-body rejected when not Diverted; a diverted body remains a valid porkchop/encounter target with results consistent with its new path; double-run determinism via the kernel gates with diversions in the command script)
- [x] T018 Harness astro support in `crates/sojourn-harness/src/scenario.rs` + `src/main.rs`: scenario flag to register the astro module with the test catalogue, astro-command envelope helper (kind + RON-authored payload → postcard `ModulePayload`), `conformance --module astro` wiring
- [x] T019 Foundational integration tests in `crates/sojourn-astro/tests/conformance.rs` + `tests/soi.rs`: spawn craft via command, coast a simulated year crossing an SOI (soi-crossing event fires, state continuous to integration tolerance, rebasing exact), kernel conformance suite passes for the astro module

**Checkpoint**: a craft coasts honestly through the test system, deterministically, via scenarios

---

## Phase 3: User Story 1 — The Truth: Propagate Everything Honestly (Priority: P1) 🎯 MVP

**Goal**: All perturbations verified against analytic physics; determinism gates green with astro installed.

**Independent Test**: the validation suite (periods, energy drift, J2 regression) and perturbation tests pass; harness verify/roundtrip/replay green on an astro coast scenario.

- [x] T020 [P] [US1] Author the sourced validation cases in `data/astro/validation.ron` (two-body periods, Hohmann LEO→GEO Δv, J2 regression 800 km/51.6°, hyperbolic flyby, textbook Lambert transfer case, Edelbaum spiral, synodic recurrence — values + tolerances + textbook citations per research R13) and the loader in `crates/sojourn-astro/src/bodies/mod.rs` or `tests/common`
- [x] T021 [US1] Core validation tests in `crates/sojourn-astro/tests/validation.rs`: two-body period within 0.01%, energy drift ≤ documented bound over a 10-year coast, J2 nodal regression within 0.5% (SC-001/SC-002)
- [x] T022 [US1] Perturbation behaviour tests in `crates/sojourn-astro/tests/propagation.rs`: drag decays low orbit monotonically (and not above interface), SRP displaces high-area craft + switches off in shadow deterministically, low-thrust energy change equals thrust work within tolerance, mass decreases per exhaust-velocity relation (no thrust without mass flow)
- [x] T023 [US1] Tier/warp-invariance scenario `scenarios/astro_coast.ron` + test: tier escalation engages by state (periapsis proximity), and `sojourn-harness verify` proves identical history across stepping patterns with astro installed
- [x] T024 [US1] Wire astro gates into CI in `.github/workflows/ci.yml`: verify/roundtrip on `astro_coast.ron`, `conformance --module astro`, validation suite in the test job (SC-003)

**Checkpoint**: MVP — the truth layer is proven against physics and the determinism wall

---

## Phase 4: User Story 2 — Plan a Burn, Fly It, Feel the Equation (Priority: P1)

**Goal**: Manoeuvre nodes with rocket-equation budgeting; finite-burn execution with losses; seeded execution error; the first planning queries.

**Independent Test**: Hohmann plan matches analytic Δv; flown finite burn matches within tolerance; propellant matches the rocket equation to 0.1%; infeasible plans flagged; all deterministic.

- [x] T025 [US2] Node machinery in `crates/sojourn-astro/src/maneuver/node.rs`: `ManeuverNode` store, create-node/edit-node/delete-node/commit-plan commands (state-validity rejections per contract), kernel maneuver-node event scheduling at epochs
- [x] T026 [US2] Conic planning tier in `crates/sojourn-astro/src/planner/twobody.rs`: universal-variable conic propagation, orbit summaries, SOI-patched node-outcome prediction with `Regime` tagging (FR-ASTRO-401/402)
- [x] T027 [US2] Delta-v budgeting in `crates/sojourn-astro/src/maneuver/budget.rs`: rocket-equation budget from the propulsion endpoint, per-node and per-chain feasibility, plan-invalidated events on state change (FR-ASTRO-302)
- [x] T028 [US2] Finite-burn executor in `crates/sojourn-astro/src/maneuver/burn.rs`: burn-tier integration of committed nodes (thrust-duration from interface, gravity/steering losses emerge), propellant exhaustion cut + propellant-exhausted event + dependent-stage invalidation (FR-ASTRO-303/307)
- [x] T029 [US2] Execution error in `crates/sojourn-astro/src/maneuver/error.rs`: Box–Muller (libm) magnitude/pointing dispersions from stream `astro/exec-error`, sigmas from config, zero when disabled (FR-ASTRO-304)
- [x] T030 [US2] Planning-query surface (first slice) in `crates/sojourn-astro/src/planner/query.rs`: `AstroSnapshot` extraction, orbit_summary, predict_trajectory, predict_with_nodes, dv_budget, node_feasibility — pure-function discipline + the purity test (fingerprint unchanged by queries)
- [x] T031 [US2] Manoeuvre tests + scenario in `crates/sojourn-astro/tests/maneuvers.rs` + `scenarios/astro_transfer.ron`: planned Hohmann Δv vs analytic (SC-001), flown-vs-planned within finite-burn tolerance, propellant vs rocket equation ≤0.1% (SC-005), chain budgeting, infeasibility flags, seeded exec-error reproducibility; add `astro_transfer.ron` to CI verify/roundtrip

**Checkpoint**: both P1 stories complete — plan, budget, fly, and trust the numbers

---

## Phase 5: User Story 3 — Windows: When to Go (Priority: P2)

**Goal**: Porkchop solving from real geometry; windows emerge, never scripted.

**Independent Test**: synodic recurrence found within 2% of analytic; optimal cell matches analytic transfer; unsolvable cells explicit; grid under latency budget.

- [x] T032 [US3] Lambert solver in `crates/sojourn-astro/src/planner/lambert.rs`: universal-variable formulation, multi-revolution options, data-capped iterations, explicit no-convergence result; unit tests against a textbook Lambert case (sourced in validation.ron)
- [x] T033 [US3] Porkchop in `crates/sojourn-astro/src/planner/porkchop.rs`: departure×arrival grid over railed ephemerides (Δv/C3/TOF/solvable per cell), `cell_to_plan` conversion to node specs; wire into the query surface
- [x] T034 [US3] Porkchop tests + bench in `crates/sojourn-astro/tests/planner.rs` + `crates/sojourn-harness/benches/kernel.rs` (astro bench group): synodic recurrence within 2% (SC-006), cell→plan reproduces grid Δv within planner tolerance, unsolvable-cell honesty, 40×40 grid < 100 ms budget

---

## Phase 6: User Story 4 — Steal Speed: Flybys and Assist Chains (Priority: P2)

**Goal**: Hyperbolic-encounter solving, validity guards, manual chain composition with end-to-end verification.

**Independent Test**: textbook flyby reproduced within tolerance; flown matches predicted; sub-surface periapsis flagged; two-assist chain verified against propagation.

- [x] T035 [US4] Flyby solver in `crates/sojourn-astro/src/planner/flyby.rs`: v∞/periapsis → turn angle/outbound state, validity vs radius/atmosphere-interface, chain composition (`FlybyLeg` list → `ChainReport` with end-to-end divergence); query-surface wiring (FR-ASTRO-404, chain representation = future search contract)
- [x] T036 [US4] Flyby validation + scenario in `crates/sojourn-astro/tests/planner.rs` + `scenarios/astro_flyby.ron`: the sourced flyby validation case within 0.5% (SC-001), flown flyby matches prediction within reconciliation tolerance, two-leg chain end-to-end check; scenario into CI verify
- [x] T037 [US4] Invalid-encounter guards test in `crates/sojourn-astro/tests/planner.rs`: sub-surface periapsis flagged at planning; flown trajectory intersecting the surface raises the impact event and halts propagation of that craft (FlightStatus terminal)

---

## Phase 7: User Story 5 — Patience as Propellant: Low-Thrust Arcs (Priority: P2)

**Goal**: Guidance-law arcs flown by the propagator; Edelbaum-class planning estimates; power-limited honesty.

**Independent Test**: spiral duration/propellant within 5% of estimate; thrust scales with available power; exhaustion mid-arc handled.

- [x] T038 [US5] Guidance arcs in `crates/sojourn-astro/src/maneuver/guidance.rs`: `GuidanceArc` + tangential law (data-extensible enum), set-guidance-arc/clear commands, burn-tier integration over arcs with per-substep power-limited thrust
- [x] T039 [US5] Low-thrust planning in `crates/sojourn-astro/src/planner/lowthrust.rs`: Edelbaum-class circular-to-circular estimate (duration, propellant), arc reconciliation; query-surface wiring (FR-ASTRO-405)
- [x] T040 [US5] Low-thrust tests + scenario in `crates/sojourn-astro/tests/maneuvers.rs` + `scenarios/astro_lowthrust.ron`: spiral vs estimate ≤5% (SC-004), thrust ∝ available power (no free thrust), exhaustion mid-arc → ballistic + event; scenario into CI verify

---

## Phase 8: User Story 8 — Stay on Course: Drift, Error and Correction (Priority: P2)

**Goal**: Continuous reconciliation with honest thresholds; TCM targeting; station-keeping; Lagrange-region behaviour.

**Independent Test**: divergence quantified and surfaced past threshold; TCM re-targets within tolerance, seed-reproducible; halo-class orbit departs without station-keeping and persists with it.

- [x] T041 [US8] Reconciliation in `crates/sojourn-astro/src/planner/reconcile.rs`: predicted-vs-propagated divergence per committed plan at boundaries, threshold events, low-confidence regime tagging for multi-body regions; named Lagrange-region locations (L1/L2 of the planet–moon pair) computed from the catalogue (FR-ASTRO-203/402)
- [x] T042 [US8] TCM targeter in `crates/sojourn-astro/src/maneuver/tcm.rs`: finite-difference differential correction toward the plan's aim point, bounded deterministic iterations, `solve_tcm` query + correction-node creation
- [x] T043 [US8] Drift/correction tests + scenario in `crates/sojourn-astro/tests/maneuvers.rs` + `scenarios/astro_tcm.ron`: seeded-error transfer accumulates divergence → surfaced; TCM brings arrival within documented re-target tolerance, bit-identical per seed; station-keeping schedule (schedule/cancel commands) keeps an L-region orbit that departs on the predicted timescale without it (FR-ASTRO-306)

---

## Phase 9: User Story 6 — Brake with the Sky: Aerocapture and Aerobraking (Priority: P3)

**Goal**: Entry corridors computed from the atmosphere model; passes flown with the same drag physics; violations honest.

**Independent Test**: corridor-centre entry captures as predicted; shallow skips out; steep flags + events; multi-pass apoapsis drop monotonic.

- [x] T044 [US6] Aero planning in `crates/sojourn-astro/src/planner/aero.rs`: corridor computation (shallow/steep flight-path-angle bounds, predicted exit orbit, per-pass apoapsis reduction) from the body's exponential atmosphere; aero-violation event from flown passes exceeding the documented depth/load limit (FR-ASTRO-407); query-surface wiring; planned-pass marking so atmosphere-entry handoff doesn't fire during intended aero passes
- [x] T045 [US6] Aero tests + scenario in `crates/sojourn-astro/tests/planner.rs` + `scenarios/astro_aero.ron`: corridor-centre capture within tolerance, shallow skip-out predicted+flown, steep limit flagged at planning + evented when flown, aerobraking passes monotonic (US6 acceptance set)

---

## Phase 10: User Story 7 — See Further: Low-Energy Transfers (Priority: P3)

**Goal**: Research-gated WSB route planning whose savings the propagator confirms.

**Independent Test**: gate closed → absent; gate open → reference route captures ballistically with documented Δv saving.

- [x] T046 [US7] Low-energy routes in `crates/sojourn-astro/src/planner/lowenergy.rs`: gate input via set-research-gate command (default closed), catalogue/config-defined reference route family for the planet–moon pair, route estimates with regime tags; query returns empty when gated (FR-ASTRO-406)
- [x] T047 [US7] Low-energy tests + scenario in `crates/sojourn-astro/tests/planner.rs` + `scenarios/astro_lowenergy.ron`: gating behaviour both ways; flown reference route achieves ballistic capture with the documented saving vs the direct-transfer baseline

---

## Phase 11: Polish & Cross-Cutting Concerns

- [ ] T048 [P] Full-load performance: extend the synthetic load profile/bench to include astro (3,000+ railed bodies in catalogue, 200 craft mixed coast/burn) in `crates/sojourn-harness/benches/kernel.rs` + scenario; record SC-007 results in `specs/003-astrodynamics/perf-results.md` (≥1 sim-year/min; query latency budgets)
- [x] T049 [P] Rustdoc completeness for `crates/sojourn-astro` (`#![deny(missing_docs)]` on the public surface; doctest examples for the planning queries)
- [x] T050 Verification traceability in `specs/003-astrodynamics/verification.md`: SC-001…SC-008 → proving tests/benches/CI jobs; SC-008 planning-verb coverage matrix (node, chain, porkchop, flyby, low-thrust, low-energy, aero, TCM, station-keeping) with gaps closed by added scenarios
- [x] T051 Quickstart validation: run every command in `specs/003-astrodynamics/quickstart.md` on a clean checkout; fix drift
- [x] T052 Final sweep: `cargo fmt`, `clippy -D warnings`, full workspace tests, all harness gates (verify×5 scenarios, roundtrip, replay, conformance astro+toy+synthetic, mutate, killtest) green; commit-ready tree

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Ph1) → Foundational (Ph2) → stories. **T005 (kernel amendment) blocks T016/T017/T018**.
- **US1 (Ph3)** needs only Ph2. **US2 (Ph4)** needs Ph2 (+ CI wiring from T024 helps but doesn't block).
- **US3 (Ph5)** needs US2's query surface (T030) and twobody tier (T026). **US4 (Ph6)** needs T026/T030.
- **US5 (Ph7)** needs US2's burn executor pattern (T028) + T030. **US8 (Ph8)** needs US2 (plans to reconcile) and benefits from US3/US4 scenarios.
- **US6 (Ph9)** needs drag (Ph2) + planning machinery (US2). **US7 (Ph10)** needs T016's gate command + T026.
- Polish last.

### Parallel Opportunities

- Ph1: T002–T004 after T001. Ph2: T006/T007/T008/T010/T012 parallel after T005 starts; T009 after T008; T013 after T007/T008/T012; T014 after T013; T016 after T011/T014/T015.
- After Ph4: US3, US4, US5 are mutually independent (different planner files) — parallelizable across developers; US8 after any of them.
- Polish: T048/T049 parallel.

## Implementation Strategy

**MVP first**: Ph1 + Ph2 + US1 = the proven truth layer (propagation validated against analytic
physics, deterministic under the kernel gates) — the constitution's Principle II made real.
US2 completes the P1 promise (plan/fly/budget). Then the planner verbs (US3/4/5/8) in parallel
where staffing allows, P3 stories last. Every checkpoint leaves the full workspace green —
kernel suites included (the amendment in T005 must not disturb any FA-01 gate).

## Notes

- Task count: 53 (Setup 4, Foundational 16, US1 5, US2 7, US3 3, US4 3, US5 3, US8 3, US6 2, US7 2, Polish 5).
- Spec traceability: FR-ASTRO-1xx → T007–T017b, T020–T024; 2xx → T010/T011/T041; 3xx → T025–T029, T038, T042–T043; 4xx → T026, T030, T032–T037, T039, T041, T044, T046; 5xx → T012; 6xx → T020–T024, T048; FR-ASTRO-107 → T017/T017b; kernel amendment → T005/T006.
- The kernel amendment (T005) is additive; all FA-01 tests must stay green unmodified except the documented contract-doc updates (T006).
