# Phase 1 Data Model: Vehicle Designer & Propulsion (FA-04)

Entities, fields, relationships, validation and derivations for `sojourn-vehicle`. Ordered stores
are `BTreeMap` (determinism). All quantitative content is sourced data (Principles I/II); the
**engine carries no per-design/per-tech magic numbers**. "Slice state" is serialized in the module
slice (small: the design library + production counts). "Derived" = computed at query time from the
design + FA-05 maturity + supplied gravity (R3), never stored.

---

## 1. Component & propulsion data (immutable module data; `data/vehicle/`)

### Component (`components.ron`)
| Field | Type | Rules |
|---|---|---|
| `id` | `ComponentId` | unique |
| `class` | enum | Structure \| Tank \| Power \| Thermal \| Avionics \| Comms \| LifeSupport \| Payload \| EdlKit \| Landing \| Rcs \| Docking \| Engine |
| `tech` | `TechId` (FA-05) | the technology that researches it (availability/maturity via `maturity()`) |
| `dry_mass_kg` | f64 | structural/component mass |
| params | class-specific | e.g. Tank: `capacity_kg`, propellant type; Power: `gen_w`, `solar` flag; Thermal: `reject_w_per_kg`; LifeSupport: `closure_fraction`, `per_crew_day_kg`; Structure: `thrust_limit_n` |
| `source` | string | mandatory (CI-enforced) |

### Propulsion model (`propulsion.ron`) — Engine-class components' physical model
| Field | Type | Rules |
|---|---|---|
| `id`, `tech`, `source` | — | as above |
| `family` | enum | Chemical \| Electric \| NuclearThermal \| NuclearElectric \| Frontier |
| `exhaust_velocity_m_s` | f64 | = Isp·g₀ (one canonical quantity, FA-02 contract) |
| `max_thrust_n` | f64 | at full throttle + rated power |
| `throttle` | `(f64, f64)` | deep-throttle limits |
| `power_limited` | bool | Electric/NuclearElectric ⇒ true (thrust ∝ available power) |
| `rated_power_w` | f64 | EP power draw at full thrust |
| `engine_mass_kg`, `feed_mass_kg` | f64 | mass model |
| `reactor_mass_kg`, `shield_mass_kg` | f64 | nuclear families (first-class dry mass) |
| `waste_heat_w` | f64 / derived | heat to reject (drives radiator mass, R7) |
| `propellant`, `density_kg_m3` | — | propellant model |
| `boiloff_frac_per_day` | f64 | cryo boil-off (FA-02 reads resulting `propellant()`) |

### Params (`params.ron`), Classes (`classes.ron`), Validation (`validation.ron`)
- `params.ron` — reliability-block params, cost coefficients + learning-curve exponent, life-support
  sizing constants, EDL (heat-shield/ballistic) constants, solar-distance PV model. All sourced.
- `classes.ron` — vehicle-class templates (required component classes, default modes/staging).
- `validation.ron` — analytic cases + tolerances (rocket-equation Δv, T/W, power-limited thrust,
  mass-fraction identities) gating CI.

---

## 2. The design (slice state)

### VehicleDesign — *slice state*
| Field | Type | Rules |
|---|---|---|
| `id` | `DesignId` | unique; versioned |
| `faction` | `FactionId` | owner (opaque) |
| `class` | string | a `classes.ron` template id |
| `stages` | `[Stage]` | ordered; each a set of component instances + a propulsion mode |
| `redundancy` | `[RedundancyBlock]` | declared parallel blocks (reliability-block-diagram, R8) |
| `mission_reqs` | `{ dv?, endurance_days?, target_body_g?, dose_limit? }` | optional stated requirements the guards check against |
| `parent` | `Option<DesignId>` | derivative lineage (heritage discounts via the parent's techs) |
| `production_count` | u32 | cumulative units produced (learning curve, R11) |

`Stage`: `{ components: [ComponentId], mode: PropulsionMode, propellant_kg }`. Validation: referenced
components/techs exist; stages well-ordered; redundancy blocks reference real components.

### VehicleSlice — *the owned, serialized state*
`designs: BTreeMap<DesignId, VehicleDesign>`, `next_design`, `data_hash` (component-data version pin).
(Component/propulsion/params/class data is immutable module data, hashed + pinned — not duplicated.)

---

## 3. Derived outputs (query-time; NOT stored — R3)

Computed by `query.rs` over a `DesignSnapshot` (design + FA-05 maturity + supplied gravity). Each
carries a **traceability tree** (R12) to sourced leaves.

| Output | Derivation |
|---|---|
| **Mass** | dry = Σ component `dry_mass_kg` (+ derived reactor/radiator/power/feed/shield); wet = dry + Σ propellant |
| **Δv** | per stage/mode `v_e·ln(m0/m1)`; modes never blended (edge case) |
| **Thrust / T-W** | engine thrust; T/W = thrust / (mass × supplied `g`) per gravity field |
| **Power balance** | gen (PV×solar-distance, RTG, fission) − demand, per mode; margin ≥ 0 in some mode |
| **Thermal balance** | waste-heat → radiator area/mass (carried as dry mass); power↔radiator fixed point (R7) |
| **Reliability** | per-component = FA-05 `maturity().reliability`; composed by reliability-block-diagram (series ∏, redundancy parallel 1−∏(1−r)) (R8) |
| **Life-support sizing** | consumables/closure-fraction mass + endurance, shield mass vs dose, accommodation (crewed; R9) |
| **EDL suitability** | T/W vs target-body g; heat-shield adequacy + ballistic coefficient (atmospheric); throttle/guidance fit (R10) |
| **Cost / build-time** | mass × maturity coefficients × learning curve(`production_count`) (R11) |
| **Red-flags** | the `guards` pass (R13) over all the above |

### Propulsion Endpoint (derived; the FA-02 binding)
Per engine, the designer derives a `PropulsionEndpoint`-shaped `EngineDef` (exhaust velocity, max
thrust, throttle, rated power, power-limited flag, masses, boil-off) — carried **inline into FA-02's
`SpawnCraft`** at spawn (R2). Power-limited EP delivers thrust ∝ available power; NEP reactor +
radiator mass are in dry mass.

---

## 4. Module slice & manifest (kernel contract)

### VehicleModule manifest
| Field | Value |
|---|---|
| `id` | `vehicle` |
| `owned_slice` | `vehicle/slice` (design library + production counts) |
| `publishes` | `vehicle/status` (design count) |
| `reads` | `kernel/status` |
| `streams` | — (derivations are deterministic, no randomness) |
| `emits` | `vehicle-produced` (heritage-relevant production; consumed by the host → FA-05 RegisterHeritage) |
| `subscribes` | — |
| `cadence` | high / no-op step (event/command-driven; designs don't evolve with time) |

- **Heritage**: lives in **FA-05** (its `RegisterHeritage`); FA-04 **reads** it via `maturity()` and
  applies derivative discounts. `vehicle-produced` events let the host drive FA-05 heritage.
- **Pinning**: component-data content hash pinned in saves (extends FA-02/03/05; R14).
- **Conformance/determinism**: ordered stores, libm-only, no streams/wall-clock → passes
  `conformance --module vehicle` and the harness double-run/roundtrip/replay gates.

---

## Entity relationship summary

```text
Component (class, tech, mass, params, source) ──researched by──▶ FA-05 Technology (maturity)
     │ composed into                                                   │ maturity().reliability
     ▼                                                                 ▼
VehicleDesign (stages, modes, redundancy, mission_reqs, parent, production_count)
     │ derive (pure, query-time, + supplied gravity)
     ├─▶ Mass · Δv · T/W · Power · Thermal · Reliability(block-diagram) · LifeSupport · EDL · Cost
     │        each with a Traceability tree → sourced leaves
     ├─▶ Red-flags (guards)
     └─▶ EngineDef(s) ──inline at spawn──▶ FA-02 Craft (flight-time mass/propellant; `consume`)
vehicle-produced event ──host──▶ FA-05 RegisterHeritage ──▶ raises maturity ──▶ raises reliability
```
