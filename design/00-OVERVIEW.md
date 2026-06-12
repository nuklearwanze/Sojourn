# SOJOURN — Game Design Document, Part 0: Overview & Core Loop

> Hard-science 4X strategy about the exploration, industrialisation and settlement of the Solar
> System, 2026–2126. Inspirations: *Aurora 4x* (depth, data-driven UI, designer-style ship
> building), *The Expanse* (plausible propulsion, the tyranny of delta-v, politics of space),
> *Children of a Dead Earth* (orbital realism), *Terra Invicta* (research breadth — minus aliens).

---

## 1. Vision Statement

Sojourn is a single-player, pausable real-time grand strategy game in which the player runs one
of ten space organisations starting in **January 2026**, with exactly the hardware, budgets and
scientific knowledge humanity actually has at that date. There is no combat, no aliens, no
techno-magic. The drama comes from physics, money, politics, engineering risk and the slow,
hard-won expansion of human and robotic presence across the Solar System.

**The fantasy:** "I ran the agency that made humanity multi-planetary — and every step of it
was plausible."

**The core tension:** Everything is possible, nothing is free. Delta-v, mass, power, crew-time,
budget and political capital are the six currencies; every decision spends at least two of them.

## 2. Design Pillars (non-negotiable)

| # | Pillar | Consequence |
|---|--------|-------------|
| P1 | **Scientific plausibility first** | Every technology, resource process and trajectory in the game is either flown hardware, a funded program, or published peer-reviewed concept. Each number in the data files carries a source tag. No reactionless drives, no FTL, no "top speed", no artificial scarcity invented for balance. |
| P2 | **Physics is the gameplay** | Orbital mechanics, the rocket equation, power/thermal budgets and life-support closure are simulated, not flavour text. Players plan transfers on porkchop plots, not on a hex grid. |
| P3 | **Research is a process, not a purchase** | Basic science → insight → engineering program → test campaign → flight heritage. Dead ends, overruns, failures-that-teach, rare breakthroughs and deliberate leapfrogging are core mechanics. |
| P4 | **Data-dense 2D presentation** | Clean, text-friendly UI in the lineage of Aurora 4x / EVE Online. Tables, plots and schematic maps over 3D spectacle. Every screen is information-first. |
| P5 | **Asymmetric but fair factions** | National agencies and private companies play by different economic and political rules but share one physical reality. |
| P6 | **Educational honesty** | An in-game encyclopedia ("Sojournal") explains the real science behind every mechanic, with references. The game should make the player smarter about real spaceflight. |

**Explicitly out of scope (reserved for future expansions):** weapons, combat, sabotage, alien
civilisations, interstellar travel, terraforming beyond paraterraforming-scale habitats.

## 3. Player Factions

The player picks one of ten organisations. All ten exist in every game; non-player factions are
simulated competitors/partners run by the AI. Asymmetry is expressed via starting assets,
funding model, political constraints and research bonuses — never via different physics.

### 3.1 National agencies (appropriation-funded)

| Faction | Starting strengths (2026, factual) | Funding model | Special constraints |
|---|---|---|---|
| **NASA** | Largest budget (~$25 B/yr), SLS/Orion, Artemis program, deep-space network, ISS leadership, commercial contracting ecosystem | Annual congressional appropriation; high volatility with election cycles; directed programs (Congress can force/cancel projects) | Strict planetary-protection compliance; crew-safety politics (a fatal accident freezes crewed flight 2–4 years) |
| **ESA** | ~€8 B/yr, Ariane 6, strong science fleet (JUICE, BepiColombo), ExoMars heritage, Moonlight comms initiative | Multi-year ministerial cycles → stable but slow; geo-return rule raises hardware costs ~10% but spreads political support | Consensus mechanic: major program changes take 6–18 months of "ministerial" lead time |
| **Roscosmos** | Soyuz/Proton/Angara, unmatched long-duration crew heritage, nuclear-power expertise (Zeus/TEM nuclear tug concept), engine technology | State budget, low and sanction-sensitive; can sell seats/engines for cash | Component-import restrictions raise avionics costs; strong NTP/NEP research discount |
| **CSA** | Small (~CA$0.9 B/yr) but world-leading robotics (Canadarm3), strong partnerships | Stable small appropriation + partnership revenue | Cannot lead crewed launch early; thrives via barter: contributes robotics to others' missions for crew seats and data shares |
| **JAXA** | H3, sample-return mastery (Hayabusa 1/2, MMX), SLIM precision landing, ISS Kibo | Stable mid-size appropriation | Precision-landing and sample-return research discounts; small crewed budget |

### 3.2 Private companies (revenue-funded, fictional but archetypal)

| Faction | Archetype | Starting assets | Revenue model |
|---|---|---|---|
| **Helion Launch Systems** | Reusable-launch disruptor (SpaceX-like) | Partially reusable medium-lift vehicle in service, super-heavy fully-reusable program at TRL 5 | Commercial launch market + agency contracts; vertical integration |
| **Meridian Aerospace** | Patient billionaire-funded (Blue-Origin-like) | Suborbital tourism line, heavy-lift engine at TRL 7, deep cash reserves | Owner injections (finite), tourism, engine sales |
| **Astrolith Resources** | Prospecting & ISRU startup | Cubesat prospector line, thermal-mining patents, no launch capability | Venture rounds, data sales, future propellant sales — must survive to revenue |
| **Orbital Forge** | In-space manufacturing & stations | ISS-attached module, micro-g pharma/fiber pilot line | Product sales (ZBLAN, protein crystals), station leasing, agency anchor contracts |
| **Caravel Logistics** | Tug, depot & delivery services | Storable-propellant tug at TRL 8, depot design study | Per-kg delivery contracts, propellant resale, satellite servicing |

Private factions can go **bankrupt** (game over) and must manage runway, raises, debt and
contract pipelines. Agencies cannot go bankrupt but can be **gutted** (budget collapse to
caretaker level — soft fail state).

### 3.3 Non-player world

CNSA (Chinese agency) is always AI-controlled in v1.0 (story/competition driver: Tiangong,
ILRS lunar program), plus minor agencies (ISRO, KARI, UAE) as partnership/contract sources.
A global **launch market**, **science community** and **political weather** layer connects all
factions (see 03-ECONOMY.md and 05-WORLD.md).

## 4. The Core Loop

Three nested loops:

1. **Operational loop (days–weeks game time):** monitor missions, respond to events
   (anomalies, solar storms, budget news), manage launch manifests, allocate crew-time and DSN
   passes, approve manoeuvres at nodes.
2. **Program loop (months–years):** design vehicles, run engineering programs through TRL
   gates, fly test campaigns, bid for/award contracts, negotiate partnerships, build
   infrastructure, plan transfer-window campaigns (Mars windows every ~26 months are the
   game's natural heartbeat).
3. **Strategic loop (years–decades):** research portfolio strategy, choose architecture bets
   (e.g., NTP vs NEP vs distributed-launch chemical for Mars), site selection, settlement
   sustainability, legacy goals.

A session typically alternates: pause → review event queue → adjust plans → set time
acceleration → next event interrupts.

## 5. Time Model

- Continuous simulated time, fixed-timestep deterministic core (see constitution).
- Time warp: 1 s/s up to 1 year/min, with **automatic interrupt-and-pause** on configurable
  event classes (manoeuvre nodes, anomalies, reviews, budget votes, discoveries) — the
  Aurora-style "increment until something matters" model.
- Sub-stepping: trajectories and resource flows integrate at warp-appropriate resolution;
  burns and EDL always resolve at fine timestep.
- Calendar matters: launch windows, fiscal years, election cycles, eclipse seasons, dust-storm
  seasons on Mars, lunar day/night (354 h) for surface power.

## 6. Map & Physical Scope

- Full Solar System: Sun, 8 planets, Pluto + major dwarfs, ~150 significant moons, curated
  catalogue of ~3,000 asteroids/comets (real orbital elements from MPC data; the rest of the
  belt abstracted as a prospecting statistics layer).
- 2D, zoomable, multi-focus system map (heliocentric ↔ planetocentric ↔ local ops views),
  logarithmic zoom, trajectory overlays, SOI/Lagrange-point regions. See 06-UI-UX.md.
- Surface play is site-based, not tile-based: bodies expose **Sites** (e.g., Shackleton rim,
  Jezero, Valles Marineris caves) with surveyed properties (resources, slope, thermal,
  illumination, comms visibility, science value). Bases are built at sites from modules.

## 7. What the Player Actually Does (verb list)

Survey • research • prototype • test • design vehicles • build & launch • plan trajectories •
operate missions (robotic & crewed) • mine & process resources • construct stations & bases •
trade & contract • lobby & campaign • hire & develop people • respond to crises • publish or
patent • set policy positions • found a settlement.

## 8. Progression Arc (typical, not scripted)

| Era | Years (typ.) | Theme | Representative capabilities |
|---|---|---|---|
| 0 – Foothold | 2026–2032 | Reusable launch matures, cislunar race, first ISRU demos | Crew on Moon, propellant transfer demo, polar ice ground-truth |
| 1 – Cislunar economy | 2030–2042 | Depots, lunar oxygen pilot plant, commercial LEO stations | Reusable landers, 40 kWe surface fission, NTP first flight |
| 2 – Mars & the Belt | 2038–2055 | First crewed Mars, asteroid volatiles, NEP cargo tugs | Mars ISRU at scale, MWe NEP, regolith construction |
| 3 – Settlement | 2050–2075 | Permanent bases, closed-loop life support, in-space industry | Spin-gravity habitats, local food >50%, ocean-world access |
| 4 – Maturity | 2070–2126 | Outer-planet infrastructure, possible fusion, answer "are we alone (here)?" | Cryobot through Europan ice, fission-fragment/fusion tugs (if breakthroughs occur) |

Eras are emergent, not gated — a player can rush or stall; the AI world advances regardless.

## 9. Goals, Scoring & End States

Sandbox with structured goals; game formally scores at 2126 (configurable 25/50/100-yr runs).

- **Milestones ("Firsts")** — ~120 scored historic firsts (first commercial station, first
  child... no — first crewed Mars landing, first sample from Enceladus plume, first kg of
  lunar-derived propellant sold, etc.). World-first earns prestige + funding effects;
  faction-first earns smaller score. Full list in 05-WORLD.md §6.
- **Grand Goals** (pick at start, can change with penalty):
  1. *Pathfinder* — accumulate exploration firsts and science.
  2. *Homestead* — a settlement that survives a 5-year Earth-resupply embargo stress test
     (sustainability index ≥ threshold).
  3. *Prospector* — in-space economy: ≥ X t/yr off-Earth resources sold at profit.
  4. *Seeker* — resolve the astrobiology question for ≥ 3 candidate worlds (positive or
     conclusive negative).
- **Soft-fail states:** agency gutting, private bankruptcy, crewed-program loss-of-crew
  spiral. Game continues in observer/rebuild mode.

## 10. Difficulty, Determinism & Replayability

- Difficulty alters political/economic harshness and anomaly base rates — never physics.
- Seeded runs: tech-tree dead-end/breakthrough rolls and astrobiology ground truth are seeded
  per game (see 01-RESEARCH.md §6, 05-WORLD.md §4) → each playthrough's "true map of what
  works and what's out there" differs within plausible bounds.
- Ironman + save-anywhere modes; full simulation replayable from event log (determinism).

## 11. Document Map

| File | Contents |
|---|---|
| 00-OVERVIEW.md | This file |
| 01-RESEARCH.md | Science model, TRL engine, dead ends, breakthroughs, leapfrogging, personnel |
| 02-TECH-TREE.md | Full technology tree, all domains, all nodes |
| 03-ECONOMY.md | Budgets, markets, contracts, ISRU economics, logistics, infrastructure |
| 04-SPACEFLIGHT.md | Astrodynamics model, propulsion catalogue, vehicle designer, life support, EDL |
| 05-WORLD.md | Solar-system data model, sites & resources, astrobiology, planetary protection, policy, events, milestones |
| 06-UI-UX.md | Screens, interaction model, presentation standards |
