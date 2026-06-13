# Contract: Economy Commands & Events (FA-06)

Commands routed through the kernel as `Command::ModulePayload { module: "economy", kind, payload }`
(the FA-03/04/05 pattern), applied in `EconomyModule::on_command`. Trust-the-caller for structural
validity; cross-slice physics arrives **inside the payload** as composed values (the plan→preview→
commit pattern). Events are emitted via the data registry. Implements R5/R7/R8/R13/R16.

## Commands (`EconomyCommand`)

| Kind | Payload (abridged) | Effect |
|---|---|---|
| `DispatchShipment` | `{ faction, edge, cargo, vehicle, edge_price: EdgePrice }` | create a `Shipment`; `Ordered`→`WaitingWindow`/`InTransit` per `edge_price`; debit Δv (in-space) or mass-to-orbit/Funds (launch, R5) |
| `BuyLaunch` / `SellLaunch` | `{ faction, orbit_class, mass_kg }` | trade mass-to-orbit capacity ↔ Funds at the launch-market price (R12) |
| `BuildDepot` / `AssignTug` / `ScheduleCycler` | `{ faction, node / edge, … }` | place/assign reusable logistics assets (R6) |
| `ApplyAppropriation` | `{ faction, amount, directed, political_modifier }` | agency funding roll-over; opaque political inputs (R8) |
| `InjectFinancing` | `{ faction, kind, amount }` | private equity/debt/owner injection (R8) |
| `CommissionFacility` / `UpgradeFacility` | `{ faction, kind, level }` | capex debit; sizes capacity / ops pool (R14) |
| `SiteIsruPlant` / `OperateIsru` | `{ faction, node, process, grade_belief: GradeBelief, power_w }` | site/run a plant; seeded yield; credit local stock (R10) |
| `PostRfp` / `BidContract` / `AwardContract` / `ReportContract` | `{ … }` | contract lifecycle `Posted→Bid→Awarded→Fulfilled|Failed` (R13) |
| `FormPartnership` / `RenegePartnership` / `License` | `{ pair / tech_ref, terms }` | trust state + IP royalties (R13) |
| `OpenProject` / `DeliverToProject` | `{ faction, target_node, required / shipment }` | the Slice 7 delivery-accounting seam (R15) |

- **Validation**: ids resolve; amounts ≥ 0; transactions are **conservation-checked** and rejected on
  negative balance/stock (`EconomyError::Insufficient*`). A command that cannot complete returns a
  `CommandOutcome` carrying the violated constraint (no silent partial application).
- **Determinism**: all stochastic effects (overrun, ISRU yield, market move, contract generation,
  ops anomaly) draw from named seeded streams via `ctx.rng(path)`; no wall-clock.

## Step cadence (R7)

- `cadence_ticks = 86_400` (daily). Each `step`: advance shipments (window/arrival), run ISRU output,
  accrue facility opex, advance funding periods (and fire `budget-cycle`/`bankruptcy`/`agency-gutted`).
- **Market sub-tick**: when `tick % market_period_days == 0` (data, default 30), update launch/niche
  prices (seeded `market`) and generate RFPs (seeded `contracts`).

## Events (data registry — `data/kernel/event-classes.ron`)

| Event | Class | When |
|---|---|---|
| `budget-cycle` | LogOnly | fiscal period roll-over / continuing-resolution |
| `contract-awarded` | Interrupt | an RFP is awarded |
| `bankruptcy` | Interrupt | a private faction's cash < 0 |
| `agency-gutted` | Interrupt | an agency's appropriation < caretaker floor |
| `shipment-arrived` | LogOnly | a shipment reaches its destination |
| `isru-online` | LogOnly | a plant reaches production scale |
| `market-shock` | Interrupt | a launch/market price move beyond a sourced threshold |
| `ops-oversubscribed` | LogOnly | ops/comms pool exceeded; degradation applied |

Interrupt-class events feed the FA-01 interrupt-and-pause loop ("stop on something that matters").
**No kernel change** — events are data registry entries.
