# Sojourn

> **A hard-science, no-combat 4X strategy game about exploring, industrialising and settling the Solar System, 2026–2126.**

Sojourn is a single-player, pausable real-time grand strategy game in which you run one of ten space organisations starting in **January 2026** — with exactly the hardware, budgets and scientific knowledge humanity actually has at that date. There is no combat, no aliens, no techno-magic. The drama comes from physics, money, politics, engineering risk and the slow, hard-won expansion of human and robotic presence across the Solar System.

**The fantasy:** *"I ran the agency that made humanity multi-planetary — and every step of it was plausible."*

**The core tension:** Everything is possible, nothing is free. Delta-v, mass, power, crew-time, budget and political capital are the six currencies; every decision spends at least two of them.

Inspirations: *Aurora 4x* (depth, data-driven UI, designer-style ship building), *The Expanse* (plausible propulsion, the tyranny of delta-v), *Children of a Dead Earth* (orbital realism), *Terra Invicta* (research breadth — minus the aliens).

---

## Design Pillars

These are non-negotiable. The whole game is built to honour them.

| # | Pillar | What it means |
|---|--------|---------------|
| **P1** | **Scientific plausibility first** | Every technology, resource process and trajectory is flown hardware, a funded program, or a published peer-reviewed concept. Every number in the data files carries a `source` tag. No reactionless drives, no FTL, no "top speed", no scarcity invented for balance. |
| **P2** | **Physics is the gameplay** | Orbital mechanics, the rocket equation, power/thermal budgets and life-support closure are *simulated*, not flavour text. You plan transfers on porkchop plots, not on a hex grid. |
| **P3** | **Research is a process, not a purchase** | Basic science → insight → engineering program → test campaign → flight heritage. Dead ends, overruns, failures-that-teach, rare breakthroughs and deliberate leapfrogging are core mechanics. |
| **P4** | **Data-dense 2D presentation** | A clean, text-friendly UI in the lineage of Aurora 4x / EVE Online. Tables, plots and schematic maps over 3D spectacle. Every screen is information-first. |
| **P5** | **Asymmetric but fair factions** | National agencies and private companies play by different economic and political rules but share **one** physical reality. |
| **P6** | **Educational honesty** | An in-game encyclopedia ("Sojournal") explains the real science behind every mechanic, with references. The game should make you smarter about real spaceflight. |

**Explicitly out of scope (reserved for future expansions):** weapons, combat, sabotage, alien civilisations, interstellar travel, terraforming beyond paraterraforming-scale habitats.

---

## The Factions

You pick one of ten organisations. All ten exist in every game; the others are simulated competitors and partners. Asymmetry comes from starting assets, funding model, political constraints and research bonuses — **never** from different physics.

### National agencies (appropriation-funded)

| Faction | Starting strengths (2026) | Funding | Special constraint |
|---|---|---|---|
| **NASA** | Largest budget, SLS/Orion, Artemis, Deep Space Network, ISS leadership | Annual congressional appropriation; volatile with elections; directed programs | Strict planetary-protection; a fatal accident freezes crewed flight for years |
| **ESA** | Ariane 6, strong science fleet (JUICE, BepiColombo), ExoMars heritage | Multi-year ministerial cycles → stable but slow; geo-return raises costs ~10% | Consensus: big program changes take 6–18 months of lead time |
| **Roscosmos** | Soyuz/Proton/Angara, long-duration crew heritage, nuclear expertise | State budget, sanction-sensitive; can sell seats/engines | Import restrictions raise avionics cost; strong NTP/NEP research discount |
| **CSA** | World-leading robotics (Canadarm3), strong partnerships | Small appropriation + partnership revenue | Cannot lead crewed launch early; thrives by barter (robotics for crew seats) |
| **JAXA** | H3, sample-return mastery (Hayabusa, MMX), SLIM precision landing | Stable mid-size appropriation | Precision-landing & sample-return discounts; small crewed budget |

### Private companies (revenue-funded archetypes)

| Faction | Archetype | Revenue model |
|---|---|---|
| **Helion Launch Systems** | Reusable-launch disruptor | Commercial launch market + agency contracts; vertical integration |
| **Meridian Aerospace** | Patient billionaire-funded | Owner injections (finite), tourism, engine sales |
| **Astrolith Resources** | Prospecting & ISRU startup | Venture rounds, data sales, future propellant sales — must survive to revenue |
| **Orbital Forge** | In-space manufacturing & stations | Product sales (ZBLAN fiber, protein crystals), station leasing |
| **Caravel Logistics** | Tug, depot & delivery services | Per-kg delivery contracts, propellant resale, satellite servicing |

Private factions can go **bankrupt** (game over) and must manage runway, raises, debt and contract pipelines. Agencies cannot go bankrupt but can be **gutted** — a budget collapse to caretaker level (soft fail).

A global **launch market**, **science community** and **political weather** layer connects all factions, plus an always-AI **CNSA** and minor agencies (ISRO, KARI, UAE) as competitors and contract sources.

---

## The Core Loop

Three nested loops play out across the game's timescales:

1. **Operational (days–weeks):** monitor missions, respond to events (anomalies, solar storms, budget news), manage launch manifests, allocate crew-time and DSN passes, approve manoeuvres at nodes.
2. **Program (months–years):** design vehicles, run engineering programs through TRL gates, fly test campaigns, bid on contracts, build infrastructure, plan transfer-window campaigns. *Mars windows every ~26 months are the game's natural heartbeat.*
3. **Strategic (years–decades):** set research-portfolio strategy, make architecture bets (NTP vs NEP vs distributed-launch chemical for Mars), choose sites, pursue settlement sustainability and legacy goals.

A session typically alternates: **pause → review the event queue → adjust plans → set time acceleration → next event interrupts.**

**Time model:** continuous simulated time on a fixed-timestep deterministic core. Time warp runs from 1 s/s up to ~1 year/min, with **automatic interrupt-and-pause** on configurable event classes (manoeuvre nodes, anomalies, reviews, budget votes, discoveries) — the Aurora-style "increment until something matters" model. The calendar matters: launch windows, fiscal years, election cycles, eclipse seasons, Mars dust-storm seasons, and the 354-hour lunar night for surface power.

---

## Major Game Mechanics

### 1. Research — a modelled process, not a tech-point shop

Every capability is produced by **two interacting tracks**:

- **Track A — Science (understanding).** A graph of **Knowledge Domains** (Materials, Nuclear Physics, Astrodynamics, Closed-Ecology, per-body Geosciences, Astrobiology, Human Factors, …). Each is a continuous **Understanding Level (UL, 0–100)**, not a boolean unlock. UL gates *which* engineering programs are available and sets their *risk floor*. You raise UL by funding **Research Programs** (which consume **Research Points**) **and by flying missions** — a Europa flyby dumps real understanding into Geoscience and Astrobiology that no lab can buy.

- **Track B — Engineering (implementation).** **Engineering Programs** turn understanding into concrete **Technologies** (an engine, a lander, an ISRU plant) by consuming **Design Effort**. Every program advances its technology up the standard **NASA TRL 1–9 ladder** through a **test campaign**.

> **Mantra:** Science says *"possible."* Engineering says *"ready."* Operations says *"reliable."*

**TRL is the spine.** A technology isn't "researched" — it is *advanced through Technology Readiness Levels*. Each step has a cost, a minimum time you cannot buy your way under, and a facility requirement. A tech can only fly at **TRL ≥ 6** (with steep reliability penalties), is comfortable at 7+, and is only "boring and dependable" at 9. **Reliability** is a function of TRL + accumulated flight units + relevant domain UL. **Flight Heritage** from each successful use raises reliability toward a ceiling and discounts derivative programs.

**What makes the tree feel alive** (all seeded per game, so playthroughs differ):

- **Dead ends** — some approaches are dead ends *within a TRL band* in a given seed; progress stalls and costs balloon. You get hints before it's confirmed. Mitigate by pursuing parallel approaches or investing in the underlying science to "see further."
- **Cost & schedule overruns** — every program rolls realistic, Augustine-style variance driven by TRL jump size, UL margin, staffing quality, facility adequacy and political interference.
- **Failures that teach** — a failed test costs money and schedule but **injects UL** ("we learned why the injector screeched"). Repeated failure without UL growth is a dead-end signal.
- **Breakthroughs** — sustained investment in *basic* science accrues hidden "insight pressure"; crossing a threshold can trigger a step-change that discounts a cluster of techs or unlocks a locked branch early. Roughly once per 8–15 years in a heavily-invested domain; rushing applied work almost never triggers one.
- **Leapfrogging** — deliberately skip a generation by over-investing in basic science to satisfy a higher tier's prerequisites via UL rather than via the previous product. Costs more up front, risks more (no intermediate flight heritage), can vault you ahead of rivals.

**The global science tide.** Science is not private. Every domain has a **World UL** advancing from everyone's aggregate investment plus a real-world baseline; your private UL is the world's plus your lead or lag. You can **publish** (prestige + world UL, lose exclusivity) or **hold/patent** (keep the lead, earn licensing). Trailing a domain makes your research there cheaper (you're reproducing known results), which bounds runaway leads and keeps the AI relevant.

**Personnel.** Scientists and engineers (with discipline tags, skill ratings and traits like *Visionary*, *Closer*, *Maverick*, *Safe Hands*), plus Program Managers, Astronauts, Mission Controllers and Diplomats. Hire, poach, train and retain them. The crew pipeline (select → train → assign → manage radiation dose, psychological load and physical health over a career) is crucial for crewed play. Losing key people can even *reduce* your effective UL in a niche domain — institutional memory loss, modelled.

### 2. The Tech Tree

A **web, not parallel ladders** — cross-branch gates are deliberate (NEP needs Power + Thermal + Electric Propulsion + Reactor; a Mars settlement needs EDL + ISRU + ECLSS + Power + Construction). The branches:

- **A — Knowledge Domains** (17 science tracks with synergy links)
- **B — Propulsion:** chemical, electric (ion/Hall/MPD/VASIMR), nuclear-thermal (NTP), nuclear-electric (NEP), frontier/late-game (fission-fragment, fusion — optional endgame), and propellant-logistics tech (depots, on-orbit transfer, zero-boil-off)
- **C — Power & Thermal:** photovoltaics, RTG/radioisotope, surface fission, storage, conversion cycles, radiators (a frequently-binding bottleneck)
- **D — Structures, Materials & Manufacturing:** lightweight structures, cryotanks, inflatable habitats, radiation shielding, in-space additive manufacturing, regolith construction
- **E — GNC, Autonomy & Comms:** navigation, precision/hazard-relative landing, rendezvous & docking, mission autonomy, laser comms & relay constellations
- **F — Life Support & Crew:** ECLSS closure fraction, bioregenerative food loops, artificial gravity, radiation protection, EVA, medical autonomy
- **G — ISRU & Resource Processing:** lunar polar water, regolith oxygen, Mars-atmosphere propellant (Sabatier), asteroid volatiles, plant scale-up
- **H — Launch & Earth-to-Orbit:** expendable → partially reusable → fully reusable → on-orbit-refuel architectures
- **I — In-Space Vehicles & Surface Systems:** capsules, tugs, landers, transit habitats, rovers, station and base modules, body-specific explorers (Mars helicopter, Venus aerobot, Titan rotorcraft, Europa cryobot)
- **J — Science Instruments & Astrobiology:** remote sensing, in-situ geochem, the staged life-detection suite, subsurface access, sample-return & containment chains
- **K — Operations, Automation & Data:** fleet management, logistics planning, robotic construction, digital-twin test-cost reduction

Two guarantees hold the design together: **every capability category is reachable by ≥2 candidate paths** (so per-game dead-end seeding never bricks a strategy), and **every node carries a `source` field** — a node without a citable basis doesn't ship.

### 3. Spaceflight & Physics

This is the simulation core, and it obeys conservation laws.

- **Astrodynamics:** patched-conics for fast planning plus a true **n-body propagator** for authoritative state. Perturbations that matter are modelled (third-body, J2/oblateness, solar radiation pressure, atmospheric drag, low-thrust acceleration). The game *reconciles plans against the real propagation* — a plan is never free of consequences.
- **Manoeuvre planning** (your core spaceflight verb): **porkchop plots** for ballistic transfers, **manoeuvre nodes** on the timeline, **gravity-assist/flyby** designers (VEEGA-style), **low-thrust spiral** planners for electric propulsion, **low-energy (weak-stability-boundary) transfers** unlocked by Astrodynamics UL, and **aerocapture/aerobraking** as planned, risky Δv-savers.
- **The rocket equation as a felt constraint:** `Δv = Isp · g₀ · ln(m0 / mf)`. The designer surfaces mass fractions live. Staging trade-offs, payload-fraction cliffs, the brutal cost of high Δv with chemical, boil-off eating your Δv on long coasts — and why ISRU/refuelling/depots are transformative (they reset m0 at a new node).
- **Honest propulsion couplings:** electric propulsion is power-limited (high Isp ⇒ low thrust ⇒ long burns ⇒ carry and *cool* the power source); NEP drags its reactor and radiators everywhere; NTP gives high thrust *and* Isp but heavy reactors and hydrogen boil-off. Waste heat is a first-class mass.
- **Entry, Descent & Landing (EDL):** its own simulated phase, because it's where missions die. Atmospheric entry heating, the **Mars EDL gap** (landing >1–2 t is genuinely hard), propulsive descent on airless bodies, rendezvous-and-anchor on microgravity bodies. Outcomes depend on vehicle suitability, site hazards and a reliability roll.
- **Operations under light-lag:** comms light-time is real (1.3 s to the Moon, 3–22 min to Mars, hours to the outer system). Beyond a threshold, real-time teleop is impossible → onboard autonomy and pre-planned sequences matter, and a finite **ops-capacity** pool limits how many craft you can babysit.

### 4. The Vehicle Designer

A **spreadsheet-grade composer** (Aurora-style, physics-checked). You build vehicles from **researched component Technologies** and the game computes emergent performance:

- **Inputs:** structure/tanks, propulsion, power, thermal/radiators, avionics/GNC, comms, life support, ISRU/science/cargo payloads, landing/EDL kit, docking, RCS.
- **Computed outputs:** dry & wet mass, Δv per stage/mode, thrust & T/W in each relevant gravity field, power & thermal balance (margin must be ≥ 0), life-support closure & endurance, payload capacity, composed reliability, unit cost & build time (with learning curve), and suitability checks (*Can it actually land here? Survive this radiation dose? Close its power budget at Jupiter?*).
- **Realism guards:** the designer red-flags the impossible — negative power margin, radiators too small for the heat load, Δv short of the mission, a lander whose T/W < local gravity, crewed endurance < mission duration, dose over limit. You *can* fly marginal designs, but the risk numbers tell the truth. **No physics cheats; only informed gambles.**

Vehicle classes (all designer-built): launch vehicles, crew capsules, cargo spacecraft, tugs, landers, ascent vehicles, transit habitats/cyclers/spin-habs, rovers, station and base modules, ISRU plants, relay sats, and body-specific science explorers.

### 5. Economy & Logistics

A closed-ish, plausible space economy where the dominant cost is always **mass × delta-v to where you need it**.

- **Six currencies:** Funds, Delta-v/Propellant, Mass-to-orbit, Crew-time, Ops capacity, Political/Reputation capital. The art of the game is converting between them at good exchange rates.
- **Budgeting:** agencies run an appropriation model (annual/multi-year budgets set by politics, directed funds, fiscal-year cliffs); companies run a revenue model where **cash runway is sacred** (model burn, revenue, financing — run out and it's bankruptcy). All costs are estimated with **P50/P80 uncertainty** and realised with overruns; **learning curves** (Wright's law) make reusable, high-cadence hardware cheap and bespoke one-offs expensive.
- **Resources have a location.** Every resource unit has a **delta-v address** — a tonne of water in LEO, at EML-1, on the lunar surface and on Phobos are four different goods with wildly different values. The economy is fundamentally a **logistics network priced in delta-v**, with depots as buffer nodes and reusable tugs/cyclers amortising over many trips.
- **ISRU economics** are the heart of the off-Earth economy: lunar polar water → propellant, Mars Sabatier propellant (which often makes a crewed Mars *return* feasible at all), regolith oxygen/metals/construction feedstock, asteroid volatiles. ISRU is justified only when the launch cost it saves beats the cost to build, operate, amortise and *deliver* the plant — and the game makes you feel that break-even.
- **Markets, contracts & partnerships:** a global launch market sets $/kg by orbit class; agencies post RFPs (CLPS/COTS-style) that companies bid on; consortia co-fund programs and share TRL credit, IP and crew seats; a data/IP market lets you sell, license or hold. Strategic materials (Pu-238/Am-241 for RTGs, enriched nuclear fuel, rare-earths) are genuinely scarce and competed for.

### 6. The World — Solar System, Sites & Astrobiology

- **The board:** the full Solar System — Sun, 8 planets, Pluto + major dwarfs, ~150 significant moons, and a curated catalogue of **~3,000 asteroids and comets with real orbital elements** (from JPL/MPC data); the rest of the belt and Kuiper region abstracted as **statistical prospecting fields** you survey to convert into known targets. Ephemerides are propagated from real elements so windows and assist opportunities are physically correct across 2026–2126.
- **Sites, not tiles.** Bodies expose specific characterised **Sites** seeded from real targets (Shackleton's permanently-shadowed craters, Jezero's delta deposits, Valles Marineris caves, Europa's ice, Enceladus' plume). Site properties — resource grade, illumination, slope, comms visibility, science value, hazard, planetary-protection category — are revealed progressively by survey, with modelled uncertainty. *Build a mine on a bad ice grade and you lose money; surveying first is strategy.*
- **Planetary protection** is a real, modelled COSPAR-style regime: body categories I–V set sterilisation and containment requirements; crash a non-sterile lander into a Special Region and you pay a science and reputation cost — you can ruin the pristine astrobiology experiment for everyone, including future you.
- **Astrobiology — no aliens, but maybe microbes.** Life elsewhere is modelled honestly as an open scientific question with a **per-game seeded ground truth**. Candidate habitats (Mars subsurface, Europa/Enceladus oceans, Titan, Ceres brines, Venus clouds) carry a hidden truth flag set within plausibility bounds. **Detection is a staged process** — orbital biosignature hints → in-situ chemistry → microscopy/metabolism → sample return & independent confirmation — giving probabilistic evidence, not a "LIFE FOUND" popup. False positives and abiotic explanations compete; consensus forms over time. Discovered life is a **science object, never an actor.**
- **Politics & events:** a lightweight geopolitical/PR layer drives budgets, approvals and drama (never combat). Per-faction relationships and prestige, public and political mood, policy and treaties (launch licensing, nuclear-launch approval, export controls, debris regulation), and a **seeded, state-driven event system** — a low-TRL, under-tested, over-subscribed craft *earns* its anomaly probability.

---

## Goals & Scoring

A sandbox with structured goals; the game formally scores at 2126 (configurable 25/50/100-year runs).

- **Milestones ("Firsts"):** ~120 scored historic firsts. **World-first** earns prestige and funding effects; **faction-first** earns a smaller score. (First fully-reusable orbital flight, first kg of lunar-derived propellant sold, first crewed Mars landing, first Mars ascent on local propellant, first sample from an Enceladus plume, first cryobot through Europan ice, first conclusive astrobiology result…)
- **Grand Goals** (pick one at start, changeable with a penalty):
  - **Pathfinder** — accumulate exploration firsts and science.
  - **Homestead** — build a settlement that survives a 5-year Earth-resupply embargo stress test.
  - **Prospector** — run an in-space economy selling off-Earth resources at a profit.
  - **Seeker** — resolve the astrobiology question (positive *or* conclusive negative) for ≥ 3 candidate worlds.
- **Soft-fail states:** agency gutting, private bankruptcy, a crewed-program loss-of-crew spiral. The game continues in observer/rebuild mode.

**Replayability** comes from the per-game seed: tech-tree dead-end/breakthrough rolls and astrobiology ground truth differ each game *within plausible bounds*, so every playthrough's "true map of what works and what's out there" is different. Ironman and save-anywhere modes are both supported, and any run is fully replayable from its event log (determinism). Difficulty alters political/economic harshness and anomaly rates — **never physics.**

---

## UI / UX

**Target feel: Aurora 4x's depth + EVE Online's readability.** A clean, data-dense, mostly-2D interface where text and tables are first-class, plots are everywhere, and every number is traceable to a cause. No twitch input — everything is **plan → preview → commit.** SI units only.

**Presentation principles:** information-first; full **traceability** (*"why is this 6.2 km/s?"* → expand the mass/Isp breakdown); destructive actions always preview their consequences; fully playable paused; progressive disclosure (newcomer summaries on top, Aurora-grade detail one click down); colour-blind-safe palettes and scalable text from 1280×720 to 4K.

A persistent shell (top bar with date/time-warp/funds/alerts, left nav, central work area, right inspector, bottom event ticker) wraps the major screens:

| Screen | Purpose |
|---|---|
| **System Map** (hero screen) | 2D, logarithmically zoomable, multi-focus (heliocentric ↔ planetocentric ↔ local-ops). Real orbits, your craft & trajectories, SOIs, Lagrange regions, comms coverage; toggleable layers (resources, traffic, planetary-protection zones, science). |
| **Trajectory / Manoeuvre Planner** | Porkchop plots, manoeuvre-node editor, low-thrust arc planner, flyby & aerocapture designers; live Δv-vs-available check; queue burns, auto-pause at nodes. |
| **Research & Development** | Science portfolio (domain UL bars, RP sliders, world-tide deltas, breakthrough-pressure hints) and Engineering programs (TRL ladders, test-campaign status, P50/P80 vs actual, dead-end warnings); tech-tree graph view. |
| **Vehicle Designer** | The spreadsheet-grade composer: component picker, live mass/Δv/power/thermal/reliability/cost, realism red-flags, save as reusable class, side-by-side comparison. |
| **Operations / Fleet** | Every craft & asset (filter/sort/group), status, fuel, health, task; mission timelines, launch manifest, crew assignments & dose readouts. |
| **Economy & Contracts** | Budget/cash-flow dashboards, resource ledgers by location, market prices, the RFP board, partnerships, facilities, learning-curve tracking. |
| **Bases & Construction** | Site browser, base/station builder (emergent power margin, ECLSS closure, sustainability index), construction-project & ISRU-plant trackers. |
| **Personnel** | Scientists/engineers/PMs/astronauts/controllers/diplomats; skills, traits, assignments, recruitment, morale, crew health careers. |
| **World / Politics** | Faction relationships & prestige, public mood, policy/treaty state, the milestone race board, rival activity feed. |
| **Science Returns & Astrobiology** | Belief-state per body, incoming data, the staged astrobiology evidence tracker, discoveries log. |
| **Sojournal** | The searchable, cross-linked, source-cited in-game encyclopedia — educational-honesty backbone and soft tutorial. |
| **Alerts / Event Log** | Chronological, filterable feed; each event links to its screen; configurable which event classes pause the game. |

**Bespoke widgets worth highlighting:** the interactive porkchop plot, the per-vehicle Δv ladder/budget bar, the TRL ladder with test-campaign & risk overlays, domain UL bars with world-tide ghosts and breakthrough shimmer, the resource-by-location (delta-v-addressed) ledger, the logistics-graph view, the live base schematic, and the probabilistic astrobiology evidence meter.

---

## Project Status & Architecture

Sojourn is in active early development. The work is **spec-driven** (constitution → specify → clarify → plan → tasks → analyze → implement) and governed by a formal [constitution](.specify/memory/constitution.md) whose nine principles are enforced in code review and CI. Design intent lives in [`design/`](design/); the simulation core is being built crate-by-crate in Rust.

The architecture follows the constitution's hard rules:

- **The deterministic simulation core runs headless** with no dependency on UI, rendering or input. A given seed plus a given sequence of decisions always produces bit-identical state; the core is fully replayable from its event log. The UI (not yet built) will read state and issue commands across a defined boundary and contain **no** game logic.
- **Physics is the authoritative simulation.** The numerical propagator is the source of truth; planning aids (patched-conics, porkchop plots) are approximations reconciled against it. The physics engine contains **no per-technology magic numbers** — it reads all constants from sourced data files.
- **Content is data; mechanics are code.** Bodies, sites, tech-tree nodes, propulsion/economy constants and events live in versioned, schema-validated [`data/`](data/) files — every plausibility-bearing field carrying a `source` tag, validated in CI.

### Workspace crates

| Crate | Role |
|---|---|
| `sojourn-core` | Deterministic kernel: fixed-timestep scheduler, event store & journal, seeded RNG streams, save/migrate, state hashing, module contracts |
| `sojourn-astro` | Astrodynamics: in-crate vectors, Kepler & Lambert solvers, n-body integrator, frames/SOIs, propulsion & manoeuvre models, trajectory planners (porkchop, flyby, low-energy, low-thrust) |
| `sojourn-world` | Solar-system data model: body/site catalogue, ephemerides, prospecting fields, the belief-state layer (what the player knows vs ground truth), the Sojournal |
| `sojourn-research` | The two-track research model: domain UL curves, TRL engine, dead-end/breakthrough/tide seeding, reliability, personnel & astronauts |
| `sojourn-worldbuild` | Offline build tool that turns raw sourced inputs into validated world data |
| `sojourn-harness` | Determinism & integrity harness: double-run, round-trip, kill-test, mutation and synthetic scenarios |

**Tech stack:** Rust (edition 2024, pinned toolchain), `serde`/`postcard` for serialization, `libm` for transcendentals (float-determinism policy), `ron` for data fixtures, `blake3` for state hashing, `slotmap` for entity stores. **No third-party physics/math dependencies** — vectors, Kepler/Lambert solvers and the integrator are all in-crate so realism is auditable. **No UI in the workspace yet.**

### Building & testing

```bash
cargo test --workspace      # headless integration + determinism tests
cargo clippy --workspace --all-targets -- -D warnings
```

CI runs fmt + clippy + a dependency audit (enforcing no UI crates and licence policy) and the full test suite on Windows and Linux. The test gates required by the constitution include physics validation against known analytic cases (Hohmann Δv, two-body periods, simple flybys, rocket-equation identities), a determinism test that runs a seed+decision-script twice and asserts bit-identical state, data schema + source-presence validation, and save/load round-trip identity.

---

## License

Licensed under either of **MIT** or **Apache-2.0** at your option.

---

> *Sojourn — everything is possible, nothing is free.*
