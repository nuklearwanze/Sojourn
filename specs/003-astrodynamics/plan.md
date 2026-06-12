# Implementation Plan: Astrodynamics & Flight (FA-02)

**Branch**: `003-astrodynamics` | **Date**: 2026-06-13 | **Spec**: `specs/003-astrodynamics/spec.md`
**Input**: Feature specification from `/specs/003-astrodynamics/spec.md`

## Summary

Build `sojourn-astro`, the first real game-system module on the FA-01 kernel: the authoritative
deterministic propagator (fixed-step numerical integration of craft under multi-body gravity
from railed celestial bodies, with J2, SRP, drag and continuous low-thrust), reference frames
and SOI bookkeeping with state-continuous handoffs, manoeuvre nodes with rocket-equation
budgeting and finite-burn execution, seeded execution error and TCMs, and the analytic planning
tier (two-body/patched-conic predictions, Lambert-based porkchop solving, hyperbolic-encounter
flyby chaining, low-thrust arc estimates, research-gated low-energy routes, aerocapture
corridors) delivered through a read-only planning-query surface. Small bodies are divertible
(rail → craft-grade propagation → re-rail). Everything is validated headlessly against sourced
analytic cases (Hohmann, periods, J2 regression, flyby) as CI gates, and the module passes the
kernel's conformance/determinism suite. One additive kernel amendment is required: a typed
module-command payload (`ModulePayload`) so domain commands (create node, divert body) flow
through the journal without the kernel learning domain types.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01).
**Primary Dependencies**: `sojourn-core` (kernel contracts), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `slotmap` (craft/node stores), `ron` (data fixtures), `thiserror`. **No new third-party math/physics dependencies**: vectors, Kepler solver, Lambert solver and integrator are in-crate (research R1–R5). Harness gains astro scenarios; no UI anywhere.
**Storage**: Data files only — `data/astro/` (test catalogue, fixture engines, config: step tiers, thresholds, diversion budget, error model) and `data/astro/validation.ron` (analytic cases + tolerances), all with `source` fields, validated in CI.
**Testing**: `cargo test` (unit + integration incl. the analytic validation suite), kernel conformance suite (`conformance --module astro`), harness determinism gates (verify/roundtrip/replay with astro scenarios), criterion benches for the propagation budget.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-astro`) implementing the FA-01 `SimModule` contract + a public read-only planning-query API; harness/bench extensions.
**Performance Goals**: SC-007: synthetic full load (3,000+ railed bodies, 200 propagated craft, mixed coast/burn) sustains ≥ 1 sim-year/wall-minute on the reference machine; planner queries (≤40×40 porkchop, node prediction, flyby solve) < 100 ms each; railed-body positions computed on demand (never stepped).
**Constraints**: Kernel determinism obligations in full (ordered iteration, libm-only transcendentals, declared streams for execution error, no wall-clock); state-driven step tiering only (warp invariance); planning queries pure (no mutation, no streams, unjournaled — FR-ASTRO-409); SI units; no top speed/reactionless motion; craft exert no gravity on anything; no craft-craft collision (defaults recorded in research R12).
**Scale/Scope**: 3,000+ railed bodies (positions on demand), ≥200 craft, ≤16 simultaneously diverted small bodies (config default, data-tunable), century spans, ~10 planning verbs.

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS | Every quantitative constant (test-catalogue bodies, fixture engines, tolerances, atmosphere models, error sigmas) lives in `data/astro/*.ron` with `source` citations (textbook/standards references for the idealised test catalogue; real-data sources arrive with FA-03); `validate-data` extended to cover it in CI. |
| II. Physics authoritative / no magic numbers | PASS (this slice is the embodiment) | The numerical propagator is the source of truth; planners are labelled approximations reconciled against it (FR-ASTRO-401/402); the engine reads all constants from data; analytic validation cases (Hohmann, period, J2, flyby) gate CI per the constitution's testing mandate. |
| III. Deterministic core | PASS | Module conforms to the kernel contract: fixed-step integration with state-driven tiering, named streams for execution error, ordered stores, libm-only; double-run/replay/conformance gates run with astro scenarios. |
| IV. Headless / decoupled | PASS | Pure library module + planning-query functions; zero UI deps; everything exercised via harness scenarios. |
| V. Data-driven content | PASS | Catalogue, engines, tolerances, step tiers, thresholds, event classes all in schema-validated data; new event classes added via the existing data registry (no kernel changes for events). |
| VI–VIII | N/A / PASS | Research gate consumed as input (VI lands with FA-05); reconciliation honesty satisfies VIII (no fiction presented as truth). |
| IX. No combat/aliens | PASS | Domain-pure orbital mechanics. |
| Engineering constraints | PASS | SI everywhere; performance budgets tracked by bench; saves round-trip via slice serde (kernel-driven). |
| **Kernel contract amendment** | NOTED (additive) | FA-01's `Command` enum cannot carry typed domain commands (`ModuleCommand{key, value:i64}` is too weak for "create node (epoch, Δv vector)"). Plan adds `Command::ModulePayload { module, kind, payload: Vec<u8> }` + `SimModule::on_command(...)` — additive, domain-agnostic, journaled like any command. Contract docs updated as part of this slice (contracts/astro-commands-events.md). This keeps FR-CORE-505 (no domain logic in kernel) intact. |

**Post-Phase-1 re-check (2026-06-13)**: design artifacts introduce no violations; the kernel
amendment is additive and domain-agnostic. Gate remains PASS.

## Project Structure

### Documentation (this feature)

```text
specs/003-astrodynamics/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── planning-queries.md      # FR-ASTRO-409: the read-only planner surface (FA-10's seam)
│   ├── propulsion-interface.md  # FR-ASTRO-501: the contract FA-04 implements
│   ├── body-catalog.md          # FR-ASTRO-103/107: the contract FA-03 implements (+ rails/divert)
│   └── astro-commands-events.md # Module commands (via kernel ModulePayload) + event classes
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 (additive amendment: Command::ModulePayload +
│   └── src/…                    #   SimModule::on_command; contract docs updated)
├── sojourn-astro/               # THIS SLICE — pure library, SimModule implementor
│   ├── Cargo.toml               # deps: sojourn-core, serde, postcard, libm, slotmap, ron, thiserror
│   └── src/
│       ├── lib.rs               # public surface: AstroModule, planning queries, interfaces
│       ├── math.rs              # Vec3<f64>, state vectors, libm-only helpers (in-crate, no dep)
│       ├── bodies/              # catalogue interface, rails (Kepler ephemeris), divert lifecycle
│       ├── frames.rs            # heliocentric/body-centred/rotating conversions; rebasing
│       ├── soi.rs               # SOI hierarchy, crossing detection, dominant-body bookkeeping
│       ├── forces/              # point gravity, J2, SRP (+occultation), drag, thrust
│       ├── integrator.rs        # fixed-step RK4 with state-tiered substeps (research R1/R2)
│       ├── propulsion.rs        # PropulsionEndpoint trait + fixture engines (FA-04 contract)
│       ├── craft.rs             # craft store (slice state), mass/propellant, flight status
│       ├── maneuver/            # nodes, plans, finite-burn execution, exec error, TCM, stationkeeping
│       ├── planner/             # twobody.rs, lambert.rs, porkchop.rs, flyby.rs, lowthrust.rs,
│       │                        # lowenergy.rs, aero.rs, reconcile.rs, query.rs (FR-ASTRO-409 surface)
│       └── module.rs            # SimModule impl: manifest, step, on_event, on_command, publish, serde
│   └── tests/                   # validation.rs (analytic suite), propagation.rs, maneuvers.rs,
│                                # planner.rs, soi.rs, divert.rs, conformance.rs
├── sojourn-harness/             # + astro scenario support (module registration), scenarios, benches
data/
├── kernel/event-classes.ron     # + soi-crossing, impact, atmosphere-entry, plan-invalidated,
│                                #   propellant-exhausted, aero-violation (data-only addition)
└── astro/
    ├── test-catalog.ron         # sourced idealised system (star, planets, moon, atmosphere body)
    ├── engines.ron              # fixture engines (hydrolox-class, ion-class) with sources
    ├── config.ron               # step tiers, divergence thresholds, diversion budget, error sigmas
    └── validation.ron           # analytic cases + tolerances (Hohmann, period, J2, flyby, spiral)
scenarios/                       # + astro_transfer.ron, astro_flyby.ron, astro_lowthrust.ron …
```

**Structure Decision**: `sojourn-astro` is the first sibling module crate the workspace was
designed for. It depends only on `sojourn-core` and the shared deterministic building blocks;
the kernel gains one additive, domain-agnostic command-routing capability. The planning-query
surface is plain public crate API (pure functions over snapshots) — exactly the seam a Tauri
host or the harness calls between steps, IPC-serializable like the core's DTOs.

## Complexity Tracking

No constitution violations to justify — table intentionally empty.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
