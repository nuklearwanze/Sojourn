# Phase 0 Research — Politics, Events, Milestones & Astrobiology (FA-09)

All NEEDS-CLARIFICATION items from the Technical Context are resolved here. The four `/speckit-clarify`
decisions (weighted-aggregate consensus + band, highest-prestige tiebreak, graded-by-overage
contamination, daily-Bernoulli-hazard events) and the three `/speckit-specify` decisions (abstracted AI,
per-faction beliefs, Grand-Goal pass/fail + score) are treated as settled and recorded against the
relevant items below.

---

## R1 — Crate boundary & composed-value seams (the decoupling)

- **Decision**: A new crate **`sojourn-polity`** depending **only on `sojourn-core`**. Every upstream
  gameplay fact enters as a **composed value** the host assembles: FA-03 candidate-world `presence_prob`
  priors + site COSPAR categories/Special-Region flags; FA-05 tech maturity + the global science tide;
  FA-06 budgets/valuations/off-Earth tonnage/strategic supply; FA-07/08 mission/landing/embargo facts +
  **loss-of-crew**. The harness bridges FA-03's `data/world/astrobiology.ron` and `data/world/sites.ron`
  into the `CandidatePrior` / `SiteProtection` input shapes.
- **Rationale**: The FA-04 C1 / FA-06 R1 / FA-08 lesson — a hard crate dep on the upstream gameplay crates
  buys nothing and costs testability + a new graph edge. Core-only keeps every sub-system unit-testable
  with stub inputs and the dependency audit green. Astrobiology priors and site PP data are **owned by
  FA-03** (spec assumption); FA-09 reads them, never redefines them.
- **Alternatives considered**: (a) Extend `sojourn-world` — rejected: pulls political/scoring gameplay
  into the physical-data crate and couples FA-03 to FA-05/06/08 outputs. (b) Crate dep on
  `sojourn-world`/`-economy`/`-crew` — rejected: new edges, harder stubbing, contradicts the established
  pattern.

## R2 — Milestone catalogue & the world-/faction-first ledger

- **Decision**: `data/polity/milestones.ron` is a data-driven catalogue of ~120 firsts — `id`, `era`
  (foothold/cislunar/frontier/endgame), `description`, `weight` (prestige/score significance),
  machine-checkable **award conditions** over composed facts, and `source`. At this slice author a
  **representative sourced subset spanning all eras** plus the full structure (the FA-05 "representative
  sourced node subset" precedent); the catalogue is completable in data without code change. The ledger
  stores, per milestone, the **world-first** claim (first global claimant + tick) and a set of
  **faction-first** claimants. Award = all conditions satisfied; world-first awards full `weight`,
  faction-first a sourced lesser fraction. Firsts are **permanent** once awarded (FR-PEA-107).
- **Same-tick world-first tiebreak (clarified)**: the claimant with the **highest current prestige** wins;
  ties broken by **lowest faction id** — deterministic and seed-stable (prestige + id are both ordered
  state, no RNG needed).
- **Rationale**: Data-driven significance keeps Principle I/V; the ledger is the scoring spine (US1) and
  the AI race. Highest-prestige tiebreak rewards the established leader and is trivially reproducible.
- **Alternatives**: kernel-queue-order tiebreak (rejected — couples the rule to event-ordering internals,
  harder to explain to the player); seeded coin (rejected — opaque outcome); full 120 firsts now
  (deferred — author incrementally in data, like FA-05's tech nodes).

## R3 — Public/political mood, budgets & approvals

- **Decision**: A **bounded** per-faction mood ∈ [−1, +1] (or [0,1] normalised), updated by composed
  outcome events with **sourced deltas** and **exponential decay** toward a faction baseline
  (`libm::exp`). Loss-of-crew applies a large negative delta **and** opens a multi-year crewed-flight
  **approval-freeze** window (sourced from NASA's post-accident 2–4-yr crewed-flight pauses). Mood maps to
  three modifiers via sourced curves: appropriation factor (agencies), valuation/contract-access factor
  (companies), and approval latency/denial. All saturate within bounds (FR-PEA-204).
- **Rationale**: Matches OVERVIEW §3 funding asymmetry and 05-WORLD.md §5; gives loss-of-crew (FA-08) its
  strategic teeth (FR-PEA-202/SC-003). Budget effects are **factors over** composed FA-06 inputs — never
  value generators (FR-PEA-206), preserving economy conservation.
- **Alternatives**: unbounded mood (rejected — overflow/instability); linear decay (rejected — exponential
  matches news-cycle fade and is the FA-08-style libm curve).

## R4 — Astrobiology: seeded ground truth, per-faction beliefs, weighted consensus

- **Decision (ground truth)**: at `InitWorld`, for each composed `CandidatePrior` (in body-id order) draw
  a hidden `life_present: bool` = `uniform(stream "polity/ground-truth") < presence_prob`. Stored in the
  slice, **never query-exposed** (FR-PEA-302). Across seeds the realised positive fraction matches the
  priors (SC-004).
- **Decision (evidence)**: evidence arrives via `CollectEvidence{faction, candidate, stage, quality}`
  commands, where `stage ∈ {OrbitalHint, InSitu, Microscopy, SampleReturn}`. Each evidence item carries a
  **seeded likelihood** generated **conditioned on the ground truth**: a positive world can emit positive
  evidence at any stage; a negative world can emit **false hints only at pre-conclusive stages**, capped
  so they cannot drive consensus past the band. The strongest stage (SampleReturn) for a negative world is
  necessarily non-positive — this is the mechanism that makes FR-PEA-305 (never conclusive-positive for
  negative truth) structural, not a post-hoc clamp.
- **Decision (per-faction belief)**: each faction holds a posterior in **log-odds**; an evidence item
  updates that faction's log-odds by a **sourced likelihood-ratio** for `(stage, quality)`, attenuated by
  the strongest live **abiotic-competitor** hypothesis (which "explains away" pre-conclusive evidence).
- **Decision (consensus, clarified)**: the **community consensus** is a **prestige/science-output-weighted
  aggregate** of the per-faction posteriors (weights data-driven; reputable factions sway more). A
  candidate is **conclusive** when the consensus probability crosses the **band ≥ 0.9 (positive) / ≤ 0.1
  (negative)** AND at least one **SampleReturn-tier** evidence item exists (FR-PEA-304). Factions may
  **publicly disagree** (their individual posteriors differ) until then; the disagreement state is
  queryable (FR-PEA-308). A conclusive resolution emits a top-tier milestone (US1), tightens PP (US6) and
  shifts mood (US2) (FR-PEA-306).
- **Decision (contamination confound)**: forward contamination (R7) **degrades the evidence quality** a
  candidate can yield thereafter (FR-PEA-307), so ruining a Special Region forecloses *Seeker* there.
- **Rationale**: log-odds additive updates are the standard, stable Bayesian form (libm::log/exp); the
  weighted aggregate is the clarified consensus rule; conditioning evidence generation on ground truth is
  the only honest way to guarantee no false "life confirmed" (Principle VIII).
- **Alternatives**: equal-weight mean / supermajority / log-odds pool consensus (rejected per clarify in
  favour of prestige-weighted aggregate); post-hoc clamping of a positive result on a negative world
  (rejected — dishonest and fragile; structural conditioning is correct).

## R5 — Event system: daily Bernoulli multiplicative-hazard scheduler

- **Decision (clarified)**: events fire from a data-driven catalogue of classes. Each **named event source**
  is evaluated on the **daily step** (`cadence_ticks = 86_400`) as a **Bernoulli draw** `uniform(stream
  "polity/events/<source>") < clamp(base_rate × Π factors, 0, 1)`, where the factors come from composed
  state (maturity/TRL, test heritage, ops oversubscription, environment) — the **FA-08 multiplicative-
  hazard** primitive (`hazard.rs`). A riskier configuration earns a strictly higher probability (SC-006);
  the same seed + decisions reproduce the same events (FR-PEA-404). Each class declares an **interrupt vs
  log** classification feeding the FA-01 loop; rival-milestone events are emitted when the AI world claims
  a first.
- **Rationale**: identical mechanism to FA-08's SPE/ECLSS/EDL rolls — proven deterministic across stepping
  patterns, trivially mutate-testable, uniform with the codebase.
- **Alternatives**: seeded inter-arrival times (rejected — re-deriving on every state change complicates
  determinism); hybrid hazard+calendar (rejected for v1 — the daily Bernoulli covers discrete events by
  gating their hazard on a "window-open" composed factor; revisit if needed).

## R6 — Policy & treaties: levers, gating, drift, lobbying

- **Decision**: `data/polity/policy.ron` defines bounded levers (launch licensing & range access,
  nuclear-launch approval, planetary-protection stringency, export controls, debris/sustainability), each
  with a `level` in a sourced range, **gating rules** (a mission/partnership lacking a required lever is
  gated or penalised — FR-PEA-502), seeded **drift** (a daily/periodic nudge on stream
  "polity/policy-drift") and **lobby** mechanics (a `Lobby` command nudges a level on stream
  "polity/lobby", bounded). PP **stringency is the single source** consumed by R7 (FR-PEA-504). Export
  controls modify partnership feasibility / component cost as a factor over composed FA-06 inputs.
- **Rationale**: makes politics mechanical friction (US5) and the seam for PP (US6) and the AI world (US7).
- **Alternatives**: hard binary policy (rejected — graded levels allow drift/lobby nuance); unbounded
  lobbying (rejected — must clamp to the lever range).

## R7 — Planetary protection: categories, Special Regions, graded contamination

- **Decision**: `data/polity/protection.ron` carries COSPAR categories I–V, per-body Special-Region flags
  and **bioburden limits** (sourced from COSPAR policy), composed with the FA-03 site data and the active
  stringency (R6). **Forward contamination (clarified: graded by overage)**: when a lander's composed
  bioburden exceeds the limit in a Special Region, pristine-value degradation =
  `f(overage_ratio) × crash_or_soft_factor` — a small breach partially confounds future evidence (R4), a
  gross breach effectively ruins it; the degradation function is **monotone in overage** and data-driven.
  A compliant (≤ limit) sterilised lander incurs **no** penalty (sterilisation cost/mass paid upstream).
  **Back contamination**: a sample return from a potentially-habitable world requires a composed
  **containment-chain** flag; absent it the return is gated/penalised (FR-PEA-604). Outcomes deterministic
  given composed mission facts + active categories (FR-PEA-605).
- **Rationale**: graded degradation is the clarified richer model and is what makes "cutting corners"
  tempting-but-consequential (05-WORLD.md §3) and ties PP to the *Seeker* goal.
- **Alternatives**: binary ruin / two-tier soft-vs-crash (rejected per clarify in favour of graded; the
  crash/soft factor is retained as a multiplier within the graded model).

## R8 — The AI world: abstracted, heuristic, seeded

- **Decision (clarified: abstracted)**: AI factions run a **lightweight, heuristic, seeded** behaviour
  model over **composed capability estimates** — not a mirror of FA-04…08. Each AI faction has a capability
  profile (funding, competence, tech level from the composed science tide), a goal/target queue, and a
  daily seeded decision step (stream "polity/ai/<faction>") that advances the tide, pursues/claims
  milestones, generates contracts/partnership offers and suffers accidents (via the R5 event hazards). AI
  capability is **clamped to a plausibility envelope** (no impossible tech — FR-PEA-702). **Difficulty**
  multiplies funding/competence only (FR-PEA-703), never the envelope.
- **Rationale**: matches 05-WORLD.md §7 ("simplified versions") and the clarify decision; bounds scope and
  keeps determinism simple while still producing a credible milestone race.
- **Alternatives**: full system mirror / hybrid-deep-CNSA (rejected per clarify — too large; abstracted is
  sufficient for the race and the contract market).

## R9 — Grand Goals & scoring: pass/fail + secondary composite

- **Decision (clarified)**: four Grand Goals, selectable at start, changeable mid-game with a sourced
  **penalty**. Each computes a deterministic **pass/fail verdict** from composed inputs — Pathfinder
  (exploration firsts/science count ≥ threshold), Homestead (composed FA-07/08 embargo-survival index ≥
  threshold), Prospector (composed FA-06 profitable off-Earth tonnage ≥ threshold), Seeker (≥ 3 candidates
  conclusively resolved — R4). At the configured horizon the run resolves to the **selected goal's
  pass/fail** (primary) **plus a secondary composite score** = sourced-weighted blend of prestige +
  milestone total + Grand-Goal progress (FR-PEA-803). Soft-fail states (agency gutting, bankruptcy,
  loss-of-crew spiral) are flagged and continue in observer mode (FR-PEA-804), not hard game-over.
- **Rationale**: the clarified scoring shape; composes the outputs of every other sub-system; deterministic
  from composed inputs.
- **Alternatives**: composite-only / leaderboard (rejected per clarify in favour of goal pass/fail +
  score).

## R10 — Determinism, seeded streams & the daily step

- **Decision**: `cadence_ticks = 86_400` (daily, the FA-08 cadence). Named streams: `polity/ground-truth`,
  `polity/events/<source>`, `polity/policy-drift`, `polity/lobby`, `polity/ai/<faction>`,
  `polity/contamination`, `polity/evidence-noise`. Two-pass roll-then-apply per step (as FA-08) for
  ordering safety. Ordered `BTreeMap`/`BTreeSet` stores keyed by `FactionId`/`MilestoneId`/`CandidateId`/
  `PolicyId`; libm-only transcendentals; no wall-clock.
- **Rationale**: the proven FA-08 determinism shape; double-run / roundtrip / mutate must stay green.
- **Alternatives**: per-tick stepping (rejected — daily cadence matches the political/event timescale and
  the kernel envelope).

## R11 — Float & units policy

- **Decision**: probabilities and mood in bounded ranges; `libm::exp`/`libm::log` for decay and log-odds;
  uniform draws via the FA-01 `(next_u64() >> 11) / 2^53` idiom; SI where physical (mass/tonnage kg/t,
  time days). No `f64::exp`/`std` transcendentals (FA-01 cross-platform policy).
- **Rationale**: identical to FA-02…08; cross-platform bit-identical.

## R12 — Traceability

- **Decision**: a `trace.rs` tree (reused FA-04/06/07/08 shape): any derived figure (a faction's prestige,
  the composite score, a candidate's consensus, a contamination penalty) decomposes to **sourced leaves**,
  so every number is explainable to the player and `all_leaves_sourced` is testable.
- **Rationale**: Principle I/VIII auditability; the Sojournal seam (FR-PEA-906).

## R13 — Data sourcing & analytic validation gates

- **Decision**: `data/polity/validation.ron` analytic cases, gated in `validate-data polity`:
  (1) **ground-truth prior fidelity** — over N seeds, realised positive fraction ≈ prior within tolerance;
  (2) **consensus band** — a candidate flips to conclusive only when consensus crosses ≥0.9/≤0.1 AND a
  SampleReturn item exists, and **never conclusive-positive for negative ground truth**;
  (3) **contamination monotonicity** — degradation strictly increases with bioburden overage;
  (4) **event hazard** — monotone in each risk factor and clamped to [0,1];
  (5) **tiebreak determinism** — highest-prestige (then id) is reproducible;
  (6) **score determinism** — composite score is a pure function of composed inputs;
  (7) **mood bounds** — mood saturates within range; loss-of-crew effect strictly deeper/longer than a
  routine failure. All milestone weights, mood/event/policy/PP/astrobiology/AI/goal/score constants carry
  `source`; CI fails on any missing source.
- **Rationale**: the constitution's analytic-validation mandate; these are the Principle-I/III/VIII teeth.

## R14 — Query surface & the FA-10 (UI) seam

- **Decision**: a read-only `WorldSnapshot` (slice + composed inputs) exposing: the milestone ledger
  (claimed/unclaimed, world/faction-first, per-faction prestige); per-faction mood + derived
  budget/valuation/approval modifiers; policy lever levels + gating verdicts; PP category/Special-Region
  status + contamination records + pristine value; per-candidate **per-faction posteriors + the weighted
  consensus + conclusive status + public-disagreement** (but **never** ground truth); the event feed;
  Grand-Goal progress/verdict; and the final pass/fail + composite score. Pure functions over stored state
  + composed inputs (no hidden truth, no mutation) — the FA-10 UI and the Sojournal read this.
- **Rationale**: same read-only honest-seam pattern as FA-08's `CrewSnapshot`.

## R15 — Initial world setup & faction roster

- **Decision**: an `InitWorld` command captures the composed faction roster (ten factions: one player +
  nine AI, with funding-model class), the `CandidatePrior` list (FA-03) and `SiteProtection` list (FA-03),
  and draws the seeded ground truth. This is the FA-08 `OccupyAsset` capture pattern — composed values
  enter at command time so the daily step can evolve stored state. Difficulty + the science tide are
  refreshed via `UpdateWorld` (composed) as they change.
- **Rationale**: the daily step needs the captured priors/roster; capturing at command time keeps the slice
  core-only while letting the step evolve events/mood/consensus/AI.
- **Alternatives**: query-time composition every tick (rejected — the step mutates stored state and must
  own the captured inputs, the FA-06 `OperateIsru` / FA-08 lesson).
