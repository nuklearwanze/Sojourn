# Contract: Planning-Query Surface (FR-ASTRO-409)

The read-only planner API — the seam FA-10's trajectory screens (and the harness) consume.
Clarified 2026-06-13: pure functions over a snapshot; kernel views stay flat scalars.

## Rules

1. **Pure**: every query is a function of (`AstroSnapshot`, parameters) → DTO. No mutation, no
   random streams, no journal entries, no fingerprint participation. Calling a query any number
   of times, in any order, at any warp, changes nothing.
2. **Snapshot-based**: `AstroModule::snapshot(&slice, &catalog, tick) → AstroSnapshot` extracts a
   cheap, owned copy at a completed tick boundary (consistent with the kernel's query model).
   Queries on hypothetical states are allowed (designer/what-if flows): a snapshot can be built
   from caller-supplied craft states.
3. **Serializable**: all parameters and DTOs derive serde — the surface is IPC-bridgeable
   exactly like the kernel API.
4. **Honest**: every prediction DTO carries a `Regime` tag; low-confidence regimes
   (multi-body regions, near-SOI legs) are flagged, never silently wrong (FR-ASTRO-402).
5. **Bounded**: every solver has data-configured iteration caps and grid limits; queries return
   explicit unsolvable/no-convergence results rather than looping or fabricating.

## Surface (signatures, Rust-shaped)

```rust
// Orbits & trajectories
fn orbit_summary(snap, craft: CraftId) -> OrbitSummary;
fn predict_trajectory(snap, craft: CraftId, span_s: f64, max_samples: u32)
    -> Vec<TrajectorySample>;            // planner-tier conic prediction, SOI-patched
fn predict_with_nodes(snap, craft: CraftId, nodes: &[NodeSpec], …) -> PlanPrediction;

// Budgets
fn dv_budget(snap, craft: CraftId) -> DvBudget;      // rocket-equation remaining Δv
fn node_feasibility(snap, craft: CraftId, node: &NodeSpec) -> Feasibility;

// Transfers & windows
fn porkchop(snap, q: &PorkchopQuery) -> PorkchopGrid;
fn cell_to_plan(snap, q: &PorkchopQuery, cell: (u16, u16), craft: CraftId) -> Vec<NodeSpec>;

// Encounters
fn solve_flyby(snap, q: &FlybyQuery) -> EncounterSolution;
fn verify_chain(snap, chain: &[FlybyLeg]) -> ChainReport;   // end-to-end divergence

// Low-thrust & low-energy
fn lowthrust_arc(snap, craft: CraftId, q: &ArcQuery) -> ArcEstimate;   // duration/propellant
fn lowenergy_routes(snap, q: &RouteQuery) -> Vec<RouteEstimate>;       // empty when gate closed

// Aero
fn aero_corridor(snap, q: &AeroQuery) -> AeroCorridor;

// Reconciliation & correction
fn reconcile(snap, plan: &Plan) -> ReconciliationReport;
fn solve_tcm(snap, craft: CraftId, aim: &AimPoint, epoch_tick: u64) -> TcmSolution;
```

## Performance budgets (SC-007)

| Query | Budget (reference machine) |
|---|---|
| porkchop ≤ 40×40 grid | < 100 ms |
| node prediction / feasibility / orbit summary | < 10 ms |
| flyby solve / chain verify (≤ 4 legs) | < 100 ms |
| lowthrust arc estimate / aero corridor / TCM solve | < 100 ms |

## Conformance tests

- Purity: a query between two steps leaves the state fingerprint unchanged (asserted in suite).
- Determinism: identical snapshot + parameters → identical DTO bytes.
- Honesty: every suite scenario that enters a documented low-confidence regime yields a
  `LowConfidence` tag; reconciliation thresholds fire in the divergence scenarios.
