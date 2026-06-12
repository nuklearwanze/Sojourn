# Sojourn — Per-Slice `/speckit.specify` Prompts

Sojourn is too large for a single plan. After the top-level `/speckit.specify` (see
[`specify-prompt.md`](./specify-prompt.md)) and a `/speckit.clarify` pass, build the game **one
slice at a time**, each on its own branch with its own `specify → clarify → plan → tasks →
analyze → implement` cycle.

Each block below is a ready, copy-paste prompt. Every prompt:

- describes **what** and **why**, not a tech stack;
- names the authoritative design doc(s) in `../design/` and the binding
  `.specify/memory/constitution.md`;
- states its dependencies on earlier slices so the agent plans against existing interfaces rather
  than re-inventing them.

Build them **in order** — later slices assume the contracts established by earlier ones. Run
`/speckit.analyze` before every `/speckit.implement`.

---

## Slice 1 — Simulation core & time

> Depends on: nothing (foundation). Establishes the contracts every later slice builds on.

```
/speckit.specify Build the simulation core for Sojourn: the deterministic, headless, pausable real-time engine that every other system plugs into. Authoritative design: design/04-SPACEFLIGHT.md (implementer notes), design/00-OVERVIEW.md (time model, currencies, loops) and .specify/memory/constitution.md (Principles II, III, IV, V) — read them.

WHY: the whole game's credibility rests on a core that is reproducible from a seed plus the player's decisions, decoupled from any UI, and able to fast-forward through quiet years yet stop the instant something needs a human. Determinism is what makes saves, replays, shared seeds, automated testing and bug reproduction possible.

The core must provide: a fixed-timestep advance with a single authoritative world state; a seeded pseudo-random stream (and sub-streams) so an identical seed plus identical decisions reproduces an identical history; an event-driven time-warp scheduler that runs at selectable rates and performs interrupt-and-pause — automatically halting on manoeuvre nodes, mission milestones, anomalies, design/program reviews, budget votes, discoveries and player-defined watch conditions; an in-order event/decision log that is sufficient to replay the game from the beginning; and save/load that round-trips the entire world state exactly. Expose a headless API to step time, query state, submit decisions and register interrupt conditions, with no presentation logic inside the core. Define the module boundaries and data-ownership rules that later slices (astrodynamics, world, research, economy, bases, life support, politics, UI) will depend on. Include a deterministic test harness that runs the core with no UI and a double-run determinism check (same seed+inputs ⇒ identical state hash and identical event log). Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 2 — Astrodynamics

> Depends on: Slice 1 (steps time, owns state, deterministic PRNG).

```
/speckit.specify Build Sojourn's astrodynamics: the orbital-mechanics layer that is the physical source of truth for where everything is and how it moves. Authoritative design: design/04-SPACEFLIGHT.md (astrodynamics, manoeuvre planning, propulsion physics) and .specify/memory/constitution.md (Principles I, II, III) — read them. Build on the Slice 1 core's time-step, state ownership and determinism.

WHY: realistic trajectories are the game's central felt constraint. The rocket equation, transfer windows and gravity assists must emerge from real propagation, not from scripted values, and there must be no top speed and no reactionless motion.

Provide two reconciled fidelity tiers: a fast patched-conic / analytic planner for instant what-ifs, and an authoritative n-body numerical propagator (the simulated truth) with the perturbations that matter — third-body, oblateness/J2, solar radiation pressure, atmospheric drag where relevant, and continuous low-thrust acceleration. Model reference frames and spheres of influence. Provide planning tools as computed results, not magic: manoeuvre-node planning with delta-v budgeting, porkchop/launch-window solving, gravity-assist/flyby chaining, low-thrust spiral arcs, low-energy/weak-stability-boundary transfers (gated by research elsewhere), and aerocapture/aerobraking geometry. Apply thrust from a propulsion interface defined by mass, Isp/exhaust velocity, thrust, throttle and available power (the propulsion model itself is Slice 4; here, define and consume the interface). Trajectory-correction manoeuvres and execution error must be representable. The propagator must be a deterministic fixed-step integrator and must be validated in headless tests against analytic cases — Hohmann transfer delta-v, two-body orbital periods, and a known flyby — within stated tolerances. Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 3 — World data model

> Depends on: Slices 1–2 (state core; ephemeris consumes the propagator/frames).

```
/speckit.specify Build Sojourn's world data model: the Solar System the game is played in, plus the belief-state layer that separates what a faction knows from what is true. Authoritative design: design/05-WORLD.md and design/00-OVERVIEW.md (world scope); also .specify/memory/constitution.md (Principles I, V, VIII) — read them. Build on the Slice 1 core and the Slice 2 propagator/frames.

WHY: every mission, resource and discovery is anchored to a real place with real orbital data, and the game's honesty depends on the player only ever acting on imperfect, mission-improvable knowledge — never on hidden ground truth.

Model: the Sun, planets, ~150 significant moons, and on the order of 3,000 catalogued small bodies (asteroids/comets) from real orbital elements, plus statistical prospecting fields for the uncatalogued population. Model dynamical locations as first-class nodes — orbits, Lagrange points, halo/NRHO and other useful staging points — usable by the logistics graph later. For each body and surveyable Site, hold ground-truth physical properties (composition, resource type/grade, illumination, slope, thermal, comms geometry, hazards, planetary-protection category) AND a separate per-faction belief-state with modelled uncertainty that missions and instruments refine over time. Provide the source-cited in-game encyclopedia data ("Sojournal") describing real bodies and concepts. All catalogue and physical data must be data-driven with provenance/source fields and schema-validated. Provide query interfaces the UI and other systems will use (what is here, what do we believe is here, how certain are we). Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 4 — Vehicle designer & propulsion model

> Depends on: Slices 1–2 (consumes the propulsion interface Slice 2 defined).

```
/speckit.specify Build Sojourn's vehicle designer and the propulsion/vehicle physics behind it: the system where players compose spacecraft, launchers, landers and rovers from researched components and get honest, derived performance. Authoritative design: design/04-SPACEFLIGHT.md (propulsion physics model, vehicle designer, EDL) and design/02-TECH-TREE.md (propulsion and vehicle branches); also .specify/memory/constitution.md (Principles I, II, VII) — read them. Build on the Slice 1 core and implement the propulsion interface consumed by the Slice 2 astrodynamics.

WHY: the tyranny of mass and delta-v should be felt at design time. Every vehicle's capability must fall out of physics — the rocket equation, power-limited electric thrust, heavy-reactor nuclear propulsion, radiators as real mass — not from arbitrary stat points.

Provide a component-composition designer (Aurora-style, physics-checked) that builds vehicles from researched parts and computes, with full traceability of every number: total/dry/propellant mass, delta-v, thrust, thrust-to-weight, power generation and demand, thermal/radiator balance, reliability, and cost. Model propulsion families per the tech tree — chemical, electric (ion/Hall/MPD/VASIMR/electrospray), nuclear-thermal, nuclear-electric, and gated frontier options — each as physical parameter sets (Isp, thrust, power draw, throttle, mass, reliability) feeding the Slice 2 interface; treat electric propulsion as power-limited and nuclear-electric radiators as first-class mass. Compute reliability as a function of technology maturity (TRL), flight heritage and relevant domain understanding (the research values come from Slice 5; define and consume the interface). Define vehicle classes (launch vehicles, in-space stages/tugs, crewed vehicles, landers, rovers, station/base modules) and surface realism red-flags (e.g. negative margins, impossible T/W, radiator shortfalls). All component data must be data-driven with sources; no per-design magic numbers in engine code. Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 5 — Research system

> Depends on: Slices 1, 4 (feeds TRL/heritage/understanding into the designer & reliability).

```
/speckit.specify Build Sojourn's research system: research as a modelled process, not a purchase. Authoritative design: design/01-RESEARCH.md and design/02-TECH-TREE.md; also .specify/memory/constitution.md (Principles I, VI, VIII) — read them. Build on the Slice 1 core; expose the technology-maturity, flight-heritage and domain-understanding values that Slice 4 (vehicle/propulsion) consumes for capability and reliability.

WHY: the game's identity is that you cannot buy a technology — you fund basic science until understanding makes engineering possible, then mature it through test campaigns, with real risk of dead ends, overruns, failures-that-teach, rare breakthroughs and deliberate leapfrogging. This is what makes the tech tree feel earned.

Implement the two-track model: Track A Science raises continuous Understanding Levels (0–100) across knowledge domains; Track B Engineering advances technology programs through TRL 1–9 via test campaigns that are gated by the relevant domains' Understanding Levels. Generate research/engineering progress from funded scientists, engineers, project managers, facilities and missions. Model: cost and schedule uncertainty with overruns; seeded dead ends per TRL band with parallel-approach mitigation; failures that nonetheless inject understanding; rare breakthroughs that are both seeded and earned (roughly once per many years in a heavily-invested domain); leapfrogging by over-investing basic science to skip technology generations; and a global science "tide" (a World Understanding Level with publish-vs-patent choices, licensing, partnership, buying-in, and cheaper catch-up than frontier work). The per-game seed fixes which approaches are dead ends and when breakthroughs are possible. Model personnel as managed assets with traits, recruitment/poaching/training, an astronaut training/health/radiation career pipeline, and tacit-knowledge loss when teams disband. Web-shaped tech tree with cross-branch gates; every node carries a real-world source and capability categories remain reachable even when specific nodes are seeded shut. Expose the maturity/heritage/understanding query interface used by vehicle design, economy and politics. Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 6 — Economy & logistics

> Depends on: Slices 1–5 (resources by location need world; costs need designs & research).

```
/speckit.specify Build Sojourn's economy and logistics: the six-currency resource simulation and the transport graph that moves everything. Authoritative design: design/03-ECONOMY.md and design/04-SPACEFLIGHT.md (logistics) plus design/00-OVERVIEW.md (currencies, factions); also .specify/memory/constitution.md (Principles VII, VIII, IX) — read them. Build on the Slice 1 core, the Slice 3 world (locations/resources), the Slice 2 astrodynamics (transfer windows/delta-v) and Slice 4–5 (vehicle costs, learning, technology).

WHY: the felt scarcity must always be real — mass, delta-v, money, crew-time, ops capacity and political capital — never invented. Resources are not global piles; they sit at a place and cost delta-v and time to reach, and ISRU only pays off when the physics and economics actually close.

Implement six currencies (funds, delta-v/propellant, mass-to-orbit, crew-time, ops capacity, political/reputation capital) and two funding models: agency appropriations (annual/multi-year budgets, directed funds, fiscal cliffs) and private cash-runway with financing and bankruptcy. Model resources addressed by location and delta-v, with strategic-material scarcity (e.g. RTG fuel, fissile-material policy, rare elements). Implement realistic ISRU break-even economics — lunar-ice propellant, Mars Sabatier propellant, regolith oxygen/metals, asteroid volatiles — including scale-up dynamics so ISRU is a genuine investment decision, not free fuel. Implement a cost model with P50/P80 uncertainty and learning curves (costs fall with units built). Implement markets and contracts: a launch market, service-contract/RFP-bid system (CLPS/COTS-style), partnerships/consortia (barter, geo-return), data/IP licensing, tourism and in-space manufacturing revenue. Model the logistics layer as a directed transport graph whose nodes are the Slice 3 dynamical locations and whose edges are window-constrained transfers priced in delta-v and time-of-flight, with depots, reusable tugs and cyclers, and finite ops/comms capacity under light-time delay. Model capital facilities (labs, test stands, pads, ground segment/DSN, stations, depots). All economic constants are data-driven with sources. Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 7 — Bases & construction

> Depends on: Slices 1, 3, 5, 6 (sites, modules, ISRU plants, logistics of materials).

```
/speckit.specify Build Sojourn's bases and construction: orbital stations and surface bases assembled from modules with emergent properties. Authoritative design: design/03-ECONOMY.md (construction, facilities), design/05-WORLD.md (sites, planetary protection) and design/02-TECH-TREE.md (regolith construction, ISRU plants); also .specify/memory/constitution.md (Principles I, VII, VIII) — read them. Build on the Slice 1 core, Slice 3 sites, Slice 5 technology, and Slice 6 economy/logistics (materials, ISRU output, crew-time, ops capacity).

WHY: settlement is the game's destination, and a base should be the sum of physical truths — power margin, life-support closure, shielding, population capacity, sustainability — not an abstract level bar. Building far from Earth must be a genuine logistics and self-sufficiency problem.

Implement: orbital stations and surface bases composed from modules (habitat, power, ECLSS, ISRU, science, storage, manufacturing, shielding) sited at the Slice 3 Sites; emergent base properties computed from the modules — power margin, ECLSS closure fraction, population capacity, radiation shielding (including regolith-built shielding), sustainability/self-sufficiency index, and hazard exposure; construction projects with schedules, delivered-mass and crew-time demands routed through the Slice 6 logistics graph, and on-site fabrication/ISRU that reduces import needs over time; and the ability for a settlement to progress toward self-sufficiency. Respect planetary-protection categories from the world model when siting and building. All module/construction parameters are data-driven with sources; base properties derive from physics, not hand-set values. Expose base state to economy (production/consumption), life support (Slice 8) and politics (Slice 9). Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 8 — Life support & crew

> Depends on: Slices 1, 4, 5, 7 (vehicles, crew pipeline, bases as habitats).

```
/speckit.specify Build Sojourn's life-support and crew model, the system that makes crewed missions materially harder than robotic ones. Authoritative design: design/04-SPACEFLIGHT.md (life support, crewed difficulty, EDL) and design/01-RESEARCH.md (crew pipeline) plus design/02-TECH-TREE.md (life support & crew branch); also .specify/memory/constitution.md (Principles I, VII, VIII) — read them. Build on the Slice 1 core, Slice 4 vehicles, Slice 5 personnel/research, and Slice 7 bases.

WHY: humans in space are the hardest part of spaceflight, and the difference between a robotic probe and a crewed expedition should be felt in mass, risk, time and consequence — not flavour text.

Model: consumables versus ECLSS closure fraction (open-loop → physico-chemical → high-closure → bioregenerative), so closing the loop trades mass for technology and risk; radiation dose accumulation against limits, solar-particle-event storm shelters, and career dose tracking tied to the Slice 5 crew pipeline; physiological deconditioning (bone/muscle/cardiovascular/vision) with countermeasures including artificial gravity; psychology under isolation, confinement and comms-lag; spares, maintenance and failure of life-support hardware; and entry/descent/landing and aerocapture risk per body type, including the hard Mars EDL case. Crewed mission and base viability must depend on these physical states, and loss-of-crew must be a real, modelled consequence. Operations occur under light-time delay and finite ops capacity (from Slice 6). All physiological and ECLSS parameters are data-driven with sources. Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 9 — Politics, events & milestones

> Depends on: Slices 1, 3, 5, 6, 8 (mood drives budgets; PP & astrobiology need world/crew).

```
/speckit.specify Build Sojourn's politics, events, milestones and astrobiology systems — the non-combat competitive and narrative layer. Authoritative design: design/05-WORLD.md (politics, events, astrobiology, planetary protection) and design/00-OVERVIEW.md (firsts, Grand Goals, factions); also .specify/memory/constitution.md (Principles I, VIII, IX) — read them. Build on the Slice 1 core, Slice 3 world/belief-state, Slice 5 research, Slice 6 economy, and Slice 8 crew. There is NO combat and NO aliens — competition is for firsts, economics and prestige; any discovered life is a science object, never an actor.

WHY: without weapons, the game's tension comes from a race for historic firsts, the politics of money and approval, and the slow, honest unveiling of whether life exists elsewhere. These must be modelled as real pressure, not set dressing.

Implement: faction relationships and prestige; public/political mood that drives budgets, approvals and private valuations; policy and treaties (launch licensing, nuclear-launch approval, planetary-protection stringency, export controls) with real gameplay consequences; a COSPAR-style planetary-protection regime with forward- and back-contamination consequences and Special Regions; and a seeded-plus-state-driven event system that feeds the Slice 1 interrupt-and-pause loop. Implement the astrobiology question honestly: a per-game seeded ground truth on candidate worlds (e.g. Mars subsurface, Europa/Enceladus, Titan, Ceres, Venus clouds) resolved through a staged, probabilistic, mission-driven evidence process with abiotic competitors and scientific consensus forming over time — never a binary "life found" popup. Implement an AI world: AI-run factions that research, build, fly, contract, partner and race for milestones. Implement ~120 scored historic firsts (world-first vs faction-first) and selectable Grand Goals (Pathfinder, Homestead, Prospector, Seeker) as win/scoring conditions. Everything data-driven and seed-respecting. Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## Slice 10 — UI / UX

> Depends on: all prior slices (reads the headless core; holds no game logic).

```
/speckit.specify Build Sojourn's user interface: the data-dense, mostly-2D presentation layer in the lineage of Aurora 4x and EVE Online, reading entirely from the headless simulation core. Authoritative design: design/06-UI-UX.md (all screens, widgets, interaction model) and design/00-OVERVIEW.md; also .specify/memory/constitution.md (Principles IV, VIII; SI-units and accessibility constraints) — read them. Build strictly on the interfaces exposed by Slices 1–9; the UI must contain no game logic and no authoritative state.

WHY: Sojourn lives or dies on legibility — players must be able to read a thousand numbers, trust every derived figure, plan irreversible manoeuvres with confidence, and learn the real science as they play. Text and tables are first-class, not a fallback.

Implement a persistent shell plus the screens: a zoomable, multi-focus System Map (hero screen); a Trajectory/Manoeuvre Planner with porkchop plots, manoeuvre nodes, gravity-assist and low-thrust planning; a Research & Development screen showing domain Understanding Levels and TRL ladders with full traceability; a Vehicle Designer surfacing every derived number and realism red-flag; Operations/Fleet; Economy & Contracts; Bases & Construction; Personnel; World/Politics; a Science-Returns & Astrobiology screen with a staged evidence meter; the source-cited Sojournal encyclopedia; and a configurable Alerts/Event Log. Provide bespoke widgets (porkchop plot, delta-v ladder, TRL ladder, understanding bars, resource-by-location ledger, logistics-graph view, base schematic, astrobiology evidence meter). Enforce plan→preview→commit for irreversible actions, progressive disclosure for newcomers, full traceability of every derived value back to its inputs, SI units only, keyboard-rich and accessible interaction, and performance with virtualised tables for thousands of bodies and large fleets/ledgers at high time-warp. The UI subscribes to and commands the core; it never owns simulation state. Flag ambiguities for /speckit.clarify. Do not choose a tech stack.
```

---

## After all ten slices

Once the slices are individually green, do an integration pass: a full headless game run from seed to a Grand-Goal condition with determinism and save round-trip checks, then a realism-review sweep (`/speckit.analyze`) against the constitution's plausibility, determinism and scope principles before any release tag.
