# SOJOURN — Part 6: UI / UX & Presentation

> Target feel: **Aurora 4x's depth + EVE Online's readability**. A clean, data-dense, mostly-2D
> interface where text and tables are first-class, plots are everywhere, and every number is
> traceable to a cause. No twitch input; everything is plan-and-confirm. SI units only.

---

## 1. Presentation principles

1. **Information-first.** Tables, plots and schematic maps over spectacle. Lots of text is fine
   and expected; the player is here to *think*.
2. **Traceability.** Any derived number (a vehicle's Δv, a program's risk, a budget line) is
   inspectable down to its inputs ("why is this 6.2 km/s?" → expand the mass/Isp breakdown).
3. **Plan → preview → commit.** Destructive/irreversible actions (burns, launches, cancellations)
   always show consequences and a confirm step. The sim never surprises you with something you
   couldn't have seen.
4. **Pause-friendly.** The game is fully playable paused; time-warp is a tool, not a pressure.
   Configurable interrupts surface what matters (00-OVERVIEW.md §5).
5. **Progressive disclosure.** Newcomer-friendly summaries on top; full Aurora-grade detail one
   click down. The Sojournal (05-WORLD.md §8) is always a hover/click away.
6. **SI units, consistent formatting, colour-blind-safe palettes, scalable text** (accessibility).

## 2. Top-level structure

A persistent shell: top bar (date, time-warp controls, funds/political-capital, alerts), left
nav to major screens, central work area, right context/inspector panel, bottom event/log ticker.

Major screens:

### S1 — System Map (the hero screen)
2D, logarithmically zoomable, multi-focus: heliocentric ↔ planetocentric ↔ local-ops. Shows
bodies, real orbits, your craft & their trajectories, SOIs, Lagrange regions, transport-graph
nodes, comms/relay coverage. Layers toggle (resources, comms, traffic, planetary-protection
zones, science). Click a craft/body/site to inspect; right-click to plan. Frame selector
(inertial/rotating). Trajectory overlays with Δv/TOF labels.

### S2 — Trajectory / Manoeuvre Planner
Porkchop plots, manoeuvre-node editor, low-thrust arc planner, flyby/assist designer, aerocapture
planner. Live Δv vs available-Δv check against the selected vehicle. Save plans, queue burns,
auto-pause at nodes.

### S3 — Research & Development
Two linked views: **Science portfolio** (Domain UL bars, active research programs, RP allocation
sliders, world-tide deltas, breakthrough "insight pressure" hints) and **Engineering programs**
(TRL ladders, test-campaign status, DE allocation, cost/schedule P50/P80 vs actual, risk index,
dead-end warnings). Assign lead personnel. Tech-tree graph view with prereqs and source tags.

### S4 — Vehicle Designer
The spreadsheet-grade composer (04-SPACEFLIGHT.md §4): component picker (only researched techs),
live computed mass/Δv/power/thermal/reliability/cost, realism red-flags, save as reusable
class/template, version & iterate. Side-by-side design comparison.

### S5 — Operations / Fleet
Table of all craft & assets (filter/sort/group), status, location, fuel, health, current task,
ops-capacity & comms load. Mission timelines (Gantt-like), launch manifest & cadence planner,
crew assignments & health/dose readouts. The "what is everything doing" screen.

### S6 — Economy & Contracts
Budget/cash-flow dashboards (P&L for companies, appropriation tracker for agencies), resource
ledgers by **location** (delta-v address), market prices & trends, contract/RFP board (post/bid),
partnerships & IP/licensing, facilities (capex/opex/capacity/upgrades), learning-curve tracking.

### S7 — Bases & Construction
Site browser (surveyed properties, resources, hazards, PP category), base/station builder
(module layout, emergent properties: power margin, ECLSS closure, population cap, sustainability
index), construction-project tracker (delivery sequence → assembly → commissioning), ISRU plant
status.

### S8 — Personnel
Scientists/engineers/PMs/astronauts/controllers/diplomats: pools, skills, traits, assignments,
recruitment/training/poaching, morale & retention, crew pipeline & health careers.

### S9 — World / Politics
Faction relationships & prestige, public/political mood, policy/treaty state & lobbying, the
milestone race board (world-first vs faction-first tracker), rival activity feed.

### S10 — Science Returns & Astrobiology
Belief-state per body (Geoscience UL), incoming data, the staged astrobiology evidence tracker
(probabilistic, per candidate world), discoveries log. Where "are we alone here?" slowly resolves.

### S11 — Sojournal (encyclopedia)
Searchable, cross-linked, source-cited entries for everything; doubles as soft tutorial & the
educational-honesty layer.

### S12 — Alerts / Event Log
Chronological, filterable feed; each event links to the relevant screen; configurable which event
classes pause the game.

## 3. Interaction model

- **Mouse-first, keyboard-rich**: hotkeys for time-warp, screen switching, common actions;
  power-user friendly (EVE/Aurora audience).
- **Inspectors everywhere**: hover for summary, click to pin a detail panel, expand for the full
  derivation.
- **Context actions**: right-click any object for verbs (survey, plan transfer, assign, inspect,
  compare).
- **Queues & automation**: standing orders (resupply this base, keep this depot above X tonnes,
  auto-correct station-keeping), so the late game doesn't drown in micromanagement — with always
  the option to take manual control.

## 4. Key custom widgets (worth bespoke design)

- **Porkchop plot** (interactive Δv/TOF/C3 contour picker).
- **Δv ladder / budget bar** (per-vehicle, live against plan).
- **TRL ladder** with test-campaign and risk overlays.
- **Domain UL bars** with world-tide ghost and breakthrough-pressure shimmer.
- **Resource-by-location ledger** (the delta-v-addressed inventory).
- **Logistics-graph view** (nodes = dynamical locations, edges = transfers priced in Δv/TOF).
- **Base schematic** with live emergent-property gauges (power/closure/sustainability).
- **Astrobiology evidence meter** (probabilistic, multi-stage, per candidate world).

## 5. Onboarding

Layered: (a) guided "first agency" scenario teaching the core loop on rails; (b) contextual tips
tied to first use of each system; (c) the Sojournal as the deep reference; (d) optional historical
scenarios (Artemis-era cislunar, a Mars-window campaign) as structured tutorials. Difficulty &
assist toggles (e.g., trajectory-solver help on/off) let the hardcore turn off training wheels.

## 6. Tech & rendering notes (for implementers)

- 2D vector/Canvas/WebGL2-class rendering for maps/plots; the heavy lifting is data viz, not 3D.
- UI is **decoupled from the deterministic sim core** (constitution): the core emits state; the
  UI reads it. The same core runs headless for tests. No game logic in the view layer.
- Must stay readable from ~1280×720 up to 4K; dense tables need virtualisation for thousands of
  rows (asteroid catalogue, fleet, ledger). Colour-blind-safe, scalable fonts, full keyboard nav.
