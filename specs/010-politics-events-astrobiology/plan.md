# Implementation Plan: Politics, Events, Milestones & Astrobiology (FA-09)

**Branch**: `010-politics-events-astrobiology` | **Date**: 2026-06-14 | **Spec**: `specs/010-politics-events-astrobiology/spec.md`
**Input**: Feature specification from `/specs/010-politics-events-astrobiology/spec.md`

## Summary

Build `sojourn-polity`, **the non-combat competitive and narrative layer** — the system that makes a
weaponless game *tense*. It owns three sources of pressure and the machinery around them: the **race for
~120 historic firsts** (world-first > faction-first, with a global ledger so the player races an AI
world), the **politics of money and approval** (public/political mood that swings on successes and
failures — loss-of-crew most of all — and drives budgets, valuations, approvals, policy and treaties),
and the **honest astrobiology question** (a per-game **seeded ground truth** on candidate worlds resolved
only through a staged, probabilistic, mission-driven evidence process against abiotic competitors, with a
**per-faction belief → prestige-weighted community consensus** that can publicly disagree until it crosses
a confidence band). Around these sit the **seeded, state-driven event system** (the daily Bernoulli
multiplicative-hazard scheduler feeding the FA-01 interrupt-and-pause loop), the **COSPAR planetary-
protection regime** (categories I–V, Special Regions, **graded** forward contamination + back-contamination
chain), the **abstracted AI world** (heuristic, seeded rivals that research/build/fly/contract/race
without cheating physics), and the **Grand Goals** (Pathfinder/Homestead/Prospector/Seeker) that resolve a
run to a **pass/fail verdict plus a secondary composite score** at the horizon.

Architecturally this is the **capstone slice**: the broadest by surface, but it follows the now-settled
FA-04 C1 / FA-06 R1 / FA-08 decoupling exactly — **`sojourn-polity` depends only on `sojourn-core`**.
Every upstream gameplay fact (achievements/launches/landings, research-tech maturity + the science tide,
budgets/valuations/markets/supply, FA-03 candidate-world astrobiology **priors** and site **planetary-
protection categories**, FA-08 **loss-of-crew**) flows in as **composed values / opaque inputs** the host
assembles — so the dependency graph gains **no new cross-gameplay-crate edge** and every sub-system is
unit-testable with stubs. This slice **consumes** FA-03's `data/world/astrobiology.ron` priors and site PP
data (it does not redefine them) and is the **first heavily multi-stream seeded slice after FA-08**: all
stochastics (ground-truth draw, event scheduling, policy drift, lobbying, AI decisions, contamination
rolls, evidence noise) derive from **named seeded streams** on the daily step. **No kernel/upstream-crate
change** — their outputs are read as values.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01…08).
**Primary Dependencies**: `sojourn-core` (kernel contracts) **only** as a crate dependency; `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy; mood decay + posterior/log-odds + consensus weighting use `libm::exp`/`libm::log`), `ron` (data), `thiserror`, `rand_core` (the kernel stream trait), `blake3` (data content-hash pin, as FA-02…08). **No `sojourn-world`/`-research`/`-economy`/`-vehicle`/`-base`/`-crew` crate dependency** — their outputs (FA-03 candidate priors + site PP categories, FA-05 tech maturity + science tide, FA-06 budgets/valuations/tonnage/supply, FA-07/08 mission/embargo facts + loss-of-crew) flow in as composed values/opaque inputs (the FA-04 C1 decoupling). No new third-party deps: the milestone ledger, the bounded mood model, the per-faction Bayesian belief update + prestige-weighted consensus, the daily Bernoulli multiplicative-hazard event scheduler, policy gating/drift/lobby, graded contamination, the abstracted AI heuristics and the Grand-Goal pass/fail + composite score are all in-crate on the kernel's seeded streams.
**Storage**: Data files only — `data/polity/` (the ~120-first milestone catalogue [a representative sourced subset across all eras at this slice, structured for completion]; mood coefficients + decay + loss-of-crew severity/recovery + mood→budget/valuation/approval curves; event-class catalogue + base rates + hazard-factor refs + interrupt/log classification; policy/treaty levers + bounds + drift/lobby params + gating rules; COSPAR categories I–V + Special Regions + bioburden limits + sterilisation refs + contamination-grading params; astrobiology evidence-stage likelihoods + abiotic-competitor params + prestige-weighted consensus weighting + the ≥0.9/≤0.1 confidence band + sample-return gate; AI behaviour tuning + difficulty multipliers; Grand-Goal thresholds + change penalty + composite-score weights) and `data/polity/validation.ron` (analytic cases), all carrying `source` provenance and validated in CI. The astrobiology **priors** remain owned by FA-03 (`data/world/astrobiology.ron`) and the site PP categories by FA-03 (`data/world/sites.ron`); the host bridges them in as composed values.
**Testing**: `cargo test` (unit + integration per user story: milestones/world-vs-faction-first + prestige tiebreak; mood/approval + loss-of-crew severity; astrobiology ground-truth fidelity + staged evidence + per-faction consensus + conclusive band; event hazard monotonicity + interrupt classes + double-run identity; policy gating/drift/lobby; planetary-protection graded contamination + back-contamination; AI plausibility envelope + tide; Grand-Goal pass/fail + composite score); **analytic validation gates** (ground-truth prior fidelity across seeds; consensus crossing only at the band + sample-return gate; never-conclusive-positive-for-negative-truth; contamination degradation monotone in overage; event hazard monotone + clamped; tiebreak determinism; score determinism) per the constitution testing mandate; kernel conformance (`conformance --module polity`); harness determinism gates (verify/roundtrip/**mutate**); `validate-data` extended to polity.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-polity`) implementing the FA-01 `SimModule` contract + a public read-only world/political-state query API; harness/bench/data extensions. No kernel/upstream-crate change.
**Performance Goals**: Tiny hot state (10 factions, ~120 firsts, 6 candidates × 10 per-faction posteriors, a handful of policy levers + event sources); the daily step evaluates per-source event hazards, mood decay, policy drift, AI heuristics and consensus recompute in time proportional to (factions × candidates + event sources); read-only queries are sub-millisecond pure computations; the full world sustains the kernel envelope (≥1 sim-year/min) at high warp across century horizons.
**Constraints**: Full kernel determinism (ordered `BTreeMap`/`BTreeSet` stores, libm-only, no wall-clock; **all stochastic outcomes from named seeded streams** threaded via `ctx.rng(path)`); **no magic numbers** (Principle II/V — all milestone weights, mood coefficients, event rates, policy bounds, PP limits, consensus weights/band, AI tuning, goal thresholds, score weights from sourced data); probabilities in [0,1], SI units; **no combat/aliens** (Principle IX — discovered life is a science object, never an actor); educational honesty (Principle VIII — no binary "life found", no misinformation); acts on composed values, never hidden truth (the astrobiology ground truth is never query-exposed); analytic-case CI gates; polity-data version pinned in saves.
**Scale/Scope**: A sourced parameter set across nine sub-systems (firsts, mood, astrobiology, events, policy, planetary-protection, AI, goals, scoring); all ten factions (one player + nine AI); century horizons (durable, daily-stepped, seeded-event-driven).

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS | Every quantitative entry (milestone significance weights, mood/appropriation coefficients, event base rates, COSPAR categories/bioburden limits + sterilisation refs, astrobiology evidence likelihoods/consensus band, AI tuning, Grand-Goal thresholds, score weights) lives in `data/polity/*` with a `source`; the astrobiology **priors** stay sourced in FA-03's `data/world/astrobiology.ron`; `validate-data` (extended) fails CI on any missing source. Speculative endgame firsts are gated behind the FA-05 Breakthrough system. |
| II. Physics authoritative / no magic numbers | PASS | This layer adds **no physics**; it reads composed physical outcomes and applies sourced political/statistical models (bounded mood, Bayesian belief update, multiplicative-hazard events, graded contamination) with **all** constants from data. No event invents a physics-violating outcome (FR-PEA-405); analytic gates pin the statistical models. |
| III. Deterministic core | PASS | The **second heavily-seeded slice** (after FA-08): the ground-truth draw, every event, policy drift, lobby, AI decision, contamination roll and evidence-noise term draws from a **named seeded stream** (`ctx.rng(path)`); ordered stores; libm-only; no wall-clock; double-run / roundtrip / **mutate** / conformance gates. |
| IV. Headless / decoupled | PASS | Pure library module + read-only world/political-state query functions; zero UI deps; depends only on `sojourn-core`; everything driven/audited via harness scenarios. |
| V. Data-driven content | PASS | All nine sub-systems' params/catalogues are schema-validated data; new event classes are data registry entries; the milestone catalogue is data, not code. |
| VI. Research a modelled process | N/A (consumed) | Tech maturity and the global **science tide** come from FA-05 as composed values driving AI progress, event hazards and breakthrough-gated firsts; this slice reads them, never reinventing research. |
| VII. Tyranny of mass / Δv | PASS (amplifies) | Mood/policy are **modifiers over** the FA-06 economy (money = proxy for mass-to-orbit), never value generators (FR-PEA-206); loss-of-crew (the FA-08 mass/abort consequence) here becomes a strategic, long-lasting political cost — the felt downstream of the tyranny. |
| VIII. Educational honesty | PASS | Astrobiology is modelled honestly: seeded ground truth, **no binary popup**, staged probabilistic evidence with abiotic competitors and public scientific disagreement; conclusive-positive is impossible for negative ground truth (FR-PEA-305); Sojournal entries (FR-PEA-906) carry sources. |
| IX. No combat/aliens | PASS | Competition is firsts/economics/prestige only; discovered life is a **science object**, never an actor (FR-PEA-905); no weapons/sabotage; reserved features are not built. |
| Engineering constraints | PASS | SI/probabilities-in-[0,1]; sub-ms queries; polity-data version pinned in saves (extends the FA-02…08 hash pattern); fully offline. |
| **Cross-slice coupling** | NOTED (none added) | `sojourn-polity` depends **only on `sojourn-core`**; FA-03/05/06/07/08 outputs flow in as composed values (the FA-04/06/08 decoupling), so the dependency graph gains no new crate edges and the slice is unit-testable with stubs. No kernel/upstream change. |

**Initial gate (pre-Phase-0)**: PASS. **Post-Phase-1 re-check (2026-06-14)**: design artifacts introduce no
new violations; no kernel amendment; no upstream-crate change (composed-value decoupling); single-writer
preserved; all randomness is seeded-stream; ground truth never query-exposed. Gate remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/010-politics-events-astrobiology/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R15)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── polity-queries.md        # The read-only world/political-state query surface (FA-10 UI seam)
│   ├── polity-commands.md       # Commands (via ModulePayload) + emitted events + event classes
│   ├── polity-data.md           # Milestone/mood/event/policy/PP/astrobiology/AI/goal data formats, sourcing, analytic gates
│   └── integration-seams.md     # The composed-value inputs (FA-03 priors + PP, FA-05 tide, FA-06 econ, FA-07/08 mission + loss-of-crew) and outputs
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged this slice
├── sojourn-world/               # FA-03 — unchanged (candidate priors + site PP categories consumed as values)
├── sojourn-research/            # FA-05 — unchanged (tech maturity + science tide consumed as values)
├── sojourn-economy/             # FA-06 — unchanged (budgets/valuations/tonnage/supply consumed as values)
├── sojourn-base/  sojourn-crew/ # FA-07/08 — unchanged (mission/embargo facts + loss-of-crew consumed as values)
├── sojourn-polity/              # THIS SLICE — pure library, SimModule implementor (dep: core only)
│   ├── Cargo.toml               # deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core, blake3
│   └── src/
│       ├── lib.rs               # public surface: PolityModule, PolityCommand, polity queries
│       ├── ids.rs               # FactionId, MilestoneId, BodyId(u32), CandidateId, PolicyId, EventClassId, GoalKind
│       ├── params.rs            # the sourced parameter set (milestones/mood/events/policy/protection/astrobiology/ai/goals) load + validation + content_hash
│       ├── inputs.rs            # composed-value shapes (Achievement, CandidatePrior, SiteProtection, MissionFacts, EconomyFacts, CrewFacts, ScienceTide, Difficulty)
│       ├── faction.rs           # faction political state: relationships, prestige, partnership/consortium, funding-model class
│       ├── milestones.rs        # the firsts catalogue + award logic (world/faction-first ledger; prestige-then-id tiebreak)
│       ├── mood.rs              # bounded public/political mood + decay + outcome→delta + mood→budget/valuation/approval modifiers
│       ├── astrobiology.rs      # seeded ground-truth draw; per-faction posteriors; staged evidence + abiotic competitors; prestige-weighted consensus + band + sample-return gate
│       ├── events.rs            # event catalogue + daily Bernoulli multiplicative-hazard scheduler (per named source); interrupt/log
│       ├── policy.rs            # policy/treaty levers + gating + seeded drift + lobbying; PP-stringency single source
│       ├── protection.rs        # COSPAR categories I–V, Special Regions, graded forward contamination (overage × crash/soft), back-contamination chain, pristine-value
│       ├── ai.rs               # abstracted heuristic seeded AI-faction behaviour (goals, plausibility envelope, difficulty tuning, tide advance)
│       ├── goals.rs             # Grand-Goal progress + pass/fail verdict; change penalty
│       ├── score.rs             # final scoring: Grand-Goal verdict (primary) + secondary composite (prestige + milestones + goal progress); soft-fail
│       ├── hazard.rs            # shared multiplicative-hazard composition (base × Π factors, clamped) — the FA-08 primitive
│       ├── trace.rs             # traceability tree (any derived figure → sourced leaves)
│       ├── query.rs             # WorldSnapshot (slice + composed inputs) + pure read-only queries
│       └── module.rs            # SimModule: polity slice, commands, daily seeded step (events + mood decay + drift + AI + consensus), publish, save/load_slice
│   └── tests/                   # milestones.rs, mood.rs, astrobiology.rs, events.rs, policy.rs, protection.rs,
│                                # ai.rs, goals.rs, conformance.rs, validation.rs, common/mod.rs
├── sojourn-harness/             # + `polity` scenario flag, validate-data polity, conformance --module polity, bench;
│                                #   bridges FA-03 astrobiology priors + site PP into the composed-value inputs
data/
└── polity/
    ├── milestones.ron           # ~120 firsts (id, era, description, weight, award conditions) — representative sourced subset, sourced
    ├── mood.ron                 # mood coefficients + decay + loss-of-crew severity/recovery + mood→budget/valuation/approval curves sourced
    ├── events.ron               # event-class catalogue + base rates + hazard-factor refs + interrupt/log classification sourced
    ├── policy.ron               # policy/treaty levers + bounds + drift/lobby params + gating rules sourced
    ├── protection.ron           # COSPAR categories + Special Regions + bioburden limits + sterilisation refs + contamination-grading params sourced
    ├── astrobiology.ron         # evidence-stage likelihoods + abiotic-competitor params + prestige-weighted consensus weighting + ≥0.9/≤0.1 band + sample-return gate sourced
    ├── ai.ron                   # AI behaviour tuning + difficulty multipliers sourced
    ├── goals.ron                # Grand-Goal thresholds + change penalty + composite-score weights sourced
    ├── params.ron               # cross-cutting (horizon, score composite weights, global bounds) sourced
    └── validation.ron           # analytic cases + tolerances (prior fidelity, consensus band, contamination monotone, hazard monotone, tiebreak/score determinism)
scenarios/                       # + politics_world.ron (init world → achievements/firsts race → mood swings → policy → evidence → contamination → events → goal scoring)
```

**Structure Decision**: A **new crate `sojourn-polity`** (the political/world-affairs/scoring layer,
distinct from FA-03's physical `sojourn-world`), depending **only on `sojourn-core`**. The clarify step
deferred crate placement here: rather than extend `sojourn-world` (which would either pull a gameplay
dep into FA-03 or bloat it), FA-09 is its own slice that **consumes FA-03's astrobiology priors and site
PP categories as composed values the host bridges** — preserving the FA-04 C1 decoupling and keeping the
slice unit-testable with stub candidates/sites/economy/crew facts. It is **broad** (nine sub-systems) but
architecturally uniform: one module crate over `sojourn-core`, the established command/event/query/seeded-
stream patterns from FA-05/FA-08, a shared **multiplicative-hazard** primitive (`hazard.rs`) backing the
event scheduler, and a **daily seeded step** (`cadence_ticks = 86_400`, the FA-08 cadence) that advances
events, mood decay, policy drift, the AI world and the consensus recompute. The per-game **astrobiology
ground truth is drawn once at `InitWorld`** from the composed FA-03 priors on a named stream and is
**never query-exposed**; per-faction posteriors + the prestige-weighted consensus are the only observable
astrobiology state. The breadth is managed by decomposition into independent sub-modules, each behind its
own user-story tests and analytic gates.

## Complexity Tracking

No constitution violations to justify — table intentionally empty. This is the **broadest** slice by
surface (nine sub-systems, eight user stories), but it adds **no new architectural complexity**: one
module crate depending only on core, the proven command/event/query/seeded-stream patterns, the FA-08
multiplicative-hazard primitive reused for events, composed-value decoupling, and no kernel or upstream
change. Breadth is contained by per-sub-system decomposition (milestones, mood, astrobiology, events,
policy, protection, ai, goals, score) over a shared hazard primitive, each independently testable with
stubs and gated by analytic cases. The MVP is **US1 (milestones/prestige)** alone; the remaining stories
layer on without restructuring.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
