# Feature Specification: Economy & Logistics (FA-06)

**Feature Branch**: `007-economy-logistics`
**Created**: 2026-06-14
**Status**: Draft
**Input**: User description: "Build Sojourn's economy and logistics: the six-currency resource simulation and the transport graph that moves everything. Authoritative design: design/03-ECONOMY.md and design/04-SPACEFLIGHT.md (logistics) plus design/00-OVERVIEW.md (currencies, factions); also .specify/memory/constitution.md (Principles VII, VIII, IX). Build on Slice 1 core, Slice 3 world (locations/resources), Slice 2 astrodynamics (transfer windows/delta-v) and Slice 4–5 (vehicle costs, learning, technology)."

## Overview

The economy is a **resource-flow + transport-graph simulation**: a per-faction ledger of six
currencies and physical resources that are addressed by **location and delta-v**, moved over a
window-constrained transport graph priced in propellant and time-of-flight. Money is always a
proxy for the physical constraint — the player can trace any dollar back to the mass and delta-v
behind it (Principle VII). ISRU pays off only when the physics and economics actually close; it is
never free fuel. The slice consumes the world (locations, resources, surveyed belief-state),
astrodynamics (windows/Δv/TOF), the vehicle designer (cost basis, capacity, learning) and research
(maturity/heritage/understanding) through their existing interfaces, and owns no logic of theirs.

**Scope boundary**: this slice provides the economic *substrate* — currencies, location-addressed
resources, logistics, cost, markets, ISRU economics, and capital facilities as economic assets.
Assembling bases/stations from modules is **Slice 7 (Bases & Construction)**; this slice exposes a
generic project / resource-delivery accounting primitive that Slice 7 consumes. Per-faction AI
economic decision-making is **Slice 9**; here the external market and contract counterparties are a
parametric, seeded world layer, and all market/contract mechanisms are faction-agnostic.

## Clarifications

### Session 2026-06-14

- Q: What is the scope and scale of the logistics transport graph's node set — which dynamical locations does the economy instantiate as graph nodes? → A: A curated set of staging nodes + active sites (Earth surface, LEO/GTO/GEO, EML-1/2, LLO, lunar surface sites, key NEAs, Mars-system nodes, plus Sites a faction is actively operating at); the ~3,000-body catalogue stays a prospecting-statistics layer, not graph nodes. Tens-to-low-hundreds of nodes typical.
- Q: How is launch-to-orbit modeled versus in-space transfers, and what role does the mass-to-orbit currency play? → A: Surface→orbit edges consume the mass-to-orbit capacity currency (sized by owned launchers + launch-market capacity bought in Funds at $/kg by orbit class); in-space edges consume propellant/Δv on assigned vehicles. Mass-to-orbit is a managed launch-capacity budget supporting both self-launch and buy.
- Q: Does FA-06 own its own tradable-commodity taxonomy, or strictly reuse the FA-03 world resource taxonomy? → A: FA-06 owns a tradable-commodity taxonomy that references FA-03's in-situ resource types as the raw-material base and extends it with economic goods the world model doesn't define (processed propellants/grades, manufactured products, consumables, spares) and services (launch, data/IP licences). FA-03 stays the geology source of truth.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The six-currency ledger & location-addressed resources (Priority: P1)

A faction holds explicit balances of the six currencies — Funds, Δv/propellant, mass-to-orbit
capacity, crew-time, ops capacity, and political/reputation capital — and holds physical resources
as stocks that *sit at a place*. A tonne of water in LEO, at EML-1, on the lunar surface, and on
Phobos are four different goods. Every balance change is a recorded transaction; nothing is created
or destroyed except by a modelled process.

**Why this priority**: this is the spine every other economic mechanic debits and credits. Without
a conserved, location-addressed, auditable ledger there is no economy. It is independently
demonstrable on its own.

**Independent Test**: load a sourced starting state; submit a script of credit/debit transactions;
assert balances and per-location stocks are correct, that conservation holds (no negative stock, no
un-caused creation), that the same commodity at two locations is two distinct goods, and that the
transaction history reproduces the sequence. Double-run ⇒ bit-identical.

**Acceptance Scenarios**:

1. **Given** a faction with sourced opening balances, **When** a transaction debits Funds and
   credits a propellant stock at LEO, **Then** both the currency balance and the LEO stock reflect
   the change and a transaction record is appended with its cause.
2. **Given** 1 t of water at EML-1 and 1 t of water on the lunar surface, **When** the player
   queries inventory, **Then** the two are reported as separate location-addressed goods with
   independent quantities.
3. **Given** a stock of 100 units, **When** a transaction attempts to debit 150, **Then** the
   system rejects it as a constraint violation and the stock is unchanged.
4. **Given** any decision script, **When** it completes, **Then** total accounted inflow/outflow
   balances and no stock was created without a modelled process.

---

### User Story 2 - Logistics network priced in delta-v (Priority: P1)

The economy runs on a directed transport graph whose nodes are the world's dynamical locations and
whose edges are transfers with a Δv cost, time-of-flight and launch-window availability. Moving
goods means assigning a vehicle to an edge, paying propellant and time, and respecting windows.
Depots buffer cargo/propellant; reusable tugs and cyclers amortise over many trips.

**Why this priority**: this is the central felt mechanic — resources cost delta-v and time to
reach, and the map *is* a delta-v network. It is the reason the ledger's location-addressing
matters.

**Independent Test**: build a small graph from sourced locations; order a shipment of cargo from
node A to node B; assert it consumes the correct propellant/Δv (priced from the astrodynamics
planner), waits for an open window if none is current, delivers after the time-of-flight, and
updates both endpoints' stocks; assert a reusable tug returns and can be reassigned.

**Acceptance Scenarios**:

1. **Given** a tug with finite propellant at LEO and a window to EML-1, **When** a shipment is
   ordered, **Then** the system debits the edge's propellant/Δv, moves cargo, and credits EML-1
   after the time-of-flight.
2. **Given** an edge whose next window is in the future, **When** a shipment is ordered, **Then**
   it waits for the next window rather than departing instantly.
3. **Given** a tug with insufficient Δv for the edge, **When** a shipment is ordered, **Then** the
   system flags the shortfall and does not silently complete the move.
4. **Given** a depot at EML-1, **When** propellant is delivered to it and later drawn by another
   vehicle, **Then** the depot buffers the stock and the tanker architecture closes.

---

### User Story 3 - Funding models: appropriations and cash-runway (Priority: P1)

Agencies live on periodic appropriations (annual/multi-year budgets, directed funds, carry-over,
fiscal cliffs) and cannot go bankrupt but can be *gutted*. Private companies live on a cash runway
(burn, revenue, financing) and go *bankrupt* if cash runs out. The two models express factional
asymmetry without changing physics.

**Why this priority**: funding is what makes the Funds currency meaningful and drives the
agency-vs-private asymmetry that is core to the game's identity. It is independently demonstrable.

**Independent Test**: run an agency faction across a fiscal period and assert appropriation,
directed funds and carry-over apply per data; collapse its budget and assert the gutted soft-fail
state. Run a private faction with burn > revenue until cash is exhausted and assert bankruptcy —
both deterministically from identical inputs.

**Acceptance Scenarios**:

1. **Given** an agency with a sourced baseline and a directed-funds line, **When** the fiscal
   period advances, **Then** the appropriation (incl. earmark and carry-over rule) is applied.
2. **Given** a private faction with monthly burn exceeding revenue, **When** cash reaches zero,
   **Then** the faction enters the bankruptcy (game-over) state.
3. **Given** an agency whose appropriation collapses below caretaker level, **When** the period
   advances, **Then** the faction enters the gutted soft-fail state (not bankruptcy).
4. **Given** two factions with different funding profiles and identical physics, **When** the same
   script runs, **Then** their financial outcomes differ only by their data-defined funding
   parameters.

---

### User Story 4 - Cost model: P50/P80 uncertainty & learning curves (Priority: P2)

Costs are estimated with explicit P50/P80 uncertainty and realised with overruns; unit cost falls
with cumulative production along a learning curve (Wright's law). Reusable, high-cadence hardware
gets cheap; bespoke one-offs stay expensive. Every cost is traceable to its physical/maturity
basis.

**Why this priority**: the cost model turns the vehicle designer's physical mass/maturity into
money honestly and makes reuse and standardisation pay — but the economy can demonstrate motion and
funding (P1) without it.

**Independent Test**: estimate an item's cost from a sourced mass/maturity basis; assert P50 < P80;
draw realised cost on a seeded stream and assert overruns are possible and reproducible; produce N
identical units and assert realised unit cost falls monotonically along the sourced exponent.

**Acceptance Scenarios**:

1. **Given** an item with a sourced mass and maturity, **When** its cost is estimated, **Then** the
   estimate yields a P50 and a P80 with P50 < P80, traceable to mass and learning state.
2. **Given** a seeded run, **When** realised cost is drawn, **Then** it can exceed P50 and the same
   seed reproduces the same realisation.
3. **Given** cumulative production rising from 1 to N units, **When** unit cost is computed, **Then**
   it falls monotonically along the data-defined learning curve.

---

### User Story 5 - ISRU break-even economics & scale-up (Priority: P2)

An ISRU plant converts a local in-situ resource into a useful commodity (lunar ice → LOX/LH₂; Mars
CO₂+water → CH₄/O₂; regolith → O₂/metals/feedstock; asteroid volatiles → water/propellant) at a
yield governed by sourced process parameters, surveyed grade, power and plant mass. It pays off
only when launch-cost-saved exceeds the plant's delivered+build+operate+amortised cost; early
first-of-a-kind plants are unreliable and barely profitable.

**Why this priority**: ISRU is the heart of the off-Earth economy and the loop toward
self-sufficiency, but it builds on the ledger, logistics and cost model (P1/early-P2).

**Independent Test**: site a sourced lunar-ice plant; assert its propellant output feeds the local
stock; compute break-even and assert the plant is net-negative below it and net-positive only above
it; assert a pilot plant ramps (lower yield/reliability) before reaching production.

**Acceptance Scenarios**:

1. **Given** a surveyed ice grade, power and plant mass, **When** the plant runs, **Then** it
   produces propellant at the sourced yield and credits the local stock.
2. **Given** a plant below its break-even scale, **When** its lifetime economics are evaluated,
   **Then** the net result is negative — ISRU is not free fuel.
3. **Given** a first-of-a-kind plant, **When** it starts up, **Then** it exhibits a reduced yield
   and reliability ramp before reaching nameplate (scale-up dynamics).

---

### User Story 6 - Markets, contracts, partnerships & IP (Priority: P2)

A living external economy: a launch market whose $/kg by orbit class moves with world capacity
(buy when cheaper than self-launch, sell spare capacity); CLPS/COTS-style service contracts
(post RFP → bid → award → fulfil/penalty); partnerships/consortia with per-faction trust state
(barter, geo-return, shared TRL/IP/seats/data); and a data/IP-licensing, tourism and in-space
manufacturing revenue layer with realistic price ceilings.

**Why this priority**: markets make the faction not alone and open revenue paths, but the core
economy functions without them; they layer on top of the ledger and funding.

**Independent Test**: post an RFP, have it bid and awarded, fulfil it (assert revenue + heritage)
and fail it (assert penalty); change world launch capacity and assert the $/kg responds; form a
partnership and assert trust state updates and a betrayal carries lasting reputation cost.

**Acceptance Scenarios**:

1. **Given** a posted RFP to deliver X to a lunar site, **When** a faction bids and wins and
   delivers, **Then** it earns the contract revenue and flight heritage; **on failure** it incurs
   the penalty.
2. **Given** a rise in world reusable-launch capacity, **When** the market tick runs, **Then** the
   launch $/kg falls.
3. **Given** a consortium, **When** a partner reneges, **Then** its trust/reputation state degrades
   with lasting effect.
4. **Given** matured technology or science data, **When** it is licensed/sold, **Then** the faction
   earns royalties/revenue within the data-defined market size.

---

### User Story 7 - Capital facilities & ground segment (Priority: P3)

Owned/leased fixed assets — R&D labs/test stands, manufacturing/integration lines, launch
pads/ranges, mission control, antenna networks/DSN, relays — with capex, opex, capacity and upgrade
paths. Facility capacity gates the rate of the activity it enables; ops capacity and DSN/relay
passes are the finite pool that active craft consume, expandable via investment, and range access
is a scheduling (and political) constraint.

**Why this priority**: facilities size the ops/comms pool and production throughput that the rest of
the economy assumes, but a thin default pool lets P1/P2 stand alone; the facility model is the
refinement.

**Independent Test**: stand up a sourced ground-segment facility and assert it sizes the ops/comms
pool; assign more active craft than the pool supports and assert measurable degradation (reduced
data return / raised anomaly risk); upgrade the facility and assert the pool grows.

**Acceptance Scenarios**:

1. **Given** a mission-control + DSN facility, **When** it is commissioned, **Then** it sizes the
   faction's ops-capacity and comms pool with its sourced capacity.
2. **Given** more active craft than the ops pool supports, **When** the tick runs, **Then**
   outcomes degrade (data return down, anomaly risk up) rather than being silently free.
3. **Given** a production line, **When** build throughput is requested above its capacity, **Then**
   the excess queues; upgrading the line raises throughput.

---

### Edge Cases

- **Negative/over-draw**: a debit that would drive a currency or stock negative is rejected or
  flagged as a constraint violation; the economy never goes silently into impossible state.
- **No open window**: a shipment ordered with no current transfer window waits for the next window;
  it never teleports.
- **Insufficient Δv/payload**: a vehicle assigned an edge it cannot afford (Δv or capacity) flags a
  shortfall instead of completing.
- **Bankruptcy mid-flight**: a private faction going bankrupt with craft in transit resolves to its
  soft-fail state deterministically; in-flight commitments are handled, not lost silently.
- **ISRU below break-even / dry resource**: a plant on an exhausted or mis-surveyed deposit produces
  below estimate (belief-state uncertainty), and its economics can close negative.
- **Strategic-material starvation**: a mission needing Pu-238/enriched fuel beyond the capped supply
  is gated until supply or an alternative technology matures.
- **Ops-pool oversubscription**: beyond the pool, additional craft degrade outcomes rather than
  being free.
- **Economic data version mismatch on load**: a save referencing changed economic constants is
  detected and reported, not silently loaded.
- **Market collapse shock**: a rival breakthrough or a launch-failure shock that collapses a market
  the faction depended on propagates as a modelled external shock.

## Requirements *(mandatory)*

### Functional Requirements

#### Ledger & resources (US1)

- **FR-EC-101**: System MUST maintain, per faction, explicit balances of the six currencies — Funds,
  Δv/propellant, mass-to-orbit capacity, crew-time, ops capacity, and political/reputation capital.
- **FR-EC-102**: System MUST track physical resources as stocks **addressed by location** (a world
  dynamical node or site), so the same commodity at two locations is two distinct, separately-valued
  goods.
- **FR-EC-103**: Every change to a balance or stock MUST occur through a recorded **transaction**
  (credit/debit) carrying its cause, so the economic history is auditable and replayable.
- **FR-EC-104**: Resources MUST be **conserved** — no stock may be created or destroyed except by an
  explicitly modelled process (launch, transfer, ISRU conversion, consumption, production, trade);
  the system MUST reject (or flag) any transaction that would drive a stock or currency negative.
- **FR-EC-105**: System MUST expose queries for a faction's currency balances, the stock of a
  commodity at a location, aggregate inventory, and the transaction history.
- **FR-EC-106**: Strategic materials (Pu-238/Am-241 RTG fuel, enriched/LEU nuclear fuel,
  rare-earth/electronics-grade) MUST be representable as **capped, separately-tracked scarce
  stocks** subject to supply constraints and policy gating.
- **FR-EC-107**: FA-06 MUST own a **tradable-commodity taxonomy** that **references** the world
  model's in-situ resource taxonomy (FA-03) as its raw-material base and **extends** it with economic
  goods the world model does not define — processed propellants and grades, manufactured products
  (e.g. ZBLAN, protein crystals), consumables, spares — and **services** (launch, data/IP licences).
  Raw-material commodities MUST reference FA-03 resource ids (no duplication/drift); FA-03 remains
  the geology/ground-truth source of truth.

#### Logistics network (US2)

- **FR-EC-201**: System MUST model a **directed transport graph** whose nodes are a **curated set of
  the world model's dynamical locations** — staging nodes (Earth surface, LEO/GTO/GEO, EML-1/2, LLO,
  lunar surface sites, key NEAs, Mars-system nodes) plus any Site a faction is actively operating at
  — and whose edges are transfers carrying a Δv cost, time-of-flight and launch-window availability
  **derived from the astrodynamics layer**. The ~3,000-body catalogue remains a
  prospecting-statistics layer (FA-03), not graph nodes; nodes are added as factions begin operating
  at new Sites.
- **FR-EC-202**: Moving a commodity MUST require assigning a **vehicle** (finite propellant/Δv and
  payload capacity) to an edge, debiting the appropriate budget (**in-space transfers** debit
  propellant/Δv; **launch edges** debit mass-to-orbit/Funds per FR-EC-202a) and crew-time/ops as
  appropriate, and delivering cargo after the time-of-flight, updating both endpoints' stocks.
- **FR-EC-202a**: **Surface→orbit (launch) edges** MUST be modelled distinctly from in-space
  transfers: they consume the **mass-to-orbit capacity** currency rather than a vehicle's Δv. That
  capacity is sized by owned launch vehicles **plus** capacity bought on the launch market (Funds,
  $/kg by orbit class — FR-EC-601). In-space transfer edges consume **propellant/Δv** on the
  assigned vehicle. A faction MUST be able to either self-launch (consuming owned capacity) or buy
  launch (consuming Funds).
- **FR-EC-203**: Transfers MUST respect **window availability** — a transfer with no open window
  MUST wait for the next window or be rejected, never depart regardless of phasing.
- **FR-EC-204**: System MUST support **depots** as buffer nodes storing propellant/cargo to decouple
  production from transport (enabling tanker/refuel architectures).
- **FR-EC-205**: System MUST support **reusable tugs and cyclers** whose cost amortises over repeated
  trips (a tug returns and is reassigned; a cycler follows a fixed recurring path).
- **FR-EC-206**: Ops capacity and comms bandwidth MUST be **finite shared pools** consumed by active
  craft; oversubscription MUST degrade outcomes (reduced data return, raised anomaly risk), and
  **light-time delay** MUST be represented for control/data.

#### Funding models (US3)

- **FR-EC-301**: System MUST support an **agency appropriation** model: a periodic (annual/
  multi-year) budget from an external political input, with directed (earmarked) funds, carry-over
  rules, and fiscal-cliff events (continuing resolution / re-baselining).
- **FR-EC-302**: System MUST support a **private revenue** model: cash balance, burn, revenue, and
  financing (equity/debt/owner injection); cash exhaustion MUST trigger a **bankruptcy** (game-over)
  state.
- **FR-EC-303**: Agencies MUST NOT go bankrupt but MUST be able to be **gutted** (budget collapse to
  caretaker level) as a soft-fail state.
- **FR-EC-304**: Funding parameters (baselines, volatility, carry-over, geo-return surcharge, import
  restrictions) MUST be **faction-configurable from data**, expressing asymmetry without changing
  physics.
- **FR-EC-305**: External political inputs (mood, prestige modifiers, directed funds, sanctions)
  MUST enter through a **defined interface as opaque inputs**; the politics system is a later slice.

#### Cost model (US4)

- **FR-EC-401**: System MUST estimate the cost and schedule of an item/program with explicit
  **P50/P80** uncertainty bands, derived from sourced parameters and the item's mass/maturity
  (consuming the vehicle cost basis and research maturity).
- **FR-EC-402**: Realised cost MUST be drawn from the uncertainty model on a **seeded stream** so a
  given seed reproduces the same outcome; **overruns** (realised > P50) MUST be possible.
- **FR-EC-403**: Unit cost MUST fall with cumulative production along a **learning curve**
  (Wright's-law exponent from data); reuse/high-cadence MUST get cheaper, bespoke one-offs stay
  expensive.
- **FR-EC-404**: Every cost figure MUST be **traceable** to its physical/maturity basis (mass,
  learning state, maturity) so the player can see the mass/Δv behind any dollar (Principle VII).

#### ISRU economics (US5)

- **FR-EC-501**: System MUST model **ISRU plants** converting a local resource to a useful commodity
  (lunar ice → LOX/LH₂; Mars Sabatier CH₄/O₂; regolith → O₂/metals/feedstock; asteroid volatiles →
  water/propellant) at a yield governed by sourced process parameters, **surveyed grade** (world
  belief-state), available power, and plant mass.
- **FR-EC-502**: System MUST compute **break-even**: net = (launch cost saved at the destination
  node) − (plant delivery mass + build + operate + amortise); ISRU MUST pay off only when the
  physics and economics close — **never free fuel**.
- **FR-EC-503**: ISRU plants MUST exhibit **scale-up dynamics** — a pilot→production ramp with its
  own learning and reliability ramp, so first-of-a-kind plants are unreliable and barely profitable.
- **FR-EC-504**: ISRU output MUST **feed back** into propellant supply at its location (resupplying
  logistics) and into construction feedstock, closing the loop toward base self-sufficiency.
- **FR-EC-505**: ISRU yield uncertainty (grade/accessibility) MUST be modelled on **seeded streams**
  consistent with the world model's surveyed belief-state.

#### Markets & contracts (US6)

- **FR-EC-601**: System MUST model a **launch market** setting $/kg by orbit class from world
  supply/demand, letting a faction buy launch when cheaper than self-launch and sell spare capacity
  for revenue, with prices moving as world launch capacity changes.
- **FR-EC-602**: System MUST model **service contracts**: an issuer posts an RFP; factions bid; the
  winner earns revenue + heritage on success and incurs penalties on failure.
- **FR-EC-603**: System MUST model **partnerships/consortia**: co-funding and shared
  TRL/IP/crew-seats/data with per-faction **trust/relationship state** that betrayal degrades at
  lasting reputation cost (barter, geo-return).
- **FR-EC-604**: System MUST model **data/IP licensing** (sell data, license matured tech for
  royalties) and **novel revenue** (tourism tiers, in-space manufacturing) with realistic price
  ceilings/market sizes from data.
- **FR-EC-605**: Market and contract mechanisms MUST be **faction-agnostic** (usable by player and,
  in a later slice, AI factions) and **seed-driven** where stochastic.

#### Facilities & ground segment (US7)

- **FR-EC-701**: System MUST model **capital facilities** (R&D labs/test stands, manufacturing/
  integration lines, launch pads/ranges, mission control, antenna networks/DSN, relays) as
  owned/leased assets with capex, opex, capacity and upgrade paths.
- **FR-EC-702**: Facility capacity MUST **gate the rate** of the activity it enables (production →
  build throughput/learning capacity; pads/range → launch cadence; ground segment → ops/comms).
- **FR-EC-703**: Ops capacity and DSN/relay passes MUST be the **finite pool** of FR-EC-206, sized
  by ground-segment facilities and expandable via investment; range access MUST be a scheduling (and
  political) constraint.

#### Cross-cutting

- **FR-EC-801**: All economic constants (launch $/kg by class, ISRU yields & plant mass, budget
  baselines, learning exponents, strategic-material supply, market sizes, facility capex/opex) MUST
  live in versioned, schema-validated data files, each carrying a non-empty `source`; CI MUST reject
  missing sources.
- **FR-EC-802**: The economy MUST be **deterministic** — identical seed + identical decisions ⇒
  identical economic state and identical transaction/event log; all stochastic outcomes (overruns,
  ISRU/market draws, anomalies) MUST derive from seeded streams; no wall-clock or unseeded
  randomness.
- **FR-EC-803**: Economy state MUST **round-trip** through save/load with bit-identical reload, and
  MUST pin the economic **data version** so a save cannot silently load against changed constants.
- **FR-EC-804**: The economy MUST integrate via the established **module boundary** (owning its
  slice; reading world locations/resources/belief-state, astro windows/Δv, vehicle cost, research
  maturity through their defined interfaces) without embedding those systems' logic or reaching
  hidden ground truth.
- **FR-EC-805**: Money MUST remain **secondary to physics** — every cost MUST be traceable to mass &
  Δv, and ISRU/reuse/depots/refuelling MUST be mechanically meaningful by relaxing the mass/Δv
  constraint, not via arbitrary bonuses (Principle VII).
- **FR-EC-806**: The slice MUST contain **no combat/weapons/sabotage** economy and no alien actors
  (Principle IX); competition is the milestone race, economics and politics.
- **FR-EC-807**: A **market/price tick** MAY run at a slower cadence than the core resource-flow
  timestep but MUST remain deterministic and reconcilable.
- **FR-EC-808**: The slice MUST expose a generic **project / resource-delivery accounting primitive**
  (delivered-mass + crew-time + time → completion) that Slice 7 (Bases & Construction) consumes;
  base/module construction logic itself is out of scope here.

### Key Entities

- **Account / Ledger**: a faction's balances across the six currencies and the authority for all
  credit/debit.
- **Currency**: the six (Funds, Δv/propellant, mass-to-orbit, crew-time, ops capacity, political
  capital).
- **Commodity / ResourceType**: a tradable good in FA-06's commodity taxonomy — raw materials
  referencing FA-03 resource ids, plus processed/manufactured goods, consumables, spares, and
  services (launch, data/IP licences).
- **ResourceStock**: a quantity of a commodity **at a location** — the location-addressed good.
- **Transaction**: a recorded credit/debit with cause; the audit + replay record.
- **TransportGraph / Node / Edge**: nodes = world dynamical locations; edges = window-constrained
  transfers (Δv, TOF, window).
- **Shipment / TransferOrder**: a vehicle assigned to an edge moving cargo over the time-of-flight.
- **Depot**: a buffer node storing propellant/cargo.
- **Tug / Cycler**: a reusable transport asset whose cost amortises over trips.
- **FundingProfile**: per-faction appropriation or revenue parameters (asymmetry knobs).
- **Budget / Appropriation / CashRunway**: funding state for agencies / private companies.
- **CostEstimate (P50/P80) / LearningState**: the cost model's estimate bands and cumulative-units
  state.
- **ISRUPlant**: a converter with yield, power demand, mass, scale-up and reliability ramp.
- **Market / Contract (RFP/Bid) / Partnership / License**: the external-economy mechanisms.
- **Facility**: a capital asset (capex/opex/capacity/upgrade), including ground-segment.
- **OpsCapacityPool / CommsPool**: finite shared pools consumed by active craft under light-time.
- **Project**: the generic resource-delivery/assembly accounting primitive Slice 7 consumes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From a sourced starting state, a commodity can be moved from one location to another;
  the system debits the correct propellant/Δv and time and credits the destination stock, with the
  cost traceable to mass × Δv.
- **SC-002**: A double-run of the same seed + decision script produces **bit-identical** economic
  state and an identical transaction/event log.
- **SC-003**: Resource **conservation** holds across an arbitrary decision script — accounted
  inflow/outflow balances; no stock goes negative or is created without a modelled process.
- **SC-004**: A sourced lunar-ice and a Mars-Sabatier ISRU case each return **net-negative below
  break-even and net-positive only above it** — ISRU is never free fuel.
- **SC-005**: Producing N identical units lowers realised unit cost **monotonically** along the
  sourced learning curve; a cost estimate yields **P50 < P80** and realised cost can exceed P50.
- **SC-006**: A private faction whose burn exceeds revenue until cash is exhausted enters
  **bankruptcy**; an agency whose appropriation collapses enters the **gutted** soft-fail state —
  both deterministically from identical inputs.
- **SC-007**: A posted RFP can be bid, awarded, fulfilled (revenue + heritage) or failed (penalty);
  the launch-market $/kg responds to a change in world launch capacity.
- **SC-008**: Oversubscribing the finite ops/comms pool measurably **degrades** outcomes (reduced
  data return / raised anomaly risk), and expanding ground-segment facilities relieves it.
- **SC-009**: Every economic data file passes **schema + source-presence** validation in CI; no
  plausibility-bearing constant lacks a `source`.
- **SC-010**: Economy state **round-trips** through save/load with identical reload, and a version
  mismatch on economic data is detected on load.
- **SC-011**: The economy + logistics graph sustains the target scale (a curated graph of
  tens-to-low-hundreds of nodes, thousands of location-addressed stocks, and large fleets) at high
  time-warp within the core's tick-time budget.

## Assumptions

- **Scope — base construction deferred** *(confirmed 2026-06-14)*: assembling bases/stations from
  modules is Slice 7; this slice provides the economic substrate plus a generic
  project/resource-delivery accounting primitive (FR-EC-808) that Slice 7 consumes. Capital
  facilities (econ §7) are in scope as economic assets.
- **Scope — AI agency deferred** *(confirmed 2026-06-14)*: the external launch market and contract
  counterparties are a parametric + seeded world layer here; per-faction AI economic
  decision-making is Slice 9, which drives these same faction-agnostic mechanisms.
- **Logistics ↔ astro fidelity** *(confirmed 2026-06-14)*: logistics edges are priced by the
  astrodynamics **analytic planner** (Δv/TOF/window); cargo shipments execute as deterministic
  timed transfers debiting propellant/Δv/ops, reconciled against the numerical propagator only for
  operationally-significant flights.
- Political/mood/prestige values, surveyed resource grades, vehicle cost bases, and research maturity
  arrive through the **already-defined interfaces** of the politics(later)/world/vehicle/research
  slices; the economy consumes them as opaque inputs and never reads hidden ground truth.
- **SI units** throughout; currencies tracked in SI-consistent quantities (kg, m/s, seconds/
  crew-hours), with Funds in a faction currency abstracted to a common unit.
- Tourism and in-space-manufacturing markets use **data-defined price ceilings and finite market
  sizes** — no infinite revenue.
- The **per-game seed** fixes overrun draws, ISRU grade/accessibility uncertainty, market
  fluctuations and contract generation.
- A **slower market/price tick** layered on the deterministic core timestep is acceptable provided
  it stays deterministic and reconcilable (FR-EC-807).

## Dependencies

- **FA-01 (sim-core)**: module/slice contract, command/event routing, seeded streams, save/load,
  interrupt-and-pause for budget votes / contract awards / anomalies.
- **FA-02 (astrodynamics)**: transfer windows, Δv, time-of-flight and the planner that prices
  logistics edges.
- **FA-03 (world)**: dynamical locations as graph nodes, the resource taxonomy, Sites and the
  per-faction surveyed belief-state (resource grades) for ISRU.
- **FA-04 (vehicle)**: cost basis, payload/propellant capacity and learning for vehicles, tugs and
  ISRU/facility hardware.
- **FA-05 (research)**: maturity/heritage/understanding gating ISRU and facility tech, and the
  matured-technology basis for IP licensing.
