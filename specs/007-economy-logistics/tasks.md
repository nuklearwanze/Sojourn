---
description: "Task list for Economy & Logistics (FA-06)"
---

# Tasks: Economy & Logistics (FA-06)

**Input**: Design documents from `/specs/007-economy-logistics/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R17), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution v1.0.0 mandates determinism double-run, **analytic validation
against known cases** (resource conservation, ISRU break-even sign, learning-curve monotonicity,
P50<P80, launch-price elasticity sign), data schema+source validation and save/load round-trip; every
user story carries an Independent Test.

**Organization**: by user story (US1–US7). Crate layout per plan.md: `crates/sojourn-economy`
(module; **dep `sojourn-core` only** — cross-slice physics flows in as composed values, the FA-04 C1
decoupling), `data/econ/`, harness `economy` flag. **No kernel change; no astro/world/vehicle/research
change.** Derived figures are pure query-time computations over an `EconomySnapshot`.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1…US7 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/sojourn-economy` crate: `Cargo.toml` (deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core; **`[dev-dependencies] sojourn-astro`** for the one astro-priced integration test only — test-only, no production edge, no cycle, preserves the "dep core only" decision R1; workspace lints) and `src/lib.rs` (`#![forbid(unsafe_code)]`, module decls); add `"crates/sojourn-economy"` to workspace `members` in `Cargo.toml`.
- [x] T002 [P] Scaffold `data/econ/` directory (placeholder headers noting sourcing per Principle I).
- [x] T003 [P] Confirm `clippy.toml`/`deny.toml` apply via workspace lints; `cargo clippy -p sojourn-economy` clean (empty crate).

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: no user story work begins until this phase completes.

- [x] T004 Implement id types (`FactionId`, `CommodityId`, `LocationId`, `NodeId`, `EdgeId`, `ShipmentId`, `PlantId`, `ContractId`, `FacilityId`, `ProjectId`, `OrbitClass`) in `crates/sojourn-economy/src/ids.rs` (data-model §1).
- [x] T005 Implement the commodity taxonomy loader & schema in `crates/sojourn-economy/src/commodity.rs` (data-model §2): `CommodityKind`/`Commodity`, non-empty-source validation, `Raw.resource_ref`/`Processed.from`/`Strategic.cap_ref` resolution, and the **no-combat-commodity** screen (`contracts/economy-data.md`).
- [x] T006 Implement the ledger core in `crates/sojourn-economy/src/ledger.rs` (data-model §3): `Currency`, `Account`, `StockKey`, `Leg`, `Cause`, `Transaction`, `Ledger`; **atomic apply with conservation rejection** (negative balance/stock ⇒ `EconomyError::Insufficient*`).
- [x] T007 [P] Add the econ event classes (`budget-cycle`, `contract-awarded`, `bankruptcy`, `agency-gutted`, `shipment-arrived`, `isru-online`, `market-shock`, `ops-oversubscribed`) to `data/kernel/event-classes.ron` per `contracts/economy-commands.md`.
- [x] T008 [P] Implement the `trace.rs` traceability-tree types (recursive nodes; sourced leaves; `all_leaves_sourced`) in `crates/sojourn-economy/src/trace.rs` (R17) — reused for every money→mass/Δv trace.
- [x] T009 [P] Implement the composed-value input shapes (`EdgePrice`, `GradeBelief`, `VehicleCost`, `TechMaturity`, `EconomyInputs`) in `crates/sojourn-economy/src/inputs.rs` (`contracts/integration-seams.md`, R2). Upstream ids referenced by these shapes use **local id aliases** (e.g. `pub type DesignId = u32`, `pub type TechId = String`) — the economy never imports FA-04/FA-05 types (dep core only, R1).
- [x] T010 Implement `EconomyModule` SimModule skeleton in `crates/sojourn-economy/src/module.rs`: manifest (`id="economy"`, owned slice, **streams** `cost-overrun`/`isru-yield`/`market`/`contracts`/`ops-anomaly`, emits the T007 events, `cadence_ticks=86_400`), `init`, no-op `step`, `save_slice`/`load_slice` (verify `data_hash`); define `EconomySlice` (data-model §12) + `data_hash` pin; export `EconomyModule`, `EconomyCommand`, `economy_payload` from `lib.rs`.
- [x] T011 Implement `EconomyModule::load(dir)` in `module.rs`: load all `data/econ/*` files + content-hash; export loaded defs.
- [x] T012 Wire harness `economy` flag in `crates/sojourn-harness/src/scenario.rs` (install `EconomyModule::load("data/econ")`; extend `TimedCommand` with an `economy: Option<EconomyCommand>` arm via `Command::ModulePayload`) and `crates/sojourn-harness/src/main.rs` (validate-data `econ` branch; `conformance "economy"` factory); add `sojourn-economy` to the harness `Cargo.toml`.

**Checkpoint**: workspace builds; FA-01/02/03/04/05 suites still green; empty Economy module passes `conformance --module economy`.

---

## Phase 3: User Story 1 — Six-currency ledger & location-addressed resources (Priority: P1) 🎯 MVP

**Goal**: per-faction six-currency balances + location-addressed stocks with conserved, recorded, replayable transactions.

**Independent Test**: submit credit/debit script → balances and per-location stocks correct; same commodity at two locations is two goods; over-draw rejected; conservation holds; double-run bit-identical.

### Tests for User Story 1 ⚠️

- [x] T013 [P] [US1] Ledger + location-addressed-stock test: credit/debit currencies and stocks; the same commodity at two locations is two distinct goods; queries return correct balances/stocks/inventory — in `crates/sojourn-economy/tests/ledger.rs`.
- [x] T014 [P] [US1] Conservation + rejection test: an over-draw is rejected (stock/balance unchanged); an arbitrary process script keeps Σ inflow = Σ outflow with no negative stock — in `crates/sojourn-economy/tests/ledger.rs`.

### Implementation for User Story 1

- [x] T015 [US1] Implement ledger queries (`balances`, `stock`, `inventory`, `journal`) in `crates/sojourn-economy/src/ledger.rs` + `query.rs` (FR-EC-105).
- [x] T016 [US1] Implement the generic transaction-applying command path (credit/debit with `Cause`) in `crates/sojourn-economy/src/module.rs` (`contracts/economy-commands.md`).
- [x] T017 [P] [US1] Author `data/econ/commodities.ron` (sourced taxonomy: raw refs + processed/manufactured/consumable/strategic/service) and a minimal opening-balance fixture for scenarios.
- [x] T018 [US1] Implement `EconomySnapshot::from_core`/`new` (composing the economy slice + `EconomyInputs`) and the `balances`/`stock`/`inventory`/`journal` queries in `crates/sojourn-economy/src/query.rs` (FR-EC-105, R17).

**Checkpoint**: the conserved, location-addressed ledger works and is deterministic — MVP.

---

## Phase 4: User Story 2 — Logistics network priced in delta-v (Priority: P1)

**Goal**: directed graph of curated nodes; window-constrained transfers priced by the astro planner; launch consumes mass-to-orbit, in-space consumes Δv; depots/tugs/cyclers.

**Independent Test**: dispatch cargo A→B → debits correct propellant/Δv (or mass-to-orbit for launch), waits for a window if none open, delivers after TOF, updates both stocks; a reusable tug returns; a real-planner-priced route flies end-to-end.

### Tests for User Story 2 ⚠️

- [x] T019 [P] [US2] Transfer + window test: an in-space shipment debits the vehicle's propellant/Δv, waits for `next_window_tick` when closed, delivers after TOF, credits the destination, and a reusable tug returns — in `crates/sojourn-economy/tests/logistics.rs`.
- [x] T020 [P] [US2] Launch-vs-in-space + shortfall test: a surface→orbit edge consumes **mass-to-orbit** capacity (or Funds via the market), an in-space edge consumes **Δv**; an under-fuelled/over-capacity assignment **flags** without silently completing — in `crates/sojourn-economy/tests/logistics.rs`.
- [x] T021 [P] [US2] Astro-priced integration test: build an `EdgePrice` from the **real FA-02 planner** (via the test-only `sojourn-astro` dev-dependency from T001 — production code stays core-only), feed it to `DispatchShipment`, and assert the route's Δv/TOF and delivery are consistent — in `crates/sojourn-economy/tests/integration_astro.rs`.

### Implementation for User Story 2

- [x] T022 [US2] Implement the transport graph (`Node`/`Edge`/`EdgeKind`/routing) in `crates/sojourn-economy/src/network.rs` (data-model §4, R4).
- [x] T023 [US2] Implement the shipment lifecycle (`Ordered→WaitingWindow→InTransit→Delivered`/`Rejected`) + depots/tugs/cyclers in `crates/sojourn-economy/src/shipment.rs` (data-model §4, R6).
- [x] T024 [US2] Implement `DispatchShipment`/`BuyLaunch`/`SellLaunch`/`BuildDepot`/`AssignTug`/`ScheduleCycler` handlers with the **launch (mass-to-orbit) vs in-space (Δv)** debit split in `crates/sojourn-economy/src/module.rs` (R5, `contracts/economy-commands.md`); advance shipments in `step`.
- [x] T025 [P] [US2] Author `data/econ/network.ron` (curated nodes referencing world location ids + edge templates with orbit classes) and `scenarios/economy_logistics.ron` + `scenarios/economy_astro.ron` (the latter carries a **precomputed planner-derived `EdgePrice` literal** for deterministic replay; the live planner call is the T021 integration test).
- [x] T026 [US2] Implement the `route_cost` query (composing `EdgePrice` + vehicle propellant; launch vs in-space) in `crates/sojourn-economy/src/query.rs` (FR-EC-201/202a).

**Checkpoint**: resources cost delta-v and time to move; windows bind; the astro seam is live.

---

## Phase 5: User Story 3 — Funding models: appropriations & cash-runway (Priority: P1)

**Goal**: agency appropriations (directed funds, carry-over, fiscal cliff, gutting) and private cash-runway (financing, bankruptcy), faction-configurable from data.

**Independent Test**: agency period roll-over applies appropriation/directed/carry-over; collapse → gutted; private burn>revenue → bankruptcy — all deterministic from identical inputs.

### Tests for User Story 3 ⚠️

- [x] T027 [P] [US3] Appropriation + gutting test: a fiscal roll-over applies baseline + directed + carry-over; an appropriation below the caretaker floor sets `gutted` and emits `agency-gutted` — in `crates/sojourn-economy/tests/funding.rs`.
- [x] T028 [P] [US3] Bankruptcy test: a private faction with burn > revenue until cash < 0 sets `bankrupt` and emits `bankruptcy`; two factions with different profiles and identical physics differ only by data — in `crates/sojourn-economy/tests/funding.rs`.

### Implementation for User Story 3

- [x] T029 [US3] Implement `FundingProfile`/`FundingState` + transitions in `crates/sojourn-economy/src/funding.rs` (data-model §5, R8).
- [x] T030 [US3] Implement `ApplyAppropriation`/`InjectFinancing` handlers + fiscal-period roll-over in `step` + `budget-cycle`/`bankruptcy`/`agency-gutted` emission in `crates/sojourn-economy/src/module.rs` (R8).
- [x] T031 [P] [US3] Author `data/econ/funding.ron` (per-faction agency/private profiles with sourced baselines).
- [x] T032 [US3] Implement the `funding_state` query in `crates/sojourn-economy/src/query.rs` (FR-EC-301…303).

**Checkpoint**: money flows in and out honestly; bankruptcy and gutting are real, deterministic states.

---

## Phase 6: User Story 4 — Cost model: P50/P80 & learning (Priority: P2)

**Goal**: P50/P80 cost estimates with seeded overruns; learning curve over the FA-04 cost basis; full traceability.

**Independent Test**: estimate yields P50<P80; realised overrun seeded/reproducible; rising production lowers realised unit cost monotonically.

### Tests for User Story 4 ⚠️

- [x] T033 [P] [US4] P50/P80 + overrun test: an estimate over a `VehicleCost` basis yields `p50 < p80`; the realised draw on the `cost-overrun` stream can exceed P50 and reproduces per seed — in `crates/sojourn-economy/tests/cost.rs`.
- [x] T034 [P] [US4] Learning-monotonicity test: realised unit cost is non-increasing as cumulative production rises along the (FA-04) exponent — in `crates/sojourn-economy/tests/cost.rs`.

### Implementation for User Story 4

- [x] T035 [US4] Implement the cost model (P50/P80 bands + seeded overrun realisation + learning wrap over the `VehicleCost` basis) in `crates/sojourn-economy/src/cost.rs` (data-model §6, R9).
- [x] T036 [P] [US4] Author `data/econ/cost.ron` (P50/P80 spread + overrun-shape params, sourced).
- [x] T037 [US4] Implement the `cost_estimate` query + cost traceability in `crates/sojourn-economy/src/query.rs` (FR-EC-401…404).

**Checkpoint**: cost is uncertain, learns, and traces to physics.

---

## Phase 7: User Story 5 — ISRU break-even & scale-up (Priority: P2)

**Goal**: ISRU plants converting local resources at sourced yields with seeded grade uncertainty; break-even vs launch-cost-saved; pilot→production scale-up.

**Independent Test**: a sited plant credits the local propellant stock; break-even is net-negative below scale and net-positive above; a first-of-a-kind plant ramps.

### Tests for User Story 5 ⚠️

- [x] T038 [P] [US5] ISRU output + break-even-sign test: a lunar-ice and a Mars-Sabatier plant produce at the sourced yield and credit the local stock; break-even is negative below and positive above the break-even scale — in `crates/sojourn-economy/tests/isru.rs`.
- [x] T039 [P] [US5] Scale-up + seeded-grade test: a first-of-a-kind plant exhibits a reduced yield/reliability ramp before production; the grade/accessibility draw is seeded/reproducible and consistent with the supplied `GradeBelief` — in `crates/sojourn-economy/tests/isru.rs`.

### Implementation for User Story 5

- [x] T040 [US5] Implement the ISRU process model (yield from sourced params × `GradeBelief` × power, seeded grade draw), break-even, and the scale-up/reliability ramp in `crates/sojourn-economy/src/isru.rs` (data-model §7, R10).
- [x] T041 [US5] Implement `SiteIsruPlant`/`OperateIsru` handlers + daily output credit (conserved leg) + `isru-online` emission in `crates/sojourn-economy/src/module.rs` (R10).
- [x] T042 [P] [US5] Author `data/econ/isru.ron` (ice/Sabatier/regolith/asteroid process params: yield, plant mass, power, scale-up/reliability curve, sourced).
- [x] T043 [US5] Implement the `isru_break_even` query (composing launch price + plant cost) in `crates/sojourn-economy/src/query.rs` (FR-EC-502).

**Checkpoint**: ISRU is a real investment decision — never free fuel.

---

## Phase 8: User Story 6 — Markets, contracts, partnerships & IP (Priority: P2)

**Goal**: launch market with capacity elasticity; RFP/bid/award/fulfil contracts; partnership trust; IP licensing; tourism/ISM with finite sizes — all faction-agnostic and seeded.

**Independent Test**: post→bid→award→fulfil/fail an RFP (revenue+heritage / penalty); a world-capacity rise lowers $/kg; a reneged partnership degrades trust lastingly.

### Tests for User Story 6 ⚠️

- [x] T044 [P] [US6] Contract-lifecycle test: an RFP goes `Posted→Bid→Awarded→Fulfilled` (revenue + heritage) and `→Failed` (penalty); generation is seeded/reproducible — in `crates/sojourn-economy/tests/markets.rs`.
- [x] T045 [P] [US6] Market + partnership test: raising the world-capacity index lowers launch `$/kg`; a `FormPartnership` then `RenegePartnership` degrades trust with lasting effect; tourism/ISM revenue respects its price ceiling — in `crates/sojourn-economy/tests/markets.rs`.

### Implementation for User Story 6

- [x] T046 [US6] Implement the launch market (capacity elasticity) + niche markets (tourism/ISM ceilings) + price tick in `crates/sojourn-economy/src/market.rs` (data-model §8, R12).
- [x] T047 [US6] Implement the contract lifecycle + partnership trust state + IP licensing in `crates/sojourn-economy/src/contract.rs` (data-model §8, R13).
- [x] T048 [US6] Implement `PostRfp`/`BidContract`/`AwardContract`/`ReportContract`/`FormPartnership`/`RenegePartnership`/`License` handlers + seeded RFP generation on the market sub-tick + `contract-awarded`/`market-shock` emission in `crates/sojourn-economy/src/module.rs` (R12/R13).
- [x] T049 [P] [US6] Author `data/econ/launch_market.ron`, `data/econ/markets.ron` (RFP/partnership/tourism/ISM params) and `data/econ/strategic.ron` (strategic-material supply caps + a `policy_gate` label per material), all sourced. Note: the capped supply is enforced by ledger conservation (T006); the `policy_gate` field is carried for FR-EC-106, with its **political cost deferred to FA-09** (consumed as an opaque input here).
- [x] T050 [US6] Implement the `market_price`/`niche_price`/`contracts`/`partnership` queries in `crates/sojourn-economy/src/query.rs` (FR-EC-601…605).

**Checkpoint**: the player isn't alone — a living, seeded, faction-agnostic external economy.

---

## Phase 9: User Story 7 — Capital facilities & ground segment (Priority: P3)

**Goal**: capital facilities with capex/opex/capacity/upgrade; capacity gates activity rate; ground segment sizes the finite ops/comms pool; oversubscription degrades.

**Independent Test**: a commissioned ground-segment facility sizes the ops pool; more craft than the pool supports degrades outcomes; an upgrade relieves it.

### Tests for User Story 7 ⚠️

- [x] T051 [P] [US7] Facilities + ops-pool test: a mission-control + DSN facility sizes the ops/comms pool; exceeding it degrades data return and raises anomaly probability (`ops-oversubscribed`); an upgrade grows the pool; a production line gates build throughput — in `crates/sojourn-economy/tests/facilities.rs`.

### Implementation for User Story 7

- [x] T052 [US7] Implement facilities (templates/instances, capacity gating) in `crates/sojourn-economy/src/facility.rs` and the ops/comms pool (light-time, oversubscription seeded anomaly) in `crates/sojourn-economy/src/ops.rs` (data-model §9, R14).
- [x] T053 [US7] Implement `CommissionFacility`/`UpgradeFacility` handlers + facility opex accrual + ops-pool sizing/consumption in `step` + `ops-oversubscribed` emission in `crates/sojourn-economy/src/module.rs` (R14).
- [x] T054 [P] [US7] Author `data/econ/facilities.ron` (R&D/manufacturing/pad/range/mission-control/DSN/relay templates, sourced).
- [x] T055 [US7] Implement the `facility_capacity`/`ops_utilisation` queries in `crates/sojourn-economy/src/query.rs` (FR-EC-701…703, 206).

**Checkpoint**: facilities are the board; ops capacity binds fleet size honestly.

---

## Phase 10: Polish & Cross-Cutting Concerns

- [x] T056 [P] Implement the generic project/resource-delivery primitive (`OpenProject`/`DeliverToProject` + `project_status` query) in `crates/sojourn-economy/src/project.rs` + `query.rs` — the Slice 7 seam (FR-EC-808, R15).
- [x] T057 [P] Implement money→mass/Δv traceability: the `trace_cost` query resolving any dollar figure to sourced leaves exposing its mass×Δv basis, with the `all_leaves_sourced` gate — `crates/sojourn-economy/src/{trace,query}.rs` (FR-EC-805, R17).
- [x] T058 [P] Econ-data version pinning: content-hash all `data/econ/*`, pin/verify in saves (extends FA-02/03/04/05 hash guard); actionable mismatch error — `crates/sojourn-economy/src/module.rs` (FR-EC-803, R16).
- [x] T059 Conformance + determinism wiring: `conformance --module economy`; include the economy scenarios in the harness `verify`/`roundtrip`/`mutate` gates — `crates/sojourn-harness/src/*`.
- [x] T060 [P] `validate-data econ` (schema + sources + commodity/network ref resolution + no-combat) **and the analytic gates** (conservation, ISRU break-even sign, learning monotonicity, P50<P80, launch-price elasticity sign) in `crates/sojourn-harness/src/main.rs` + `data/econ/validation.ron` (FR-EC-801, `contracts/economy-data.md`).
- [x] T061 [P] Extend CI `.github/workflows/ci.yml`: `validate-data data/econ`, `conformance --module economy`, economy determinism scenarios (incl. the astro-priced route), economy bench (smoke).
- [ ] T062 [P] Add an `economy` criterion bench (daily step + monthly market tick over the curated graph; sub-ms query derivations) — `crates/sojourn-harness/benches/economy.rs` (SC-011). **Deferred** (consistent with FA-03/FA-04/FA-05 benches; perf SC verified informally by sub-ms test/step timings).
- [x] T063 [P] Run `quickstart.md` end-to-end; confirm SC-001…SC-011.

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps.
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T004–T012). The ledger (T006) and snapshot/inputs (T009/T018) gate everything.
- **US1 (P3)** → after Foundational. The MVP (ledger + location stocks).
- **US2 (P4)** → after US1 (shipments debit/credit the ledger) + the astro EdgePrice seam.
- **US3 (P5)** → after US1 (funding credits/debits Funds).
- **US4 (P6)** → after US1 + the FA-04 `VehicleCost` basis.
- **US5 (P7)** → after US1/US2 (ISRU credits local stock; break-even uses route/launch price) + FA-03 `GradeBelief`.
- **US6 (P8)** → after US1/US2/US3 (markets price launch capacity; contracts pay Funds).
- **US7 (P9)** → after US1/US2 (ops pool consumed by active shipments/craft).
- **Polish (P10)** → after the desired stories.

### Critical-path notes
- T006 (ledger) and T018 (snapshot) gate everything; T009 (composed-value inputs) is the integration seam every story consumes; T024 (dispatch + launch/in-space split) gates US2/US5/US6/US7 economics; T008 (trace) is woven through (build first, thread it as each derivation lands).
- The analytic gates (T060) verify the economics (Principle II/VII) — keep them in sync with each derivation (conservation with US1, break-even with US5, learning/P50<P80 with US4, elasticity with US6).

### Parallel opportunities
- Setup: T002/T003 parallel.
- Foundational: T007/T008/T009 parallel; T004–T006 sequential-ish (ids → commodity/ledger).
- Within a story, `[P]` test tasks and `[P]` data-authoring tasks run in parallel.
- After Foundational + US1, US3 (funding) can proceed alongside US2 (logistics) — different files; US4 alongside US2/US3; US5 once US2 lands; US6 once US1–US3 land; US7 once US2 lands.

---

## Parallel Example: User Story 2

```text
# Tests first (parallel):
T019 transfer + window        → tests/logistics.rs
T020 launch-vs-in-space flag  → tests/logistics.rs
T021 astro-priced route       → tests/integration_astro.rs
# Data + impl (different files):
T025 network.ron + scenarios  |  T022 network.rs  |  T023 shipment.rs  |  T024 dispatch handlers  |  T026 route_cost query
```

---

## Implementation Strategy

### MVP first (US1)
Setup → Foundational → US1 → **STOP & validate**: a conserved, location-addressed six-currency ledger
with recorded, replayable transactions, deterministic and double-run-identical — the spine. Demoable.

### Incremental delivery
US1 (ledger) → US2 (logistics priced in Δv) → US3 (funding) → US4 (cost) → US5 (ISRU) → US6 (markets) →
US7 (facilities). The three P1 stories (US1–US3) are the felt-scarcity core: location-addressed
resources moved at Δv cost, with money flowing in/out and bankruptcy/gutting real. US2 proves the astro
seam; US5 closes the ISRU loop; US6 the markets; the project primitive (T056) opens the Slice 7 seam.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests written first, must fail before implementation (Constitution); the **analytic gates** are the
  Principle-II/VII enforcement (conservation, break-even sign, learning monotonicity, P50<P80, elasticity).
- FA-01/02/03/04/05 suites must stay green; **no upstream-crate change** (composed-value decoupling) and
  **no kernel change**.
- Commit after each task or logical group; **run `cargo fmt --all` before committing** (CI enforces
  `fmt --check`); auto-commit is disabled (manual `/speckit-git-commit`).

### Implementation deviations (recorded for honesty)
- **Seeded streams (T010)**: the manifest declares all five streams. Three are actively drawn —
  `isru-yield` (grade uncertainty in `OperateIsru`), `market` (price fluctuation on the monthly
  sub-tick), `ops-anomaly` (degradation draw in `SetOpsLoad`). Two are **reserved**: `cost-overrun`
  (the overrun realisation is the pure, deterministic `cost::realise(est, u)` tested directly — a
  future `RealiseCost` command will draw the `u` from the stream) and `contracts` (RFP generation is
  command-driven `PostRfp` in v1; seeded auto-generation will use it). Declared-but-undrawn streams
  don't affect determinism (they simply aren't advanced).
- **ISRU operation (T041)**: production is command-driven — `OperateIsru` advances a plant by `N`
  days (ramping scale, crediting the local stock) — rather than auto-running inside the daily
  `step`. This keeps the production trigger explicit and the step focused on shipments/funding/market;
  a scenario calls `OperateIsru` to produce.
- **Private burn cadence (T030)**: the private cash-runway burn is applied on the monthly market
  sub-tick (`burn_rate × 30`) rather than every daily step — coarser but cheaper and deterministic;
  bankruptcy is detected when the period burn exceeds cash (cash floored at zero + `bankruptcy` event).
- **Astro integration test (T021)**: reached the FA-02 planner math via the test-only `sojourn-astro`
  dev-dependency (computing a real Hohmann Δv from the catalogue's gravitational parameter); the
  `economy_astro` flavour is folded into `economy_logistics.ron` (a baked planner-derived `EdgePrice`)
  for the determinism gates — no separate scenario file.
- **T062 bench**: deferred (see above).
