# Phase 0 Research: Bases & Construction (FA-07)

Decisions resolving the Technical Context against Constitution v1.0.0 (esp. Principles I/II/VII/VIII),
the FA-01/03/05/06 contracts in the tree, and the spec's clarified scope. Format: **Decision /
Rationale / Alternatives rejected**. No `NEEDS CLARIFICATION` remain (spec clarified 2026-06-14: three
scope forks + three composition models).

---

## R1 — Crate topology: a base slice above world/research/economy, coupled only to core

**Decision.** A new crate `crates/sojourn-base` takes a **hard dependency only on `sojourn-core`**. Site
facts (PP category, illumination, thermal, slope, hazard, radiation environment, resource grade),
module-tech maturity, and construction-delivery/ISRU-output status enter as **composed values / opaque
caller inputs** assembled by the host. Later slices (FA-08 life support, FA-09 politics) depend on
`sojourn-base`.

**Rationale.** The FA-04 C1 / FA-06 R1 finding: hard-linking every upstream crate over-couples the slice
and slows tests. The base needs *values* (a site's PP category, a tech's maturity, a delivery's mass),
not the upstream engines. Core-only keeps the dependency graph acyclic with **no new crate edges** and
unit-testable with stubs.

**Alternatives rejected.** (a) Depend on world+research+economy crates — heavy coupling, repeats the C1
mistake (confirmed Q3). (b) Put base logic in the host — violates Principle IV.

---

## R2 — Composed-value integration seams (the four inputs)

**Decision.** Four narrow input shapes the host composes and the base consumes:
- **SiteFacts** `{ pp_category, illumination, thermal_k, slope_deg, comms_visible, hazard_level,
  radiation_env (dose rate), resource_grade, solar_distance_m }` — from FA-03's site belief-state.
- **TechMaturity** `{ trl, understanding, flyable }` — from FA-05's `maturity()`/`understanding()`
  (module gating: D3/D4/D6/F/G/I10/I11).
- **DeliveryStatus** `{ project_id, module_id → delivered_mass_kg, crew_time_hr }` — from FA-06's
  `Project`/delivery accounting (the host bridges deliveries → commissioning).
- **IsruOutput** `{ base_id, commodity → rate_kg_per_day }` — from the FA-06 ISRU plants a base hosts.

Carried into **commands** at decision time and into the **BaseSnapshot** at query time. Tests feed
stubs; the harness feeds the real upstream queries.

**Rationale.** Narrow value types keep coupling explicit, serializable (IPC for the Tauri host), and
honest — the base acts on the **surveyed** site facts (never ground truth) and the **delivered** mass
(never an instant placement).

**Alternatives rejected.** (a) Pass whole upstream snapshots — leaks their surfaces/ground truth. (b)
Recompute ISRU/delivery in-base — duplicates FA-06.

---

## R3 — Emergent properties are pure query-time derivations (not stored)

**Decision.** A base's emergent properties (power margin, ECLSS closure, population capacity, shielding/
dose attenuation, self-sufficiency index, embargo result, hazard exposure) are **pure functions**
computed at query time over a `BaseSnapshot` composing the base slice (modules + commissioning state)
with the composed inputs. They are **not** stored in the slice.

**Rationale.** Properties depend on site facts, research and ISRU output that change over time;
recomputing keeps them honest (a base's shielding/closure tracks its modules and the world) without a
stale stored copy or invalidation logic. The slice stays small (compositions + commissioning), so
saves/round-trips are trivial and deterministic — the FA-04/FA-06 R3 pattern.

**Alternatives rejected.** (a) Store derived properties — stale vs site/research; invalidation
complexity. (b) Recompute in `step()` each tick — wasteful; derivations are query-time, between-ticks.

---

## R4 — Module catalogue: typed, class-specific, sourced

**Decision.** A `data/base/modules.ron` catalogue of module types, each typed by class-specific params:
`Habitat{crew_accommodation, pressurized_volume_m3}`, `Power{gen_w, solar}`, `Eclss{closure_fraction,
per_crew_day_kg, crew_support}`, `IsruHost{process_ref}`, `Science{...}`, `Storage{commodity,
buffer_capacity_kg}`, `Manufacturing{output_commodity, rate_kg_per_day, power_demand_w}`,
`Shielding{material, areal_density_kg_m2}`. Each carries `dry_mass_kg`, a `tech` reference (FA-05
gating), `power_demand_w`, and a `source`. No combat module (Principle IX, name+kind screen).

**Rationale.** Implements FR-BC-101/107; class-typed params mirror the FA-04 component catalogue and
keep all per-module numbers in sourced data.

**Alternatives rejected.** (a) A flat stat list — violates "emergent from physics" + Principle II. (b)
Per-base hand-set properties — the cardinal sin the spec forbids.

---

## R5 — Power margin = Σ generation (solar-distance-scaled PV) − Σ demand

**Decision.** Power margin = Σ over operational modules of generation (PV scaled by
`(ref/solar_distance)²` from `SiteFacts`; RTG/fission constant) minus Σ demand (habitat/ECLSS/ISRU/
manufacturing/science). A negative margin is a red-flag. (Reuses the FA-04 power-balance shape.)

**Rationale.** FR-BC-103; the additive, solar-distance-honest model gives a clean analytic gate
(adding a power module raises the margin by its sourced generation — SC-001).

**Alternatives rejected.** (a) Ignore solar distance — dishonest (a Jupiter base needs huge PV area).
(b) A fixed power "level" — non-emergent.

---

## R6 — Population capacity gated by ECLSS + power; ECLSS closure composed

**Decision.** **Population capacity** = `min(Σ habitat accommodation, Σ ECLSS crew_support)`. **Power is
a separate viability flag** — a negative power margin red-flags the base (R5) rather than fractionally
reducing population (there is no per-crew power figure; U1). **ECLSS closure fraction** is the **best
closure** among ECLSS modules (consistent with FA-04 life-support sizing; multiple ECLSS modules don't
multiply closure). Consumables demand = per-crew-day × population × (1 − closure). The deferred "ECLSS
composition across modules" question resolves to best-module here.

**Rationale.** FR-BC-104; population is a *gated* emergent value (a habitat with no ECLSS supports
nobody; a base with negative power is flagged non-viable). Best-module closure is the honest near-term
model (the highest-closure system sets the loop). Keeping power a flag (not a per-crew divisor) avoids
an undefined per-crew power figure.

**Alternatives rejected.** (a) Habitat accommodation alone — ignores life-support/power gating. (b)
Additive closure > 1 — unphysical.

---

## R7 — Shielding: mass-attenuation exponential (clarified Q3)

**Decision.** Radiation shielding's **dose-attenuation factor** = `exp(−Σᵢ ρxᵢ / λᵢ)`, summing **per
material in the exponent** over each shielding module's areal density `ρxᵢ` (kg/m²) and its **sourced
mass-attenuation length `λᵢ` per material** (regolith, water, polyethylene) from `params.ron` — so a
base mixing regolith + polyethylene composes correctly (the product of per-material attenuations).
Transmitted dose = `SiteFacts.radiation_env × factor`; a transmitted dose above a sourced crew limit is
a red-flag. Regolith-built shielding (R10) adds areal density without launched mass.

**Rationale.** FR-BC-105; the standard radiation-shielding physics, data-driven per material. The
sum-in-exponent is the physically correct multi-material composition; the single-material special case
(doubling ρx squares the attenuation) is a clean analytic gate.

**Alternatives rejected.** (a) Linear/threshold — unphysical cliff, no material difference (Q3-B). (b)
Per-material lookup curves — heavier authoring for little gain (Q3-C).

---

## R8 — Construction project: delivery-driven commissioning

**Decision.** A base is built by a **ConstructionProject**: each planned module carries a
`required_mass_kg + crew_time_hr` demand. A module **commissions** (becomes operational) when the
composed `DeliveryStatus` shows its mass and crew-time landed; commissioning is **command-driven**
(`DeliverToBase`) the host issues as FA-06 deliveries arrive, so the base never reads cross-slice state
during `step`. A partial base exposes only commissioned modules' contributions (R3). Crew-time /
construction-robotics capacity gates the commissioning rate.

**Rationale.** FR-BC-201/202/203; building far from Earth is a logistics problem. Command-driven
commissioning mirrors FA-06's `OperateIsru` — the composed value arrives in the payload, keeping the
step deterministic and self-contained.

**Alternatives rejected.** (a) Read FA-06 delivery state inside `step` — cross-slice read not available.
(b) Instant placement — violates the tyranny-of-mass premise.

---

## R9 — Siting & planetary protection: trust-the-caller siting + query-time guards

**Decision.** Founding a base trusts the caller for structural validity (site exists, class valid); the
**PP-category and suitability guards** are **query-time red-flags** (composing `SiteFacts`): a Special
Region without containment, a solar-only base in permanent shadow (illumination), a shielding shortfall
vs the site dose, an unbuildable slope, no comms visibility. A **forward-contamination consequence**
(science/PP-value-loss marker) is emitted/representable on violation. Honest-seam: the guards never
silently permit a violation (the FA-04 realism-guard pattern).

**Rationale.** FR-BC-301…304 + Principle I; PP is a real modelled constraint. Query-time guards (where
the snapshot already composes site facts) keep the founding command deterministic and the checks honest
(they act on the surveyed belief-state).

**Alternatives rejected.** (a) Hard-block siting in `on_command` by reading the world slice — not
available cross-slice. (b) Silent acceptance of PP violations — dishonest, forbidden.

---

## R10 — On-site production & import reduction (FA-06/FA-07 split, confirmed Q-scope)

**Decision.** FA-06 owns resource-extraction ISRU (output composed in as `IsruOutput`). FA-07 owns the
**construction use** of local materials: **regolith construction** converts local regolith/metals (from
ISRU output / local stock) into **shielding/structure mass that is not launched**, and base
**manufacturing modules** produce spares/materials locally. The construction project's `required_mass`
for a module can be satisfied by **local production substituting for delivered mass**, so the imported-
mass demand falls measurably as local production rises (FR-BC-401…404, Principle VII). Conversion rates
are sourced (D6 regolith-construction params).

**Rationale.** Confirmed scope split; regolith shielding "you didn't launch" is the felt
mass-relaxation. Modeling it as delivered-mass substitution keeps it in the same construction accounting.

**Alternatives rejected.** (a) FA-07 re-implements extraction ISRU — duplicates FA-06 (Q-scope-B). (b)
Local production as a free bonus — dishonest; it consumes power/feedstock at sourced rates.

---

## R11 — Self-sufficiency index: limiting factor / minimum (clarified Q1)

**Decision.** The **self-sufficiency / sustainability index** = `min` over the per-loop closure ratios —
ECLSS air/water, food, materials, power, spares — each `ratio = local_supply / demand` capped at 1. The
weakest closed loop bounds the index; improving the binding loop raises it (monotonic). Loop definitions
+ demands are sourced (`params.ron`).

**Rationale.** FR-BC-501 + Q1; physically honest (one unclosed loop forces resupply) with a clean
monotonicity gate (SC-006) and a direct tie to the embargo test (R12).

**Alternatives rejected.** (a) Weighted average — a strong loop masks a fatal weak one (Q1-B). (b)
Product — a single zero zeroes the index, harder to interpret (Q1-C).

---

## R12 — Embargo stress test: analytic rate + buffer (clarified Q2)

**Decision.** The **embargo stress test** (Homestead) is an **analytic** per-loop check: a loop survives
iff `production_rate ≥ demand_rate` **or** `stored_buffer ≥ (demand − production) × embargo_span`; the
base survives iff **every loop survives**. Buffers come from **storage modules**. No time-stepping (the
dynamic sim is FA-08); a pure deterministic derivation over the closure ratios (R11) + buffers.

**Rationale.** FR-BC-502 + Q2; stays static (consistent with the dynamics→Slice-8 boundary), uses
storage modules meaningfully (a well-stocked base bridges a temporary deficit), and is deterministic.

**Alternatives rejected.** (a) Time-stepped buffer depletion — pulls a dynamic sim forward (borders
FA-08), costs warp (Q2-B). (b) Steady-state only — ignores buffers, makes storage pointless (Q2-C).

---

## R13 — Base-state query surface + traceability

**Decision.** `BaseSnapshot::from_core(&core, &base_module, inputs)` via kernel `with_slice` over the
base slice, composing the R2 inputs. Pure functions answer: a base's emergent properties + traceability
trees; its red-flags (power/shielding/PP/suitability); its **production/consumption** at its location
(for the economy); its **habitat/closure/shielding/population** state (for FA-08); its self-sufficiency
index + embargo result; its **settlement milestones**; and `compare()` of two bases. Faction-scoped
where site belief is involved.

**Rationale.** FR-BC-601/602/603 + Principle VIII; the identical read-only between-ticks seam as
FA-02…06 — pure fns over a composed snapshot, IPC-serializable for FA-08/09/10 and the Tauri host; the
trace is the honesty contract.

**Alternatives rejected.** (a) Mutable handles — break read-only/determinism. (b) Store outputs (R3).

---

## R14 — Determinism, data-version pin, events, cadence

**Decision.** Derivations are pure (libm-only, no randomness/wall-clock); **zero random streams**
(commissioning is delivery-driven and deterministic — the FA-04 pattern). Module/param/class data is
content-hashed and **pinned in saves** (extends the FA-02…06 pattern). New **event classes** (data
registry, `data/kernel/event-classes.ron`): `module-commissioned` (LogOnly), `base-operational`
(LogOnly), `pp-violation` (Interrupt), `embargo-result` (LogOnly), `settlement-milestone` (Interrupt).
**Cadence**: daily `step` (`cadence_ticks = 86_400`) advancing time-based construction bookkeeping;
commissioning lands via commands. **No kernel change.**

**Rationale.** Mirrors every prior slice's determinism discipline; interrupt-class events (PP violation,
settlement milestone) feed the FA-01 interrupt-and-pause loop ("stop on something that matters").

**Alternatives rejected.** (a) Seeded construction randomness — unjustified in v1; commissioning is
deterministic. (b) New kernel event plumbing — events are data registry entries.

---

## Summary of decisions feeding Phase 1

| # | Decision | Primary artifacts |
|---|---|---|
| R1 | `sojourn-base` above all, coupled only to core | plan structure |
| R2 | Composed-value seams (4 inputs) | contracts/integration-seams |
| R3 | Emergent properties are pure query-time derivations | contracts/base-queries, data-model |
| R4 | Typed, sourced module catalogue | data-model, contracts/base-data |
| R5 | Power margin = Σgen(solar-scaled) − Σdemand | data-model |
| R6 | Population gated by ECLSS+power; best-module closure | data-model |
| R7 | Mass-attenuation exponential shielding | data-model, contracts/base-data |
| R8 | Delivery-driven commissioning | contracts/base-commands, data-model |
| R9 | Trust-the-caller siting + query-time PP/suitability guards | contracts/base-queries, data-model |
| R10 | On-site regolith construction → import-mass substitution | data-model, contracts/base-data |
| R11 | Limiting-factor self-sufficiency index | data-model |
| R12 | Analytic rate+buffer embargo test | data-model |
| R13 | Composed base-query surface + traceability | contracts/base-queries |
| R14 | Determinism; zero streams; data-version pin; events; daily cadence | contracts/base-commands, contracts/base-data |
