# Phase 1 Data Model: Economy & Logistics (FA-06)

Entities, fields, relationships, validation rules and state transitions for `sojourn-economy`.
Types are illustrative (Rust-shaped); all quantities **SI** (kg, m/s, seconds, watts; Funds in one
common accounting unit). Ordered `BTreeMap`/`BTreeSet` throughout (determinism). "DATA" = sourced
file under `data/econ/`; "SLICE" = owned persistent state; "DERIVED" = pure query-time computation
(never stored). Cross-slice physics enters as **composed values** (see `contracts/integration-seams.md`).

---

## 1. Identity (`ids.rs`)

- `FactionId(u32)` — owner (aligns with the world/research/vehicle faction id).
- `CommodityId(String)` — a tradable good (transparent; references FA-03 resource ids for raw materials).
- `LocationId(String)` — a graph node identity (references an FA-03 world location id or Site key).
- `NodeId(u32)`, `EdgeId(u32)` — interned graph handles (stable within a save).
- `ShipmentId(u32)`, `PlantId(u32)`, `ContractId(u32)`, `FacilityId(u32)`, `ProjectId(u32)` — slotmap-style ids.
- `OrbitClass` enum — `Leo | Gto | Geo | Tli | Tmi | …` (data-extensible label set) for the launch market.

---

## 2. Commodity taxonomy (`commodity.rs`, DATA — `commodities.ron`)

- `CommodityKind` enum:
  - `Raw { resource_ref: String }` — references an **FA-03 resource id** (water, regolith, metals, volatiles).
  - `Processed { from: CommodityId }` — e.g. LOX/LH₂/CH₄ from ice/CO₂, refined metals.
  - `Manufactured` — ZBLAN fibre, protein crystals, spares, structural feedstock.
  - `Consumable` — food, O₂, N₂ buffer.
  - `Strategic { cap_ref: String }` — references a `strategic.ron` capped supply (Pu-238, enriched fuel, rare-earth).
  - `Service` — launch, data, IP licence (intangible, tradable).
- `Commodity { id: CommodityId, kind: CommodityKind, unit: Unit, source: String }`.
- **Validation**: non-empty `source`; `Raw.resource_ref` must resolve against the world resource
  taxonomy (checked in the harness integration / `validate-data` with a world catalogue present);
  `Processed.from` and `Strategic.cap_ref` resolve in-file; no duplicate ids; **no combat/weapon
  commodity** (Principle IX, name+kind screen).

---

## 3. Ledger (`ledger.rs`, SLICE)

- `Currency` enum: `Funds | DeltaV | MassToOrbit | CrewTime | OpsCapacity | PoliticalCapital`.
- `Account { balances: BTreeMap<Currency, f64> }` — per faction.
- `StockKey { commodity: CommodityId, location: LocationId }`; `stocks: BTreeMap<StockKey, f64>` (kg or units).
- `Transaction { tick: u64, faction: FactionId, legs: Vec<Leg>, cause: Cause }`
  - `Leg = Currency(Currency, f64) | Stock(StockKey, f64)` (signed delta).
  - `Cause` enum: `Launch | Transfer | IsruConvert | Consume | Produce | Trade | Appropriation | Financing | ContractReward | Penalty | FacilityOpex | Loss(LossKind)`.
- `Ledger { accounts: BTreeMap<FactionId, Account>, stocks, journal: Vec<Transaction> }` (journal optionally truncated to a window; full history is replayable from the event log).
- **Invariants**: applying a `Transaction` is **atomic**; it is **rejected** if any resulting
  balance/stock < 0 (`EconomyError::InsufficientBalance`/`InsufficientStock`); **conservation** —
  every physical process emits balanced legs (mass in = mass out + explicit `Loss` legs), so
  Σ inflow = Σ outflow over any script (gate R11/SC-003).

---

## 4. Transport network & shipments (`network.rs`, `shipment.rs`, SLICE + DATA)

- `Node { id: NodeId, location: LocationId, role: NodeRole }`; `NodeRole = Staging | Site | Depot`.
- `Edge { id: EdgeId, from: NodeId, to: NodeId, kind: EdgeKind }`;
  `EdgeKind = Launch { orbit_class: OrbitClass } | InSpace | Surface`.
- DATA (`network.ron`): the **curated** node set (referencing world location ids) + static edge
  templates (which node pairs connect, and each edge's kind). Pricing is **not** in data — it is the
  composed `EdgePrice` (R2) supplied at dispatch/query.
- `Vehicle { propellant_kg, dv_capacity_mps, payload_kg, reusable: bool, kind: VehicleKind }`
  (`VehicleKind = Launcher | Tug | Cycler | Generic`) — a value carried in the dispatch command from
  the FA-04 design (capacity basis).
- `Shipment { id, faction, edge, cargo: Vec<(CommodityId, f64)>, vehicle, depart_tick, arrive_tick, state }`.
- `ShipmentState`: **`Ordered → WaitingWindow → InTransit → Delivered`** (or `Rejected`).
  - `Ordered`→`WaitingWindow` if the supplied `EdgePrice.window_open == false` (wait for `next_window_tick`).
  - `WaitingWindow`→`InTransit` at the window; debits propellant/Δv (InSpace) **or** mass-to-orbit
    capacity / Funds (Launch, R5); sets `arrive_tick = depart + tof`.
  - `InTransit`→`Delivered` at `arrive_tick`: credits destination stock; a reusable vehicle returns to
    the tug pool.
  - Any insufficient Δv/capacity/payload ⇒ **flag** (red, no silent completion) — stays `Ordered` with a reason.
- `Depot { node, holds: BTreeMap<CommodityId, f64> }`; `Cycler { leg_schedule: Vec<(EdgeId, period_days)> }`.

---

## 5. Funding (`funding.rs`, SLICE + DATA — `funding.ron`)

- `FundingProfile` (DATA, per faction):
  - `Agency { baseline_funds, volatility, directed: Vec<DirectedLine>, carry_over: CarryOver, caretaker_floor }`.
  - `Private { starting_cash, monthly_burn_model, financing: Vec<FinancingOption> }`.
- `FundingState` (SLICE): `Agency { appropriation_remaining, directed_remaining, period_end_tick, gutted: bool }`
  | `Private { cash, burn_rate, runway_end_tick, bankrupt: bool }`.
- **Transitions**: agency period roll-over applies baseline ± modifiers (opaque political input) +
  directed funds + carry-over; appropriation below `caretaker_floor` ⇒ `gutted = true` + `agency-gutted`
  event. Private `cash < 0` ⇒ `bankrupt = true` + `bankruptcy` event. Both set a **state flag and emit
  an event** (observer-mode, not a hard halt — R8); fiscal-cliff events emit `budget-cycle`.
- **Validation**: non-empty `source`; baselines/floors ≥ 0; exactly one profile variant per faction.

---

## 6. Cost model (`cost.rs`, SLICE + DATA — `cost.ron`)

- `CostUncertainty` (DATA): `{ p50_p80_spread, overrun_shape, … , source }` — the distribution params.
- `CostEstimate` (DERIVED): `{ p50, p80, traceable_basis }` from the **VehicleCost basis** (R2) + maturity.
- `RealisedCost` (SLICE, per program): drawn from the distribution on the `cost-overrun` seeded stream
  (reproducible; can exceed P50). Learning (unit cost ↓ with cumulative production) is the **FA-04 basis**;
  FA-06 stores only realised outcomes, not a duplicate learning state.
- **Validation/gates**: `p50 < p80` (SC-005); realised draw deterministic per seed; learning
  monotonicity gate over the vehicle basis.

---

## 7. ISRU (`isru.rs`, SLICE + DATA — `isru.ron`)

- `IsruProcess` (DATA): `{ id, input: CommodityId(Raw), output: CommodityId(Processed), base_yield_per_day, power_demand_w, plant_mass_kg, scaleup_curve: Vec<(units, yield_mult, reliability)>, source }`
  for **lunar-ice electrolysis, Mars Sabatier, regolith O₂/metals, asteroid volatiles**.
- `IsruPlant` (SLICE): `{ id, faction, node, process, scale_level, online_tick, cumulative_output }`.
- DERIVED: **yield** = `base_yield × scaleup_mult × grade_factor(GradeBelief) × power_factor` with
  grade/accessibility drawn on the `isru-yield` seeded stream; **break-even** = launch-cost-saved (from
  the launch market / route to the destination node) − (plant delivery mass + build + operate +
  amortise). Output **credits the local stock** each operating day (conserved leg).
- **Transitions**: `Sited → PilotRamp(scale_level↑ over scaleup_curve) → Production`; reliability ramp
  applies on the same stream. **Validation**: non-empty `source`; yields/masses/power ≥ 0; `input` is
  `Raw`, `output` is `Processed`; break-even sign gate (SC-004).

---

## 8. Markets & contracts (`market.rs`, `contract.rs`, SLICE + DATA — `launch_market.ron`, `markets.ron`)

- `LaunchMarket` (SLICE+DATA): `{ price_per_kg: BTreeMap<OrbitClass, f64>, world_capacity_index, elasticity, source }`;
  price moves with `world_capacity_index` (parametric world model, seeded fluctuation on `market`).
- `NicheMarket` (DATA): tourism tiers, in-space-manufacturing products — `{ size_cap, price_ceiling, source }` (finite).
- `Contract` (SLICE): `{ id, issuer: FactionId, deliverable, reward_funds, heritage, penalty, state }`.
  - `ContractState`: **`Posted → Bid(bids) → Awarded(winner) → InProgress → Fulfilled | Failed`**.
  - RFPs generated on the `contracts` seeded stream from `markets.ron` generator params.
- `Partnership` (SLICE): `{ pair: (FactionId, FactionId), trust: f64, terms: Terms }`; reneging lowers
  `trust` with lasting effect.
- `License` (SLICE): `{ tech_ref, licensee, royalty_rate }` over FA-05 maturity (R2).
- **Validation**: non-empty `source`; price ceilings/sizes ≥ 0; faction-agnostic mechanisms; seeded
  where stochastic.

---

## 9. Facilities & ops (`facility.rs`, `ops.rs`, SLICE + DATA — `facilities.ron`)

- `Facility` (DATA template + SLICE instance): `{ id, kind, capex, opex_per_day, capacity, level, source }`;
  `FacilityKind = Lab | TestStand | ProductionLine | Pad | Range | MissionControl | DeepSpaceNetwork | Relay`.
- Capacity **gates rate**: `ProductionLine`→build throughput; `Pad/Range`→launch cadence;
  `MissionControl/DeepSpaceNetwork/Relay`→**ops/comms pool size**.
- `OpsPool` (DERIVED+SLICE): `{ capacity, used }` per faction; `LightTime` per node/edge (a delay
  value). Active craft consume the pool; **oversubscription** (used > capacity) degrades outcomes —
  reduced data return + raised anomaly probability on the `ops-anomaly` seeded stream (`ops-oversubscribed` event).
- **Validation**: non-empty `source`; capex/opex/capacity ≥ 0.

---

## 10. Strategic materials (`commodity.rs`/`ledger.rs`, DATA — `strategic.ron`)

- `StrategicSupply` (DATA): `{ id, annual_production_cap, world_stock, policy_gate, source }`
  (Pu-238/Am-241, HEU/LEU, rare-earth/electronics-grade).
- Modelled as **capped scarce stocks** competed for; a mission needing more than the cap is **gated**
  until supply or an alternative technology matures (FR-EC-106; consumes FA-05 maturity).

---

## 11. Project primitive (`project.rs`, SLICE — the Slice 7 seam)

- `Project { id, faction, target_node, required: Vec<(CommodityId, f64)>, crew_time_req, time_req, delivered, state }`;
  `ProjectState = Open → InProgress → Complete`. Advances as logistics deliveries land; FA-06 owns the
  **accounting only** (FR-EC-808). Slice 7 consumes it to assemble bases/stations.

---

## 12. Module state & wiring (`module.rs`, SLICE)

- `EconomySlice { ledger, network, shipments, funding: BTreeMap<FactionId, FundingState>, plants,
  market, contracts, partnerships, facilities: BTreeMap<FactionId, Vec<Facility>>, ops:
  BTreeMap<FactionId, OpsPool>, projects, realised_costs, next_ids…, data_hash }`.
- `EconomyModule { commodities, network_defs, funding_defs, isru_defs, market_defs, facility_defs,
  strategic_defs, cost_defs }` (loaded DATA) + `load(dir)`.
- Manifest: `id = "economy"`, owns the economy slice, **streams** `["cost-overrun","isru-yield",
  "market","contracts","ops-anomaly"]`, emits the R16 events, `cadence_ticks = 86_400`, daily `step`
  (advance shipments, ISRU, funding periods; market sub-tick when due), `save_slice`/`load_slice`
  (verify `data_hash`).
- DERIVED (`query.rs`): `EconomySnapshot` composes the slice + R2 inputs; pure queries per R17.

---

## 13. Traceability (`trace.rs`, DERIVED)

- `TraceTree` (reused shape from FA-04): `Leaf { name, value, source } | Node { op, value, inputs }`.
- Every **dollar figure** resolves to a tree whose leaves are **sourced** economic constants and whose
  internal nodes expose the **mass × Δv** basis (route Δv × propellant, plant mass, learning state) —
  the FR-EC-805 / Principle VII honesty contract; CI-checkable (`all_leaves_sourced`).

---

## 14. Entity relationship summary

```text
Faction ──1:1── Account (six currencies)         Faction ──1:1── FundingState
Faction ──1:N── Stock@Location                   Faction ──1:N── Facility ──sizes── OpsPool
Commodity ──refs── FA-03 resource id (Raw)       Node ──refs── FA-03 location id
Node ──N── Edge ──priced by── EdgePrice(astro)   Shipment ──on── Edge, ──carries── Vehicle(FA-04 capacity)
IsruPlant ──at── Node, ──runs── IsruProcess, ──uses── GradeBelief(FA-03), ──gated by── TechMaturity(FA-05)
Contract/Partnership/License ── faction-agnostic, seeded
CostEstimate ──over── VehicleCost(FA-04) + TechMaturity(FA-05)
Project ──consumes── deliveries (Slice 7 seam)
Transaction ──conserves── all of the above (Σ in = Σ out)
```

All cross-slice arrows are **composed values**, not crate dependencies (R1/R2).
