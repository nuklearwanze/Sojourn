# Implementation Plan: Life Support & Crew (FA-08)

**Branch**: `009-life-support-crew` | **Date**: 2026-06-14 | **Spec**: `specs/009-life-support-crew/spec.md`
**Input**: Feature specification from `/specs/009-life-support-crew/spec.md`

## Summary

Build `sojourn-crew`, the slice that makes **crewed missions materially harder than robotic ones** —
the **dynamic, time-evolving** model of crewed life that FA-07 deferred to it. A crewed asset (a vehicle
in transit or an occupied base) consumes O₂/water/food over time against its ECLSS closure fraction;
each crew member accumulates radiation dose (a continuous GCR rate plus seeded SPE storms, attenuated by
shielding) with a **REID** career limit; micro-gravity deconditioning and psychological load build over
the mission, reducing a **multiplicative** crew-capability metric; ECLSS hardware degrades and can fail;
and entry/descent/landing carries a seeded crew-risk roll. Every stochastic outcome (SPE storms, ECLSS
failures, EDL rolls, anomalies) is a **multiplicative hazard** — `base_rate × ∏(sourced factor
multipliers)` — so a low-maturity, under-maintained, over-subscribed, high-psych craft **earns** its
failure probability. **Loss-of-crew is a real, modelled consequence** (a physical loss + an emitted
event); its political fallout is FA-09's.

This is the project's **first genuinely dynamic, seeded-stream slice** (closer to FA-05's stepped model
than FA-04/07's pure derivations): the per-crew/per-asset health state is **stored slice state evolved
on the daily step** with named seeded streams, while derived queries (REID, capability, viability) are
pure functions over that state. Per the three confirmed scope decisions and the FA-04/06/07 decoupling,
**`sojourn-crew` depends only on `sojourn-core`**: vehicle/base static sizing (closure capability, shield
attenuation, population, endurance, EDL suitability), the crew roster + age/sex + traits + ECLSS-tech
maturity, and ops capacity/light-time/abort-reach all flow in as **composed values** the host assembles.
**No kernel/vehicle/research/economy/base change** — their outputs are read as values.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01…07).
**Primary Dependencies**: `sojourn-core` (kernel contracts) **only** as a crate dependency; `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy; REID/hazard curves use `libm::exp`/`libm::pow`), `ron` (data), `thiserror`, `rand_core` (the kernel stream trait). **No `sojourn-vehicle`/`-research`/`-economy`/`-base` crate dependency** — their outputs flow in as composed values/opaque inputs (the FA-04 C1 / FA-06 R1 decoupling). No new third-party deps: consumables depletion, the REID dose→risk curve, deconditioning/psych accrual, the multiplicative-hazard failure/anomaly/EDL model and the capability product are in-crate on the kernel's seeded streams.
**Storage**: Data files only — `data/crew/` (consumables rates + closure tiers; radiation: GCR rates, SPE storm params, storm-shelter attenuation, dose→REID curve; physiology: deconditioning rates + countermeasure/artificial-gravity effectiveness + capability curves; psychology: load accrual + anomaly hazard; ECLSS: failure base rates + maturity/maintenance/heritage multipliers; EDL: per-body difficulty incl. the Mars gap; hazard base rates + viability thresholds + the 3% REID threshold) and `data/crew/validation.ron` (analytic cases), all carrying `source` provenance and validated in CI.
**Testing**: `cargo test` (unit + integration: consumables/closure, dose+REID, deconditioning+artificial-gravity, psychology+anomaly, ECLSS failure, EDL crew-risk, exposure/loss-of-crew); **analytic validation gates** (make-up-mass identity, REID monotonicity in dose, multiplicative-hazard factor monotonicity, Mars-EDL > airless) per the constitution testing mandate; kernel conformance (`conformance --module crew`); harness determinism gates (verify/roundtrip/mutate — the **first slice that exercises mutate heavily** given its seeded streams); `validate-data` extended to crew.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-crew`) implementing the FA-01 `SimModule` contract + a public read-only crew-query API; harness/bench/data extensions. No kernel/vehicle/research/economy/base change.
**Performance Goals**: The daily step advances per-asset consumption/dose/deconditioning/psych + seeded daily rolls in time proportional to active crewed assets × crew; derived queries (REID, capability, viability) are sub-millisecond pure computations; dozens of crewed assets × hundreds of crew sustain the kernel envelope (≥1 sim-year/min) at high warp.
**Constraints**: Full kernel determinism (ordered `BTreeMap`/`BTreeSet` stores, libm-only, no wall-clock; **all stochastic outcomes from named seeded streams** threaded via `ctx.rng(path)`); **no per-asset/per-crew magic numbers** (Principle II — all physiology/ECLSS/radiation/EDL constants from sourced data); SI units (dose Sv, mass kg, time days/seconds, volume m³); crewed materially harder than robotic (Principle VII); acts on composed values, never hidden truth; analytic-case CI gates; crew-data version pinned in saves.
**Scale/Scope**: A sourced parameter set across the six sub-systems; dozens of per-faction crewed assets; hundreds of per-astronaut health records; century horizons (durable, daily-stepped, seeded-event-driven).

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS | Every physiology/ECLSS/radiation/EDL parameter (GCR rates, dose→REID curve, deconditioning rates, countermeasure effectiveness, ECLSS failure rates, per-body EDL difficulty, hazard base rates) lives in `data/crew/*` with a `source`; `validate-data` (extended) fails CI on any missing source. |
| II. Physics authoritative / no magic numbers | PASS | Crew/ECLSS outcomes derive from sourced models (consumables make-up, REID dose→risk, multiplicative-hazard reliability, deconditioning accrual) reading **all** constants from data; the engine carries no per-asset numbers; analytic gates (make-up identity, REID monotonicity, hazard factor monotonicity, Mars>airless) gate CI. |
| III. Deterministic core | PASS | The **first heavily-seeded slice**: every SPE storm, ECLSS failure, EDL roll and anomaly draws from a **named seeded stream** (`ctx.rng(path)`); ordered stores; libm-only; no wall-clock; double-run / roundtrip / **mutate** / replay / conformance gates. |
| IV. Headless / decoupled | PASS | Pure library module + read-only crew-query functions; zero UI deps; depends only on `sojourn-core`; everything driven/audited via harness scenarios. |
| V. Data-driven content | PASS | All six sub-systems' params + thresholds are schema-validated data; new event classes are data registry entries. |
| VI. Research a modelled process | N/A (consumed) | ECLSS-tech maturity/heritage and the astronaut roster come from FA-05 as composed values; this slice reads them, never reinventing research. |
| VII. Tyranny of mass / Δv (this slice is the embodiment) | PASS | Crewed is materially harder than robotic: consumables + ECLSS mass, radiation/dose ceilings, deconditioning, ECLSS-failure and EDL crew-risk all apply only to crew (a robotic asset carries none); closing the ECLSS loop trades launched mass for technology and risk. |
| VIII. Educational honesty | PASS | Real radiation (GCR/SPE, REID), physiology (deconditioning, countermeasures) and EDL models with sourced parameters; loss-of-crew is never silently absorbed; no misinformation. |
| IX. No combat/aliens | PASS | Crew loss is a modelled **safety** consequence (consumables/dose/ECLSS/EDL), never combat; no weapons. |
| Engineering constraints | PASS | SI everywhere; sub-ms derivations; crew-data version pinned in saves (extends the FA-02…07 hash pattern); fully offline. |
| **Cross-slice coupling** | NOTED (none added) | `sojourn-crew` depends **only on `sojourn-core`**; vehicle/research/economy/base outputs flow in as composed values (the FA-04/06/07 decoupling), so the dependency graph gains no new crate edges and the slice is unit-testable with stubs. No kernel/upstream change. |

**Post-Phase-1 re-check (2026-06-14)**: design artifacts introduce no new violations; no kernel
amendment; no upstream-crate change (composed-value decoupling); single-writer preserved; all randomness
is seeded-stream. Gate remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/009-life-support-crew/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R14)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── crew-queries.md          # The read-only crew-state query surface (FA-09/10 seam)
│   ├── crew-commands.md         # Commands (via ModulePayload) + events (spe/eclss-failure/anomaly/loss-of-crew/grounded)
│   ├── crew-data.md             # Consumables/radiation/physiology/psych/ECLSS/EDL data formats, sourcing, analytic gates
│   └── integration-seams.md     # The composed-value inputs: asset sizing, env facts, crew roster, maturity, ops/abort
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged this slice
├── sojourn-vehicle/             # FA-04 — unchanged (life-support sizing + EDL suitability consumed as values)
├── sojourn-research/            # FA-05 — unchanged (crew roster + age/sex + ECLSS maturity consumed as values)
├── sojourn-economy/             # FA-06 — unchanged (ops/crew-time/light-time/abort-reach consumed as values)
├── sojourn-base/                # FA-07 — unchanged (base static habitat/closure/shielding state consumed as values)
├── sojourn-crew/                # THIS SLICE — pure library, SimModule implementor (dep: core only)
│   ├── Cargo.toml               # deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core
│   └── src/
│       ├── lib.rs               # public surface: CrewModule, CrewCommand, crew queries
│       ├── ids.rs               # FactionId, AssetId, AstronautId(String), MissionId
│       ├── params.rs            # the sourced parameter set (consumables/radiation/physiology/psych/eclss/edl) load + validation
│       ├── inputs.rs            # composed-value shapes (AssetSizing, EnvFacts, CrewRoster/AstronautFacts, TechMaturity, OpsLoad, CrewInputs)
│       ├── asset.rs             # CrewedAsset + CrewMember slice state (consumables, ECLSS, per-member health)
│       ├── consumables.rs       # consumption depletion + ECLSS-closure make-up; viability
│       ├── radiation.rs         # GCR + seeded SPE accrual; shielding/shelter; dose→REID curve (age/sex)
│       ├── physiology.rs        # deconditioning accrual + countermeasures (artificial gravity); capability factor
│       ├── psychology.rs        # psych load accrual (duration/comms-lag/confinement); anomaly contribution
│       ├── eclss.rs             # ECLSS reliability (maturity/heritage), degradation, maintenance, multiplicative-hazard failure
│       ├── edl.rs               # EDL crew-risk multiplicative hazard (suitability × body × crew state); Mars gap
│       ├── hazard.rs            # the shared multiplicative-hazard composition (base × Π factors, clamped) + capability product
│       ├── trace.rs             # traceability tree (any derived figure → sourced leaves)
│       ├── query.rs             # CrewSnapshot (slice + composed inputs) + pure derivation queries (REID/capability/viability)
│       └── module.rs            # SimModule: crew slice, commands, daily seeded step, publish, save/load_slice
│   └── tests/                   # consumables.rs, radiation.rs, physiology.rs, psychology.rs, eclss.rs, edl.rs,
│                                # exposure.rs, conformance.rs, validation.rs, common/mod.rs
├── sojourn-harness/             # + `crew` scenario flag, validate-data crew, conformance --module crew, bench
data/
└── crew/
    ├── consumables.ron          # per-crew-day O2/water/food/N2 rates + closure-tier params sourced
    ├── radiation.ron            # GCR rates per environment, SPE storm arrival/magnitude, shelter attenuation, dose→REID curve sourced
    ├── physiology.ron           # deconditioning rates + countermeasure/artificial-gravity effectiveness + capability curves sourced
    ├── psychology.ron           # psych-load accrual + comms-lag/confinement sensitivities + anomaly hazard sourced
    ├── eclss.ron                # ECLSS failure base rates + maturity/maintenance/heritage multipliers sourced
    ├── edl.ron                  # per-body EDL difficulty (Mars gap) + suitability factors sourced
    ├── params.ron               # hazard base rates, viability thresholds, the 3% REID threshold sourced
    └── validation.ron           # analytic cases + tolerances (make-up identity, REID monotonic, hazard monotonic, Mars>airless)
scenarios/                       # + crew_mission.ron (occupy → accrue dose/decon/psych → SPE → ECLSS failure → EDL → loss-of-crew)
```

**Structure Decision**: `sojourn-crew` is the **first dynamic, seeded-stream slice** — its per-crew/
per-asset **health state is stored slice state evolved on the daily step** (consumption, dose accrual,
deconditioning, psych, ECLSS degradation, seeded daily rolls), unlike the pure query-time derivations of
FA-04/07. Following the FA-04 C1 / FA-06 R1 lesson it still takes a hard dependency **only on
`sojourn-core`**, consuming vehicle/base sizing, the crew roster + age/sex, ECLSS-tech maturity and
ops/light-time/abort-reach as **composed inputs the host assembles** — so the dependency graph gains no
new crate edges and the slice is unit-testable with stubs. The **derived** figures (REID, capability,
viability) are pure functions over the stored state + composed sizing; the shared **multiplicative-hazard**
composition (`hazard.rs`) backs every seeded event (Q1) and the capability **product** (Q3). The daily
seeded step mirrors FA-05's pattern (`ctx.rng(path)` per named stream), resolving the cadence question.

## Complexity Tracking

No constitution violations to justify — table intentionally empty. The slice is **broad and dynamic**
(six interacting sub-systems + seeded streams) but architecturally consistent: one module crate
depending only on core, the established command/event/query/seeded-stream patterns, composed-value
decoupling, no kernel or upstream change. Breadth is managed by decomposition into independent
sub-modules (consumables, radiation, physiology, psychology, eclss, edl) over a shared hazard primitive,
each behind its own tests and analytic gates.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
