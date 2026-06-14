# Contract — Read-only world/political-state query surface (`WorldSnapshot`)

The honest, mutation-free seam the FA-10 UI and the Sojournal read. Pure functions over the stored
`PolitySlice` + captured composed inputs. **No query ever exposes the astrobiology ground truth.**

`WorldSnapshot::from_core(core, module) -> Result<WorldSnapshot, CoreError>` (reads the stored slice; the
composed inputs are already captured) and `from_parts(module, slice)` (tests).

## Milestones & prestige (US1)

- `milestone(id) -> Option<MilestoneView>` — `{ id, era, description, weight, world_first:
  Option<(FactionId, tick)>, faction_firsts: Vec<FactionId> }`.
- `ledger() -> Vec<MilestoneView>` — full catalogue with claim state.
- `unclaimed_world_firsts() -> Vec<MilestoneId>` — the race board.
- `prestige(faction) -> f64` — accrued prestige (traceable to firsts/events).

## Politics & mood (US2)

- `mood(faction) -> f64` (bounded).
- `modifiers(faction) -> { appropriation_factor, valuation_factor, approval_latency_days, crewed_frozen:
  bool }` — the concrete levers mood drives, as factors over composed FA-06 inputs.
- `approval(faction, program) -> Granted | Delayed(days) | Denied` (function of mood + policy).

## Astrobiology (US3) — never the ground truth

- `posterior(faction, candidate) -> f64` — that faction's belief (probability of life present).
- `consensus(candidate) -> f64` — the **prestige-weighted** community consensus.
- `conclusive(candidate) -> Option<Positive | Negative>` — set only when the band is crossed **and** a
  SampleReturn item exists.
- `disagreement(candidate) -> bool` — factions publicly disagree (spread over a sourced width).
- `evidence_value(candidate) -> f64` — achievable evidence quality (degraded by contamination).
- *(No `ground_truth` accessor exists anywhere in the public surface.)*

## Policy & treaties (US5)

- `policy(id) -> f64` — current lever level.
- `gate(mission_or_partnership_facts) -> Ok | Gated(reason) | Penalised(factor)`.

## Planetary protection (US6)

- `protection(body) -> { category, special_region, bioburden_limit, pristine_value }`.
- `contamination_records(body) -> Vec<ContaminationRecord>`.

## Events (US4)

- `events(filter) -> Vec<EventView>` (paged) — class, tick, faction, payload, interrupt/log.

## AI world (US7)

- `science_tide() -> { global, per_faction }`.
- `ai_targets(faction) -> Vec<MilestoneId>` (visible intentions, for narrative texture).

## Grand Goals & scoring (US8)

- `grand_goal(faction) -> { kind, progress, verdict: Pass | Fail | Pending }`.
- `score(faction) -> { goal_verdict, composite: f64, soft_fail: Option<AgencyGutted|Bankrupt|LocSpiral> }`.

## Guarantees

- Pure & read-only; identical results for identical state across stepping patterns (determinism).
- Every numeric view decomposes through `trace.rs` to **sourced leaves** (`all_leaves_sourced`).
- The ground truth, AI internal RNG and any hidden state are **not** reachable through this surface.
