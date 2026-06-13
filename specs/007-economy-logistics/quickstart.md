# Quickstart: Economy & Logistics (FA-06)

A headless walk through the slice that proves the spec's success criteria. Everything runs under the
deterministic harness; no UI. Commands are `Command::ModulePayload { module: "economy", … }`; queries
are pure functions over an `EconomySnapshot`. Cross-slice physics is fed as composed values
(`contracts/integration-seams.md`).

## 0. Build & validate data

```text
cargo test -p sojourn-economy
cargo run -p sojourn-harness -- validate-data data/econ      # schema + sources + analytic gates
cargo run -p sojourn-harness -- conformance --module economy # manifest, double-run, serde round-trip, cadence
```

## 1. Six-currency ledger & location-addressed stocks (US1 → SC-001/003)

1. Load a faction with sourced opening balances; credit 1 t of water at `LEO` and 1 t at `EML-1`.
2. Query `stock(faction, "water", "LEO")` and `…"EML-1"` → two distinct goods.
3. Attempt to debit 1.5 t from the 1 t `LEO` stock → **rejected** (conservation), stock unchanged.
4. Run a scripted set of processes; assert `Σ inflow = Σ outflow` and no negative stock.

## 2. Logistics priced in delta-v (US2 → SC-001)

1. Compose an `EdgePrice` (real, from the FA-02 planner in the integration test) for `LEO→EML-1`.
2. `DispatchShipment` a tug carrying water; if `window_open == false` it **waits** for `next_window_tick`.
3. At the window it debits the tug's propellant/Δv; after `tof` it **delivers** and credits `EML-1`.
4. A launch edge `Earth→LEO` instead debits **mass-to-orbit** capacity (or Funds via the market), not Δv.

## 3. Funding: appropriations & bankruptcy (US3 → SC-006)

1. Roll an agency faction's fiscal period → appropriation + directed funds + carry-over applied.
2. Collapse its appropriation below the caretaker floor → `agency-gutted` event + `gutted` flag.
3. Run a private faction with burn > revenue until cash < 0 → `bankruptcy` event + `bankrupt` flag.

## 4. Cost: P50/P80 & learning (US4 → SC-005)

1. `cost_estimate(faction, design)` over a `VehicleCost` basis → `p50 < p80`, traceable.
2. Draw realised cost on the `cost-overrun` seed → can exceed P50; reproducible across runs.
3. Raise cumulative production → realised unit cost falls monotonically (learning, from the FA-04 basis).

## 5. ISRU break-even & scale-up (US5 → SC-004)

1. Site a lunar-ice plant with a `GradeBelief`; `OperateIsru` → credits local LOX/LH₂ stock (seeded yield).
2. `isru_break_even` below the break-even scale → **net negative**; above it → **net positive** (no free fuel).
3. A first-of-a-kind plant ramps (reduced yield/reliability) before reaching production.

## 6. Markets, contracts & partnerships (US6 → SC-007)

1. `PostRfp` → `BidContract` → `AwardContract` → fulfil (revenue + heritage) or fail (penalty).
2. Raise the world-capacity index on a market tick → launch `$/kg` falls.
3. `FormPartnership` then `RenegePartnership` → trust degrades with lasting reputation cost.

## 7. Facilities & ops pool (US7 → SC-008)

1. `CommissionFacility` a mission-control + DSN → sizes the faction's ops/comms pool.
2. Activate more craft than the pool supports → `ops-oversubscribed`: data return down, anomaly risk up.
3. `UpgradeFacility` → the pool grows and degradation clears.

## 8. Determinism, save/load, perf (cross-cutting → SC-002/009/010/011)

```text
cargo run -p sojourn-harness -- verify    scenarios/economy_logistics.ron     # double-run bit-identical (SC-002)
cargo run -p sojourn-harness -- roundtrip scenarios/economy_logistics.ron --save-at-ticks <t1>,<t2>  # SC-010
cargo run -p sojourn-harness -- mutate --all                                  # gates have teeth
```

- SC-009: every `data/econ/*` entry has a `source`; CI `validate-data econ` enforces it.
- SC-011: with the curated graph (tens-to-low-hundreds of nodes), thousands of stocks and large
  fleets, the kernel sustains ≥1 sim-year/min at high warp (daily step + monthly market tick).

## Success-criteria coverage map

| SC | Proven by |
|---|---|
| SC-001 | §1–§2 (move a good, debit Δv/propellant, credit destination, trace to mass×Δv) |
| SC-002 | §8 `verify` (double-run identical) |
| SC-003 | §1.4 conservation gate |
| SC-004 | §5 ISRU break-even sign |
| SC-005 | §4 P50<P80 + learning monotonicity |
| SC-006 | §3 bankruptcy + gutting |
| SC-007 | §6 contracts + launch-price elasticity |
| SC-008 | §7 ops-pool oversubscription/relief |
| SC-009 | §0 `validate-data econ` source presence |
| SC-010 | §8 `roundtrip` + econ-data version pin |
| SC-011 | §8 perf envelope on the curated graph |
