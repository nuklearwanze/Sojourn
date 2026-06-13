# Phase 0 Research: Research & Personnel (FA-05)

Decisions resolving the Technical Context against Constitution v1.0.0 (esp. Principle VI), the
FA-01/02/03 contracts in the tree, and the spec's clarified scope. Format: **Decision / Rationale /
Alternatives rejected**. No `NEEDS CLARIFICATION` remain (spec clarified 2026-06-13).

---

## R1 — Crate topology: a physics-independent module above the kernel

**Decision.** A new crate `crates/sojourn-research` depends on **`sojourn-core` only** and implements
the kernel `SimModule` contract. It does **not** depend on `sojourn-astro` or `sojourn-world`. FA-04
(vehicle/propulsion) will depend on `sojourn-research` to read maturity/heritage.

**Rationale.** Research is a self-contained process — it consumes funded inputs and mission-injection
events and produces understanding/maturity/heritage; it needs nothing from the propagator or the world
catalogue. Keeping the dependency arrow pointed *toward* research (FA-04 → research) preserves the
acyclic workspace and lets research be developed and tested in complete isolation from physics.

**Alternatives rejected.** (a) Depend on astro/world for mission data — unnecessary; UL injections
arrive as journaled commands, not by reaching into other slices. (b) Fold research into an existing
crate — violates the module boundary the kernel enforces.

---

## R2 — Time-stepped, fixed cadence (not event-driven)

**Decision.** The research module is **time-stepped** at a single fixed cadence (default daily;
data-tunable). Each step it: generates RP/DE from the roster+facilities, applies the portfolio
allocation to advance domain ULs (diminishing returns + synergy) and program TRL progress (S-curve),
advances the world tide, accrues breakthrough insight pressure, ages personnel/training, and rolls
seeded test-campaign and overrun outcomes at TRL-step boundaries. No state-driven step escalation
(unlike FA-02's tiers); a fixed cadence is sufficient because research has no fast/slow physical
regimes.

**Rationale.** Research progress is a continuous accrual over months/years; a coarse fixed cadence
captures it cheaply and deterministically (warp-invariant: outcomes depend on elapsed sim-time, not on
how the kernel chunks steps — the FA-01 warp-invariance rule). Daily granularity keeps century games
light while resolving multi-month campaigns.

**Alternatives rejected.** (a) Event-driven like FA-03's world module — research has genuine per-tick
dynamics (accrual), so it must step. (b) Per-second stepping — wasteful; nothing changes meaningfully
sub-day. (c) State-driven tiers — no physical regime to escalate for.

---

## R3 — Understanding Levels: representation, growth, gating

**Decision.** Each `(faction, domain)` Understanding Level is an `f64` in `[0,100]`. Growth per step
= allocated RP × staffing/facility efficiency × a **diminishing-returns** factor (sharply rising cost
above ~70) × **synergy** bonus from coupled domains' ULs, all from `domains.ron`/`params.ron`. UL
**gates** engineering-program availability (per-program domain-UL floors) and sets a per-program risk
floor. libm-only transcendentals.

**Rationale.** Continuous UL (not boolean unlocks) is the core of Track A (design §1); diminishing
returns + synergy are the documented cost curve (§3). f64 + libm matches the FA-01/02/03 float policy
for cross-platform determinism.

**Alternatives rejected.** (a) Integer/boolean tech levels — violates "not a purchase" (Principle VI).
(b) Fixed-point — unneeded; the workspace's f64+libm policy is already deterministic.

---

## R4 — TRL step model: cost, min-time floor, facility + UL gate, S-curve, P50/P80

**Decision.** An Engineering Program targets a Technology and advances TRL 1→9. Each step carries a
sourced **cost**, a **minimum duration** (a hard floor — funding past it converts to rising overrun
risk, never sub-floor time), a **facility-capability requirement**, and a **domain-UL gate**. Progress
within a step is **S-curved** (slow-fast-slow). Cost/schedule are estimated as **P50/P80** and realised
with variance from a seeded `research/overrun` stream, modulated by TRL-jump size, UL margin, staffing,
facility adequacy and an opaque political-interference input.

**Rationale.** TRL is the literal spine (design §2). The min-time floor encodes the mythical-man-month;
P50/P80 + overruns are the Augustine-style realism the design calls for (§4.2). Facility/UL gating is
the cross-track coupling that makes science gate engineering.

**Alternatives rejected.** (a) Buyable schedule — breaks the floor and the realism pillar. (b) Single
deterministic duration — removes the uncertainty that defines the mechanic.

---

## R5 — Test campaigns & failure-that-teaches

**Decision.** Each TRL step (especially 4–7) runs a **test campaign**: a seeded success/failure roll
(from `research/test` stream) whose failure probability derives from TRL-immaturity, UL margin,
staffing and facility adequacy. A **failure** costs money/schedule but **injects UL** into the relevant
domain (failure-that-teaches); a spectacular flight-test failure (TRL-7 demo) additionally emits a
`test-failure` event with a political/PR-eligible payload (FA-09). **Repeated failure without UL
growth** is the documented dead-end signal (R6).

**Rationale.** Directly implements design §4.3. Tying the dead-end signal to "failure without UL
growth" makes the hint emergent rather than a flag.

**Alternatives rejected.** (a) Failures that only cost — loses the "we learned why it screeched"
realism. (b) Pure-RNG anomalies — the design forbids pure RNG; probability is earned from state.

---

## R6 — Dead-end seeding with constructive capability-reachability (clarified Q2:A)

**Decision.** At world creation, a `research/seed` stream fixes, per `(engineering approach, TRL band)`,
whether that approach is a **dead end** within that band. The seeding algorithm is **constructive**: it
processes each **capability category** (from `capability-categories.ron`) and **never closes that
category's last viable path** — so every category retains ≥1 viable path in every seed *by
construction*. A CI **reachability sweep** over a sampled seed set additionally verifies the invariant
(defense in depth). Dead ends are surfaced only through emergent hints (rising risk index, stalled
error bars, repeated failures without UL growth) before a `dead-end-confirmed` event.

**Rationale.** The clarified answer: structural guarantee + verification, mirroring FA-03's truth-free
snapshot + standing audit. Constructive seeding makes "no seed bricks a strategy" impossible to violate;
the sweep catches algorithm regressions.

**Alternatives rejected.** (a) Detective-only (reject/re-roll bad seeds) — wastes seeds and can't prove
the property holds for an arbitrary seed. (b) Constructive-only — no regression guard. (c) Exposing the
dead-end flag directly — kills the skill of *recognising* a dead end.

---

## R7 — Breakthroughs: seeded thresholds + earned insight pressure

**Decision.** Each `(faction, domain)` accrues hidden **insight pressure** per step, weighted toward
**basic-science** investment (applied-only work accrues little). Crossing a **seeded threshold** (from
`research/seed`) triggers a **Breakthrough** delivering one of: a tech-cluster discount, an early
branch unlock, or a hidden-path reveal past a presumed dead end; a `breakthrough` event carries a
sourced Sojournal reference. Cadence is tuned (data) to ~once per 8–15 years for a heavily-invested
domain.

**Rationale.** Implements design §4.4 exactly: rare, seeded *and* earned, basic-science-weighted,
announced with a citation (Principle VIII).

**Alternatives rejected.** (a) Pure-random breakthroughs — forbidden (earned, not RNG). (b) Guaranteed
breakthroughs on spend — removes rarity/risk.

---

## R8 — Leapfrogging

**Decision.** A higher-tier technology whose prerequisites are **domain-UL floors** (rather than
intermediate products) becomes **available** once those ULs are reached by over-investment, even with no
intermediate product/heritage — at higher program cost/risk (no heritage discount, wider P80). The tree
data encodes which prereqs are UL-satisfiable (leapfroggable) vs product-required.

**Rationale.** Implements design §4.5; the tree's UL-floor prereqs are exactly the leapfrog seams.

**Alternatives rejected.** (a) Forbid skipping generations — removes a documented strategy. (b) Free
leapfrog — must cost the lost heritage and higher risk.

---

## R9 — Global science tide (knowledge half here; money in FA-06, clarified Q2)

**Decision.** Each domain has a **World UL** = exogenous baseline + aggregate of all factions' activity
(from `research/tide` params). A faction's private UL = World UL + its lead/lag. Factions **publish**
(raise World UL + prestige-eligible `publish` event, lose exclusivity) or **hold/patent** (keep lead,
slower tide). Trailing a domain by N levels makes RP cheaper there (catch-up discount, bounded by World
UL). The slice exposes declared **licence / partner / buy-in** interfaces that grant documented TRL/IP
credit; the **monetary settlement** is FA-06's.

**Rationale.** Implements design §5; the clarified knowledge-vs-money split keeps research money-free
(honest seam), while still owning the World-UL dynamics that bound runaway leads and keep the AI
relevant.

**Alternatives rejected.** (a) Model licensing income here — FA-06's job. (b) Per-faction-isolated UL
(no tide) — removes the competitive substrate and lets leads run away.

---

## R10 — Reliability: scalar per-use + heritage (clarified Q1:A)

**Decision.** A technology's **reliability** is a **scalar per-use success probability ∈ [0,1]**,
computed from a data-defined curve over `(TRL, accumulated flight-units, relevant domain UL)`, exposed
**alongside its raw inputs**. Flying below TRL 6 is refused; TRL-6 carries the steep documented penalty
easing through 7–9. **Flight Heritage** accrues from operational-use events (`register-heritage`,
driven by FA-04+), raising reliability asymptotically toward a per-tech ceiling and **discounting
derivative programs** (a declared derivative starts partway up the ladder).

**Rationale.** The clarified contract: one number FA-04 can gate mission risk on, with inputs exposed
for any duration/phase layering; the maturity→reliability model stays in FA-05 (design §2).

**Alternatives rejected.** (a) Structured/time-based reliability now — speculative; defer to FA-04 if
needed. (b) Raw inputs only — pushes the maturity model into FA-04, wrong home.

---

## R11 — Personnel: roster, traits, transitions, tacit-knowledge

**Decision.** Personnel are managed assets: scientists, engineers, programme managers, mission
controllers, diplomats (and astronauts, R12), each with a discipline, skill rating, age, morale and
sourced **traits** (Visionary/Closer/Maverick/Safe Hands…) whose modifiers (from `traits.ron`) shift
low-TRL vs qual progress, breakthrough odds, overrun variance and reliability. Hire/poach (relations-cost
signal)/train (multi-year)/retire are deterministic roster transitions. RP/DE **efficiency multipliers**
respond to over/under-staffing, domain mismatch and facility bottlenecks. Losing key personnel reduces
a faction's **effective UL** in their niche domain — a *recompute* over roster state, never a mutation
of the stored ground UL (tacit-knowledge as a derived value).

**Rationale.** Implements design §7 lightly; modelling effective-UL as a recompute keeps the ground
state clean and round-trips trivially (the loss is reproducible from roster + ground UL).

**Alternatives rejected.** (a) Subtract from stored UL on loss — corrupts ground state and can't recover
on re-hire. (b) No traits — loses the personnel-as-lever mechanic.

---

## R12 — Astronaut career pipeline + the FA-08 interface (clarified Q1 scope)

**Decision.** Astronauts are a personnel sub-type with a pipeline: **select → train** (multi-year,
facility/analog-gated) **→ ready → age out**, holding running **career dose** and **health** budgets and
morale. This slice owns the roster and career state and exposes readiness. A declared
**`crew-feedback`** command lets FA-08 apply in-mission **dose/health/psychological deltas**; crossing a
documented career limit deterministically removes an astronaut from the ready pool. In-mission ECLSS and
acute physiology are FA-08's.

**Rationale.** The clarified FA-05↔FA-08 split: FA-05 owns the career roster up to the mission boundary;
FA-08 feeds deltas back through this interface.

**Alternatives rejected.** (a) Whole crew lifecycle here — pulls ECLSS forward (FA-08's). (b) No astronaut
model here — drops a named half of the slice.

---

## R13 — Facilities & funding as opaque caller inputs

**Decision.** This slice takes **funded-staff** (the roster it owns), **facility-capability descriptors**
(a caller-supplied available-capability set per scenario), and an **RP/DE budget** (carried on the
allocation command) as inputs. It exposes facility-requirement gates and efficiency multipliers but
moves no money and builds no facilities. FA-06 binds real facilities, budgets and markets later without
data migration.

**Rationale.** The honest-seam pattern proven in FA-02 (research-gate stand-in) and FA-03 (opaque
faction ids / caller-supplied entitlement). Lets the research engine be exercised fully before the
economy exists.

**Alternatives rejected.** (a) Model facilities/economy here — FA-06's scope. (b) Abstract funding to a
flat RP/DE pool — loses the personnel/facility levers (Principle VI's "fund people and facilities").

---

## R14 — Faction-parametric engine (one model for all factions)

**Decision.** All state (UL, programs, technologies, heritage, personnel, insight) is **keyed by
faction**; every faction — player and AI — runs the **same** engine under identical rules (FR-POL-007).
FA-09 later supplies AI factions' *decisions* as journaled commands; difficulty tunes their funded
inputs/competence, never the model. This slice runs N factions in scenarios and the bench.

**Rationale.** FR-POL-007 mandates identical systems/physics for AI; a single faction-parametric engine
is the only way to honour that and keep determinism. The tide (US4) already needs multi-faction state.

**Alternatives rejected.** (a) Simplified competitor model — violates FR-POL-007 and forks the engine.
(b) Player-only now — the tide can't be modelled and FA-09 would need a second engine.

---

## R15 — Kernel/data touchpoints (no kernel change)

**Decision.** **No kernel code change.** Commands route through FA-02's `Command::ModulePayload` →
`SimModule::on_command`. New event classes (`breakthrough`, `dead-end-confirmed`, `test-failure`,
`program-milestone`, `trl-advance`, `publish`) are **data-registry** additions to `event-classes.ron`.
Research-data content is hashed and **pinned in saves** (extends FA-02's catalogue-hash guard).

**Rationale.** The module pattern is now established (FA-02/FA-03); nothing about research needs a kernel
primitive that `ModulePayload` + the data registry don't already provide.

**Alternatives rejected.** (a) A kernel "research service" — over-generalises. (b) Kernel event-class
code — the registry is data by design.

---

## R16 — Query surface

**Decision.** `ResearchSnapshot::from_core(&core, &module)` via kernel `with_slice`; pure functions
answer, **faction-scoped**: `maturity(faction, tech)` (TRL, scalar reliability + raw inputs, flyability),
`heritage(faction, tech)`, `understanding(faction, domain)` (private + world UL, gating status),
`program_status(faction, program)` (TRL, P50/P80-vs-actual, risk/dead-end index), and personnel/roster
summaries. No query returns another faction's private state.

**Rationale.** Identical seam to FA-02's planning queries / FA-03's world queries — pure fns over a
snapshot, called between ticks, IPC-serializable for the Tauri host and FA-04/06/09.

**Alternatives rejected.** (a) Mutable handles — break the read-only/determinism contract. (b)
Cross-faction visibility — leaks private leads; the tide is the only shared channel.

---

## Summary of decisions feeding Phase 1

| # | Decision | Primary artifacts |
|---|---|---|
| R1 | `sojourn-research` on `sojourn-core` only; FA-04 depends on it | plan structure |
| R2 | Time-stepped fixed cadence, warp-invariant | data-model §module, contracts/research-commands |
| R3 | f64 UL with diminishing returns + synergy; UL gating | data-model §domains |
| R4 | TRL steps: cost/min-time floor/facility+UL gate/S-curve/P50-P80 | contracts/tech-tree-data, data-model §programs |
| R5 | Test campaigns + failure-that-teaches + dead-end signal | data-model §campaigns |
| R6 | Constructive dead-end seeding + reachability sweep | contracts/tech-tree-data, data-model §seeding |
| R7 | Insight-pressure breakthroughs (seeded+earned) | data-model §breakthrough |
| R8 | Leapfrogging via UL-satisfiable prereqs | contracts/tech-tree-data |
| R9 | Tide: World UL + publish/hold + catch-up; money in FA-06 | contracts/research-commands, data-model §tide |
| R10 | Scalar per-use reliability + heritage | contracts/maturity-queries, data-model §reliability |
| R11 | Personnel roster/traits/transitions; tacit-knowledge recompute | data-model §personnel |
| R12 | Astronaut pipeline + FA-08 crew-feedback interface | contracts/crew-interface |
| R13 | Facilities/funding as opaque caller inputs | contracts/research-commands |
| R14 | Faction-parametric engine; FA-09 drives AI later | data-model §module |
| R15 | No kernel change; data-registry events; data-version pin | contracts/research-commands |
| R16 | `with_slice` + pure faction-scoped query surface | contracts/maturity-queries |
