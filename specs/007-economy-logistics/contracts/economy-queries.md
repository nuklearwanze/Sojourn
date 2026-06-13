# Contract: Economy Query Surface (FA-06)

The read-only, between-ticks surface FA-07/09/10 and the Tauri host consume. Pure functions over an
`EconomySnapshot` that composes the economy slice with the four composed-value inputs
(`integration-seams.md`). No mutation, no hidden truth, IPC-serializable. Implements FR-EC-105,
FR-EC-805, and the R17 design-query decision.

## Snapshot construction

```text
EconomySnapshot::from_core(&core, &economy_module, inputs: &EconomyInputs) -> Result<EconomySnapshot, CoreError>
EconomySnapshot::new(slice, module_defs, inputs)   // tests: build directly from parts
```

- `from_core` reads the `"economy"` slice via kernel `with_slice` (read-only, between ticks).
- `EconomyInputs` bundles the composed values the host assembles per `integration-seams.md`
  (edge prices, grade beliefs, vehicle costs, tech maturities). Tests pass stubs.

## Queries (pure, faction-scoped where noted)

| Query | Returns | Notes |
|---|---|---|
| `balances(faction)` | `BTreeMap<Currency, f64>` | the six currency balances (FR-EC-101) |
| `stock(faction, commodity, location)` | `f64` | location-addressed stock (FR-EC-102) |
| `inventory(faction)` | `Vec<(StockKey, f64)>` | aggregate stocks |
| `journal(faction, since_tick)` | `Vec<Transaction>` | audit/replay history (FR-EC-103) |
| `route_cost(faction, from, to, vehicle)` | `RouteCost { dv, tof, propellant_kg, funds_or_capacity }` | composes `EdgePrice`; launch vs in-space per R5 (FR-EC-201/202a) |
| `isru_break_even(faction, plant)` | `BreakEven { net, saved, plant_cost, positive: bool }` | composes launch price + GradeBelief (FR-EC-502) |
| `cost_estimate(faction, design)` | `CostEstimate { p50, p80, trace }` | composes VehicleCost + maturity (FR-EC-401) |
| `market_price(orbit_class)` / `niche_price(market)` | `f64` | launch $/kg; tourism/ISM ceilings (FR-EC-601/604) |
| `contracts(faction)` / `partnership(a, b)` | `Vec<Contract>` / `Partnership` | lifecycle + trust (FR-EC-602/603) |
| `facility_capacity(faction, kind)` / `ops_utilisation(faction)` | `f64` / `OpsUtil { capacity, used, degraded }` | FR-EC-701/703/206 |
| `funding_state(faction)` | `FundingView { kind, headroom, bankrupt, gutted }` | FR-EC-301…303 |
| `project_status(faction, project)` | `ProjectView { delivered, remaining, state }` | the Slice 7 seam (FR-EC-808) |
| `trace_cost(faction, figure_ref)` | `TraceTree` | money → **mass × Δv** sourced leaves (FR-EC-805) |

## Guarantees

- **Read-only / deterministic**: no query mutates the slice; identical snapshot + inputs ⇒ identical
  results (no wall-clock, libm-only).
- **No hidden truth**: ISRU/route queries act on the **belief-state** grade and the **planner** price
  supplied as inputs — never ground truth or the authoritative propagator state.
- **Traceability**: every dollar figure is drillable to sourced leaves exposing its mass/Δv basis
  (`all_leaves_sourced` is CI-checkable).
- **Faction scoping**: balances, funding, beliefs and contracts are per faction; market prices and
  the commodity taxonomy are global.
