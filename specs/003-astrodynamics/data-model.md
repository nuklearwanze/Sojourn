# Data Model: Astrodynamics & Flight (FA-02)

Rust-shaped pseudo-declarations; the module's slice obeys the kernel's serde/canonical-encoding
obligations (ordered containers only). SI units throughout: metres, m/s, kg, ns.

## 1. Math & state primitives

```rust
struct Vec3 { x: f64, y: f64, z: f64 }            // in-crate; libm-only ops
struct StateVec { r: Vec3, v: Vec3 }              // position m, velocity m/s
struct Elements {                                  // Keplerian elements (rails, planner)
    sma: f64, ecc: f64, inc: f64, raan: f64, argp: f64,
    mean_anomaly_at_epoch: f64, epoch_ns: i64,
}
enum Regime { WellConditioned, LowConfidence }     // planner honesty tag (FR-ASTRO-402)
```

## 2. Bodies (consumed catalogue + motion state)

```rust
struct BodyId(u32);                                // catalogue index, stable per data version
struct BodyDef {                                   // DATA (data/astro/test-catalog.ron; FA-03 later)
    id: BodyId, name_id: String,
    mu: f64,                                       // gravitational parameter m³/s²  [source]
    radius: f64,                                   // mean radius m                  [source]
    j2: Option<f64>,                               // oblateness                     [source]
    rotation_period_s: Option<f64>,
    atmosphere: Option<AtmosphereDef>,             // exponential model              [source]
    gravitating: bool,                             // far-field gravity flag (clarified rule)
    divertible: bool,                              // small bodies only (FR-ASTRO-107)
    parent: Option<BodyId>,                        // orbit hierarchy
    elements: Elements,                            // rail definition about parent
    srp_reference: Option<f64>,                    // solar flux at 1 AU-equivalent  [source]
    source: String,
}
struct AtmosphereDef { interface_alt_m: f64, rho0: f64, scale_height_m: f64, source: String }

// Motion state lives in the astro slice (BodyDef is immutable data):
enum BodyMotion {
    Railed,                                        // default: state_at(t) from elements
    Diverted { state: StateVec, dominant: BodyId, mass: f64 },   // craft-grade propagation
    ReRailed { elements: Elements },               // post-diversion rail
}
// Transitions (FR-ASTRO-107): Railed → Diverted (DivertBody command; small+divertible only;
// budget-checked) → ReRailed (ReRailBody command once stable) — each continuity-exact at the
// transition tick, journaled, deterministic. Planets/major moons: never leave Railed.
```

## 3. Craft (owned slice state)

```rust
struct CraftId(u64);                               // slotmap-backed stable handle
enum FlightStatus { Propagating, Impacted { body: BodyId, tick: u64 },
                    EntryHandoff { body: BodyId, tick: u64 } }   // EDL-slice handoff
struct Craft {
    id: CraftId, name_id: String,
    dominant: BodyId,                              // SOI owner; state is in its frame (R6)
    state: StateVec,                               // body-centred inertial
    engine: EngineRef,                             // fixture engine id now; FA-04 endpoint later
    mass: MassState,                               // total/dry/propellant (kg)
    throttle: f64,                                 // 0..1
    guidance: Option<GuidanceArc>,                 // active continuous-thrust law
    status: FlightStatus,
    step_tier: Tier,                               // derived each step; persisted for escalation decl
}
struct MassState { dry: f64, propellant: f64 }     // total = dry + propellant; invariants: ≥ 0
// Invariants: craft exert no gravity; no craft-craft collision (R12); no state change
// without integrated force or journaled command (FR-ASTRO-104).
```

## 4. Propulsion interface (contract to FA-04)

```rust
trait PropulsionEndpoint {                         // contracts/propulsion-interface.md
    fn total_mass(&self) -> f64;
    fn dry_mass(&self) -> f64;
    fn propellant(&self) -> f64;
    fn exhaust_velocity(&self) -> f64;             // m/s (Isp·g0)
    fn max_thrust(&self) -> f64;                   // N
    fn throttle_range(&self) -> (f64, f64);
    fn available_power(&self) -> f64;              // W; power-limited mode caps thrust
    fn power_limited(&self) -> bool;
    fn drag_area(&self) -> f64;                    // m² (cannonball drag)
    fn srp_area(&self) -> f64;                     // m² (cannonball SRP)
    fn consume(&mut self, propellant_kg: f64);     // debited by the burn executor
}
struct EngineDef { /* DATA fixtures: hydrolox-class, ion-class — sourced parameters */ }
```

## 5. Manoeuvres & plans

```rust
struct NodeId(u64);
struct ManeuverNode {
    id: NodeId, craft: CraftId, epoch_tick: u64,
    dv_prn: Vec3,                                  // prograde/radial/normal m/s (impulsive plan)
    predicted: ConicPrediction,                    // planner-tier outcome + regime tag
    feasibility: Feasibility,                      // Ok | InsufficientDeltaV {..} | Invalidated {..}
    kernel_event: bool,                            // scheduled maneuver-node event
}
struct Plan {                                      // ordered chain
    nodes: Vec<NodeId>, aim: Option<AimPoint>,     // e.g. target body + arrival window
    total_dv: f64, divergence: DivergenceState,
}
struct GuidanceArc { law: GuidanceLawId, start_tick: u64, end_condition: EndCondition }
enum GuidanceLawId { Tangential /* data-extensible */ }
struct ExecutionErrorCfg { sigma_mag: f64, sigma_point_rad: f64, enabled: bool, source: String } // DATA
struct StationKeepingSchedule { craft: CraftId, cadence_ticks: u64, budget_dv: f64 }
// Burn execution: finite burn from node → thrust-tier integration; losses emerge;
// propellant via rocket equation through the endpoint; exhaustion → cut + event + invalidate.
```

## 6. Planner DTOs (read-only query surface, FR-ASTRO-409)

```rust
struct AstroSnapshot { /* cheap copy: craft states + catalogue handle + tick — pure inputs */ }
struct OrbitSummary { elements: Elements, period_s: Option<f64>, apsides: (f64, f64), regime: Regime }
struct TrajectorySample { t_ns: i64, r: Vec3, frame: FrameId }     // polyline point
struct PorkchopQuery { from: Location, to: Location, depart_span: (i64, i64),
                       arrive_span: (i64, i64), grid: (u16, u16), max_revs: u8 }
struct PorkchopCell { dv_total: f64, c3: f64, tof_s: f64, solvable: bool }
struct EncounterSolution { v_inf_in: Vec3, v_inf_out: Vec3, periapsis_m: f64,
                           turn_angle_rad: f64, valid: bool, regime: Regime }
struct AeroCorridor { shallow_limit: f64, steep_limit: f64,        // flight-path angles
                      predicted_exit: OrbitSummary, per_pass_apo_drop: f64, limits_source: String }
struct ReconciliationReport { plan_predicted: StateVec, propagated: StateVec,
                              miss_distance_m: f64, threshold_m: f64, exceeded: bool }
struct TcmSolution { node: ManeuverNode, residual_miss_m: f64 }
// All DTOs serde-serializable (IPC-ready); queries are pure functions:
// fn porkchop(&AstroSnapshot, &PorkchopQuery) -> PorkchopGrid, etc. Never journaled.
```

## 7. Module integration (kernel contracts)

```rust
// Manifest: id "astro"; owned slice "astro/slice"; publishes "astro/status" ONLY (flat
// scalars: craft_count, active_burns, diverted_count, worst_divergence_m, next_node_tick,
// any_fine_tier). Per-craft detail flows through the planning-query surface, not views.
// KNOWN FOLLOW-UP: per-craft watchability (watch templates over craft fields, e.g.
// "propellant below X") requires per-craft views when FA-10/ops demand it. Reads
// "kernel/status"; emits maneuver-node,
// soi-crossing, impact, atmosphere-entry, plan-invalidated, propellant-exhausted,
// aero-violation (all in data/kernel/event-classes.ron); subscribes module-command;
// streams ["astro/exec-error"]; cadence = coast tier (config); escalations: any craft
// in burn/encounter/atmosphere tier (published flag field).
// Commands arrive via kernel Command::ModulePayload { module:"astro", kind, payload } →
// AstroCommand enum (postcard): CreateNode, EditNode, DeleteNode, CommitPlan, SetThrottle,
// SetGuidanceArc, ScheduleStationKeeping, CancelStationKeeping, DivertBody, ReRailBody,
// SetResearchGate, SpawnCraft (fixtures/scenarios), DespawnCraft.
```

## 8. Configuration & validation data

```rust
struct AstroConfig {                               // DATA data/astro/config.ron [sources]
    coast_step_s: u64, encounter_step_s: u64, burn_substep_s: f64,
    encounter_radius_factor: f64,                  // tier trigger
    divergence_threshold_m: f64,
    diversion_budget: u32,                         // default 16
    exec_error: ExecutionErrorCfg,
    lambert_max_iter: u32, lambert_tol: f64,
    source: String,
}
struct ValidationCase { id: String, kind: CaseKind, params: …, expected: f64,
                        tolerance_frac: f64, source: String }    // DATA validation.ron (R13)
```

## 9. Relationships & lifecycle summary

- `Craft` N—1 dominant `BodyDef`; state rebases on SOI crossing (continuity-exact, evented).
- `ManeuverNode` N—1 `Craft`; chained in `Plan`s; kernel `maneuver-node` events at epochs.
- `BodyMotion` lifecycle: Railed → Diverted → ReRailed (small+divertible only, budget-bounded).
- `FlightStatus` lifecycle: Propagating → Impacted | EntryHandoff (terminal here; later slices consume).
- Planner DTOs derive from `AstroSnapshot` — never stored in the slice, never fingerprinted.
- Every quantitative field above that bears on plausibility traces to a `source` in data.
