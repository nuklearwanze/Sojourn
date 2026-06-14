# GP-01 — Economy & Budget Spine (FA-12)

**Spec dir:** `specs/013-economy-budget` · **Depends on:** GP-00 · **Speckit:** `speckit/gameplay/gp-01-economy-budget.md`

Makes ESA's money real. The six currencies become live constraints, appropriations arrive on the fiscal calendar, the conserved ledger tracks everything, and the player can spend — most importantly **buy a launch**, the first action that will (in GP-04) make a craft exist. This is the loop's economic spine and the first interactive screen (S6).

## Goal & player-facing capability

Receive the annual appropriation; see the conserved ledger and resources addressed by location/Δv; allocate discretionary budget; **buy launch capacity** on the launch market (debiting funds and booking mass-to-orbit) via plan→preview→commit; view and bid on service contracts / RFPs; see facilities and partnerships. Overspending and the next fiscal vote are visible pressures.

## Orchestration intents → command fan-out

In `sojourn-game`:
- `Intent::AllocateBudget { directed_splits }` → reversible economy allocation (`SetAllocation`-style via `ModuleCommand`/`Transact` bookkeeping).
- `Intent::BuyLaunch { orbit_class, mass_kg }` → `EconomyCommand::BuyLaunch` (debits funds, books mass-to-orbit, finite annual capacity). Gated; preview = funds Δ + mass-to-orbit Δ + remaining capacity, traced to launch-market price.
- `Intent::AcceptContract { id }` / `Intent::BidContract { id, bid }` → `EconomyCommand::Transact` with the contract `Cause`. Gated.
- Appropriations: `Session` re-arms `RegisterFunding`; on period end the core applies `ApplyAppropriation` (amount scaled by the political modifier from GP-08 later; constant for now).

The preview composes core-computed deltas; the conserved-ledger invariant (legs sum to zero) is the economy slice's existing guarantee.

## Cross-system causality & state touched

Funds/mass-to-orbit/ops-capacity become the currencies later increments spend: research allocation (GP-02) draws funds; production (GP-03) draws funds + facility capacity; a launch booked here is consumed when GP-04 spawns the craft; logistics shipments (GP-06) price edges in Δv + funds. State: economy slice (journalled). No new module.

## ESA data

Reuses `data/econ/*` (funding profiles, launch market, cost params, facilities, network) — confirm ESA's `funding.ron` profile and launch-market prices carry sources; add any ESA-specific lines to `data/scenario/esa_2026.ron`.

## UI/UX — S6 Economy & Contracts (now interactive)

Overview tier: the six-currency header expanded into a **budget dashboard** (annual budget, committed, directed, discretionary remaining, next vote countdown).

Subscreens:
- **Budget & appropriations** — appropriation **timeline** widget (past/next votes on the fiscal calendar), directed vs discretionary split, allocation controls (reversible sliders; committing an allocation is `Direct`).
- **Resource ledger by location** — virtualised table: location · Δv-from-LEO · commodity · qty; the existing resource-by-location widget, now live and filterable.
- **Launch market** — available launch services by orbit class + price + remaining annual capacity; "Buy launch" opens the gate (preview: funds Δ, mass-to-orbit Δ, capacity left, trace to price). The booked capacity shows as a pending asset.
- **Contracts / RFP board** — open service contracts and RFPs; "Accept" / "Bid" gated; shows value, type, partner.
- **Facilities** — labs/pads/ground-segment with capacity utilisation (ops-capacity).
- **Partnerships / consortia** — barter and geo-return lines (read-only summary for now).

Inspector: selected ledger line / contract / launch service, with trace.

Plan→preview→commit verbs: Buy launch (Build/Launch kind), Accept/Bid contract. Empty states: no contracts yet → "RFP board empty — partners post work as the world develops."

View-model: `EconomyView` is extended into the dashboard + per-subscreen builders (`LaunchMarketVM`, `ContractsVM`, `LedgerVM`), unit-tested. Renderer wires the buy/bid gates to `sojourn-game`.

## Testability

Harness scenario `economy_play.ron`: boot ESA → advance to appropriation → assert balance += sourced amount → buy a launch → assert funds debited + mass-to-orbit booked + capacity decremented + ledger conserved (legs sum 0) → bid a contract → assert reservation. Determinism + round-trip. View-model tests for the dashboard and launch-market math (display only). Human: buy a launch in the app and watch the top-bar currencies move.

## Acceptance criteria

Appropriations arrive on the calendar; the ledger stays conserved across every transaction; buying a launch debits funds and books mass-to-orbit with a traced, core-computed preview behind a gate; contracts can be accepted/bid; all economic numbers are sourced; renderer holds no economic logic.

## Out of scope

Spawning the craft from a booked launch (GP-04). Production cost of designs (GP-03). Logistics shipments along edges (GP-06). The mood→budget modifier (GP-08, constant until then).
