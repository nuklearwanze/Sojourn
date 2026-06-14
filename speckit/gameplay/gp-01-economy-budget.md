# GP-01 — Economy & Budget Spine · `/speckit` set (FA-12)

**Branch:** `013-economy-budget` · **Design:** `gameplay/02-ECONOMY-BUDGET.md` · **Depends:** GP-00

## /speckit.specify

```
/speckit.specify Make ESA's money real and give the player the first interactive screen: the Economy & Contracts screen (S6). Authoritative design: gameplay/02-ECONOMY-BUDGET.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles IV, V, VII, VIII) — read them.

WHY: GP-00 boots ESA with opening balances but the six currencies are inert and the player cannot spend. The economy is the loop's spine: appropriations fund everything, the conserved ledger tracks everything, and buying a launch is the first action that will (in GP-04) make a craft exist.

Make the annual appropriation arrive on ESA's fiscal calendar and apply to the funds balance; keep the six-currency ledger conserved across every transaction (legs sum to zero — the economy slice already guarantees this); let the player view resources addressed by location and Δv-cost; allocate discretionary versus directed budget; BUY LAUNCH capacity on the launch market (debiting funds and booking finite mass-to-orbit capacity) behind a plan→preview→commit gate whose consequence preview is core-computed and traced to the launch-market price; and view and accept/bid on service contracts and RFPs. The next fiscal vote and any overspend must be visible pressures.

All cross-system causality (one player action expanding to validated economy commands, with a single composed core-computed preview) lives in the stateless orchestration crate from GP-00, not in the renderer. The renderer collects the draft intent, shows the preview, and submits.

Acceptance: appropriations apply on the calendar; the ledger stays conserved across every transaction; buying a launch debits funds and books mass-to-orbit with a traced core-computed preview behind a gate; contracts can be accepted/bid; every economic number is sourced data; the renderer holds no economic logic. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- The political modifier on appropriations is constant until GP-08 — confirm the placeholder value and that the hook exists for GP-08 to make it live.
- Launch-market structure: orbit classes, prices, and the annual capacity model — confirm these are already in `data/econ/launch_market.ron` with sources, or what to add.
- Contract/RFP availability at the 2026 start (likely empty) and how new contracts appear over time.
- Whether allocation is committed (Direct) or itself gated.

## /speckit.plan — guidance

- In `sojourn-game`, add intents `BuyLaunch`, `AcceptContract`/`BidContract`, `AllocateBudget` expanding to the real `EconomyCommand::{BuyLaunch, Transact (with Cause), SetAllocation-equivalent}`. The composed `Preview` must come from core-computed legs (funds Δ, mass-to-orbit Δ, capacity left), never invented.
- Appropriation cadence: `Session` re-arms `EconomyCommand::RegisterFunding`; the core applies `ApplyAppropriation` at period end. Do not move this into the renderer.
- View-model: extend `EconomyView` into a budget dashboard + `LaunchMarketVM`, `ContractsVM`, `LedgerVM`. Renderer: S6 subscreens (Budget & appropriations, Resource ledger by location, Launch market, Contracts/RFP, Facilities, Partnerships) wired to the gates; reuse the resource-by-location widget.
- Tests: harness `economy_play.ron` (appropriation applied; buy launch debits + books + decrements capacity + ledger conserved; bid reserves); determinism + round-trip; view-model unit tests for the dashboard and launch-market display math.

## /speckit.tasks & /speckit.analyze — notes

Separate intents/preview composition, appropriation cadence, view-model builders, S6 renderer subscreens, tests. `/speckit.analyze` must confirm: ledger conservation invariant exercised in tests (Principle VII/realism), all prices/funding sourced (Principle V), preview core-computed and traced (Principle VIII), no economic logic in renderer (Principle IV), `sojourn-core` audit still green.
