# Phase 1 Data Model — Politics, Events, Milestones & Astrobiology (FA-09)

All stores are ordered (`BTreeMap`/`BTreeSet`) for determinism. "Stored" = owned slice state evolved on
the daily step; "Composed" = opaque inputs captured at command time from the host (FA-03/05/06/07/08);
"Derived" = pure functions over stored + composed (read-only queries). The astrobiology **ground truth**
is stored but **never query-exposed**.

## 1. Identity types (`ids.rs`)

- **FactionId(u32)** — one of the ten organisations (player + nine AI).
- **MilestoneId(u32)** — a catalogued historic first.
- **BodyId(u32)** — a Solar-System body (matches the FA-03 catalogue ids; e.g. Mars=4, Europa=111).
- **CandidateId(BodyId)** — a candidate-world astrobiology target (a `BodyId` with a prior).
- **PolicyId(String, transparent)** — a policy/treaty lever key (e.g. `"nuclear-launch"`).
- **EventClassId(String, transparent)** — an event-class/source key.
- **GoalKind** — `Pathfinder | Homestead | Prospector | Seeker`.

## 2. Composed-value inputs (`inputs.rs`) — host-assembled, captured at command time

- **FactionInit**: `{ id, funding_model: Agency|Company, baseline_mood, ai: bool }`.
- **Achievement** (US1): `{ faction, milestone: MilestoneId, facts: AchievementFacts }` — the composed
  proof that a milestone's award conditions are met (e.g. `propellant_sold_kg`, `body`, `crewed`,
  `reusable`, `isru_origin`). Award conditions are evaluated against these facts.
- **CandidatePrior** (FA-03, US3): `{ candidate: CandidateId, presence_prob, tier: Subsurface|Ocean|Atmospheric|Brine }`
  — bridged from `data/world/astrobiology.ron`.
- **EvidenceInput** (US3): `{ faction, candidate, stage: OrbitalHint|InSitu|Microscopy|SampleReturn, quality ∈ [0,1] }`.
- **SiteProtection** (FA-03, US6): `{ body: BodyId, category: 1..=5, special_region: bool, bioburden_limit }`
  — bridged from `data/world/sites.ron` + active stringency.
- **MissionFacts** (US6): `{ body, special_region: bool, lander_bioburden, crash: bool, sample_return: bool, containment_chain: bool }`.
- **EconomyFacts** (US2/US8): `{ faction, base_appropriation, base_valuation, off_earth_tonnage_profit }`
  — composed FA-06; mood applies modifiers over these (never generates value).
- **CrewFacts** (US2): `{ faction, loss_of_crew: bool, routine_failure: bool, success: bool, world_first: bool }`
  — composed outcome stream (FA-08 loss-of-crew among them).
- **HomesteadFacts** (US8): `{ faction, embargo_survival_index }` — composed FA-07/08.
- **ScienceTide** (US7): `{ global_level, per_faction_level: map }` — composed FA-05.
- **Difficulty**: `{ ai_funding_mult, ai_competence_mult, event_rate_mult }` — political/economic harshness
  only; never physics.

## 3. Sourced parameters (`params.rs`, `data/polity/*`) — DATA, immutable, content-hashed

- **MilestoneCatalogue** (`milestones.ron`): `[{ id, era, description, weight, faction_first_fraction,
  conditions: [Condition], source }]` where `Condition` is a machine-checkable predicate over
  `AchievementFacts`. Validation: unique ids; `weight > 0`; `0 < faction_first_fraction < 1`; non-empty
  `source`; every condition references a known fact key.
- **MoodParams** (`mood.ron`): outcome→delta coefficients (`world_first`, `success`, `routine_failure`,
  `loss_of_crew`, `economic_cycle`), `decay_per_day`, mood bounds, `loc_recovery_days` (crewed-flight
  freeze window), and mood→{appropriation, valuation, approval-latency} curves. Validation: bounds ordered;
  `|loss_of_crew| > |routine_failure|`; `loc_recovery_days ≥` a sourced multi-year floor; `source`.
- **EventCatalogue** (`events.ron`): `[{ class: EventClassId, base_rate, factor_refs, interrupt: bool,
  source }]`. Validation: `0 ≤ base_rate ≤ 1`; known factor refs; `source`.
- **PolicyParams** (`policy.ron`): `[{ id: PolicyId, min, max, default, drift_per_period, lobby_step,
  gates: [Gate], source }]` where a `Gate` maps a required level to a mission/partnership predicate.
  Validation: `min ≤ default ≤ max`; PP-stringency lever present; `source`.
- **ProtectionParams** (`protection.ron`): per-body `{ body, category, special_region, bioburden_limit,
  sterilisation_ref }` + `contamination: { overage_curve, crash_factor, soft_factor, backcontam_penalty }`.
  Validation: category ∈ 1..=5; `bioburden_limit > 0`; `overage_curve` monotone; `crash_factor ≥
  soft_factor ≥ 1`; `source`.
- **AstrobiologyParams** (`astrobiology.ron`): `{ stage_lr: { OrbitalHint, InSitu, Microscopy, SampleReturn },
  abiotic_competitor: {...}, consensus_weight: PrestigeWeighted, band_positive: 0.9, band_negative: 0.1,
  sample_return_required: true, false_hint_cap, source }`. Validation: `0 < band_negative < band_positive < 1`;
  LRs ordered (SampleReturn strongest); `false_hint_cap < band_positive`; `source`.
- **AiParams** (`ai.ron`): heuristic weights, plausibility-envelope caps, difficulty hooks, tide-advance
  rate. Validation: envelope caps present; `source`.
- **GoalParams** (`goals.ron`): per-goal thresholds (`pathfinder_firsts`, `homestead_index`,
  `prospector_tonnage`, `seeker_worlds = 3`), `change_penalty`, composite-score weights `{ prestige_w,
  milestone_w, goal_w }`. Validation: thresholds > 0; weights sum documented; `source`.
- **PolityParams** (`params.ron`): horizon defaults, global mood/probability bounds, score normalisation.
- **content_hash**: blake3 over normalised (CRLF→LF) concatenated data texts joined with `"\0"` (the
  FA-02…08 pattern); saves pin it (`DataVersionUnavailable` on mismatch).

## 4. Stored slice state (`PolitySlice` in `module.rs`)

- **factions: BTreeMap<FactionId, FactionState>** — `FactionState { funding_model, relationships:
  BTreeMap<FactionId, f64>, partnership: BTreeSet<FactionId>, prestige, mood, mood_baseline,
  loc_freeze_until_tick, grand_goal: Option<GoalKind>, goal_changes: u32 }`.
- **ledger: BTreeMap<MilestoneId, MilestoneClaim>** — `MilestoneClaim { world_first:
  Option<(FactionId, u64)>, faction_firsts: BTreeMap<FactionId, u64> }`.
- **candidates: BTreeMap<CandidateId, CandidateState>** — `CandidateState { tier, ground_truth: bool
  (HIDDEN), posteriors: BTreeMap<FactionId, f64 /*log-odds*/>, evidence: Vec<EvidenceRecord>,
  abiotic_strength, consensus: f64, conclusive: Option<Resolution /*Positive|Negative*/>, pristine_value:
  f64 /*starts 1.0*/ }`.
- **policy: BTreeMap<PolicyId, f64 /*current level*/>**.
- **protection: BTreeMap<BodyId, ProtectionState>** — `{ category, special_region, bioburden_limit,
  pristine_value }` (pristine value shared with the candidate when applicable).
- **events: Vec<EventRecord>** (bounded/paged) — emitted-event history for the feed.
- **ai: BTreeMap<FactionId, AiState>** — `{ target_queue, capability, pending }`.
- **science_tide: ScienceTide** (captured/refreshed composed).
- **difficulty: Difficulty** (captured).
- **last_tick: u64**, **data_hash: [u8;32]**.

State transitions (daily step, two-pass roll-then-apply, all seeded):
1. **events** — per source: Bernoulli `uniform(stream events/<source>) < clamp(base × Π factors)`; emit at
   interrupt/log class; AI milestone claims emit `rival-milestone`.
2. **mood** — decay toward baseline (`libm::exp`); apply queued outcome deltas; expire `loc_freeze`.
3. **policy** — seeded drift per lever (bounded); apply pending lobby nudges.
4. **ai** — per AI faction seeded heuristic: advance tide, pursue/claim milestones, emit contracts; clamp
   to plausibility envelope.
5. **astrobiology** — recompute each candidate's per-faction posteriors from accrued evidence + abiotic
   strength; recompute the **prestige-weighted consensus**; flip to `conclusive` iff band crossed AND a
   SampleReturn item exists (and, structurally, never positive on negative ground truth); on flip emit the
   top-tier milestone + tighten PP + shift mood.
6. **goals/score** — recompute Grand-Goal progress; at horizon, freeze the pass/fail verdict + composite.

## 5. Derived (query) figures (`query.rs`, `WorldSnapshot`) — pure, read-only

- **milestone ledger view**: per milestone, claimed/unclaimed, world-first claimant, faction-firsts.
- **prestige(faction)**, **mood(faction)** + derived **appropriation/valuation/approval** modifiers.
- **policy(level)** + **gate verdict(mission/partnership)** (gated/penalised/ok).
- **protection(body)**: category, special-region, bioburden limit, **pristine_value**, contamination
  records.
- **astrobiology(candidate)**: per-faction posteriors, the **weighted consensus**, **conclusive** status,
  **public-disagreement** flag, achievable evidence value — **never the ground truth**.
- **event feed** (paged).
- **grand_goal(faction)**: progress + **pass/fail verdict**.
- **score(faction)**: the Grand-Goal verdict (primary) + **secondary composite** (prestige + milestones +
  goal progress); soft-fail flags.
- All decompose through `trace.rs` to **sourced leaves** (`all_leaves_sourced`).

## 6. Invariants (tested)

- A world-first is held by **exactly one** faction; same-tick ties → **highest prestige, then lowest id**.
- Firsts are **permanent** once awarded.
- Mood stays within bounds; loss-of-crew effect strictly **deeper + longer** than a routine failure.
- Ground truth is **never** returned by any query.
- A candidate is **conclusive-positive only if** ground truth is positive; conclusive requires band cross
  **and** a SampleReturn item.
- Contamination pristine-value degradation is **monotone in bioburden overage** (× crash/soft factor); a
  compliant lander degrades nothing.
- Event probability is **monotone** in each risk factor and **clamped** to [0,1].
- Policy levels stay within `[min, max]` under drift + lobby.
- AI capability never exceeds the plausibility envelope; difficulty changes funding/competence only.
- The whole slice round-trips through save/load **bit-identically**; `data_hash` pins the data version.
- No combat/weapon/alien-actor entity exists; discovered life is a science object.
