# Contract: Integration Seams — Composed-Value Inputs (FA-06)

`sojourn-economy` depends **only on `sojourn-core`** (R1). Every cross-slice physics value enters as a
**plain, serializable input** the host composes from the upstream query surfaces — the FA-04 C1
decoupling. This file defines those four value shapes and who fills them. Tests pass stubs; the
harness/Tauri host fills them from the real upstream modules.

## `EconomyInputs` (bundle passed to commands at decision time and to the snapshot at query time)

```text
EconomyInputs {
    edge_prices:  BTreeMap<(LocationId, LocationId), EdgePrice>,   // from FA-02 planner
    grades:       BTreeMap<(FactionId, LocationId), GradeBelief>,  // from FA-03 belief-state
    vehicle_costs: BTreeMap<DesignId, VehicleCost>,                // from FA-04 design snapshot
    maturities:   BTreeMap<(FactionId, TechId), TechMaturity>,     // from FA-05 research snapshot
}
```

## The four value shapes

| Shape | Fields | Source query (host-side) | Honesty note |
|---|---|---|---|
| `EdgePrice` | `dv_mps, tof_s, next_window_tick: u64, window_open: bool` | FA-02 `porkchop_departure_dv` / `lambert_solve` / `lowthrust_arc` for `(from,to)` at the current tick | the **planner** price (two-tier fidelity), not the authoritative propagator |
| `GradeBelief` | `resource_id: String, believed_grade: f64, certainty: f64` | FA-03 `believed_site` / `certainty_site` | the **faction belief-state**, never ground truth |
| `VehicleCost` | `unit_cost_basis, build_days_basis, payload_kg, propellant_kg, dv_mps` | FA-04 `DesignSnapshot::cost` / `derive` | the physical cost/capacity basis (learning already applied) |
| `TechMaturity` | `trl: u8, understanding: f64, flyable: bool` | FA-05 `maturity` / `understanding` | maturity gates ISRU/facility tech & IP licensing |

## Why composed values, not crate deps

- **Testability**: the economy is unit-tested with stub inputs — no world/astro/vehicle/research crate
  needed to test ledger conservation, break-even sign, funding, markets (mirrors FA-04's stub maturity map).
- **Honesty**: the economy structurally acts on the **belief-state** grade and the **planner** price —
  it *cannot* reach hidden truth or authoritative flight state, because it never holds them.
- **Acyclic graph, no new edges**: `economy → core` only; the dependency arrow to FA-07/09 points the
  other way. The host composes inputs at the IPC boundary.

## Host composition (harness & Tauri)

The harness `economy` scenario flag installs `EconomyModule`; scenarios carry a precomputed
planner-derived `EdgePrice` literal (deterministic). The **live** astro-priced integration test lives
in `crates/sojourn-economy/tests/integration_astro.rs` and reaches the real FA-02 planner through a
**test-only `sojourn-astro` dev-dependency** (no production crate edge, no cycle — production code
stays core-only per R1) to fill `edge_prices` before issuing `DispatchShipment`, proving the seam
end-to-end. Stub-fed unit tests cover everything else. No upstream module is modified.
