# Implementation Plan: Economy & Logistics (FA-06)

**Branch**: `007-economy-logistics` | **Date**: 2026-06-14 | **Spec**: `specs/007-economy-logistics/spec.md`
**Input**: Feature specification from `/specs/007-economy-logistics/spec.md`

## Summary

Build `sojourn-economy`, the slice where **felt scarcity is real**: a per-faction ledger of the six
currencies (Funds, Δv/propellant, mass-to-orbit, crew-time, ops capacity, political capital) and
physical resources held as **location-addressed stocks**, moved over a **directed transport graph
priced in delta-v and time-of-flight**. Every balance change is a conserved, recorded transaction;
money is always a proxy the player can trace back to mass × Δv (Principle VII). On top of the ledger
and graph sit the two **funding models** (agency appropriations with directed funds / fiscal cliffs /
gutting; private cash-runway with financing / bankruptcy), a **P50/P80 cost model** with learning
curves, **ISRU break-even economics** with scale-up dynamics, a faction-agnostic **markets &
contracts** layer (launch market, RFP/bid, partnerships/trust, IP licensing, tourism, in-space
manufacturing), and **capital facilities & ground segment** sizing the finite ops/comms pool. ISRU
pays off only when the physics and economics actually close — never free fuel.

Per the FA-04 C1 lesson (decouple via composed values, not hard crate deps), **`sojourn-economy`
depends only on `sojourn-core`**. All cross-slice physics enters as **opaque caller inputs / composed
snapshot values**: astro edge prices (Δv/TOF/window from the FA-02 planner), world location ids and
surveyed resource grades (FA-03), vehicle cost bases and payload/propellant capacities (FA-04), and
technology maturity/understanding (FA-05) flow in as plain values the host composes — so the slice is
**unit-testable with stubs** and carries no upstream-crate coupling. The slice is a `SimModule` on
FA-01 with a daily resource-flow `step` and a **slower, data-configured market tick** (resolving the
deferred cadence question), seeded streams for every stochastic outcome (overruns, ISRU yield, market
moves, contract generation, ops anomalies), and a read-only **economy-query surface** (balances,
route cost, break-even, cost estimate, market prices, ops utilisation, money→mass/Δv traceability)
for FA-07/09/10. **No kernel change; no change to astro/world/vehicle/research** — the economy reads
their outputs as values. The three confirmed scope decisions hold: base construction is Slice 7 (the
economy exposes a generic project/resource-delivery primitive); AI economic agency is Slice 9 (the
world market/contracts are a parametric + seeded layer here); logistics edges are priced by the
astro analytic planner with cargo flown as deterministic timed transfers.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/02/03/04/05).
**Primary Dependencies**: `sojourn-core` (kernel contracts) **only** as a crate dependency; `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`, `rand_core` (the kernel stream trait). **No `sojourn-astro`/`-world`/`-vehicle`/`-research` crate dependency** — their outputs (edge Δv/TOF/window, location ids, surveyed grades, vehicle cost/capacity, tech maturity) flow in as composed values/opaque inputs (the FA-04 C1 decoupling). No new third-party economics/graph deps: the ledger, conserved transactions, transport graph, break-even, P50/P80 draw, learning curve and reliability/anomaly draws are in-crate on the kernel's seeded streams.
**Storage**: Data files only — `data/econ/` (commodity taxonomy referencing FA-03 resource ids; launch market; per-faction funding profiles; ISRU process params; cost-uncertainty params; facilities/ground-segment; strategic-material supply caps; transport-network node/edge templates; market/contract/tourism params) and `data/econ/validation.ron` (analytic cases: conservation, ISRU break-even, learning monotonicity, P50<P80), all carrying `source` provenance and validated in CI.
**Testing**: `cargo test` (unit + integration: ledger conservation, location-addressed stocks, logistics transfers + windows, funding/bankruptcy/gutting, P50/P80 + learning, ISRU break-even + scale-up, markets/contracts/partnerships, facilities/ops-pool); **analytic validation gates** (conservation identity, ISRU break-even sign, learning-curve monotonicity, P50<P80) per the constitution testing mandate; kernel conformance (`conformance --module economy`); harness determinism gates (verify/roundtrip/mutate); an **astro-priced integration test** (a route priced by the real FA-02 planner fed to the economy); `validate-data` extended to econ.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-economy`) implementing the FA-01 `SimModule` contract + a public read-only economy-query API; harness/bench/data extensions. No kernel/astro/world/vehicle/research change.
**Performance Goals**: The economy steps daily and the market on a slower tick; per-tick work is proportional to active shipments + plants + the **curated graph (tens-to-low-hundreds of nodes)**, not the ~3,000-body catalogue; query-surface derivations are sub-millisecond pure computations; the kernel envelope (≥1 sim-year/min at high warp) holds across century horizons with large fleets and thousands of location-addressed stocks.
**Constraints**: Full kernel determinism (ordered `BTreeMap`/`BTreeSet` stores, libm-only, no wall-clock, seeded streams threaded explicitly — overruns/ISRU/market/contracts/anomalies all seeded); **resource conservation** (no stock created/destroyed except by a modelled process); **no magic numbers** (Principle II — all economic constants from sourced data); SI units (Funds abstracted to one common accounting unit); money traceable to mass × Δv (Principle VII); single-writer (economy owns its slice; reads others' outputs as values); analytic-case CI gates; econ-data version pinned in saves.
**Scale/Scope**: Six currencies × ten factions; a curated transport graph (tens-to-low-hundreds of nodes referencing world location ids); thousands of location-addressed stocks; the two funding models; the P50/P80 cost model; the four ISRU process families (lunar ice, Mars Sabatier, regolith O₂/metals, asteroid volatiles); the markets/contracts/partnership/IP/tourism/ISM layer; capital facilities + ground segment; century horizons (durable state, daily-stepped, monthly market tick).

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS | Every economic constant (launch $/kg by class, ISRU yields & plant mass, budget baselines, learning exponents, strategic-material supply caps, market sizes, facility capex/opex, tourism/ISM ceilings) lives in `data/econ/*` with a `source`; `validate-data` (extended) fails CI on any missing source; no speculative figures without a citable basis. |
| II. Physics authoritative / no magic numbers | PASS | The economy *derives* from physics: route cost = real Δv/TOF (astro planner) × vehicle propellant; ISRU break-even = real launch-cost-saved vs plant mass/power; the engine carries no per-route/per-plant magic numbers (all constants from data). Money stays a proxy for mass × Δv (FR-EC-805). Analytic gates (conservation, break-even sign, learning monotonicity, P50<P80) gate CI. |
| III. Deterministic core | PASS | Ordered stores, libm-only, no wall-clock; every stochastic outcome (overrun draw, ISRU grade/accessibility, market move, contract generation, ops-anomaly) derives from a **named seeded stream**; the slower market tick is a deterministic sub-schedule of the daily step; double-run / roundtrip / mutate / replay / conformance gates. |
| IV. Headless / decoupled | PASS | Pure library module + read-only economy-query functions; zero UI deps; everything driven/audited via harness scenarios; depends only on `sojourn-core`. |
| V. Data-driven content | PASS | Commodities, funding profiles, market/ISRU/facility/strategic-material/network params and validation cases are schema-validated data; mechanics in code, content in data; new event classes are data registry entries. |
| VI. Research a modelled process | N/A (consumed) | Technology maturity/understanding (ISRU & facility gating, IP licensing) come from FA-05's contract as composed values; this slice never reinvents research. |
| VII. Tyranny of mass / Δv (this slice is an embodiment) | PASS | Mass and Δv are the dominant, surfaced constraints: resources are location-addressed and priced in Δv; mass-to-orbit is a first-class currency; every cost is traceable to its physical basis; ISRU/reuse/depots/refuelling are meaningful **because** they relax the mass/Δv constraint (rocket equation reset at a node), never via arbitrary bonuses. |
| VIII. Educational honesty | PASS | Sourced constants; money→mass/Δv traceability is the honesty contract; ISRU break-even tells the truth (no free fuel); no misinformation. |
| IX. No combat/aliens | PASS | No weapons/sabotage economy; competition is the milestone race, economics and politics; discovered life is never an economic actor. |
| Engineering constraints | PASS | SI everywhere; daily step + monthly market tick within tick-time budget; econ-data version pinned in saves (extends FA-02/03/04/05 hash pattern); fully offline; no hidden randomness. |
| **Cross-slice coupling** | NOTED (none added) | `sojourn-economy` depends **only on `sojourn-core`**; astro/world/vehicle/research outputs flow in as composed values (the FA-04 C1 decoupling), so the dependency graph gains no new crate edges and the slice is unit-testable with stubs. No kernel/astro/world/vehicle/research change. |

**Post-Phase-1 re-check (2026-06-14)**: design artifacts introduce no new violations; no kernel
amendment; no upstream-crate change (composed-value decoupling); single-writer preserved; the market
tick is a deterministic sub-schedule. Gate remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/007-economy-logistics/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R17)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── economy-queries.md       # The read-only economy-query + money→mass/Δv traceability surface (FA-07/09/10 seam)
│   ├── economy-commands.md      # Commands (via ModulePayload) + events (budget/contract/bankruptcy/shipment/isru/shock)
│   ├── economy-data.md          # Commodity/funding/ISRU/market/facility/network/strategic data formats, sourcing, analytic gates
│   └── integration-seams.md     # The composed-value inputs: astro edge price, world grade, vehicle cost, research maturity
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged this slice
├── sojourn-astro/               # FA-02 — unchanged (planner outputs consumed as composed values)
├── sojourn-world/               # FA-03 — unchanged (location ids + resource grades consumed as values)
├── sojourn-vehicle/             # FA-04 — unchanged (cost basis + capacity consumed as values)
├── sojourn-research/            # FA-05 — unchanged (maturity/understanding consumed as values)
├── sojourn-economy/             # THIS SLICE — pure library, SimModule implementor (dep: core only)
│   ├── Cargo.toml               # deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core
│   └── src/
│       ├── lib.rs               # public surface: EconomyModule, EconomyCommand, economy queries
│       ├── ids.rs               # FactionId, CommodityId, LocationId(String), NodeId, EdgeId, ShipmentId, PlantId, ContractId, FacilityId
│       ├── commodity.rs         # FA-06 commodity taxonomy (references FA-03 resource ids; processed/manufactured/services) load + validation
│       ├── ledger.rs            # six-currency accounts + location-addressed stocks; conserved Transaction; queries
│       ├── network.rs           # transport graph (curated nodes ref world locations; edges); routing over priced edges
│       ├── shipment.rs          # shipment lifecycle (ordered→waiting-window→in-transit→delivered); tugs/cyclers/depots
│       ├── funding.rs           # agency appropriation (directed funds/carry-over/cliff/gutting) + private cash-runway/financing/bankruptcy
│       ├── cost.rs              # P50/P80 estimate + seeded overrun realisation; learning-curve wrapper over the vehicle cost basis
│       ├── isru.rs              # ISRU process models, break-even, scale-up + reliability ramp (seeded grade/accessibility)
│       ├── market.rs            # launch market ($/kg by orbit class, capacity elasticity); tourism/ISM markets; price tick
│       ├── contract.rs          # RFP/bid/award/fulfil/fail lifecycle; partnerships + trust state; IP licensing
│       ├── facility.rs          # capital facilities + ground segment; capacity gating; ops/comms pool sizing
│       ├── ops.rs               # finite ops/comms pool, light-time, oversubscription degradation (seeded anomaly)
│       ├── project.rs           # generic project/resource-delivery accounting primitive (the Slice 7 seam)
│       ├── trace.rs             # money→mass/Δv traceability tree (sourced leaves)
│       ├── query.rs             # EconomySnapshot (slice + composed upstream values) + pure derivation queries
│       └── module.rs            # SimModule: economy slice, commands, daily step + monthly market tick, publish, save/load_slice
│   └── tests/                   # ledger.rs, logistics.rs, funding.rs, cost.rs, isru.rs, markets.rs, facilities.rs,
│                                # integration_astro.rs (real planner price), conformance.rs, validation.rs, common/mod.rs
├── sojourn-harness/             # + `economy` scenario flag, validate-data econ, conformance --module economy, bench
data/
└── econ/
    ├── commodities.ron          # tradable-commodity taxonomy (FA-03 resource refs + processed/manufactured/consumables/spares/services) sourced
    ├── funding.ron              # per-faction funding profiles (agency baselines/volatility/carry-over/geo-return; private burn/revenue/financing) sourced
    ├── launch_market.ron        # $/kg by orbit class, capacity-elasticity, world-capacity baseline sourced
    ├── isru.ron                 # process params (yield, plant mass, power, scale-up/reliability ramp) for ice/Sabatier/regolith/asteroid sourced
    ├── cost.ron                 # P50/P80 spread + overrun model params (learning exponents reference the vehicle cost basis) sourced
    ├── facilities.ron           # facility types incl. ground segment/DSN (capex/opex/capacity/upgrade) sourced
    ├── strategic.ron            # strategic-material supply caps (Pu-238/Am-241, enriched/LEU, rare-earth) sourced
    ├── network.ron              # curated transport-graph node set (ref world location ids) + edge templates sourced
    ├── markets.ron              # RFP-generator params, partnership/trust params, tourism/ISM market sizes & price ceilings sourced
    └── validation.ron           # analytic validation cases + tolerances (conservation, ISRU break-even, learning monotonicity, P50<P80)
scenarios/                       # + economy_logistics.ron (ledger + shipments + funding + ISRU + contracts), economy_astro.ron (astro-priced route)
```

**Structure Decision**: `sojourn-economy` is the **widest-consuming slice yet** — it sits above the
world, astro, vehicle and research outputs — but, applying the FA-04 C1 lesson, it takes a **hard
dependency only on `sojourn-core`** and consumes every cross-slice physics value (edge Δv/TOF/window,
location ids, surveyed grades, vehicle cost/capacity, tech maturity) as **composed inputs the host
assembles**. This keeps the dependency graph acyclic with no new crate edges, makes the slice
unit-testable with stubs, and matches the read-only between-ticks seam FA-02/03/04/05 expose. The
economy owns its slice (ledger, graph, shipments, plants, contracts, facilities, market state) as the
single writer; derived figures (route cost, break-even, prices, traceability) are **pure query-time
computations** over an `EconomySnapshot` that composes the slice with the live upstream values, so
they track the world/research without storing stale copies. The daily resource-flow step plus a
data-configured **monthly market tick** resolves the cadence question the spec deferred to planning.

## Complexity Tracking

No constitution violations to justify — table intentionally empty. The slice is **broad** (many
sub-systems) but architecturally plain: one module crate depending only on core, the established
command/event/query/seeded-stream patterns, composed-value decoupling, no kernel or upstream change.
Breadth is managed by decomposition into independent sub-modules (ledger, network, funding, cost,
isru, market, contract, facility) each behind its own tests and analytic gates.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
