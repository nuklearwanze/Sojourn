# Contract: Component / Propulsion / Params Data + Analytic Gates (FR-VD-101/301/803)

How the designer's physics enters the game as sourced, schema-validated data, and how its outputs
are validated against analytic cases.

## Data files (`data/vehicle/`)

- `components.ron` — the **component catalogue** across the part classes (structures/tanks, power,
  thermal/radiators, avionics/GNC, comms, life-support & accommodation, payloads, EDL kit, landing
  gear, RCS, docking, engines). Each entry: id, class, **researching `tech`** (FA-05), `dry_mass_kg`,
  class-specific params, and a **mandatory `source`**.
- `propulsion.ron` — the **propulsion family models** (chemical; electric; nuclear-thermal;
  nuclear-electric; frontier): exhaust velocity, thrust, throttle, `power_limited`, rated power, the
  mass model (engine/feed/reactor/shield), waste heat, propellant + density, boil-off. All sourced.
- `params.ron` — reliability-block params, cost coefficients + learning-curve exponent, life-support
  sizing constants, EDL (heat-shield/ballistic) constants, the solar-distance PV model. All sourced.
- `classes.ron` — vehicle-class templates (required component classes, default modes/staging).
- `validation.ron` — analytic cases + tolerances.

Content is hashed and **pinned in saves** (extends FA-02/03/05). No combat/weapons components;
nuclear-pulse (Orion, design B5.5) is absent except as a locked historical entry (Principle IX).

## Validation (`validate-data vehicle`, CI)

- Schema + non-empty `source` for every component, propulsion model, param and class.
- Every component's `tech` resolves (against the FA-05 tech tree); every class template's required
  components exist; redundancy/staging references resolve.
- No combat/weapons component class or category.

## Analytic validation gates (FR-VD-803, constitution testing mandate)

`validation.ron` cases the designer's derivations MUST reproduce within tolerance — run as CI gates:

| Case | Asserts |
|---|---|
| Rocket-equation Δv | a known (v_e, m0, m1) → `Δv = v_e·ln(m0/m1)` to tolerance |
| Thrust-to-weight | thrust / (mass·g) for a known config matches |
| Power-limited EP thrust | delivered thrust ∝ available/rated power |
| Mass-fraction identity | dry + propellant + payload = total; staging sums consistently |
| Reliability composition | series ∏ and parallel 1−∏(1−r) match hand-computed values |

These make the physics auditable (Principle II) and are part of the determinism suite.
