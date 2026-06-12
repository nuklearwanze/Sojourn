# SOJOURN — Part 3: Economy & Logistics Simulation

> Design intent: a closed-ish, plausible space economy where the dominant cost is always
> **mass × delta-v to where you need it**. Money buys mass-to-orbit; everything else is the
> struggle to do more with less mass once you're up the gravity well. ISRU's entire point is to
> stop paying Earth's gravity tax. All prices/figures are design-intent defaults in `data/econ/`,
> each with a source tag (current launch prices, ISRU study yields, NASA/ESA budgets, etc.).

---

## 1. Currencies (six, per the core tension)

| Currency | What it is | How gained | How spent |
|---|---|---|---|
| **Funds** ($/€/₽…) | Money | Appropriations / revenue / contracts / sales / financing | Everything purchasable |
| **Delta-v / Propellant** | The physical budget of motion | Tanks + propellant (bought or ISRU) | Every manoeuvre |
| **Mass-to-orbit** | Launch capacity & cadence | Owned/contracted launch | Deploying anything |
| **Crew-time** | Trained astronaut-hours | Crew pipeline | Crewed ops, EVA, science |
| **Ops capacity** | Controllers + DSN/relay passes | Facilities/personnel | Operating active craft |
| **Political/Reputation capital** | Goodwill, prestige | Successes, publishing, partnerships | Budget fights, approvals, partnerships |

The art of the game is converting between them at good exchange rates (e.g., spend Funds +
science to build lunar ISRU → convert Mass-to-orbit you'd have spent on propellant into payload).

## 2. Budgeting

### 2.1 Agencies (appropriation model)
- **Annual/multi-year budget** set by a political process (05-WORLD.md §5): baseline ± modifiers
  from prestige, public/political mood, economic cycle, recent successes/failures, election years.
- **Directed funds**: some budget is earmarked by politics to specific programs (you didn't ask
  for that heavy-lift line item, but Congress did; cancelling costs political capital).
- **Carry-over rules** differ (NASA: use-it-or-lose-it pressure; ESA: smoother multi-year).
- **Fiscal-year cliff** events: continuing resolutions, shutdown risk, re-baselining.

### 2.2 Private companies (revenue model)
- **Cash runway** is sacred: model cash, burn, revenue, and **financing** (equity rounds,
  debt, owner injections for Meridian). Running out of cash = bankruptcy = game over.
- **Revenue streams**: launch contracts, delivery $/kg, data/IP licensing, tourism, in-space
  product sales (ZBLAN fiber, pharma crystals, later propellant/metals), station leasing.
- **Valuation & raises**: hit milestones to raise at better terms; failures dilute or close the
  window. A *Prospector*-style company lives or dies by reaching first off-Earth revenue before
  runway ends.

### 2.3 Cost model (shared)
Costs are **estimated with uncertainty** (P50/P80) and *realised* with overruns (01-RESEARCH.md
§4.2). Learning curves: unit cost drops with production count (Wright's law) — reusable, high-
cadence hardware gets cheap; bespoke one-offs stay expensive. This is why reuse and standardised
buses dominate good economies.

## 3. Resources

Two classes: **Earth-supplied** (bought, must be launched) and **In-situ** (mined off-Earth).

### 3.1 Bulk commodities
Propellants (LH₂, LOX, CH₄, kerosene, storables, xenon/krypton), water, structural metals,
regolith/aggregate, silicon, polymers/feedstock, consumables (food, O₂, N₂ buffer), spares.

### 3.2 Constrained strategic materials
- **Pu-238 / Am-241** (RTG fuel): globally scarce, slow production, capped supply you compete for
  — gates deep-space/night missions until surface fission matures. (Real bottleneck.)
- **Highly-enriched vs LEU** nuclear fuel: policy-gated; LEU paths cost performance, HEU paths
  cost political capital and proliferation reputation.
- **Helium-3 / fusion fuels**: only relevant in fusion-breakthrough endgames; modelled as
  presently uneconomic (no fake He-3 gold rush).
- **Rare-earths / electronics-grade materials**: feed advanced avionics; import-restricted for
  some factions (Roscosmos avionics constraint).

### 3.3 Location matters
Every resource unit has a **location** and a **delta-v address**. A tonne of water in LEO, at
EML-1, on the lunar surface, and on Phobos are four different goods with wildly different values.
The economy is fundamentally a **logistics network priced in delta-v** (§6).

## 4. ISRU economics (the heart of the off-Earth economy)

ISRU is justified only when *(launch cost saved) > (cost to build + operate + amortise the
plant) + (mass of the plant delivered)*. The game makes the player feel this break-even.

- **Lunar polar water → propellant**: extract ice, electrolyse to LOX/LH₂, sell/use at EML-1 or
  LLO. Break-even depends on plant mass, ice grade (surveyed!), power (lunar night problem),
  and the price of Earth-launched propellant at that node. Source: lunar-ice ISRU & cislunar
  propellant-market studies.
- **Mars propellant (Sabatier CH₄+O₂ from CO₂ + water)**: makes crewed Mars **return** feasible
  without launching ascent propellant from Earth — often the single decision that makes a Mars
  campaign close. Source: MOXIE + Mars ISRU architectures.
- **Regolith O₂ / metals / construction feedstock**: turns mass-to-orbit into local mass; gates
  large surface bases (you cannot launch a base's worth of shielding — you sinter it, D6).
- **Asteroid/comet volatiles**: water for propellant/life-support at high-value deep-space nodes;
  C-type NEAs as "gas stations." Modelled with realistic uncertainty on grade and accessibility.
- **Scale-up dynamics**: pilot → production has its own learning curve and reliability ramp;
  early plants are unreliable and barely profitable — exactly like real first-of-a-kind plants.

> ISRU output feeds back into propellant supply (B6 tie-in) and construction (D6), closing the
> loop that lets a base approach self-sufficiency (the *Homestead* goal).

## 5. Markets, contracts, partnerships & IP

A living external economy so the player isn't alone.

- **Launch market**: global supply/demand sets $/kg by orbit class; your reusable fleet can
  *sell* launch to others (revenue) or you can *buy* when cheaper than self-launch. Prices move
  with world capacity (if everyone fields reuse, $/kg collapses — and ISRU's relative value
  rises).
- **Service contracts** (the CLPS/COTS/commercial-station model): agencies post **RFPs**
  (deliver X kg to lunar surface; host this payload; provide a crew taxi). Companies **bid**;
  winning brings revenue + heritage but penalties for failure. Agencies can run programs in-house
  *or* buy services — a core strategic axis (NASA's whole modern model).
- **Partnerships / consortia**: co-fund programs, share TRL credit, IP, crew seats, and data
  (CSA's barter playstyle; ESA geo-return; ISS-style multilateral stations). Trust/relationship
  state per faction; betrayal has lasting reputation cost.
- **Data & IP market**: sell science data, license matured tech (01-RESEARCH.md §5), or hold for
  competitive lead. Patents earn royalties but slow the global tide in your favour.
- **Tourism & novel revenue**: suborbital → orbital → lunar flyby tourism; in-space manufacturing
  products (ZBLAN, protein crystals, bioprinting) with real-ish niche markets and price ceilings.

## 6. Logistics network (delta-v as the map)

The economy runs on a **directed transport graph** whose nodes are dynamical locations (Earth
surface, LEO, GTO, EML-1/2, LLO, lunar surface sites, NEAs, Mars system, etc.) and whose edges
are **transfers** with a delta-v cost, time-of-flight, and **launch-window** availability.

- Moving goods = assigning vehicles to edges, paying propellant + time, respecting windows.
- **Depots** (B6.3) are buffer nodes that decouple production from transport (store propellant/
  cargo, enable tanker architectures, smooth windows).
- **Reusable tugs/cyclers** amortise over many trips; **cyclers** (Aldrin-type) trade a fixed
  recurring path for cheap repeated Earth–Mars transit of crew/cargo.
- **Ops capacity & comms**: every active craft consumes controller attention + DSN/relay
  bandwidth (a shared, finite pool you expand via E6 and ground-segment facilities). Over-
  subscription degrades data return and raises anomaly risk — a real constraint on fleet size.

## 7. Facilities & ground segment (capital infrastructure)

Owned/leased fixed assets that generate or enable capability (tie to 01-RESEARCH.md §3 and ops):

- **R&D**: labs, test stands (engine/thermal-vac/structural), wind tunnels, centrifuge,
  radiation source, neutral-buoyancy/analog sites, clean rooms, fab.
- **Manufacturing**: integration & assembly buildings, production lines (learning-curve
  capacity), propellant production (Earth-side).
- **Launch & recovery**: pads, mobile launchers, droneships/landing zones, range access (a
  scheduling + political resource).
- **Ground segment**: mission control centres, antenna networks/DSN time, relay constellations.
- **Space-side infrastructure** (built via missions): LEO/cislunar stations, depots, lunar/Mars
  surface bases, ISRU plants, surface power, comms relays, construction yards.

Facilities have capex, opex, capacity, and upgrade paths; they are the board you build on.

## 8. Construction & infrastructure projects

Big builds are **projects** (like programs): site → modules → delivery sequence → assembly →
commissioning. Surface/orbital construction needs: delivered or ISRU-made materials, power,
construction robotics/crew-time, and time. Bases grow by adding modules (I11): power, ECLSS,
ISRU, greenhouse, workshop, medical, storage, comms, radiation shelter. A base has emergent
properties (population capacity, closure %, power margin, sustainability index) that determine
whether it's a beachhead, an outpost, or a settlement.

## 9. Economic feedback loops & failure modes

- **Virtuous**: reuse → cheap launch → depots/ISRU viable → cheaper deep-space → more missions →
  more science/heritage → cheaper everything.
- **Vicious**: overrun → budget cut → program slip → prestige loss → bigger cut (agency gutting);
  or burn > revenue → bad raise → death spiral (startup bankruptcy).
- **External shocks**: launch failure (yours or market), funding crisis, a rival's breakthrough
  collapsing a market you depended on, a loss-of-crew accident freezing crewed revenue/politics.

## 10. Implementer notes

- Economy core is a **resource-flow + transport-graph simulation** on the deterministic timestep;
  prices update on a slower market tick.
- All economic constants (launch $/kg by class, ISRU yields & plant mass, budget baselines,
  learning-curve exponents, RTG fuel supply) live in `data/econ/*.json` with sources; balancing
  is data-driven so it can be tuned against real figures without touching the sim.
- Keep money *secondary* to physics: the UI should always let the player see the **mass &
  delta-v** behind any dollar cost, because that's the real constraint the dollars are proxying.
