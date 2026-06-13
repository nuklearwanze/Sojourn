# Implementation Plan: Vehicle Designer & Propulsion (FA-04)

**Branch**: `006-vehicle-designer` | **Date**: 2026-06-13 | **Spec**: `specs/006-vehicle-designer/spec.md`
**Input**: Feature specification from `/specs/006-vehicle-designer/spec.md`

## Summary

Build `sojourn-vehicle`, the slice where the tyranny of mass and Δv is *felt at design time*: a
component-composition designer that builds vehicles from researched parts and **derives** every
number from physics — total/dry/propellant mass, per-stage/mode Δv (the rocket equation), thrust and
T/W at gravity fields of interest, power generation/demand and margin, thermal/radiator balance,
composed reliability, static life-support sizing, EDL suitability, and a physical cost/build estimate
— **with full traceability** to sourced data and **no per-design magic numbers** (Principle II). The
propulsion families (chemical; electric ion/Hall/MPD/VASIMR/electrospray; nuclear-thermal;
nuclear-electric; gated frontier) are physical models, each producing the **`PropulsionEndpoint`
parameters** FA-02 flies — electric propulsion power-limited, nuclear-electric radiators carried as
first-class mass. Component reliability comes from FA-05's `maturity()` (TRL + flight-units + domain
UL), composed across the vehicle by a **reliability-block-diagram**; realism guards red-flag the
physically impossible while leaving marginal designs buildable.

Per the clarified boundary, **FA-04 owns design-time state** (the design library + cumulative
production counts) and **FA-02 keeps flight-time craft state** (live mass/propellant): a craft is
spawned from a design by carrying the designer's engine parameters + masses **inline** into FA-02's
existing `SpawnCraft`, and FA-02's burn-executor `consume` stays the sole live-mass mutation. That is
the one **additive `sojourn-astro` change** (inline engine parameters on spawn; the fixture path is
unchanged). No kernel change. The slice is a `SimModule` on FA-01 depending on `sojourn-astro` (the
endpoint shape + gravity) and `sojourn-research` (maturity); its derived outputs are **pure
query-time computations** composing the design with the current FA-05 maturity, so reliability and
cost track research without stored duplication.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/02/03/05).
**Primary Dependencies**: `sojourn-core` (kernel contracts), `sojourn-astro` (`PropulsionEndpoint`/`EngineDef` shape consumed by the propagator; body μ/radius for T/W), `sojourn-research` (the `maturity()`/`heritage()`/`understanding()` query contract), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`. No new third-party physics deps: the rocket equation, power/thermal balances, reliability-block-diagram and learning curve are in-crate. FA-06 will depend on `sojourn-vehicle` (cost estimate).
**Storage**: Data files only — `data/vehicle/` (component catalogue, propulsion family models, reliability/cost/life-support/EDL/solar-distance params, vehicle-class templates), all with `source` provenance and validated in CI.
**Testing**: `cargo test` (unit + integration: compose/derive, propulsion endpoints, reliability composition, power/thermal, guards, classes, cost, traceability); **analytic validation gates** (rocket-equation Δv, T/W, power-limited-EP thrust, mass-fraction identities) per the constitution testing mandate; kernel conformance (`conformance --module vehicle`); harness determinism gates; an FA-02 integration test flying a designer-built engine; `validate-data` extended to vehicle.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-vehicle`) implementing the FA-01 `SimModule` contract + a public read-only design-query API; one additive astro change; harness/bench/data extensions.
**Performance Goals**: Designer derivations are **sub-millisecond pure computations**; the design-query surface returns < 50 ms; with the full component catalogue + design library the kernel envelope (≥1 sim-year/min) holds trivially (the module is event/command-driven, no time-stepping).
**Constraints**: Full kernel determinism (ordered stores, libm-only, no wall-clock, no hidden randomness — derivations are pure); **no per-design/per-tech magic numbers** (Principle II — the engine reads all constants from data); SI units; reliability from FA-05 (no invented reliability); single-writer preserved (FA-04 owns design-time state, FA-02 owns flight-time craft state); analytic-case CI gates.
**Scale/Scope**: A sourced component catalogue (~40–80 components across the part classes); the 5 propulsion families as physical models; the documented vehicle-class templates; a per-faction design library (tens of designs); century horizons (designs are durable state, not stepped).

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS | Every component parameter, propulsion model value, reliability/cost/life-support/EDL constant and class template lives in `data/vehicle/*` with a `source`; `validate-data` (extended) fails CI on any missing source; speculative propulsion (frontier B5) is gated behind FA-05 research/breakthrough and flagged. |
| II. Physics authoritative / no magic numbers (this slice is an embodiment) | PASS | The designer computes capability from physics (rocket equation, power-limited EP, mass models, thermal balance) reading **all** constants from data; the engine carries no per-design/per-tech numbers; analytic validation cases (rocket-equation Δv, T/W, power-limited thrust, mass fractions) gate CI per the testing mandate. |
| III. Deterministic core | PASS | Derivations are pure deterministic functions (libm-only, no randomness, no wall-clock); the small design-library slice is exercised by double-run/roundtrip/replay/conformance gates. |
| IV. Headless / decoupled | PASS | Pure library module + read-only design-query functions; zero UI deps; everything driven/audited via harness scenarios. |
| V. Data-driven content | PASS | Components, propulsion models, all tuning params and class templates are schema-validated data; no new event-class registry churn beyond the heritage/production events (data-only). |
| VI. Research a modelled process | N/A (consumed) | Reliability/availability/heritage come from FA-05's contract; this slice reads them, never reinventing them. |
| VII. Tyranny of mass / Δv (this slice is the embodiment) | PASS | Mass and Δv are the dominant, surfaced constraints; money traces to physical mass (the cost estimate is mass/maturity-driven); ISRU/refuel relax m0 honestly via the rocket equation; crewed life-support sizing makes crewed harder. |
| VIII. Educational honesty | PASS | Full traceability (every number → sourced leaves) is the honesty contract; realism guards never present a physics cheat as feasible — only informed gambles. |
| IX. No combat/aliens | PASS | No weapons; nuclear-pulse (Orion, design B5.5) is intentionally absent except as a locked historical entry, inherited from the tech tree. |
| Engineering constraints | PASS | SI everywhere; sub-ms derivations; component-data version pinned in saves (extends FA-02/FA-03/FA-05 hash pattern); fully offline. |
| **Astro contract amendment** | NOTED (additive) | One additive `sojourn-astro` change: `SpawnCraft` accepts **inline engine parameters** (the `PropulsionEndpoint` shape) in addition to a catalogue-id, so designer-built engines fly without an engine-catalogue file. Empty/unused ⇒ identical to today; the FA-02 fixture path (engine-by-id) is unchanged and all FA-02 gates stay green. No kernel change. |

**Post-Phase-1 re-check (2026-06-13)**: design artifacts introduce no new violations; the astro
change is additive and leaves FA-02 gates green; single-writer is preserved (design-time vs
flight-time split); no kernel amendment. Gate remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/006-vehicle-designer/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R15)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── design-queries.md        # FR-VD-801: the read-only design-query + traceability surface (FA-06/09/10 seam)
│   ├── vehicle-commands.md       # FR-VD-802: commands (via ModulePayload) + events
│   ├── component-data.md         # FR-VD-101/301/803: component + propulsion + params data format, sourcing, analytic gates
│   └── propulsion-binding.md     # FR-VD-302 + the additive astro change: designer engines → FA-02 inline-spawn
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged this slice
├── sojourn-astro/               # FA-02 — ONE additive change: SpawnCraft accepts inline engine params (designer engines)
├── sojourn-research/            # FA-05 — unchanged (consumed via the maturity contract)
├── sojourn-vehicle/             # THIS SLICE — pure library, SimModule implementor (deps: core, astro, research)
│   ├── Cargo.toml               # deps: sojourn-core, sojourn-astro, sojourn-research, serde, postcard, libm, ron, thiserror
│   └── src/
│       ├── lib.rs               # public surface: VehicleModule, VehicleCommand, design queries
│       ├── ids.rs               # DesignId, ComponentId, FactionId
│       ├── catalogue.rs         # component catalogue + propulsion-family models (DATA) load + validation
│       ├── design.rs            # VehicleDesign: composition, staging, modes, redundancy blocks, class templates
│       ├── mass.rs              # dry/wet mass, mass fractions
│       ├── deltav.rs            # rocket equation per stage/mode; thrust + T/W vs supplied gravity
│       ├── propulsion.rs        # family models → PropulsionEndpoint params (EngineDef); power-limited EP; NEP radiator mass
│       ├── power.rs             # power balance (gen/demand/margin); solar-distance PV scaling
│       ├── thermal.rs           # thermal/radiator balance; waste-heat; power↔radiator coupling fixed-point
│       ├── reliability.rs       # reliability-block-diagram from FA-05 maturity (series × declared redundancy)
│       ├── lifesupport.rs       # static life-support sizing (consumables/closure/shield/accommodation)
│       ├── edl.rs               # landing/EDL suitability checks (T/W vs local g, heat-shield, ballistic coeff)
│       ├── cost.rs             # physical cost + build-time + learning curve (cumulative count)
│       ├── guards.rs            # realism red-flags
│       ├── trace.rs             # traceability tree (every derived output → sourced leaves)
│       ├── query.rs             # DesignSnapshot (FA-04 design + FA-05 maturity + supplied gravity) + pure derivation queries
│       └── module.rs            # SimModule: design-library slice, commands, no-op step, publish, save/load_slice
│   └── tests/                   # compose.rs, propulsion.rs, reliability.rs, power_thermal.rs, guards.rs,
│                                # classes.rs, cost.rs, trace.rs, integration_fa02.rs, conformance.rs, validation.rs
├── sojourn-harness/             # + `vehicle` scenario flag, validate-data vehicle, conformance --module vehicle, bench
data/
└── vehicle/
    ├── components.ron           # component catalogue (structures/tanks/power/thermal/avionics/comms/payload/EDL/landing/RCS/docking) sourced
    ├── propulsion.ron           # propulsion family physical models (Isp/thrust/power/throttle/mass/reliability-curve) sourced
    ├── params.ron               # reliability-block params, cost + learning-curve, life-support sizing, EDL, solar-distance model (sourced)
    ├── classes.ron              # vehicle-class templates
    └── validation.ron           # analytic validation cases + tolerances (rocket-equation Δv, T/W, power-limited thrust)
scenarios/                       # + vehicle_design.ron, vehicle_fly.ron (designer engine flown by FA-02)
```

**Structure Decision**: `sojourn-vehicle` is the first slice with **two upstream module
dependencies** — it depends on `sojourn-astro` (the endpoint/gravity it produces for) and
`sojourn-research` (the maturity it consumes), both already in the tree and CI-verified; the arrow
points one way (FA-06 will depend on vehicle). It owns only **design-time** state (the design
library + production counts), keeping FA-02's flight-time craft state single-writer; the bridge is
the one additive astro change (inline engine params at spawn) the spec deferred to this plan. The
derived outputs are **pure query-time computations** over a `DesignSnapshot` that composes the design
with the live FA-05 maturity — the same read-only, between-ticks, IPC-serializable seam FA-02/03/05
expose, so reliability and cost stay honest as research advances without storing a stale copy.

## Complexity Tracking

No constitution violations to justify — table intentionally empty. The slice is computation-heavy
(many derived quantities) but architecturally plain: one module crate, the established command/
event/query patterns, one additive astro change, no kernel amendment.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
