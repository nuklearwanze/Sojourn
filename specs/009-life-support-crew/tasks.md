---
description: "Task list for Life Support & Crew (FA-08)"
---

# Tasks: Life Support & Crew (FA-08)

**Input**: Design documents from `/specs/009-life-support-crew/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R14), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution v1.0.0 mandates determinism double-run, **analytic validation against
known cases** (consumables make-up identity, REID monotonicity, multiplicative-hazard factor
monotonicity, Mars-EDL > airless, capability product), data schema+source validation and save/load
round-trip; every user story carries an Independent Test. As the **first heavily-seeded slice**, the
`mutate` gate is a first-class determinism proof here.

**Organization**: by user story (US1–US7). Crate layout per plan.md: `crates/sojourn-crew` (module; **dep
`sojourn-core` only** — vehicle/base sizing, crew roster + age/sex, ECLSS maturity, ops/light-time flow
in as composed values, the FA-04/06/07 decoupling), `data/crew/`, harness `crew` flag. **No kernel
change; no vehicle/research/economy/base change.** The dynamic per-crew/per-asset health state is
**stored** and evolved on the daily seeded step; REID/capability/viability are pure derived queries.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1…US7 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/sojourn-crew` crate: `Cargo.toml` (deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core; workspace lints) and `src/lib.rs` (`#![forbid(unsafe_code)]`, module decls); add `"crates/sojourn-crew"` to workspace `members` in `Cargo.toml`.
- [x] T002 [P] Scaffold `data/crew/` directory (placeholder headers noting sourcing per Principle I).
- [x] T003 [P] Confirm `clippy.toml`/`deny.toml` apply via workspace lints; `cargo clippy -p sojourn-crew` clean (empty crate).

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: no user story work begins until this phase completes.

- [x] T004 Implement id types (`FactionId`, `AssetId`, `AstronautId`, `MissionId`) in `crates/sojourn-crew/src/ids.rs` (data-model §1).
- [x] T005 Implement the sourced parameter loaders & schema in `crates/sojourn-crew/src/params.rs` (data-model §3): the 7 param structs (consumables/radiation/physiology/psychology/eclss/edl/hazard), non-empty-source validation, REID-threshold = 3%, **Mars > any airless EDL difficulty**, and the no-combat-parameter screen (`contracts/crew-data.md`).
- [x] T006 Implement the composed-value input shapes (`AssetSizing`, `EnvFacts`, `Sex`, `AstronautFacts`, `CrewRoster`, `TechMaturity`, `OpsLoad`, `EdlSuitability`, `CrewInputs`) in `crates/sojourn-crew/src/inputs.rs` (`contracts/integration-seams.md`, R2). Note (I1): the `CrewRoster` with per-astronaut **age/sex** is **stub-fed** in tests — FA-05's current surface exposes personnel counts, not individual astronauts with age/sex; the real FA-05 roster bridge is a future host/FA-05 integration concern, not an FA-08 dependency (core-only, composed values).
- [x] T007 [P] Implement the shared **multiplicative-hazard** primitive + **capability product** (`hazard(base, factors)` = clamp(base × ∏ factors); `capability(factors)` = clamp(∏ factors)) in `crates/sojourn-crew/src/hazard.rs` (R11/R12, FR-LSC-808) — used by US3/US5/US6.
- [x] T008 [P] Implement the `trace.rs` traceability-tree types (recursive nodes; sourced leaves; `all_leaves_sourced`) in `crates/sojourn-crew/src/trace.rs` (R13).
- [x] T009 [P] Add the crew event classes (`spe-storm`, `eclss-failure`, `crew-anomaly`, `astronaut-grounded`, `loss-of-crew`) to `data/kernel/event-classes.ron` per `contracts/crew-commands.md`.
- [x] T010 Define `CrewedAsset`/`CrewMember`/`Deconditioning`/`CrewStatus`/`EclssState` (the stored dynamic state) + `CrewSlice` in `crates/sojourn-crew/src/asset.rs` + `module.rs` (data-model §4/§11).
- [x] T011 Implement `CrewModule` SimModule skeleton in `crates/sojourn-crew/src/module.rs`: manifest (`id="crew"`, owned slice, **streams** `crew/spe-storm`/`crew/eclss-failure`/`crew/edl-risk`/`crew/anomaly`, emits the T009 events, `cadence_ticks=86_400`), `init`, a daily seeded `step` skeleton (`ctx.rng(path)` per stream), `save_slice`/`load_slice` (verify `data_hash`) + `load(dir)`; export `CrewModule`, `CrewCommand`, `crew_payload` from `lib.rs`.
- [x] T012 Wire harness `crew` flag in `crates/sojourn-harness/src/scenario.rs` (install `CrewModule::load("data/crew")`; extend `TimedCommand` with a `crew: Option<CrewCommand>` arm via `Command::ModulePayload`) and `crates/sojourn-harness/src/main.rs` (validate-data `crew` branch; `conformance "crew"` factory); add `sojourn-crew` to the harness `Cargo.toml`.

**Checkpoint**: workspace builds; FA-01…07 suites still green; empty Crew module passes `conformance --module crew`.

---

## Phase 3: User Story 1 — Consumables versus ECLSS closure (Priority: P1) 🎯 MVP

**Goal**: a crewed asset consumes O₂/water/food over time; closure sets the resupply make-up; a mission that can't cover its duration is non-viable; a robotic asset is exempt.

**Independent Test**: occupy an asset for a duration → consumables deplete at the sourced rate; make-up = gross × (1 − closure) and falls as closure rises; a mission that cannot cover its duration is flagged non-viable; a robotic asset carries no constraint. Double-run bit-identical.

### Tests for User Story 1 ⚠️

- [x] T013 [P] [US1] Make-up-identity + robotic-exempt test: consumables deplete at the sourced rate; make-up = **air/water gross × (1 − closure) + food gross** (closure recycles air/water only; food open-loop, A1) and the air/water term falls as closure rises while food does not; a robotic asset (`crewed=false`) has zero consumption — in `crates/sojourn-crew/tests/consumables.rs`.
- [x] T014 [P] [US1] Viability test: a mission whose stock + resupply cannot cover its duration is flagged non-viable; resupply restores viability — in `crates/sojourn-crew/tests/consumables.rs`.

### Implementation for User Story 1

- [x] T015 [US1] Implement consumption depletion + ECLSS-closure make-up (**air/water × (1 − closure) + food open-loop**, A1) + viability in `crates/sojourn-crew/src/consumables.rs` (R5, FR-LSC-101…105).
- [x] T016 [US1] Implement `OccupyAsset`/`Resupply`/`Vacate` command handling + daily consumption in `step` in `crates/sojourn-crew/src/module.rs` (R4, `contracts/crew-commands.md`). **Consumables exhaustion** (stock reaches zero) MUST be one of the loss-of-crew triggers (U1), alongside critical ECLSS / EDL / dose (resolved in T046).
- [x] T017 [US1] Implement `CrewSnapshot::from_core`/`from_parts` (composing the crew slice + `CrewInputs`) and the `consumables`/`viability` queries in `crates/sojourn-crew/src/query.rs` (FR-LSC-701, R3/R13).
- [x] T018 [P] [US1] Author `data/crew/consumables.ron` (sourced per-crew-day rates split into **air/water (closure-recycled) vs food (open-loop)**, A1; closure tiers) and `data/crew/params.ron` (viability thresholds) + a crew fixture.

**Checkpoint**: crewed = mass; closing the loop trades resupply for ECLSS — MVP.

---

## Phase 4: User Story 2 — Radiation dose accumulation & limits (Priority: P1)

**Goal**: GCR + seeded SPE dose accrual per crew member, shielding-attenuated; storm shelter mitigates; career dose → REID (age/sex); ground at 3%.

**Independent Test**: accrue dose from GCR + a seeded SPE; sheltering cuts the SPE dose; career dose carries across two missions; REID reaching 3% grounds the astronaut; SPE deterministic per seed.

### Tests for User Story 2 ⚠️

- [x] T019 [P] [US2] Dose + REID test: career dose rises by GCR × attenuation × time + seeded SPE; sheltering reduces the SPE dose; REID (from the sourced dose→risk curve + age/sex) reaching 3% grounds the astronaut (`astronaut-grounded`) — in `crates/sojourn-crew/tests/radiation.rs`.
- [x] T020 [P] [US2] Career-dose test: a crew member's career dose carries over across two missions; a mission that would push REID past 3% is flagged non-viable — in `crates/sojourn-crew/tests/radiation.rs`.

### Implementation for User Story 2

- [x] T021 [US2] Implement GCR accrual + seeded SPE (on `crew/spe-storm`) + shelter attenuation + the **dose→REID curve** (age/sex) in `crates/sojourn-crew/src/radiation.rs` (R6, Q2).
- [x] T022 [US2] Wire daily dose accrual + the SPE roll into `step`, the `Shelter` handler, and the `astronaut-grounded` emission in `crates/sojourn-crew/src/module.rs` (R6).
- [x] T023 [US2] Implement the `reid` query in `crates/sojourn-crew/src/query.rs` (FR-LSC-203/204).
- [x] T024 [P] [US2] Author `data/crew/radiation.ron` (GCR rates per environment, SPE arrival/magnitude, shelter attenuation, the dose→REID curve, sourced).

**Checkpoint**: radiation is the deep-space ceiling; career dose follows the astronaut.

---

## Phase 5: User Story 3 — Physiological deconditioning & countermeasures (Priority: P2)

**Goal**: micro-g deconditioning accrues; countermeasures (artificial gravity strongest) slow it; deconditioning reduces crew capability.

**Independent Test**: accrue deconditioning over a long micro-g mission; an artificial-gravity mission shows materially less; deconditioning reduces a capability metric.

### Tests for User Story 3 ⚠️

- [x] T025 [P] [US3] Deconditioning + artificial-gravity + capability test: bone/muscle/cardio/vision indices rise at sourced rates; a `spin_gravity` mission shows materially less; the deconditioning capability factor falls with the indices — in `crates/sojourn-crew/tests/physiology.rs`.

### Implementation for User Story 3

- [x] T026 [US3] Implement deconditioning accrual + countermeasure/artificial-gravity effectiveness + the deconditioning capability factor in `crates/sojourn-crew/src/physiology.rs` (R7, FR-LSC-301…303).
- [x] T027 [US3] Wire daily deconditioning accrual + post-mission recovery into `step` in `crates/sojourn-crew/src/module.rs` (R7).
- [x] T028 [US3] Implement the `capability` query (the multiplicative product via `hazard.rs`) in `crates/sojourn-crew/src/query.rs` (R11, FR-LSC-303).
- [x] T029 [P] [US3] Author `data/crew/physiology.ron` (deconditioning rates + countermeasure/artificial-gravity effectiveness + capability curves, sourced).

**Checkpoint**: long micro-g is costly; spin-gravity is the real unlock.

---

## Phase 6: User Story 4 — Psychology under isolation, confinement & comms-lag (Priority: P2)

**Goal**: psych load accrues with duration/comms-lag/confinement; raises anomaly probability + reduces morale; contributes a capability factor.

**Independent Test**: psych load rises faster on a long, distant, cramped mission; higher load raises the anomaly probability; the anomaly draw is seeded/reproducible.

### Tests for User Story 4 ⚠️

- [x] T030 [P] [US4] Psychology + anomaly test: psych load rises with duration/comms-lag/confinement; a higher load yields a higher seeded anomaly probability (monotone), reproducible per seed — in `crates/sojourn-crew/tests/psychology.rs`.

### Implementation for User Story 4

- [x] T031 [US4] Implement psych-load accrual (duration/confinement/comms-lag sensitivities) + the anomaly hazard contribution + the psych capability factor in `crates/sojourn-crew/src/psychology.rs` (R8, FR-LSC-401…403).
- [x] T032 [US4] Wire daily psych accrual + the anomaly roll (`crew/anomaly`, multiplicative hazard incl. ops oversubscription) into `step` + the `crew-anomaly` emission in `crates/sojourn-crew/src/module.rs` (R8).
- [x] T033 [P] [US4] Author `data/crew/psychology.ron` (load accrual + sensitivities + anomaly hazard, sourced).

**Checkpoint**: psychology binds via anomalies; distance has a human cost.

---

## Phase 7: User Story 5 — ECLSS spares, maintenance & failure (Priority: P2)

**Goal**: ECLSS reliability from maturity/heritage; degradation; maintenance lowers the hazard; a critical failure beyond abort reach is a loss-of-crew risk.

**Independent Test**: a lower-maturity/under-maintained ECLSS fails more often (seeded); maintenance lowers the failure probability; a critical failure beyond abort reach surfaces a loss-of-crew risk, never absorbed.

### Tests for User Story 5 ⚠️

- [x] T034 [P] [US5] Failure-probability test: the failure probability is `clamp(base × maturity_mult × maintenance_mult × degradation)`; raising any factor raises it monotonically; maintenance (crew-time + spares) lowers it — in `crates/sojourn-crew/tests/eclss.rs`.
- [x] T035 [P] [US5] Critical-failure test: a critical ECLSS failure with `abort_reach=false` surfaces a loss-of-crew risk (interrupt) and is never silently absorbed — in `crates/sojourn-crew/tests/eclss.rs`.

### Implementation for User Story 5

- [x] T036 [US5] Implement ECLSS reliability (maturity/heritage) + degradation + maintenance + the multiplicative-hazard failure in `crates/sojourn-crew/src/eclss.rs` (R9, Q1).
- [x] T037 [US5] Wire daily ECLSS degradation + failure roll (`crew/eclss-failure`) into `step`, the `Maintain` handler, and the `eclss-failure`/`loss-of-crew` emission in `crates/sojourn-crew/src/module.rs` (R9/R12).
- [x] T038 [US5] Implement the `eclss_risk` query in `crates/sojourn-crew/src/query.rs` (FR-LSC-501/503).
- [x] T039 [P] [US5] Author `data/crew/eclss.ron` (failure base rate + maturity/maintenance/heritage multipliers + degradation/spares params, sourced).

**Checkpoint**: a failed loop far from home is existential; maturity and maintenance bind.

---

## Phase 8: User Story 6 — EDL & aerocapture crew risk (Priority: P2)

**Goal**: a seeded EDL crew-risk gated by vehicle suitability + body + crew state; the Mars gap; failure → loss-of-crew.

**Independent Test**: an EDL attempt's crew-risk reflects suitability + body + crew state; Mars is materially harder than an airless landing for the same mass; a failure is loss-of-crew.

### Tests for User Story 6 ⚠️

- [x] T040 [P] [US6] EDL crew-risk test: `crew_loss_prob = clamp(base × suitability × body_difficulty × crew_state)`; the Mars body yields a materially higher probability than an airless body for the same suitability; a seeded failure marks loss-of-crew — in `crates/sojourn-crew/tests/edl.rs`.

### Implementation for User Story 6

- [x] T041 [US6] Implement the EDL multiplicative-hazard crew-risk (suitability × body × crew-state; the Mars gap) in `crates/sojourn-crew/src/edl.rs` (R10, Q1).
- [x] T042 [US6] Implement the `EvaluateEdl` handler (roll `crew/edl-risk`) + the loss-of-crew consequence in `crates/sojourn-crew/src/module.rs` (R10/R12).
- [x] T043 [US6] Implement the `edl_risk` query in `crates/sojourn-crew/src/query.rs` (FR-LSC-601/602).
- [x] T044 [P] [US6] Author `data/crew/edl.ron` (per-body difficulty incl. the Mars gap + suitability multipliers, sourced).

**Checkpoint**: EDL is where missions die; the Mars gap is real and data-driven.

---

## Phase 9: User Story 7 — Crew-state exposure & loss-of-crew consequence (Priority: P3)

**Goal**: expose per-member + per-asset state read-only; viability composes the sub-systems; loss-of-crew is a physical loss + event (political fallout → FA-09).

**Independent Test**: query a crew member's full state and an asset's viability; a mission becomes non-viable when a sub-system crosses a threshold; loss-of-crew emits an event + physical consequence FA-09 can consume.

### Tests for User Story 7 ⚠️

- [x] T045 [P] [US7] Exposure + loss-of-crew test: `member`/`roster_state`/`viability` report the full state; a state crossing a threshold flags non-viable; a loss-of-crew marks the crew lost + emits `loss-of-crew` — in `crates/sojourn-crew/tests/exposure.rs`.

### Implementation for User Story 7

- [x] T046 [US7] Implement the `member`/`roster_state`/`loss_of_crew`/composite `viability` queries + the loss-of-crew physical consequence (mark lost, fail mission) for **all triggers — consumables exhaustion (U1), critical ECLSS beyond abort, EDL failure, and dose** — + traceability in `crates/sojourn-crew/src/query.rs`/`module.rs` (FR-LSC-701…703, R12/R13).

**Checkpoint**: the FA-09 seam is live; loss-of-crew is real, never silent.

---

## Phase 10: Polish & Cross-Cutting Concerns

- [x] T047 [P] Crew-data version pinning: content-hash all `data/crew/*`, pin/verify in saves (extends the FA-02…07 hash guard); actionable mismatch error — `crates/sojourn-crew/src/module.rs` (FR-LSC-803, R14).
- [x] T048 Conformance + determinism wiring: `conformance --module crew`; include the crew scenario in the harness `verify`/`roundtrip`/`mutate` gates (the seeded-stream determinism proof) — `crates/sojourn-harness/src/*`.
- [x] T049 [P] `validate-data crew` (schema + sources + Mars>airless + REID threshold + no-combat) **and the analytic gates** (make-up identity = **air/water × (1 − closure) + food open-loop** (A1), REID monotonicity, multiplicative-hazard factor monotonicity, Mars-EDL > airless, capability product) in `crates/sojourn-harness/src/main.rs` + `data/crew/validation.ron` (FR-LSC-801, `contracts/crew-data.md`).
- [ ] T050 [P] Add a `crew` criterion bench (daily seeded step over dozens of assets × hundreds of crew; sub-ms derivations) — `crates/sojourn-harness/benches/crew.rs` (SC-011). *(May be deferred consistent with FA-03/04/05/06/07 benches.)*
- [x] T051 [P] Extend CI `.github/workflows/ci.yml`: `validate-data data/crew`, `conformance --module crew`, the crew determinism scenario (verify + roundtrip), crew bench (smoke).
- [x] T052 [P] Run `quickstart.md` end-to-end; confirm SC-001…SC-011.

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps.
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T004–T012). The params (T005), inputs (T006), hazard primitive (T007), stored state (T010) and snapshot (T017) gate everything.
- **US1 (P3)** → after Foundational. The MVP (consumables/closure/viability).
- **US2 (P4)** → after US1 (dose accrues on the per-member state US1's asset hosts).
- **US3 (P5)** → after US1 (deconditioning on the per-member state) + the hazard/capability primitive.
- **US4 (P6)** → after US1/US3 (psych is a capability factor + anomaly driver) + ops load.
- **US5 (P7)** → after US1 (ECLSS on the asset) + the hazard primitive + ECLSS maturity.
- **US6 (P8)** → after US3/US4 (EDL crew-risk gated by the composite capability) + vehicle EDL suitability.
- **US7 (P9)** → after US1–US6 (exposes the full composite state + loss-of-crew).
- **Polish (P10)** → after the desired stories.

### Critical-path notes
- T005 (params) + T007 (hazard primitive) + T010 (stored state) + T017 (snapshot) gate everything; the
  daily seeded `step` (T016/T022/T027/T032/T037) grows per story; T008 (trace) is woven through.
- The analytic gates (T049) verify the physics (Principles II/VII/VIII) — keep them in sync with each
  sub-system (make-up with US1, REID with US2, capability with US3, hazard monotonicity with US5, Mars gap
  with US6).
- This slice exercises the **`mutate` gate** hardest (four seeded streams) — wire it (T048) and keep it green.

### Parallel opportunities
- Setup: T002/T003 parallel.
- Foundational: T007/T008/T009 parallel; T004–T006 sequential-ish (ids → params/inputs); T010/T011 after.
- Within a story, `[P]` test tasks and `[P]` data-authoring tasks run in parallel.
- After Foundational + US1, US2 and US5 can proceed alongside each other (different files); US3 then US4;
  US6 once US3/US4 land.

---

## Parallel Example: User Story 2

```text
# Tests first (parallel):
T019 dose + REID + grounding   → tests/radiation.rs
T020 career-dose across missions → tests/radiation.rs
# Data + impl (different files):
T024 radiation.ron  |  T021 radiation.rs  |  T022 step + Shelter + grounded event  |  T023 reid query
```

---

## Implementation Strategy

### MVP first (US1)
Setup → Foundational → US1 → **STOP & validate**: a crewed asset consumes consumables over time, closure
sets the resupply make-up, and a mission that can't cover its duration is non-viable — *crewed = mass*,
deterministic. Demoable.

### Incremental delivery
US1 (consumables/closure) → US2 (radiation/REID) → US3 (deconditioning) → US4 (psychology) → US5 (ECLSS
failure) → US6 (EDL risk) → US7 (exposure/loss-of-crew). The two P1 stories (US1–US2) are the felt
crewed-difficulty core (mass + the radiation ceiling); US5/US6 add the loss-of-crew risks; US7 is the
FA-09 seam.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests written first, must fail before implementation (Constitution); the **analytic gates** are the
  Principle-II/VII/VIII enforcement (make-up identity, REID, hazard monotonicity, Mars gap, capability).
- FA-01…07 suites must stay green; **no upstream-crate change** (composed-value decoupling) and **no
  kernel change**. All randomness is **named seeded streams** — the `mutate` gate must stay green.
- Commit after each task or logical group; **run `cargo fmt --all` before committing** (CI enforces
  `fmt --check`); auto-commit is disabled (manual `/speckit-git-commit`).

### Implementation deviations (recorded for honesty)
- **Composed values captured at command time (R3/R4 refinement)**: the plan framed `CrewInputs` as a
  query-time composition. The implementation instead **captures** the composed sizing/env/ECLSS-maturity/
  ops into the slice at command time (`OccupyAsset`/`AssignCrew`/`UpdateEnv`/`EvaluateEdl`) so the daily
  seeded `step` can evolve the **stored** dynamic state — the FA-06 `OperateIsru` pattern. Consequently
  `CrewSnapshot::from_core` reads stored state only (no query-time `CrewInputs`); `from_parts` remains for
  pure-function tests. Still **core-only, composed-value** — no upstream-crate edge.
- **EDL & maintenance results via command (T037/T042)**: `EvaluateEdl` rolls `crew/edl-risk` and applies
  the loss-of-crew **inside `on_command`** (the host triggers the descent); `Maintain` stages crew-hours/
  spares that the next daily `step` consumes. The daily `step` owns continuous accrual + the SPE/ECLSS/
  anomaly rolls; the discrete EDL attempt is an explicit command — keeps the trigger explicit and the step
  deterministic.
- **VacateAsset keeps career records (FR-LSC-203)**: vacating **retires the asset but preserves** each
  crew member's career dose + deconditioning (they follow the astronaut across missions); a re-`AssignCrew`
  preserves the existing member record and resets only the per-mission psych load. Career dose persisting
  across missions is covered by `tests/radiation.rs::career_dose_carries_across_missions`.
- **CrewRoster age/sex stub-fed (I1, unchanged from plan)**: FA-05's current surface exposes personnel
  counts, not individual astronauts with age/sex; the per-astronaut `AstronautFacts` (age/sex feeding REID)
  enter as composed values, stub-fed in tests. The real FA-05 roster bridge is a future host/FA-05
  integration concern, not an FA-08 dependency.
- **Integration-test `advance` resumes across interrupts**: the crew slice is the first to raise interrupts
  *from the daily step* (SPE storm, grounding, anomaly, loss-of-crew). The test helper `CrewH::advance`
  acknowledges them and **resumes the remaining ticks** so a multi-day step runs the whole span (the
  scenario `drive` already does this). This is test-harness fidelity, not a slice-logic change.
- **T050 bench**: deferred (consistent with the FA-03…07 benches).
