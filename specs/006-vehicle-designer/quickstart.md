# Quickstart: Vehicle Designer & Propulsion (FA-04)

Headless scenarios that exercise the slice end-to-end and double as the independent tests for each
user story. Run through `sojourn-harness` with the `vehicle` flag (installs the vehicle module +
loads `data/vehicle`; the FA-05 research module supplies maturity; FA-02 flies designer engines). No
UI anywhere.

## Build & gate

```pwsh
cargo test -p sojourn-vehicle                          # unit + integration + analytic validation
cargo run -p sojourn-harness -- validate-data data/vehicle   # schema + sources + analytic gates
cargo run -p sojourn-harness -- conformance --module vehicle
cargo run -p sojourn-harness -- verify    scenarios/vehicle_design.ron   # double-run bit-identity
cargo run -p sojourn-harness -- roundtrip scenarios/vehicle_design.ron --save-at-ticks <t1>,<t2>
cargo bench -p sojourn-harness --bench vehicle                          # derivation + query latency
```

## US1 — Compose & derive
1. Compose a vehicle from a sourced component set; assert dry/wet mass, per-stage Δv (`v_e·ln(m0/m1)`), thrust and T/W match `validation.ron` analytic values within tolerance.
2. Adding an unresearched/immature component is rejected/locked with the gating tech reported (via `availability`).
3. `trace()` of any output resolves to sourced data leaves; a CI check asserts no non-sourced constant.
4. `verify` double-run: all derived numbers bit-identical.

## US2 — Propulsion is physics
1. Instantiate each family; assert the produced `EngineDef` (exhaust velocity, max thrust, throttle, power-limited flag, masses) is FA-02-conformant.
2. An EP engine's delivered thrust scales with available power; its power source + radiator mass appear in dry mass.
3. A NEP engine carries reactor + shield + radiator mass; waste-heat rejection is computed.
4. `vehicle_fly.ron`: spawn an FA-02 craft from a designer engine (inline params); the propagator flies a coast + a burn unchanged, plan-vs-flown propellant agrees to the FA-02 tolerance.

## US3 — Reliability is earned
1. Component reliability equals FA-05 `maturity().reliability`; the composed vehicle reliability follows the reliability-block-diagram (lower with more low-TRL parts; higher with declared redundancy).
2. A sub-TRL-6 component is red-flagged (buildable, truthful risk).
3. `RegisterProduction` emits `vehicle-produced`; the host registers FA-05 heritage; the technology's reliability rises and a derivative starts higher.

## US4 — The designer refuses the impossible
1. Construct designs violating each guard (negative power margin, radiator shortfall, lander T/W < local g, Δv short of a stated requirement, over-thrust structure); assert each is red-flagged with the specific violated constraint.
2. A marginal design remains buildable with truthful reliability/risk; no guard is bypassable by a non-sourced value.

## US5 — Power & thermal balance
1. A high-power EP/NEP design: power gen/demand/margin from sourced data; radiator mass to reject the computed heat is in dry mass; an undersized radiator is flagged.
2. A solar-powered design evaluated at a distant body: PV generation falls with solar distance; margin reflects it.

## US6 — Every vehicle from one system
1. Build one design of each archetype from the shared component system; each validates and derives class-appropriate outputs.
2. Save a design (versioned class); a derivative inherits heritage discounts; `compare()` diffs two designs field-by-field.

## US7 — Cost & build-time estimate
1. `cost()` derives unit cost + build time from sourced mass/maturity params, fully traceable.
2. Rising `production_count` lowers unit cost along the learning curve.

## Determinism & performance
- `verify` + `roundtrip` + `conformance --module vehicle` pass; saves pin and verify the component-data version.
- Derivations are sub-millisecond; the design-query surface < 50 ms — checked by the `vehicle` bench.
