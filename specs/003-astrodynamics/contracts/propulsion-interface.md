# Contract: Propulsion Interface (FR-ASTRO-501/502)

Defined and consumed by `sojourn-astro`; **implemented by FA-04** (vehicle designer & propulsion).
Fixture implementations ship in this slice for tests and scenarios.

## The trait

```rust
pub trait PropulsionEndpoint {
    fn total_mass(&self) -> f64;          // kg (dry + propellant + payload)
    fn dry_mass(&self) -> f64;            // kg
    fn propellant(&self) -> f64;          // kg remaining
    fn exhaust_velocity(&self) -> f64;    // m/s (= Isp · g0; one canonical quantity)
    fn max_thrust(&self) -> f64;          // N at full throttle and full power
    fn throttle_range(&self) -> (f64, f64); // e.g. (0.4, 1.0) deep-throttle limits; (1.0,1.0) fixed
    fn available_power(&self) -> f64;     // W (electric propulsion supply)
    fn power_limited(&self) -> bool;      // thrust ∝ available_power when true
    fn drag_area(&self) -> f64;           // m², cannonball drag model
    fn srp_area(&self) -> f64;            // m², cannonball SRP model
    fn consume(&mut self, propellant_kg: f64);  // called by the burn executor only
}
```

## Physical couplings the consumer enforces (this slice)

- **Mass flow**: thrust T for substep dt consumes `T·dt / exhaust_velocity()` kg. Never thrust
  without mass flow (SC-005); `consume` of more than remaining propellant is impossible — the
  executor cuts thrust at the exhaustion instant (sub-step accuracy, FR-ASTRO-307).
- **Power limiting**: when `power_limited()`, delivered thrust ≤ the endpoint's
  `max_thrust() × (available_power / rated power)` relationship as exposed through
  `max_thrust`/`available_power` (the endpoint owns its power curve; the consumer only respects
  the cap each substep). Free thrust is structurally impossible.
- **Throttle**: commanded throttle clamps into `throttle_range()`; below-range commands are
  deterministic command rejections.
- **Rocket equation**: planning uses `Δv = v_e · ln(m0/m1)` from these exact quantities, so
  plan-vs-flown propellant agreement is testable to 0.1% (SC-005).

## Fixture implementations (this slice; sourced parameters)

| Fixture | Class | Parameters (data/astro/engines.ron, with sources) |
|---|---|---|
| `chem-hydrolox` | impulsive stand-in | Isp ≈ 450 s class, high thrust, throttle (0.6–1.0) |
| `ep-ion` | low-thrust stand-in | Isp ≈ 3000 s class, mN–N thrust, power-limited |

## Obligations on FA-04 (when it implements this)

- All quantities SI; all parameter values sourced (Principle I).
- `consume` is the only mutation path and is invoked only by this slice's burn executor during
  command-applied burns/arcs (keeps single-writer discipline intact: the endpoint state lives
  in FA-04's slice; the call crosses via the kernel's event/command machinery or a published
  mutable-endpoint binding agreed at FA-04 planning time — to be settled in FA-04's plan, with
  this trait as the binding shape).
- Boil-off and other time-dependent propellant losses are FA-04 state changes; this slice
  simply reads the resulting `propellant()` (and plans honestly against it).
