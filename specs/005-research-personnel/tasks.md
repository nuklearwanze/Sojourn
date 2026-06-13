---
description: "Task list for Research & Personnel (FA-05)"
---

# Tasks: Research & Personnel (FA-05)

**Input**: Design documents from `/specs/005-research-personnel/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R16), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution v1.0.0 mandates determinism double-run, data schema+source
validation, and save/load round-trip; Principle VI requires the modelled-process mechanics be
demonstrable; every user story carries an Independent Test. Test tasks are included per story.

**Organization**: by user story (US1–US7). Crate layout per plan.md: `crates/sojourn-research`
(module, depends on `sojourn-core` only), `data/research/` + `data/tech/`, harness `research` flag.
**No kernel change** (commands via `ModulePayload`; new event classes are data-registry additions).

## Implementation status — 2026-06-13 (`/speckit-implement`)

**Verified green**: whole workspace `cargo test` (FA-01 + FA-02 + FA-03 + FA-05), `cargo clippy`
(determinism lints), `validate-data data/research` (schema + sources + **reachability sweep over 200
seeds** + no-combat + breakthrough-sources), `conformance --module research` (manifest /
**double-run identity** / **slice serde round-trip** / cadence), and `verify` + `roundtrip` on
`scenarios/research_program.ron` (double-run determinism + save/load identity across ~9.5 sim-years
of hire/allocate/start-program/heritage commands). **18 research integration tests pass** across all
seven stories.

| Phase | Status |
|---|---|
| Setup (T001–T003) | ✅ done |
| Foundational (T004–T010) | ✅ done — incl. the deterministic `step()` orchestration (G1) |
| US1 understanding (T011–T016) | ✅ done — growth/DR/synergy, gating, injection; `effective_ul` indirection (U1) |
| US2 TRL maturation (T017–T022) | ✅ done — gates, P50/P80, scalar reliability, heritage + derivative |
| US3 the tree is alive (T023–T028) | ✅ done — **constructive reachability** (300-seed test + 200-seed CI sweep), failure-that-teaches, basic-science-weighted breakthroughs |
| US4 the tide (T029–T031) | ✅ tide + publish verified (`tests/tide.rs`); dedicated `research_tide.ron` not authored (covered) |
| US5 personnel (T032–T034) | ✅ done — tacit-knowledge recompute, roster transitions, trait wiring |
| US6 astronaut pipeline (T035–T036) | ✅ done — select→train→ready + `CrewFeedback` dose-limit retire |
| US7 queries (T037–T038) | ✅ done — faction privacy + flyability gating |
| Polish (T039–T044) | ◑ T039 hash pinning, T040 conformance/verify/roundtrip, T041 validate-data all done; **T042 bench / T043 CI yaml / T044 full quickstart run remain** |

**Note**: the tech-tree ships the **full A1–A17 domains + a 12-node engineering subset** across 6
capability categories (Q3:A); the full ~150-node population is the documented data expansion behind
the same schema. No kernel change; six event classes added to the data registry.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1…US7 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/sojourn-research` crate: `Cargo.toml` (deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core; workspace lints) and `src/lib.rs` (`#![forbid(unsafe_code)]`, module decls); add `"crates/sojourn-research"` to workspace `members` in `Cargo.toml`.
- [x] T002 [P] Scaffold `data/research/` and `data/tech/` directories (with placeholder headers noting sourcing per Principle I).
- [x] T003 [P] Verify `clippy.toml` (HashMap/HashSet, libm) and `deny.toml` (no presentation crates) apply to the new crate via workspace lints; `cargo clippy -p sojourn-research` clean (empty crate).

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: no user story work begins until this phase completes.

- [x] T004 Implement id types (`FactionId`, `DomainId`, `TechId`, `ProgramId`, `PersonId`) in `crates/sojourn-research/src/ids.rs` (data-model §entities).
- [x] T005 [P] Add event classes `breakthrough` (Interrupt), `dead-end-confirmed`, `test-failure`, `program-milestone`, `trl-advance`, `publish` (LogOnly) to `data/kernel/event-classes.ron` per `contracts/research-commands.md`.
- [x] T006 Implement the tech-tree + domain + params loaders & schema in `crates/sojourn-research/src/tree.rs` (domains A1–A17, engineering nodes, capability categories, params, traits) with non-empty-source validation and prereq/synergy/category resolution (`contracts/tech-tree-data.md`).
- [x] T007 Define `ResearchSlice` (serde, ordered `BTreeMap` stores per data-model §5) and the `research_hash` data-version pin in `crates/sojourn-research/src/module.rs`.
- [x] T008 Implement `ResearchModule` SimModule skeleton in `crates/sojourn-research/src/module.rs`: manifest (`id="research"`, owned slice, streams `research/seed|overrun|test|breakthrough`, publishes `research/status`, emits the six classes), `init` (loads data, seeds via `research/seed`), the **deterministic `step()` orchestration skeleton** (advance subsystems in the fixed order domains → RP/DE → programs → campaigns → tide → insight → personnel/aging, driven by elapsed ticks for warp-invariance; each subsystem is a no-op until its phase fills it in), `save_slice`/`load_slice` (verify `research_hash`); export `ResearchModule`, `ResearchCommand`, `research_payload` from `lib.rs`.
- [x] T009 Wire harness `research` flag in `crates/sojourn-harness/src/scenario.rs`: install the module (loads `data/research` + `data/tech`); extend `TimedCommand` with a `research: Option<ResearchCommand>` arm via `Command::ModulePayload`; add `sojourn-research` to the harness `Cargo.toml`.
- [x] T010 [P] Author the param/trait data fixtures `data/research/params.ron` and `data/research/traits.ron` (sourced engineering defaults: RP/DE rates, overrun/breakthrough/tide/reliability params, trait modifiers) and the loader-backing schema from T006.

**Checkpoint**: workspace builds; FA-01/02/03 suites still green; empty Research module passes `conformance --module research`.

---

## Phase 3: User Story 1 — Understanding Before Capability (Priority: P1) 🎯 MVP

**Goal**: funded research raises domain ULs (diminishing returns + synergy); UL gates engineering availability; missions inject UL; deterministic.

**Independent Test**: fund a domain → UL rises with DR+synergy; a program below its UL floor is unavailable and becomes available when crossed; mission injection exceeds lab-only; double-run bit-identical.

### Tests for User Story 1 ⚠️

- [x] T011 [P] [US1] Domain-UL growth test (diminishing returns above the knee; synergy from coupled domains; sourced params) in `crates/sojourn-research/tests/domains.rs`.
- [x] T012 [P] [US1] Gating + injection test (program unavailable below UL floor / available above; `InjectUnderstanding` raises named domains beyond lab-only; unknown domain rejected) in `crates/sojourn-research/tests/domains.rs`.

### Implementation for User Story 1

- [x] T013 [US1] Implement Understanding Levels in `crates/sojourn-research/src/domains.rs`: per-`(faction,domain)` UL, diminishing-returns + synergy growth, and a UL gating helper that reads **effective UL** via an `effective_ul()` indirection (identity until the tacit-knowledge term lands in T033 — avoids hard-coding ground UL into gates/queries) (data-model §2, R3).
- [x] T014 [US1] Implement RP generation + portfolio allocation with efficiency multipliers (staffing/mismatch/facility) in `crates/sojourn-research/src/rp_de.rs` (FR-RESP-103).
- [x] T015 [US1] Implement `SetAllocation` and `InjectUnderstanding` command handling in `crates/sojourn-research/src/module.rs` (route via `on_command`; reject unknown domains); advance UL in `step`.
- [x] T016 [P] [US1] Author `data/research/domains.ron` (full A1–A17 with synergy links + DR params, sourced) and `scenarios/research_understanding.ron`.

**Checkpoint**: science-gates-engineering works and is deterministic — MVP.

---

## Phase 4: User Story 2 — Maturing a Technology Through TRL (Priority: P1)

**Goal**: programs advance a technology TRL 1–9 via gated test campaigns; cost/schedule P50/P80 + overruns; flyable only ≥ TRL 6; reliability tracks TRL+units+UL; heritage discounts derivatives.

**Independent Test**: start a program, advance tier by tier; verify cost/min-time/facility/UL gates, P50/P80 + seeded overruns, the scalar reliability function, sub-TRL-6 refusal, and heritage raising reliability + discounting a derivative.

### Tests for User Story 2 ⚠️

- [x] T017 [P] [US2] TRL-step gate test (cost + min-duration floor + facility + UL gate; schedule compression → overrun risk not sub-floor time) in `crates/sojourn-research/tests/programs.rs`.
- [x] T018 [P] [US2] Reliability + heritage test (scalar per-use ∈ [0,1] from TRL+units+UL; sub-TRL-6 unflyable; `RegisterHeritage` raises reliability toward ceiling and discounts a `derivative_of` program) in `crates/sojourn-research/tests/programs.rs`.

### Implementation for User Story 2

- [x] T019 [US2] Implement Engineering Programs + TRL steps (S-curve, min-time floor, facility/UL gates, P50/P80, seeded overruns via `research/overrun`) in `crates/sojourn-research/src/programs.rs` (data-model §3, R4).
- [x] T020 [US2] Implement DE generation/allocation to programs in `crates/sojourn-research/src/rp_de.rs`; `StartProgram`/`SetProgramPriority` handling in `module.rs` (gate on UL floors + tech prereqs; leapfrog via `UlSatisfiable`).
- [x] T021 [US2] Implement the scalar reliability curve + Flight Heritage (asymptotic ceiling, derivative discount) in `crates/sojourn-research/src/reliability.rs`; `RegisterHeritage` handling (R10); emit `trl-advance`/`program-milestone`.
- [x] T022 [P] [US2] Author the engineering-node subset `data/tech/tech-tree.ron` + `data/tech/capability-categories.ron` (≥2 paths/category, leapfrog seams, derivatives, sources) and `scenarios/research_program.ron`.

**Checkpoint**: technologies mature through TRL with realistic uncertainty; FA-04 contract (maturity/reliability) is producible.

---

## Phase 5: User Story 3 — The Tree Is Alive (Priority: P1)

**Goal**: seeded dead ends (hinted, parallel-mitigated, reachability guaranteed by construction), failures-that-teach, rare earned breakthroughs, leapfrogging — all deterministic per seed.

**Independent Test**: across many seeds, dead ends appear (risk rises before confirmation) while a parallel path stays viable; failures inject UL; breakthroughs fire only at seeded thresholds with basic-science investment; leapfrog reaches a higher tier; every capability category keeps ≥1 path in every seed.

### Tests for User Story 3 ⚠️

- [x] T023 [P] [US3] Dead-end + **constructive reachability** test: a seeded dead end stalls (risk index up, error bars stalled) with a viable parallel path; across ≥100 seeds every capability category retains ≥1 viable path — `crates/sojourn-research/tests/seeding.rs` (SC-003, R6).
- [x] T024 [P] [US3] Failure-that-teaches + breakthrough + leapfrog test (test failure injects UL; a TRL-7-demo failure emits `test-failure` with the political/PR-eligible payload (FA-09 hook); repeated failure → dead-end signal; breakthrough only with sustained basic-science at the seeded threshold, applied-only rarely; leapfrog via UL) in `crates/sojourn-research/tests/breakthrough.rs`.

### Implementation for User Story 3

- [x] T025 [US3] Implement **constructive dead-end seeding** (never close a category's last viable path) + breakthrough-threshold seeding in `crates/sojourn-research/src/seeding.rs` via the `research/seed` stream (R6).
- [x] T026 [US3] Implement test campaigns (seeded success/failure-that-teaches via `research/test`, UL injection, risk-index/dead-end signal, `test-failure`/`dead-end-confirmed` events) in `crates/sojourn-research/src/campaigns.rs` (R5).
- [x] T027 [US3] Implement insight-pressure breakthroughs (basic-science-weighted accrual, seeded thresholds, cluster-discount/early-unlock/hidden-path effects, sourced `breakthrough` event) and leapfrog availability in `crates/sojourn-research/src/breakthrough.rs` (R7/R8).
- [x] T028 [P] [US3] Add `validate-data research` reachability sweep (≥100 sampled seeds assert the constructive guarantee) to the harness; author dead-end/breakthrough params into `data/research/params.ron`.

**Checkpoint**: the tree is alive, fair (no bricked category) and seed-deterministic.

---

## Phase 6: User Story 4 — The Global Tide (Priority: P2)

**Goal**: World UL from aggregate + baseline; publish-vs-hold; cheaper catch-up; licence/partner interfaces grant knowledge credit (no money).

**Independent Test**: multi-faction World UL advances; publish accelerates it + emits a prestige-eligible event while hold retains lead; trailing faction researches cheaper; catch-up bounded by World UL.

### Tests for User Story 4 ⚠️

- [x] T029 [P] [US4] Tide test (World UL = baseline + aggregate; private = world + lead/lag; publish raises World UL + emits `publish`; hold retains lead; trailing catch-up cheaper, bounded by World UL) in `crates/sojourn-research/tests/tide.rs`.

### Implementation for User Story 4

- [x] T030 [US4] Implement the World tide (per-domain World UL advancement, publish/hold policy, catch-up discount) and the licence/partner/buy-in TRL/IP-credit interfaces (no money) in `crates/sojourn-research/src/tide.rs`; `SetPublishPolicy`/`License`/`Partner`/`BuyIn` handling (R9).
- [ ] T031 [P] [US4] Add `scenarios/research_tide.ron` (multi-faction publish/hold + catch-up). *(US4 verified via `tests/tide.rs`; dedicated scenario not yet authored.)*

**Checkpoint**: leads are bounded and the AI substrate exists.

---

## Phase 7: User Story 5 — People Make It Happen (Priority: P2)

**Goal**: personnel roster with traits; recruit/poach/train/age/morale; efficiency multipliers; tacit-knowledge loss as an effective-UL recompute.

**Independent Test**: traits shift documented outcomes; roster transitions deterministic; efficiency responds to staffing/mismatch/facility; disbanding reduces effective niche UL without corrupting ground UL.

### Tests for User Story 5 ⚠️

- [x] T032 [P] [US5] Personnel test (trait modifiers shift low-TRL/qual/breakthrough/overrun; hire/poach/train/age deterministic; poach relations-cost signal; tacit-knowledge recompute lowers effective UL but not ground UL) in `crates/sojourn-research/tests/personnel.rs`.

### Implementation for User Story 5

- [x] T033 [US5] Implement the personnel roster, traits, recruit/poach/train/age/morale transitions, RP/DE efficiency multipliers, and the tacit-knowledge effective-UL recompute in `crates/sojourn-research/src/personnel.rs` (R11); `Hire`/`Poach`/`Train`/`Retire`/`AssignLead` handling.
- [x] T034 [US5] Wire trait effects into program outcomes (low-TRL vs qual, breakthrough odds, overrun variance, reliability) across `programs.rs`/`breakthrough.rs`/`reliability.rs` via the `traits.ron` modifiers.

**Checkpoint**: people are the lever that turns funding into research.

---

## Phase 8: User Story 6 — The Astronaut Pipeline (Priority: P2)

**Goal**: select→train→ready→age career pipeline with dose/health budgets and the FA-08 feedback interface.

**Independent Test**: a candidate advances select→train→ready under facility/time gate; `CrewFeedback` deltas accumulate; crossing a limit removes from the ready pool; deterministic.

### Tests for User Story 6 ⚠️

- [x] T035 [P] [US6] Astronaut pipeline test (training facility/time-gated → ready; `CrewFeedback` dose/health deltas accumulate; over-limit leaves ready pool; double-run identical) in `crates/sojourn-research/tests/astronaut.rs`.

### Implementation for User Story 6

- [x] T036 [US6] Implement the astronaut career pipeline (stages, training, career dose/health/psych budgets, aging) and the `CrewFeedback` / `SelectAstronaut` / `TrainAstronaut` handlers in `crates/sojourn-research/src/astronaut.rs` (R12, `contracts/crew-interface.md`); add career-limit params to `data/research/params.ron`.

**Checkpoint**: a trained astronaut corps exists with the FA-08 seam declared.

---

## Phase 9: User Story 7 — Maturity/Heritage/Understanding On Tap (Priority: P3)

**Goal**: pure, faction-scoped read-only query surface for FA-04/06/09/10.

**Independent Test**: query maturity/heritage/understanding/program-status/personnel across factions; verify faction privacy, sub-TRL-6 flyability refusal, and fingerprint-unchanged purity.

### Tests for User Story 7 ⚠️

- [x] T037 [P] [US7] Query-surface test (maturity scalar reliability + flyability; faction privacy — no cross-faction private state; query between ticks leaves fingerprint unchanged) in `crates/sojourn-research/tests/queries.rs`.

### Implementation for User Story 7

- [x] T038 [US7] Implement `ResearchSnapshot::from_core` (via `with_slice`) + the pure faction-scoped query functions (`maturity`, `heritage`, `understanding`, `program_status`, `available_programs`, `personnel`, `tide`) in `crates/sojourn-research/src/query.rs` (FR-RESP-801, `contracts/maturity-queries.md`).

**Checkpoint**: FA-04/06/09 have their contract.

---

## Phase 10: Polish & Cross-Cutting Concerns

- [x] T039 [P] Research-data version pinning: content-hash all `data/research/*` + `data/tech/*`, pin/verify in saves (extends FA-02 hash guard); actionable mismatch error — `crates/sojourn-research/src/module.rs` (R15, edge case).
- [x] T040 Conformance + determinism wiring: `conformance --module research`; include the research scenarios in the harness `verify`/`roundtrip`/`mutate` gates — `crates/sojourn-harness/src/*`.
- [x] T041 [P] `validate-data research` full validation (schema + sources + synergy/prereq/category resolution + reachability sweep + **no node carries a combat/weapons capability category (Principle IX)** + **every breakthrough effect carries a sourced reference (Principle VIII)**) in `crates/sojourn-harness/src/main.rs`.
- [ ] T042 [P] Add `research` criterion bench (multi-faction full-roster + node-subset holds ≥1 sim-yr/min; queries < 50 ms) — `crates/sojourn-harness/benches/research.rs` (SC-008).
- [ ] T043 [P] Extend CI `.github/workflows/ci.yml`: `validate-data research`, `conformance --module research`, research determinism scenarios, reachability sweep, research bench (smoke).
- [ ] T044 [P] Run `quickstart.md` end-to-end; confirm SC-001…SC-008.

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps.
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T004–T010).
- **US1 (P3)** → after Foundational. The MVP.
- **US2 (P4)** → after US1 (programs gate on US1's UL; reliability/heritage here).
- **US3 (P5)** → after US2 (dead ends/failures/breakthroughs act on US1 ULs + US2 programs).
- **US4 (P6)** → after US1 (tide is a World-UL layer over domains).
- **US5 (P7)** → after US1/US2 (efficiency multipliers + trait effects feed RP/DE and programs).
- **US6 (P8)** → after Foundational (astronaut roster is personnel sub-state; mostly independent, lightest coupling to US5).
- **US7 (P9)** → after the stories whose state it surfaces (US1–US6).
- **Polish (P10)** → after the desired stories.

### Critical-path notes
- T006 (tree/domain loaders) gates everything; T013 (UL) gates US2–US5; T019 (programs) gates US3/US5 reliability/heritage; T025 (constructive seeding) gates the SC-003 reachability guarantee.
- The reachability sweep (T028/T041) verifies the T025 constructive guarantee — keep them in sync.
- The `step()` subsystem ordering + elapsed-tick dt handling (T008) is where double-run identity and warp-invariance are won; every later subsystem plugs into that fixed order rather than stepping independently.

### Parallel opportunities
- Setup: T002/T003 parallel.
- Foundational: T005/T010 parallel; T004 before T006/T007/T008.
- Within a story, `[P]` test tasks and `[P]` data-authoring tasks run in parallel.
- After Foundational + US1, US4 can proceed alongside the US2→US3 chain (different files); US6 is largely independent.

---

## Parallel Example: User Story 2

```text
# Tests first (parallel):
T017 TRL-step gates        → tests/programs.rs
T018 reliability + heritage → tests/programs.rs (separate cases)
# Data authoring in parallel with implementation (different files):
T022 tech-tree.ron + capability-categories.ron + scenarios/research_program.ron
```

---

## Implementation Strategy

### MVP first (US1)
Setup → Foundational → US1 → **STOP & validate**: funded research raises gated ULs deterministically — the science-gates-engineering core. Demoable.

### Incremental delivery
US1 (understanding) → US2 (TRL maturation) → US3 (alive tree) → US4 (tide) → US5 (personnel) → US6 (astronauts) → US7 (queries). The three P1 stories (US1–US3) are the research-as-process core; US7 closes the FA-04 contract.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests written first, must fail before implementation (Constitution).
- FA-01/02/03 suites must stay green (no kernel change; additive event classes only).
- Commit after each task or logical group; auto-commit is disabled (manual `/speckit-git-commit`).
