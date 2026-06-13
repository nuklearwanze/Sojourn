# Phase 0 Research: Economy & Logistics (FA-06)

Decisions resolving the Technical Context against Constitution v1.0.0 (esp. Principles VII/VIII/IX
and I/II/III), the FA-01/02/03/04/05 contracts in the tree, and the spec's clarified scope. Format:
**Decision / Rationale / Alternatives rejected**. No `NEEDS CLARIFICATION` remain (spec clarified
2026-06-14; three scope forks confirmed; the cadence question is resolved here at R7).

---

## R1 — Crate topology: an economy above world/astro/vehicle/research, coupled only to core

**Decision.** A new crate `crates/sojourn-economy` takes a **hard dependency only on `sojourn-core`**.
All cross-slice physics — astro edge prices (Δv/TOF/window), world location ids + surveyed resource
grades, vehicle cost bases + payload/propellant capacities, research maturity/understanding — enters
as **composed values / opaque caller inputs** assembled by the host. Later slices (FA-07 bases, FA-09
politics) depend on `sojourn-economy`.

**Rationale.** The FA-04 C1 finding: hard-linking every upstream crate makes the slice un-unit-testable
and over-coupled. The economy needs *values* (a route's Δv, a site's believed grade, a design's unit
cost, a tech's maturity), not the upstream engines. Taking only core keeps the dependency graph
acyclic with **no new crate edges**, makes the slice testable with stubs, and matches the read-only
between-ticks seam every prior slice exposes.

**Alternatives rejected.** (a) Depend on astro+world+vehicle+research crates directly — heavy coupling,
slow tests, repeats the C1 mistake. (b) Put economy logic in the host — violates Principle IV (logic
belongs in a testable headless module).

---

## R2 — Composed-value integration seams (the four inputs)

**Decision.** Define four narrow input shapes the host composes and the economy consumes:
- **EdgePrice** `{ dv_mps, tof_s, next_window_tick, window_open }` — from the FA-02 planner
  (`porkchop_departure_dv`/`lambert_solve`/`lowthrust_arc`) for a given (from,to) at a time.
- **GradeBelief** `{ resource_id, believed_grade, certainty }` — from FA-03's `believed_site`/
  `certainty_site` (the faction belief-state, never ground truth).
- **VehicleCost** `{ design_id, unit_cost_basis, build_days_basis, payload_kg, propellant_kg, dv_mps }`
  — from FA-04's `DesignSnapshot`/`CostEstimate`.
- **TechMaturity** `{ tech_id, trl, understanding, flyable }` — from FA-05's `maturity()`/`understanding()`.

Carried into **commands** at decision time (the plan→preview→commit pattern) and into the
**EconomySnapshot** at query time. Tests feed stubs; the harness feeds the real upstream queries.

**Rationale.** Narrow value types keep the coupling explicit, serializable (IPC for the Tauri host),
and honest — the economy acts on the **belief-state** grade and the **planner** price, never on
hidden truth or the authoritative propagator state.

**Alternatives rejected.** (a) Pass whole upstream snapshots — leaks their surfaces and ground truth.
(b) Recompute physics in-economy — duplicates astro/vehicle, violates single-source.

---

## R3 — Six-currency ledger + location-addressed stocks; conserved transactions

**Decision.** The slice holds, per faction, an **Account** of the six currency balances and a
**stock map** keyed by `(CommodityId, LocationId)` — so the same commodity at two locations is two
goods. Every mutation is a recorded **Transaction** `{ tick, faction, legs: [(account/stock, delta)],
cause }` applied atomically; the ledger **rejects** any transaction that would drive a balance/stock
negative (conservation). Currencies and stocks are ordered `BTreeMap`s.

**Rationale.** FR-EC-101…105: a conserved, auditable, replayable spine. Transaction legs make
double-entry conservation checkable (Σ modelled inflow = Σ outflow) and give the audit/replay log for
free. Location-keyed stocks are the physical heart (Principle VII).

**Alternatives rejected.** (a) Global resource piles — the design explicitly forbids; kills the
delta-v map. (b) Mutate balances directly without transactions — no audit/replay, conservation
unverifiable.

---

## R4 — Transport graph: curated nodes, planner-priced timed transfers (clarified Q1/Q3:A)

**Decision.** A **directed graph** whose nodes are a **curated set** referencing FA-03 location ids
(staging nodes + active Sites; tens-to-low-hundreds) and whose edges are transfers. An edge's
**price (Δv/TOF/window) is supplied as an EdgePrice value** (R2) from the FA-02 analytic planner; the
economy does not propagate. Dispatching a shipment assigns a **vehicle** (finite propellant/Δv +
payload), debits propellant/Δv (and crew-time/ops), and schedules **arrival = depart_tick + TOF**;
the shipment is a deterministic **timed transfer**, reconciled against the propagator only for
operationally-significant flights (out of this slice). Nodes are added when a faction begins operating
at a new Site.

**Rationale.** Confirmed Q1 (curated node set) + Q3 (planner-priced) — matches the two-tier fidelity
design, keeps the economy performant at high warp, and keeps the slice decoupled from astro.

**Alternatives rejected.** (a) Full-catalogue graph — thousands of nodes; perf + data blowup. (b)
Propagate every cargo flight — heavy at warp; duplicates astro; rejected in clarify.

---

## R5 — Launch vs in-space transfer; mass-to-orbit as a managed capacity currency (clarified Q2:A)

**Decision.** **Surface→orbit (launch) edges** consume the **mass-to-orbit capacity** currency, not a
vehicle's Δv. That capacity is sized by **owned launch vehicles** (their per-period throughput) **plus
capacity bought on the launch market** (Funds, $/kg by orbit class — R12). **In-space transfer edges**
consume **propellant/Δv** on the assigned vehicle. A faction self-launches (consumes owned capacity)
or buys launch (consumes Funds); both deliver mass to the orbit node.

**Rationale.** Confirmed Q2: keeps mass-to-orbit a first-class currency (per the six-currency design),
distinguishes the gravity-well tax from in-space motion, and supports the vertical-integration vs
buy-launch strategic axis (Helion vs others).

**Alternatives rejected.** (a) Launch as an ordinary Δv edge — demotes currency #3, hides the launch
market. (b) Launch only via market — removes self-launch (the reusable-launch archetype).

---

## R6 — Depots, tugs, cyclers as graph-resident assets

**Decision.** **Depots** are buffer nodes that hold stocks (propellant/cargo) decoupling production
from transport; a tanker delivers to a depot, another vehicle draws later. **Tugs** are reusable
vehicles that, on arrival, return to a pool and are re-dispatchable (cost amortises over trips).
**Cyclers** follow a fixed recurring path (data-defined leg schedule) carrying cargo/crew cheaply on a
recurring cadence. All are economy-side assets keyed to nodes/edges; their *flight* is the timed
transfer (R4).

**Rationale.** FR-EC-204/205 — the architectures that make deep-space logistics close; modelled as
accounting + scheduling over the graph, not new physics.

**Alternatives rejected.** (a) One-shot expendable transfers only — loses the reuse economics central
to the design. (b) Simulate tug/cycler trajectories — astro's job; here they are scheduled transfers.

---

## R7 — Module cadence: daily resource-flow step + monthly market tick (resolves deferred cadence)

**Decision.** `EconomyModule` declares `cadence_ticks = 86_400` (daily), matching world/research/
vehicle. The **market/price tick** runs on a **data-configured slower sub-schedule** (default
~30 days) checked inside `step` (`tick % market_period == 0`), updating launch/market prices and
generating contracts. Funding periods (fiscal years/quarters) and ISRU operation advance on the daily
step. All deterministic; no separate clock.

**Rationale.** Resolves the cadence question the spec deferred. Daily granularity suffices for
resource flows, shipment arrivals and ISRU; the market needs only a slow tick (design §10 "prices
update on a slower market tick"). A sub-schedule inside the daily step keeps a single deterministic
cadence.

**Alternatives rejected.** (a) Per-tick (sub-day) economy — needless cost; nothing economic resolves
faster than a day. (b) A second module with its own cadence — over-engineered; one slice, one cadence,
internal sub-schedule.

---

## R8 — Funding models: appropriation and cash-runway as data-driven faction profiles

**Decision.** A per-faction **FundingProfile** (data) selects **agency** (periodic appropriation
baseline + volatility, directed/earmarked funds, carry-over rule, fiscal-cliff parameters; cannot go
bankrupt but can be **gutted** below a caretaker threshold) or **private** (cash, burn, revenue,
financing events — equity/debt/owner injection; **bankruptcy** when cash < 0). External political
inputs (mood/prestige modifiers, directed funds, sanctions) enter as **opaque scalar inputs** via
command (politics is FA-09). Bankruptcy/gutting set a **state flag + emit an event** (the core's
interrupt-and-pause / observer-mode per 00-OVERVIEW §9); they do not hard-halt the sim.

**Rationale.** FR-EC-301…305 + the asymmetric-but-fair pillar — asymmetry is data, not different
physics. The state-flag+event semantics resolve the deferred bankruptcy/gutting question consistently
with the core's interrupt model.

**Alternatives rejected.** (a) Hard-code faction budgets — violates Principle V/asymmetry-as-data. (b)
Bankruptcy hard-stops the sim — the overview says observer/rebuild mode; flag + event is right. (c)
Model politics here — FA-09's scope; consume opaque inputs.

---

## R9 — Cost model: P50/P80 estimate + seeded overrun; learning wraps the vehicle basis

**Decision.** A program/item cost is estimated with **P50/P80 bands** from sourced spread params over
the **VehicleCost basis** (R2) and tech maturity; the **realised** cost is drawn from the distribution
on the named seeded stream `cost-overrun` (overruns possible, reproducible). The **learning curve**
(unit cost ↓ with cumulative production) **reuses the FA-04 cost basis** (which already applies
Wright's law over production_count); FA-06 adds the uncertainty wrapper and funding context, storing
only realised-cost outcomes, not a duplicate learning state.

**Rationale.** FR-EC-401…404 + the FA-04/FA-06 split (FA-04 = physical unit cost + learning; FA-06 =
economic uncertainty + funding). Drawing on a seeded stream keeps determinism; P50/P80 from data keeps
it sourced and auditable.

**Alternatives rejected.** (a) Re-derive learning in economy — duplicates FA-04. (b) Deterministic
point cost — the design mandates P50/P80 uncertainty + overruns. (c) Unseeded random overrun — breaks
determinism.

---

## R10 — ISRU: process model, break-even, scale-up on seeded grade/accessibility

**Decision.** An **ISRUPlant** sites a **process** (lunar-ice electrolysis; Mars Sabatier; regolith
O₂/metals; asteroid volatiles) at a node, producing a commodity at a **yield** = f(sourced process
params, **believed grade** (GradeBelief, R2), available power, plant mass) with grade/accessibility
**uncertainty drawn on the `isru-yield` seeded stream** (consistent with the world belief-state).
**Break-even** = (launch cost saved at the destination node, from the launch market/route) −
(plant delivery mass + build + operate + amortise); a **scale-up ramp** (pilot→production) applies a
sourced yield/reliability multiplier curve so first-of-a-kind plants under-perform. Output credits the
**local propellant/feedstock stock** (closing the logistics loop) and the project/feedstock seam.

**Rationale.** FR-EC-501…505 — the heart of the off-Earth economy; break-even as a *computed truth*
(advisory, like the FA-04 realism guards) means the player can build a bad plant but the numbers don't
lie (no free fuel). Seeded grade draw ties ISRU to the surveyed belief-state honestly.

**Alternatives rejected.** (a) Free/fixed ISRU output — the design's cardinal sin. (b) Read ground-truth
grade — violates the honesty contract; use belief + seeded draw. (c) Instant nameplate output — drops
the first-of-a-kind realism.

---

## R11 — Resource conservation as a checked invariant

**Decision.** Every modelled process (launch, transfer, ISRU conversion, consumption, production,
trade) is expressed as a **conserving Transaction** (R3): mass in = mass out + losses (boil-off,
process inefficiency) where losses are *explicit legs to a sink*, never silent. A harness/analytic
gate runs an arbitrary script and asserts **Σ accounted inflow = Σ outflow** and no negative stock.

**Rationale.** FR-EC-104 + SC-003 — conservation is the auditable property that makes the economy
trustworthy; making losses explicit legs keeps the books balanced and the physics honest.

**Alternatives rejected.** (a) Allow implicit creation/destruction — unauditable; the design forbids.
(b) Conserve only bulk mass, ignore losses — boil-off eating Δv is a design-required mechanic.

---

## R12 — Launch market + tourism/ISM markets: parametric, seeded, faction-agnostic (clarified Q2:A)

**Decision.** A **launch market** sets $/kg by orbit class from a **world-capacity index** with a
sourced **price elasticity**; the index responds to aggregate fielded reuse (a parametric world model,
seeded fluctuation on the `market` stream). Factions **buy** launch (Funds → mass-to-orbit) when
cheaper than self-launch, or **sell** spare owned capacity. **Tourism** and **in-space manufacturing**
markets have **sourced finite sizes + price ceilings** (no infinite revenue). All mechanisms are
**faction-agnostic** (player + later AI use the same code) and seeded where stochastic.

**Rationale.** FR-EC-601/604/605 + confirmed Q2 (parametric + seeded world layer; AI agency is FA-09).
A capacity-elasticity model captures "everyone fields reuse ⇒ $/kg collapses ⇒ ISRU's relative value
rises" (the design's key feedback) without simulating AI factions here.

**Alternatives rejected.** (a) Fixed launch price — misses the central market feedback. (b) Simulate
AI faction supply now — FA-09's scope (confirmed). (c) Unbounded tourism/ISM revenue — the design caps
these.

---

## R13 — Contracts & partnerships: RFP/bid lifecycle + trust state machine

**Decision.** A **Contract** has a lifecycle (**posted → bid → awarded → in-progress →
fulfilled | failed**), carrying a deliverable (deliver X to a node / host payload / crew taxi), a
reward (Funds + heritage) and a failure **penalty**; RFPs are **generated on the seeded `contracts`
stream** from sourced generator params. A **Partnership** holds per-faction-pair **trust state**
(co-funding, shared TRL/IP/seats/data, geo-return/barter); a reneged commitment degrades trust with a
**lasting reputation cost**. **IP licensing** (sell data, license matured tech for royalties) draws on
FA-05 maturity (R2). All faction-agnostic.

**Rationale.** FR-EC-602/603/604 — the CLPS/COTS model and the consortium playstyles; a state machine
+ trust scalar is the simplest faithful model; seeded generation keeps determinism and lets FA-09's AI
drive the same mechanism.

**Alternatives rejected.** (a) Instant contract resolution — loses bid/award/fulfil drama and the
heritage/penalty stakes. (b) No trust persistence — betrayal must have lasting cost (design §5).

---

## R14 — Facilities & ground segment: capacity gating + the ops/comms pool

**Decision.** A **Facility** (R&D/test stand, manufacturing/integration line, pad/range, mission
control, antenna/DSN, relay) has **capex/opex/capacity/level** (data). Capacity **gates the rate** of
its activity (line → build throughput; pad/range → launch cadence; ground segment → ops/comms pool
size). **Ops capacity & comms** are a **finite shared pool** consumed by active craft under
**light-time** (a per-edge/per-node delay value); **oversubscription** degrades outcomes (reduced data
return, raised anomaly probability on the seeded `ops-anomaly` stream). Range access is a scheduling
(and opaque political) constraint.

**Rationale.** FR-EC-701…703 + FR-EC-206 — facilities are the board you build on; the ops-pool
oversubscription is a real fleet-size constraint (design §6) and must bite, not be free.

**Alternatives rejected.** (a) Infinite ops capacity — removes a core constraint. (b) Hard cap on
active craft — the design wants graceful degradation, not a wall.

---

## R15 — The project/resource-delivery primitive (the Slice 7 seam) (clarified Q-scope:A)

**Decision.** Expose a generic **Project** primitive: a target requiring **delivered mass + crew-time +
time** (and optionally ISRU/feedstock) routed through the logistics graph, advancing to **completion**
as deliveries land. FA-06 owns the *accounting* (what was delivered, what remains, schedule); **Slice 7
(Bases & Construction)** consumes it to assemble bases/stations from modules. No base/module assembly
logic here.

**Rationale.** Confirmed scope: keeps the slice boundary clean while giving Slice 7 the economic
substrate (delivery accounting) it needs, avoiding a later refactor.

**Alternatives rejected.** (a) Build the base assembler here — Slice 7's scope; bloats FA-06. (b) No
project primitive — forces Slice 7 to re-implement delivery accounting.

---

## R16 — Determinism, seeded streams, data-version pin, event registry

**Decision.** Named seeded streams: `cost-overrun`, `isru-yield`, `market`, `contracts`, `ops-anomaly`
(threaded via `ctx.rng(path)`). Ordered `BTreeMap`/`BTreeSet` stores; libm-only; no wall-clock.
Econ data is content-hashed and **pinned in saves** (extends the FA-02/03/04/05 pattern;
CRLF-normalized). New **event classes** (data registry, `data/kernel/event-classes.ron`):
`budget-cycle` (LogOnly), `contract-awarded` (Interrupt), `bankruptcy` (Interrupt), `agency-gutted`
(Interrupt), `shipment-arrived` (LogOnly), `isru-online` (LogOnly), `market-shock` (Interrupt),
`ops-oversubscribed` (LogOnly). **No kernel change.**

**Rationale.** Mirrors every prior slice's determinism discipline; interrupt-class events feed the
FA-01 interrupt-and-pause loop (budget votes, contract awards, bankruptcies, market shocks are exactly
the "stop on something that matters" cases).

**Alternatives rejected.** (a) Global RNG — non-deterministic. (b) Unpinned data — silent realism
drift across saves. (c) New kernel event plumbing — events are data registry entries.

---

## R17 — Economy-query surface + money→mass/Δv traceability

**Decision.** `EconomySnapshot::from_core(&core, &economy_module, inputs)` via kernel `with_slice` over
the economy slice, composing the R2 input values. Pure functions answer: a faction's balances; the
stock of a commodity at a location; a **route cost** (composing EdgePrice + vehicle propellant); an
**ISRU break-even**; a **P50/P80 cost estimate**; current **market prices**; **contract/partnership**
state; **facility capacity** and **ops-pool utilisation**; and a **traceability tree** resolving any
dollar figure to its **mass × Δv** basis (sourced leaves). Faction-scoped where belief/funding is
involved.

**Rationale.** FR-EC-805 + Principle VII/VIII — the identical read-only between-ticks seam as
FA-02/03/04/05; the money→mass/Δv trace is the honesty contract that keeps money secondary to physics,
and is IPC-serializable for FA-07/09/10 and the Tauri host.

**Alternatives rejected.** (a) Mutable query handles — break read-only/determinism. (b) Store derived
prices/break-even in the slice — stale vs market/research; recompute at query time (the R3/FA-04 R3
pattern).

---

## Summary of decisions feeding Phase 1

| # | Decision | Primary artifacts |
|---|---|---|
| R1 | `sojourn-economy` above all, coupled only to core | plan structure |
| R2 | Composed-value integration seams (4 inputs) | contracts/integration-seams |
| R3 | Six-currency ledger + location stocks; conserved transactions | data-model, contracts/economy-queries |
| R4 | Curated graph; planner-priced timed transfers | data-model, contracts/integration-seams |
| R5 | Launch = mass-to-orbit; in-space = Δv | data-model, contracts/economy-commands |
| R6 | Depots, tugs, cyclers as graph assets | data-model |
| R7 | Daily step + monthly market tick | contracts/economy-commands, module |
| R8 | Funding profiles; bankruptcy/gutting as flag+event | data-model, contracts/economy-commands |
| R9 | P50/P80 + seeded overrun; learning wraps vehicle basis | data-model, contracts/economy-data |
| R10 | ISRU process/break-even/scale-up on seeded grade | data-model, contracts/economy-data |
| R11 | Conservation as a checked invariant | contracts/economy-data (gates), data-model |
| R12 | Parametric+seeded launch/tourism/ISM markets | data-model, contracts/economy-data |
| R13 | Contract lifecycle + partnership trust + IP | data-model, contracts/economy-commands |
| R14 | Facilities + ops/comms pool degradation | data-model |
| R15 | Project/resource-delivery primitive (Slice 7 seam) | data-model, contracts/economy-queries |
| R16 | Determinism; seeded streams; data-version pin; events | contracts/economy-data, contracts/economy-commands |
| R17 | Composed economy-query surface + money→mass/Δv trace | contracts/economy-queries |
