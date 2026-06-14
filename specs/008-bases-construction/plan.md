# Implementation Plan: Bases & Construction (FA-07)

**Branch**: `008-bases-construction` | **Date**: 2026-06-14 | **Spec**: `specs/008-bases-construction/spec.md`
**Input**: Feature specification from `/specs/008-bases-construction/spec.md`

## Summary

Build `sojourn-base`, the slice where **settlement becomes the sum of physical truths**: a base or
orbital station is **composed from modules** (habitat, power, ECLSS, ISRU host, science, storage,
manufacturing, shielding) sited at a world Site or dynamical location, and its properties **emerge from
physics** — power margin, ECLSS closure fraction, population capacity, radiation shielding (a
mass-attenuation exponential model), the limiting-factor self-sufficiency index, hazard exposure —
never a hand-set level, every number traceable to sourced module/site inputs (Principles II/VIII). A
base is built by a **construction project** whose modules commission only as delivered mass and
crew-time land (the logistics/tyranny-of-mass problem), and **on-site regolith construction** turns
local material into shielding/structure that is never launched — the path to self-sufficiency, tested
by an analytic resupply-embargo check (the Homestead condition).

Per the three confirmed scope decisions and the FA-04/FA-06 decoupling lesson, **`sojourn-base` depends
only on `sojourn-core`**. Site facts (PP category, illumination, thermal, slope, hazard, radiation
environment, resource grade — on the surveyed belief-state), module-tech maturity (FA-05), and
construction-delivery/ISRU-output status (FA-06) all flow in as **composed values** the host assembles;
the host bridges FA-06's delivery accounting → base construction progress. The dynamic in-mission
simulation (consumables over time, dose accumulation, physiology, ECLSS failure) is **Slice 8**, which
consumes this static base state; resource-extraction ISRU is **Slice 6**, whose output a base hosts.
The slice is a `SimModule` on FA-01 with a daily construction step (delivery-driven commissioning), a
read-only **base-query surface** (emergent properties, red-flags, production/consumption,
self-sufficiency, embargo, milestones, compare) for FA-08/09/10, and **no kernel/world/research/economy
change** — their outputs are read as values.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01…06).
**Primary Dependencies**: `sojourn-core` (kernel contracts) **only** as a crate dependency; `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy; the shielding exponential uses `libm::exp`), `ron` (data), `thiserror`. **No `sojourn-world`/`-research`/`-economy` crate dependency** — Site facts, tech maturity and delivery/ISRU status flow in as composed values/opaque inputs (the FA-04 C1 / FA-06 R1 decoupling). No new third-party deps: the emergent-property derivations (power balance, mass-attenuation shielding, limiting-factor self-sufficiency, analytic embargo, regolith-construction substitution) are in-crate.
**Storage**: Data files only — `data/base/` (module catalogue with class-specific params; shielding mass-attenuation lengths per material; closure-loop definitions + dose limits; regolith-construction/build params; base-class templates) and `data/base/validation.ron` (analytic cases: power-margin additivity, shielding attenuation, limiting-factor index, embargo survival), all carrying `source` provenance and validated in CI.
**Testing**: `cargo test` (unit + integration: compose/derive emergent properties, construction commissioning, siting/PP guards, on-site production import-reduction, self-sufficiency + embargo, exposure/compare); **analytic validation gates** (power-margin additivity, shielding exp-attenuation, limiting-factor min, embargo rate+buffer) per the constitution testing mandate; kernel conformance (`conformance --module base`); harness determinism gates; `validate-data` extended to base.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-base`) implementing the FA-01 `SimModule` contract + a public read-only base-query API; harness/bench/data extensions. No kernel/world/research/economy change.
**Performance Goals**: Emergent-property derivations are **sub-millisecond pure computations**; the daily step advances delivery-driven commissioning in time proportional to active construction projects; the base-query surface returns < 50 ms; dozens of bases × hundreds of modules sustain the kernel envelope (≥1 sim-year/min) at high warp.
**Constraints**: Full kernel determinism (ordered `BTreeMap`/`BTreeSet` stores, libm-only, no wall-clock; commissioning is delivery-driven and deterministic); **emergent properties derive from physics/sourced data — no per-base/per-module magic numbers** (Principle II); mass traceability and local production relaxing the mass/Δv constraint (Principle VII); SI units (areal density kg/m², closure ∈ [0,1], power W, crew count); acts on the **surveyed site belief-state**, never hidden truth; analytic-case CI gates; base-data version pinned in saves.
**Scale/Scope**: A sourced module catalogue (~25–50 module types across the 8 classes); base-class templates (orbital station, surface base, settlement); per-faction bases (dozens) each with up-to-hundreds of modules; century horizons (bases are durable, construction-stepped).

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS | Every module parameter, shielding attenuation length, closure-loop param, dose limit, regolith-construction rate and class template lives in `data/base/*` with a `source`; `validate-data` (extended) fails CI on any missing source. |
| II. Physics authoritative / no magic numbers (this slice is an embodiment) | PASS | Base properties are **derived** from physics (power balance, mass-attenuation shielding, limiting-factor closure, analytic embargo) reading **all** constants from data; the engine carries no per-base/per-module numbers; analytic gates (power additivity, shielding exp-attenuation, index min, embargo) gate CI. |
| III. Deterministic core | PASS | Derivations are pure deterministic functions (libm-only, no randomness/wall-clock); commissioning is delivery-driven; double-run / roundtrip / mutate / replay / conformance gates. |
| IV. Headless / decoupled | PASS | Pure library module + read-only base-query functions; zero UI deps; depends only on `sojourn-core`; everything driven/audited via harness scenarios. |
| V. Data-driven content | PASS | Module catalogue, shielding/closure/construction params and class templates are schema-validated data; new event classes are data registry entries. |
| VI. Research a modelled process | N/A (consumed) | Module-tech maturity comes from FA-05 as composed values; this slice reads it, never reinventing research. |
| VII. Tyranny of mass / Δv (this slice is an embodiment) | PASS | Building far from Earth is a logistics problem (delivered-mass + crew-time gate commissioning); **regolith construction adds shielding/structure that is never launched**, relaxing the mass constraint honestly; self-sufficiency is the felt destination. |
| VIII. Educational honesty | PASS | Full traceability (every emergent number → sourced module/site leaves); siting guards never silently permit a PP violation; the embargo test tells the survival truth. |
| IX. No combat/aliens | PASS | A base is infrastructure, never a fortress; no weapons/defence modules. |
| Engineering constraints | PASS | SI everywhere; sub-ms derivations; base-data version pinned in saves (extends the FA-02…06 hash pattern); fully offline. |
| **Cross-slice coupling** | NOTED (none added) | `sojourn-base` depends **only on `sojourn-core`**; world/research/economy outputs flow in as composed values (the FA-04/FA-06 decoupling), so the dependency graph gains no new crate edges and the slice is unit-testable with stubs. No kernel/world/research/economy change. |

**Post-Phase-1 re-check (2026-06-14)**: design artifacts introduce no new violations; no kernel
amendment; no upstream-crate change (composed-value decoupling); single-writer preserved; commissioning
is deterministic and delivery-driven. Gate remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/008-bases-construction/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R14)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── base-queries.md          # The read-only base-query + traceability surface (FA-08/09/10 seam)
│   ├── base-commands.md         # Commands (via ModulePayload) + events (commission/operational/pp-violation/milestone)
│   ├── base-data.md             # Module/shielding/closure/construction/class data formats, sourcing, analytic gates
│   └── integration-seams.md     # The composed-value inputs: site facts, tech maturity, delivery + ISRU status
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged this slice
├── sojourn-world/               # FA-03 — unchanged (Site facts consumed as composed values)
├── sojourn-research/            # FA-05 — unchanged (module-tech maturity consumed as values)
├── sojourn-economy/             # FA-06 — unchanged (delivery accounting + ISRU output consumed as values)
├── sojourn-base/                # THIS SLICE — pure library, SimModule implementor (dep: core only)
│   ├── Cargo.toml               # deps: sojourn-core, serde, postcard, libm, ron, thiserror
│   └── src/
│       ├── lib.rs               # public surface: BaseModule, BaseCommand, base queries
│       ├── ids.rs               # FactionId, BaseId, ModuleId, ModuleTypeId(String), SiteId(String), ProjectId
│       ├── catalogue.rs         # module catalogue (DATA) load + validation; module class params
│       ├── base.rs              # Base + module composition + commissioning state; class templates
│       ├── inputs.rs            # composed-value shapes (SiteFacts, TechMaturity, DeliveryStatus, IsruOutput, BaseInputs)
│       ├── power.rs             # power balance (gen incl. solar-distance PV scaling vs demand; margin)
│       ├── shielding.rs         # mass-attenuation exponential shielding → dose attenuation (Q3)
│       ├── lifesupport.rs       # static ECLSS closure composition + population-capacity sizing (gated)
│       ├── construction.rs      # construction project: per-module delivered-mass + crew-time → commissioning
│       ├── production.rs        # on-site regolith construction / manufacturing → local material; import reduction
│       ├── sustainability.rs    # limiting-factor self-sufficiency index (Q1) + analytic embargo (Q2)
│       ├── siting.rs            # PP-category + site-suitability guards (red-flags)
│       ├── trace.rs             # traceability tree (every emergent output → sourced leaves)
│       ├── query.rs             # BaseSnapshot (base slice + composed inputs) + pure derivation queries
│       └── module.rs            # SimModule: base slice, commands, daily commissioning step, publish, save/load_slice
│   └── tests/                   # compose.rs, construction.rs, siting.rs, production.rs, sustainability.rs,
│                                # exposure.rs, conformance.rs, validation.rs, common/mod.rs
├── sojourn-harness/             # + `base` scenario flag, validate-data base, conformance --module base, bench
data/
└── base/
    ├── modules.ron              # module catalogue (habitat/power/eclss/isru-host/science/storage/manufacturing/shielding) sourced
    ├── params.ron               # shielding attenuation lengths, dose limits, closure-loop defs, regolith-construction/build rates (sourced)
    ├── classes.ron              # base-class templates (orbital station, surface base, settlement)
    └── validation.ron           # analytic validation cases + tolerances (power additivity, shielding, index min, embargo)
scenarios/                       # + base_construction.ron (found → deliver → commission → derive → embargo)
```

**Structure Decision**: `sojourn-base` consumes the world, research and economy **outputs** but, per
the FA-04 C1 / FA-06 R1 lesson, takes a hard dependency **only on `sojourn-core`** — Site facts, tech
maturity, and delivery/ISRU status arrive as **composed inputs the host assembles**, so the dependency
graph gains no new crate edges and the slice is unit-testable with stubs. It owns only the **base
composition + construction state** (modules, commissioning, projects) as the single writer; the
**emergent properties** (power margin, closure, population, shielding, self-sufficiency, embargo,
hazard) are **pure query-time derivations** over a `BaseSnapshot` that composes the slice with the live
site/maturity/ISRU inputs — the same read-only between-ticks seam FA-02…06 expose, so a base's truth
tracks the world and research without storing a stale copy. The daily construction step is
delivery-driven (commissioning as composed delivery status lands via commands), keeping commissioning
deterministic; the cadence question the spec deferred resolves to a daily step (R14).

## Complexity Tracking

No constitution violations to justify — table intentionally empty. The slice is computation-heavy (many
emergent derivations) but architecturally plain: one module crate depending only on core, the
established command/event/query patterns, composed-value decoupling, no kernel or upstream change.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
