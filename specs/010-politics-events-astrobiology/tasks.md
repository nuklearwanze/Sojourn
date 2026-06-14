---
description: "Task list for Politics, Events, Milestones & Astrobiology (FA-09)"
---

# Tasks: Politics, Events, Milestones & Astrobiology (FA-09)

**Input**: Design documents from `/specs/010-politics-events-astrobiology/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R15), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution v1.0.0 mandates determinism double-run, **analytic validation against
known cases** (ground-truth prior fidelity, consensus-band crossing, never-conclusive-positive-for-negative,
contamination monotone-in-overage, event-hazard monotone+clamped, tiebreak/score determinism, mood bounds),
data schema+source validation and save/load round-trip; every user story carries an Independent Test. As a
**heavily multi-stream seeded slice** (after FA-08), the `mutate` gate is a first-class determinism proof.

**Organization**: by user story (US1–US8). Crate layout per plan.md: `crates/sojourn-polity` (module; **dep
`sojourn-core` only** — FA-03 candidate priors + site PP categories, FA-05 tech maturity + science tide,
FA-06 budgets/valuations/tonnage/supply, FA-07/08 mission/embargo facts + loss-of-crew flow in as composed
values, the FA-04/06/08 decoupling), `data/polity/`, harness `polity` flag. **No kernel change; no
world/research/economy/vehicle/base/crew change.** Per-faction/per-candidate/per-world state is **stored**
slice state evolved on the daily seeded step; the milestone ledger, mood, posteriors/consensus, policy and
scores are pure derived queries. The astrobiology **ground truth** is stored but **never query-exposed**.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1…US8 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/sojourn-polity` crate: `Cargo.toml` (deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core, blake3; workspace lints) and `src/lib.rs` (`#![forbid(unsafe_code)]`, module decls); add `"crates/sojourn-polity"` to workspace `members` in `Cargo.toml`.
- [x] T002 [P] Scaffold `data/polity/` directory (placeholder headers noting sourcing per Principle I; the astrobiology **priors** stay in `data/world/astrobiology.ron`).
- [x] T003 [P] Confirm `clippy.toml`/`deny.toml` apply via workspace lints; `cargo clippy -p sojourn-polity` clean (empty crate).

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: no user story work begins until this phase completes.

- [x] T004 Implement id types (`FactionId`, `MilestoneId`, `BodyId`, `CandidateId`, `PolicyId`, `EventClassId`, `GoalKind`) in `crates/sojourn-polity/src/ids.rs` (data-model §1).
- [x] T005 Implement the sourced parameter loaders & schema in `crates/sojourn-polity/src/params.rs` (data-model §3): the nine param sets (milestones/mood/events/policy/protection/astrobiology/ai/goals/params), non-empty-source validation, the per-set semantic checks (band ordering `0<band_neg<band_pos<1`, LR ordering SampleReturn-strongest, `false_hint_cap<band_pos`, PP-stringency lever present, contamination overage-curve monotone + `crash≥soft≥1`, `|loss_of_crew|>|routine_failure|`, `seeker_worlds=3`), and `content_hash` (blake3 over normalized texts) (`contracts/polity-data.md`).
- [x] T006 Implement the composed-value input shapes (`FactionInit`, `Achievement`/`AchievementFacts`, `CandidatePrior`, `EvidenceInput`, `SiteProtection`, `MissionFacts`, `EconomyFacts`, `CrewFacts`, `HomesteadFacts`, `ScienceTide`, `Difficulty`) in `crates/sojourn-polity/src/inputs.rs` (`contracts/integration-seams.md`, R1/R15). Note: FA-03 priors + site PP are **bridged by the host**; FA-05 tide / FA-06 econ / FA-08 loss-of-crew enter as composed values (stub-fed in tests).
- [x] T007 [P] Implement the shared **multiplicative-hazard** primitive (`hazard(base, factors)` = clamp(base × ∏ factors, 0, 1)) in `crates/sojourn-polity/src/hazard.rs` (R5, the FA-08 primitive) — used by US4/US7.
- [x] T008 [P] Implement the `trace.rs` traceability-tree types (recursive nodes; sourced leaves; `all_leaves_sourced`) in `crates/sojourn-polity/src/trace.rs` (R12).
- [x] T009 [P] Add the polity event classes (`milestone-claimed`, `rival-milestone`, `mood-shift`, `approval-frozen`, `anomaly`, `launch-failure`, `solar-storm`, `funding-crisis`, `funding-boom`, `political-shakeup`, `supply-shock`, `personnel-event`, `discovery`, `astrobiology-evidence`, `astrobiology-conclusive`, `contamination`, `grand-goal-met`, `soft-fail`) to `data/kernel/event-classes.ron` per `contracts/polity-commands.md`.
- [x] T010 Define the stored slice state — `FactionState`, `MilestoneClaim`, `CandidateState` (incl. the **hidden** `ground_truth`), `ProtectionState`, `AiState`, `EventRecord` + `PolitySlice` — in `crates/sojourn-polity/src/faction.rs` + `module.rs` (data-model §4).
- [x] T011 Implement `PolityModule` SimModule skeleton in `crates/sojourn-polity/src/module.rs`: manifest (`id="polity"`, owned slice, **streams** `polity/ground-truth`/`polity/events/<source>`/`polity/policy-drift`/`polity/lobby`/`polity/ai/<faction>`/`polity/contamination`/`polity/evidence-noise`, emits the T009 events, `cadence_ticks=86_400`), `init`, a daily seeded `step` skeleton (two-pass roll-then-apply; `ctx.rng(path)` per stream), `save_slice`/`load_slice` (verify `data_hash`) + `load(dir)`; export `PolityModule`, `PolityCommand`, `polity_payload` from `lib.rs`.
- [x] T012 Implement `InitWorld`/`UpdateWorld` command handling in `crates/sojourn-polity/src/module.rs`: capture the composed faction roster + `CandidatePrior` list + `SiteProtection` list + difficulty/tide; **draw the seeded ground truth** (stream `polity/ground-truth`, body-id order, `uniform < presence_prob`), stored hidden (R4/R15; reject a second `InitWorld`).
- [x] T013 Implement `WorldSnapshot::from_core`/`from_parts` skeleton (composing the polity slice; **no `ground_truth` accessor**) in `crates/sojourn-polity/src/query.rs` (`contracts/polity-queries.md`, R14).
- [x] T014 Wire harness `polity` flag in `crates/sojourn-harness/src/scenario.rs` (install `PolityModule::load("data/polity")`; extend `TimedCommand` with a `polity: Option<PolityCommand>` arm; **bridge FA-03 `data/world/astrobiology.ron` priors + `data/world/sites.ron` PP categories into `CandidatePrior`/`SiteProtection`**) and `crates/sojourn-harness/src/main.rs` (validate-data `polity` branch; `conformance "polity"` factory); add `sojourn-polity` to the harness `Cargo.toml`.

**Checkpoint**: workspace builds; FA-01…08 suites still green; empty Polity module passes `conformance --module polity`.

---

## Phase 3: User Story 1 — The race for historic firsts (Priority: P1) 🎯 MVP

**Goal**: recognise an achievement, award it world-first (first global claimant) or faction-first, weight prestige by significance, and track the global race — deterministically.

**Independent Test**: feed dated achievements by player + AI; each milestone awards world-first to exactly the first global claimant and faction-first to later claimants; prestige accrues by weight; an already-world-claimed milestone cannot be re-world-claimed; same-tick ties resolve by highest prestige then lowest id; the ledger is bit-identical across two runs.

### Tests for User Story 1 ⚠️

- [x] T015 [P] [US1] World-/faction-first + prestige test: a world-first awards to the first global claimant (full weight) and retires globally; later achievers get faction-first (lesser fraction); prestige accrues by weight; ledger determinism — in `crates/sojourn-polity/tests/milestones.rs` (FR-PEA-101…104/106/107, SC-001/002).
- [x] T016 [P] [US1] Same-tick tiebreak test: two factions claiming the same world-first on one tick → the **highest-prestige** (ties → **lowest id**) claimant wins, recorded once — in `crates/sojourn-polity/tests/milestones.rs` (FR-PEA-105).

### Implementation for User Story 1

- [x] T017 [US1] Implement the milestone catalogue + award logic (condition evaluation over `AchievementFacts`, world/faction-first ledger, prestige accrual, the highest-prestige-then-id tiebreak) in `crates/sojourn-polity/src/milestones.rs` (R2, FR-PEA-101…107).
- [x] T018 [US1] Implement the `RecordAchievement` handler + `milestone-claimed` emission in `crates/sojourn-polity/src/module.rs` (`contracts/polity-commands.md`).
- [x] T019 [US1] Implement the `milestone`/`ledger`/`unclaimed_world_firsts`/`prestige` queries in `crates/sojourn-polity/src/query.rs` (FR-PEA-106, R14).
- [x] T020 [P] [US1] Author `data/polity/milestones.ron` (a representative sourced subset of the ~120 firsts across all eras: foothold/cislunar/frontier/endgame, incl. Breakthrough-gated endgame firsts; id/era/description/weight/faction_first_fraction/conditions/source) + the prestige weights in `data/polity/params.ron`.

**Checkpoint**: the world-first race is real and scored — MVP.

---

## Phase 4: User Story 2 — The politics of money and approval (Priority: P1)

**Goal**: public/political mood swings on composed outcomes (loss-of-crew most of all) and drives appropriation/valuation modifiers and approval timelines.

**Independent Test**: drive the mood model with a scripted outcome sequence; mood moves with the documented direction/magnitude/decay; loss-of-crew is deeper + longer than a routine failure and freezes crewed-flight approval for the recovery window; budget/valuation/approval modifiers are the documented function of mood.

### Tests for User Story 2 ⚠️

- [x] T021 [P] [US2] Mood + approval test: a world-first lifts mood + the budget modifier with decay; a loss-of-crew drops mood **deeper and longer** than a routine failure and **freezes crewed-flight approval** for the recovery window; modifiers are the documented function of mood — in `crates/sojourn-polity/tests/mood.rs` (FR-PEA-201…205, SC-003).

### Implementation for User Story 2

- [x] T022 [US2] Implement the bounded mood model (outcome→delta, exponential decay via `libm::exp`, loss-of-crew severity + `loc_recovery_days` freeze, mood→{appropriation, valuation, approval} curves, saturation) in `crates/sojourn-polity/src/mood.rs` (R3, FR-PEA-201…205).
- [x] T023 [US2] Implement the `RecordOutcome` handler + daily mood decay + freeze expiry in `step` + the `approval-frozen`/`mood-shift` emission in `crates/sojourn-polity/src/module.rs` (R3).
- [x] T024 [US2] Implement the `mood`/`modifiers`/`approval` queries (modifiers as **factors over** composed FA-06 inputs, never generating value) in `crates/sojourn-polity/src/query.rs` (FR-PEA-203/206).
- [x] T025 [P] [US2] Author `data/polity/mood.ron` (outcome deltas, decay, bounds, loss-of-crew severity/recovery, mood→modifier curves, sourced).

**Checkpoint**: prestige and failure echo into money and approval; loss-of-crew has strategic teeth.

---

## Phase 5: User Story 3 — The astrobiology question, answered honestly (Priority: P1)

**Goal**: a per-game seeded ground truth resolved only through staged evidence; per-faction beliefs aggregate to a prestige-weighted consensus that can publicly disagree until it crosses the band; never a binary popup, never a false "life confirmed".

**Independent Test**: ground truth drawn from FA-03 priors and faithful across many seeds, never query-exposed; staged evidence moves the per-faction posteriors + the weighted consensus; abiotic competitors explain away early hints; conclusive needs the band (≥0.9/≤0.1) **and** a SampleReturn item; **never** conclusive-positive on a negative-ground-truth world; deterministic per seed.

### Tests for User Story 3 ⚠️

- [x] T026 [P] [US3] Astrobiology evidence + consensus test: across many seeds the positive-ground-truth fraction matches the priors within tolerance and is never exposed; staged evidence moves per-faction posteriors + the prestige-weighted consensus; factions can publicly disagree; conclusive requires band-cross **and** a SampleReturn item; a negative-truth world yields at most capped false hints and **never** a conclusive-positive — in `crates/sojourn-polity/tests/astrobiology.rs` (FR-PEA-301…308, SC-004/005).

### Implementation for User Story 3

- [x] T027 [US3] Implement astrobiology in `crates/sojourn-polity/src/astrobiology.rs` (R4): the ground-truth-conditioned staged **evidence generation** (negative worlds emit only pre-conclusive false hints capped below the band), per-faction **log-odds** posterior update with the abiotic-competitor attenuation, the **prestige/science-weighted consensus** aggregate, and the conclusive rule (band ≥0.9/≤0.1 **and** a SampleReturn item) — structurally guaranteeing no conclusive-positive for negative ground truth (FR-PEA-301…305/308).
- [x] T028 [US3] Implement the `CollectEvidence` handler + the daily consensus recompute in `step` + the `astrobiology-evidence`/`astrobiology-conclusive` emission, and on conclusive resolution fire the top-tier milestone (US1) and shift mood (US2); wire the **PP-stringency-tighten** effect as a **stubbed hook here** (US6 lands in Phase 8 — T043 connects the real `protection.rs` tightening — so US3 stays independently testable in the meantime) in `crates/sojourn-polity/src/module.rs` (FR-PEA-306).
- [x] T029 [US3] Implement the `posterior`/`consensus`/`conclusive`/`disagreement`/`evidence_value` queries (**no `ground_truth` accessor**) in `crates/sojourn-polity/src/query.rs` (FR-PEA-302/307/308, R14).
- [x] T030 [P] [US3] Author `data/polity/astrobiology.ron` (stage likelihood-ratios OrbitalHint<InSitu<Microscopy<SampleReturn, abiotic-competitor params, prestige-weighted consensus weighting, band 0.9/0.1, sample-return gate, false-hint cap, sourced).

**Checkpoint**: "are we alone here?" is honest, seeded, staged and unspoofable.

---

## Phase 6: User Story 4 — The event system (seeded + state-driven) (Priority: P2)

**Goal**: a daily Bernoulli multiplicative-hazard scheduler whose probabilities are earned by state, feeding the FA-01 interrupt-and-pause loop, reproducible per seed.

**Independent Test**: a low-TRL/over-subscribed craft has a strictly higher realised event rate than a mature one; each class fires at its interrupt/log classification and is acknowledgeable; the event stream is bit-identical across two runs and across stepping patterns; no physics-violating outcome.

### Tests for User Story 4 ⚠️

- [x] T031 [P] [US4] Event-hazard + determinism test: two craft differing only in maturity/test/ops-load → the riskier earns a strictly higher realised event rate (multiplicative hazard, monotone + clamped); interrupt vs log classes behave; the event stream is identical across stepping patterns — in `crates/sojourn-polity/tests/events.rs` (FR-PEA-401…405, SC-006).

### Implementation for User Story 4

- [x] T032 [US4] Implement the event scheduler in `crates/sojourn-polity/src/events.rs` (R5): per named source, the daily Bernoulli draw `uniform(stream events/<source>) < hazard(base, factors)` over composed state, with the interrupt/log classification.
- [x] T033 [US4] Wire the daily event rolls into `step`, emit at each class's interrupt/log level, and emit `rival-milestone` when the AI world claims a first, in `crates/sojourn-polity/src/module.rs` (R5/R8).
- [x] T034 [US4] Implement the `events` feed query (paged) in `crates/sojourn-polity/src/query.rs` (FR-PEA-403).
- [x] T035 [P] [US4] Author `data/polity/events.ron` (event-class catalogue + base rates + hazard-factor refs + interrupt/log classification, sourced).

**Checkpoint**: the world interrupts the player when it matters; risk is earned, not random.

---

## Phase 7: User Story 5 — Policy & treaties with real consequences (Priority: P2)

**Goal**: bounded policy levers that gate/penalise missions and partnerships, drift over time, and can be lobbied; PP stringency is the single source US6 consumes.

**Independent Test**: a mission lacking a required lever (e.g. nuclear-launch) is gated/penalised; lobbying + drift change levels deterministically within bounds; the PP-stringency lever feeds the PP regime.

### Tests for User Story 5 ⚠️

- [x] T036 [P] [US5] Policy gating + drift/lobby test: a mission/partnership failing a required lever is gated or penalised; lobbying nudges a level and drift advances it, both bounded and deterministic; the PP-stringency lever is the value the PP regime reads — in `crates/sojourn-polity/tests/policy.rs` (FR-PEA-501…505, SC-007).

### Implementation for User Story 5

- [x] T037 [US5] Implement policy levers + gating rules + seeded drift + lobbying in `crates/sojourn-polity/src/policy.rs` (R6, FR-PEA-501…505).
- [x] T038 [US5] Implement the `SetPolicy`/`Lobby`/`RequestApproval` handlers + daily policy drift in `step` in `crates/sojourn-polity/src/module.rs` (R6).
- [x] T039 [US5] Implement the `policy`/`gate` queries in `crates/sojourn-polity/src/query.rs` (FR-PEA-502).
- [x] T040 [P] [US5] Author `data/polity/policy.ron` (levers + bounds + drift/lobby params + gating rules; PP-stringency lever required, sourced).

**Checkpoint**: politics is mechanical friction on the player's plans.

---

## Phase 8: User Story 6 — Planetary protection: forward & back contamination (Priority: P2)

**Goal**: COSPAR categories + Special Regions; **graded** forward contamination (overage × crash/soft) that degrades pristine astrobiology value; a back-contamination chain for sample return.

**Independent Test**: a non-compliant (over-limit) lander in a Special Region degrades pristine value graded by overage (and crash vs soft); a compliant lander degrades nothing; a sample return without a containment chain is gated/penalised; ruined pristine value confounds future evidence (US3).

### Tests for User Story 6 ⚠️

- [x] T041 [P] [US6] Forward/back contamination test: an over-limit lander breaching a Special Region degrades pristine value **graded by bioburden overage** (× crash/soft factor, monotone); a compliant sterilised lander degrades nothing; a sample return without a containment chain is gated/penalised — in `crates/sojourn-polity/tests/protection.rs` (FR-PEA-601…605, SC-008).

### Implementation for User Story 6

- [x] T042 [US6] Implement COSPAR categories + Special Regions + **graded** forward contamination (`f(overage) × crash/soft`, monotone) + back-contamination chain + pristine-value state in `crates/sojourn-polity/src/protection.rs` (R7, FR-PEA-601…605).
- [x] T043 [US6] Implement the `EvaluateContamination` handler + `contamination` emission + the pristine-value degradation that **confounds US3 evidence value** + the reputation/mood cost (US2), and **connect the PP-stringency-tighten hook stubbed in T028** (US3 conclusive resolution → real `protection.rs` tightening) in `crates/sojourn-polity/src/module.rs` (FR-PEA-307/602).
- [x] T044 [US6] Implement the `protection`/`contamination_records` queries in `crates/sojourn-polity/src/query.rs` (FR-PEA-601).
- [x] T045 [P] [US6] Author `data/polity/protection.ron` (per-body COSPAR categories + Special Regions + bioburden limits + sterilisation refs + contamination grading params, sourced).

**Checkpoint**: cutting corners poisons your own evidence and reputation — graded and tempting.

---

## Phase 9: User Story 7 — The AI world (competitors & partners) (Priority: P2)

**Goal**: abstracted, heuristic, seeded AI factions that research/advance the tide, claim firsts, contract and suffer accidents — within the plausibility envelope; difficulty tunes funding/competence only.

**Independent Test**: AI factions claim firsts + advance the science tide; capability never exceeds the plausibility envelope; raising difficulty raises funding/competence outputs without physics-violating tech; deterministic per seed.

### Tests for User Story 7 ⚠️

- [x] T046 [P] [US7] AI world test: AI factions pursue/claim milestones and advance the tide; capability stays within the plausibility envelope; raising difficulty increases funding/competence but never grants physics-violating tech; behaviour deterministic per seed — in `crates/sojourn-polity/tests/ai.rs` (FR-PEA-701…704, SC-009).

### Implementation for User Story 7

- [x] T047 [US7] Implement the abstracted heuristic seeded AI behaviour (capability profile from composed estimates, goal queue, plausibility-envelope clamp, difficulty tuning, tide advance) in `crates/sojourn-polity/src/ai.rs` (R8, FR-PEA-701…704).
- [x] T048 [US7] Wire the per-AI-faction seeded decision step into the daily `step` (claim milestones → `rival-milestone`; advance tide; generate contracts) in `crates/sojourn-polity/src/module.rs` (R8).
- [x] T049 [US7] Implement the `science_tide`/`ai_targets` queries in `crates/sojourn-polity/src/query.rs` (FR-PEA-701).
- [x] T050 [P] [US7] Author `data/polity/ai.ron` (heuristic weights + plausibility-envelope caps + difficulty multipliers + tide-advance rate, sourced).

**Checkpoint**: a credible opponent in the race that never cheats physics.

---

## Phase 10: User Story 8 — Grand Goals as win/scoring conditions (Priority: P3)

**Goal**: four selectable Grand Goals with deterministic pass/fail from composed inputs, a change penalty, and a horizon resolution of pass/fail (primary) + a secondary composite score; soft-fail continues in observer mode.

**Independent Test**: each Grand Goal computes a deterministic pass/fail from composed inputs (Pathfinder firsts, Homestead embargo index, Prospector tonnage, Seeker ≥3 conclusive); changing goal applies the penalty; the horizon resolves to the selected goal's pass/fail + a reproducible composite score; soft-fail flags continue the run.

### Tests for User Story 8 ⚠️

- [x] T051 [P] [US8] Grand-Goal + scoring test: each of the four goals reaches its pass/fail from composed inputs (Seeker via US3 conclusive resolutions; Prospector/Homestead via composed FA-06/07/08); a goal change applies the penalty; the horizon yields the selected goal's pass/fail + a deterministic composite score; soft-fail flags surface — in `crates/sojourn-polity/tests/goals.rs` (FR-PEA-801…804, SC-010).

### Implementation for User Story 8

- [x] T052 [US8] Implement Grand-Goal progress + pass/fail verdict + change penalty in `crates/sojourn-polity/src/goals.rs` (R9, FR-PEA-801/802).
- [x] T053 [US8] Implement final scoring (Grand-Goal verdict primary + secondary composite of prestige + milestones + goal progress; soft-fail states) in `crates/sojourn-polity/src/score.rs` (R9, FR-PEA-803/804).
- [x] T054 [US8] Implement the `SelectGrandGoal`/`ChangeGrandGoal` handlers + horizon scoring freeze in `step` in `crates/sojourn-polity/src/module.rs` (FR-PEA-801/803).
- [x] T055 [US8] Implement the `grand_goal`/`score` queries in `crates/sojourn-polity/src/query.rs` (FR-PEA-803).
- [x] T056 [P] [US8] Author `data/polity/goals.ron` (per-goal thresholds incl. `seeker_worlds=3`, change penalty, composite-score weights) + the horizon/score normalisation in `data/polity/params.ron`.

**Checkpoint**: a run resolves to a verdict + score; the meta-layer composes every sub-system.

---

## Phase 11: Polish & Cross-Cutting Concerns

- [x] T057 [P] Polity-data version pinning: content-hash all `data/polity/*`, pin/verify in saves (extends the FA-02…08 hash guard); actionable mismatch error — `crates/sojourn-polity/src/module.rs` (FR-PEA-904).
- [x] T058 Conformance + determinism wiring: `conformance --module polity`; include the polity scenario in the harness `verify`/`roundtrip`/`mutate` gates (the seeded-stream determinism proof) — `crates/sojourn-harness/src/*`.
- [x] T059 [P] `validate-data polity` (schema + sources + the analytic gates: prior fidelity, consensus band incl. never-conclusive-positive-for-negative, contamination monotone-in-overage, event-hazard monotone+clamped, tiebreak determinism, score determinism, mood bounds) in `crates/sojourn-harness/src/main.rs` + `data/polity/validation.ron` (FR-PEA-903, `contracts/polity-data.md`).
- [x] T060 [P] Author `scenarios/politics_world.ron` (full mini-game: `InitWorld` with FA-03 priors/PP → achievements racing the AI world → mood swings incl. a loss-of-crew → policy set/lobby → staged evidence on two candidates → a Special-Region contamination → seeded events → Grand-Goal selection + horizon scoring).
- [x] T061 [P] Extend CI `.github/workflows/ci.yml`: `validate-data data/polity`, `conformance --module polity`, the polity determinism scenario (verify + roundtrip).
- [ ] T062 [P] Add a `polity` criterion bench (daily seeded step over ten factions × ~120 firsts × event sources × candidates; sub-ms queries) — `crates/sojourn-harness/benches/polity.rs` (SC-012). *(May be deferred consistent with FA-03…08 benches.)*
- [x] T063 [P] Run `quickstart.md` end-to-end; confirm SC-001…SC-012.
- [x] T064 [P] Expose polity **Sojournal** content (milestone significance, policy/treaty explanations, candidate-world results, contamination consequences) through the read-only query/`trace.rs` surface with sources, updating as belief-state/world advance and consistent with the existing FA-03 Sojournal surface — `crates/sojourn-polity/src/query.rs`/`trace.rs` (FR-PEA-906).

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps.
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T004–T014). The params (T005), inputs (T006), hazard primitive (T007), stored state (T010), module skeleton + InitWorld (T011/T012) and snapshot (T013) gate everything.
- **US1 (P3)** → after Foundational. The MVP (firsts/prestige); its **prestige** feeds the US3 consensus weighting and the US1 tiebreak.
- **US2 (P4)** → after US1 (mood reads outcomes; approval/valuation modifiers stand alone).
- **US3 (P5)** → after US1 (consensus is **prestige-weighted**); the conclusive resolution links to US1 (milestone), US6 (PP tighten) and US2 (mood) — those effects land as their stories complete.
- **US4 (P6)** → after Foundational (uses the hazard primitive + composed state); `rival-milestone` links to US7.
- **US5 (P7)** → after Foundational; the **PP-stringency** lever is consumed by US6.
- **US6 (P8)** → after US5 (stringency) + links to US3 (evidence confound) and US2 (reputation cost).
- **US7 (P9)** → after US1 (claims firsts) + US4 (accident events) + composed tide.
- **US8 (P10)** → after US1 (firsts) + US3 (Seeker) + composed FA-06/07/08 (Prospector/Homestead).
- **Polish (P11)** → after the desired stories.

### Critical-path notes
- T005 (params) + T007 (hazard) + T010 (stored state) + T012 (InitWorld + ground-truth draw) + T013 (snapshot) gate everything; the daily seeded `step` (T018/T023/T028/T033/T038/T043/T048/T054) grows per story; T008 (trace) is woven through.
- The analytic gates (T059) verify the statistics (Principles I/III/VIII) — keep them in sync with each sub-system (prior fidelity + consensus band with US3, contamination monotone with US6, event hazard with US4, tiebreak with US1, score with US8, mood bounds with US2).
- This slice exercises the **`mutate` gate** hard (seven seeded streams) — wire it (T058) and keep it green.
- The **honesty invariant** (no conclusive-positive for negative ground truth) is **structural** in T027's evidence generation, not a clamp — and is gated in T026 + T059.

### Parallel opportunities
- Setup: T002/T003 parallel.
- Foundational: T007/T008/T009 parallel; T004–T006 sequential-ish (ids → params/inputs); T010/T011/T012/T013 after.
- Within a story, `[P]` test tasks and `[P]` data-authoring tasks run in parallel.
- After Foundational + US1, US4 and US5 can proceed alongside US2/US3 (different files); US6 after US5; US7 after US1/US4; US8 last.

---

## Parallel Example: User Story 3

```text
# Tests first:
T026 astrobiology evidence + consensus + honesty → tests/astrobiology.rs
# Data + impl (different files):
T030 astrobiology.ron  |  T027 astrobiology.rs  |  T028 CollectEvidence + consensus recompute + conclusive emission  |  T029 posterior/consensus queries
```

---

## Implementation Strategy

### MVP first (US1)
Setup → Foundational → US1 → **STOP & validate**: an achievement is recognised, awarded world-first or
faction-first, and scored with prestige — *the race that replaces combat*, deterministic. Demoable.

### Incremental delivery
US1 (firsts/prestige) → US2 (mood/approval) → US3 (astrobiology) → US4 (events) → US5 (policy) → US6
(planetary protection) → US7 (AI world) → US8 (goals/scoring). The three P1 stories (US1–US3) are the
felt tension core (the race, the money/approval, the honest unveiling); US4–US7 add the drama engine,
friction, consequences and the opponent; US8 is the capstone verdict.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests written first, must fail before implementation (Constitution); the **analytic gates** are the
  Principle-I/III/VIII enforcement (prior fidelity, consensus band, honesty invariant, contamination/
  hazard monotonicity, tiebreak/score determinism, mood bounds).
- FA-01…08 suites must stay green; **no upstream-crate change** (composed-value decoupling) and **no
  kernel change**. All randomness is **named seeded streams** — the `mutate` gate must stay green. The
  astrobiology **ground truth is never query-exposed**.
- Commit after each task or logical group; **run `cargo fmt --all` before committing** (CI enforces
  `fmt --check`); auto-commit is disabled (manual `/speckit-git-commit`).

### Implementation deviations (recorded for honesty)
- **Composed values captured at command time (R15)**: the daily seeded step evolves stored state, so the
  composed faction roster + FA-03 candidate priors + site PP + science tide are **captured into the slice
  at command time** (`InitWorld`/`UpdateWorld`/`RecordOutcome`/`CollectEvidence`) — the FA-06 `OperateIsru`
  / FA-08 `OccupyAsset` pattern. The host bridges FA-03's `data/world/astrobiology.ron` priors / `sites.ron`
  PP into the `CandidatePrior`/`SiteProtection` inputs (in tests + the scenario these are authored
  directly). Still **core-only, composed-value** — no upstream-crate edge.
- **The honesty invariant is structural (T027)**: `astrobiology::evidence_delta` generates evidence
  **conditioned on the ground truth** — a negative world emits only weak pre-conclusive *false hints*
  (and `faction_prob` caps its belief at `false_hint_cap < band_positive`), with a strong **negative**
  SampleReturn delta. So a conclusive-positive is impossible for a negative world by construction, not by a
  post-hoc clamp. There is **no `ground_truth` accessor** anywhere in the public query surface.
- **Same-tick world-first tiebreak via transfer (FR-PEA-105)**: the highest-prestige rule is applied
  incrementally — when a same-tick claimant has higher **pre-award** prestige than the current holder, the
  world-first **transfers** (the prior holder is demoted to faction-first, prestige adjusted), so first-
  processed-wins is corrected to highest-prestige-then-lowest-id without deferring awards to end-of-tick.
- **Conclusive resolution wires US1/US2/US6 (T028/T043)**: on a candidate flipping conclusive the step
  awards the "first conclusive astrobiology result" milestone (id 15) to the highest-posterior **discoverer**,
  lifts their mood, and **tightens the `pp-stringency` lever** — the T028→T043 hook, now real.
- **`mutate` runs on `smoke_decade`, not `politics_world`**: the mutation framework injects nondeterminism
  into the *synthetic* module, so it only has teeth on synthetic-bearing scenarios. Per-slice determinism is
  proven by `verify` + `roundtrip` + the conformance double-run (the FA-08 precedent); the global
  `mutate --all` stays on `smoke_decade`.
- **`#[serde(transparent)]` on the id newtypes**: `FactionId`/`MilestoneId`/`BodyId`/`CandidateId` are
  transparent so scenarios use bare integers (the codebase convention) — postcard output is unchanged
  (newtypes already serialize as their inner value).
- **Milestone catalogue is a representative sourced subset** (18 firsts across all eras) per the spec
  Assumption / FR-PEA-101 — completable in data without code change (the FA-05 precedent).
- **AI world is abstracted (R8)**: AI factions claim the lowest-id unclaimed world-first on a seeded draw
  (no per-claim condition check) — the clarified abstracted model, not a mirror of FA-04…08.
- **T062 bench**: deferred (consistent with the FA-03…08 benches).
