# Feature Specification: User Interface & Presentation Layer (FA-10)

**Feature Branch**: `011-user-interface`
**Created**: 2026-06-14
**Status**: Draft
**Input**: User description: "Build Sojourn's user interface: the data-dense, mostly-2D presentation layer in the lineage of Aurora 4x and EVE Online, reading entirely from the headless simulation core … persistent shell plus the screens … bespoke widgets … plan→preview→commit … progressive disclosure … full traceability … SI units only … keyboard-rich and accessible … virtualised tables … The UI subscribes to and commands the core; it never owns simulation state."

## Overview

This is **the final slice and the player's entire window into the game** — the data-dense, mostly-2D
presentation layer in the lineage of *Aurora 4x* (depth) and *EVE Online* (readability). It is the first
slice that builds **on top of** the simulation rather than as another headless module: it **reads state
and traceability from the cores exposed by Slices 1–9 and issues commands back**, and contains **no game
logic and no authoritative state of its own** (Constitution IV).

Sojourn lives or dies on **legibility**. The player is here to *think*: to read a thousand numbers, trust
every derived figure (any number drills down to its sourced inputs — the traceability the slices already
expose), plan irreversible manoeuvres with confidence (plan → preview → commit; the sim never surprises
them), and learn the real science as they play (the source-cited Sojournal is always a click away). Text
and tables are first-class, not a fallback; plots and schematic maps carry the spatial story.

The layer is a **desktop-class application** (one window legible from 1280×720 to 4K, the Aurora/EVE feel) —
a **persistent shell** (top bar with date/time-warp/funds/alerts, left nav, central work
area, right inspector, bottom event ticker) hosting **twelve fully-realised screens** (System Map, Trajectory Planner,
R&D, Vehicle Designer, Operations/Fleet, Economy & Contracts, Bases & Construction, Personnel, World/
Politics, Science-Returns & Astrobiology, Sojournal, Alerts/Event Log) plus a set of **bespoke widgets**
(porkchop plot, Δv ladder, TRL ladder, Understanding bars, resource-by-location ledger, logistics-graph
view, base schematic, astrobiology evidence meter). Cross-cutting: SI units everywhere, colour-blind-safe
and scalable and keyboard-navigable, and fast enough to render thousands of catalogued bodies and large
fleets/ledgers (virtualised) at high time-warp.

**Architectural seam (the defining constraint):** the UI is a **pure consumer** of the deterministic core.
It renders the read-only snapshots and **traceability trees** the slices publish, displays the event feed
and interrupts from the FA-01 loop, and submits the same journalled commands the headless harness submits.
It never recomputes physics, never holds the source of truth, and the **same core runs headless for tests**
with no renderer — so the UI can be swapped or evolved without touching gameplay.

## Clarifications

### Session 2026-06-14

- Q: How does the UI stay in sync with the core at high time-warp? → A: **Events + pulled snapshots** — the UI subscribes to the FA-01 event/interrupt feed for "what changed" and pulls fresh read-only snapshots on demand (on event, on navigation, on a throttled tick), rather than re-reading everything every render frame.
- Q: What boundary does the desktop UI consume the core through? → A: **In-process typed queries** — the UI links the slice crates and calls their read-only query/snapshot types directly (`WorldSnapshot`, `CrewSnapshot`, …) and submits typed commands; no serialized view-DTO layer this slice (the boundary is the public query API).
- Q: How is the UI (view layer) itself tested? → A: **View-model tests on stub snapshots** — the UI's display logic (what to show, traceability-tree expansion, plan→preview→commit gating, SI-unit formatting) is tested headlessly against hand-built stub core snapshots, with no renderer; pixel/visual rendering is out of automated scope.
- Q: Which state is core vs UI-only, and how is UI-only state persisted? → A: **Config journalled, view ephemeral** — state-changing config (e.g. pause-policy via the FA-01 `SetPausePolicy` command) is journalled core state; UI-only state (active screen, zoom, pinned inspectors, pre-commit draft plans, layout) is ephemeral/local and **not** part of the deterministic save.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The persistent shell & System Map (Priority: P1) 🎯 MVP

A player opens a running game and sees the whole Solar System: a 2D, logarithmically zoomable, multi-focus
map (heliocentric ↔ planetocentric ↔ local-ops) showing bodies, real orbits, their craft and trajectories,
SOIs, Lagrange regions and transport-graph nodes. Around it sits the persistent shell — a top bar with the
date, time-warp controls and funds/political-capital; a left nav to every screen; a right inspector; and a
bottom event ticker. They pan/zoom, toggle layers (resources, comms, traffic, planetary-protection zones,
science), click a body or craft to inspect it, set the time-warp, and pause.

**Why this priority**: The shell + map is the spine of the entire UI — navigation, the spatial mental
model, time control and the inspector all live here. On its own it is a legible, navigable, pausable window
into a live game: the MVP.

**Independent Test**: Load a deterministic saved/scenario game; the map renders the catalogued bodies and
their real orbits at multiple zoom levels and frames; clicking an object pins its inspector; toggling a
layer changes what is drawn; the time-warp controls advance/pause the core and the displayed date tracks
the core's clock; nothing on screen is computed by the UI (every value traces to a core query).

**Acceptance Scenarios**:

1. **Given** a loaded game, **When** the player zooms from heliocentric to a planet's local-ops view, **Then** the map re-centres and re-scales logarithmically and shows that body's moons/craft/nodes.
2. **Given** the map, **When** the player toggles the comms or planetary-protection layer, **Then** the corresponding overlay appears/disappears without altering any other layer.
3. **Given** a craft on the map, **When** the player clicks it, **Then** the right inspector pins its summary (location, fuel, health, current task) read from the core.
4. **Given** the top bar, **When** the player changes time-warp or pauses, **Then** the core advances at the chosen rate or holds, and the displayed date/clock match the core exactly.
5. **Given** a switch to any other screen via the left nav, **When** the player returns, **Then** the map's focus/zoom/layer state is preserved.

---

### User Story 2 - Traceability & progressive disclosure (Priority: P1)

A player sees a derived number — a vehicle's 6.2 km/s of Δv, a program's risk index, a budget line, a
candidate world's consensus — and asks "why?". They hover for a one-line summary, click to pin a detail
panel, and expand to the **full derivation down to sourced inputs** (mass/Isp breakdown, the cited data
behind each leaf). Newcomers see friendly summaries on top; the full Aurora-grade detail is one click down;
the Sojournal definition of any term is a hover away.

**Why this priority**: Traceability is the trust contract and a defining pillar (Principles II/VIII). A
data-dense UI that the player cannot interrogate is noise; "every number is traceable to a cause" is what
makes the thousand numbers legible. Progressive disclosure is what makes it approachable.

**Independent Test**: For any derived figure on any screen, an inspector can expand it into the
traceability tree the core publishes, every leaf of which shows a value and a non-empty source; the
summary/detail levels toggle; a glossary term resolves to its Sojournal entry — all without the UI
performing the computation itself.

**Acceptance Scenarios**:

1. **Given** a derived value with a core-published traceability tree, **When** the player expands it, **Then** the UI renders the operation nodes and sourced leaves exactly as the core provides them.
2. **Given** a leaf in a derivation, **When** the player inspects it, **Then** its sourced provenance (the `source` citation) is shown.
3. **Given** a newcomer view, **When** the player switches to full detail, **Then** additional columns/derivations appear without changing the underlying values.
4. **Given** any technical term, **When** the player hovers it, **Then** a Sojournal summary is offered with a link to the full entry.

---

### User Story 3 - Plan → preview → commit for irreversible actions (Priority: P1)

Before the player does anything destructive or irreversible — commit a burn, launch a vehicle, cancel a
program, scrap a design, approve a manoeuvre — the UI shows the **consequences** (Δv spent, mass/cash
committed, schedule impact, what becomes unrecoverable) and requires an explicit **confirm**. The player
can build and save a plan, preview it against the current state, and only then commit; the sim never
surprises them with an outcome they could not have seen.

**Why this priority**: Irreversibility is where a strategy game earns or loses the player's trust. Plan →
preview → commit (Principle 3 of the design) is the safety contract that makes high-stakes decisions
feel deliberate rather than punishing. It is woven through every action-bearing screen, so it is foundational.

**Independent Test**: For each irreversible command class, attempting it opens a preview showing the
consequences computed by the core (not the UI) and a confirm/cancel; cancelling submits nothing; confirming
submits exactly the previewed command; a reversible action does not force the preview gate.

**Acceptance Scenarios**:

1. **Given** a planned burn, **When** the player commits it, **Then** a preview first shows Δv spent and the resulting trajectory, and only an explicit confirm submits the command.
2. **Given** a launch, **When** the player initiates it, **Then** the mass/cost committed and the irreversible consequences are shown before confirmation.
3. **Given** a preview, **When** the player cancels, **Then** no command is submitted and the game state is unchanged.
4. **Given** a reversible action (e.g. toggling a map layer), **When** the player performs it, **Then** no confirmation gate is imposed.

---

### User Story 4 - Trajectory / Manoeuvre Planner (Priority: P2)

A mission planner opens the porkchop plot for a transfer window, reads Δv/TOF/C3 contours, picks a
departure/arrival, edits manoeuvre nodes, designs a gravity-assist or a low-thrust arc, and checks the live
**required-Δv against the selected vehicle's available Δv**. They save the plan, queue the burns, and set
auto-pause at the nodes.

**Why this priority**: Trajectory planning is the most distinctive bespoke screen and the embodiment of
"plan transfers on porkchop plots, not on a hex grid." It is P2 because it builds on the P1 shell/map/
preview spine.

**Independent Test**: Given an origin/destination/window from the core's astro queries, the porkchop widget
renders the Δv/TOF contour field the core provides and lets the player pick a point; a manoeuvre-node edit
updates the previewed trajectory; the required-vs-available Δv check reflects the selected vehicle; a saved
plan can be queued as burns with auto-pause — all values from the core.

**Acceptance Scenarios**:

1. **Given** a transfer window, **When** the player opens the porkchop plot, **Then** the Δv/TOF/C3 contours from the core are rendered and a point can be selected.
2. **Given** a selected transfer, **When** the player edits a manoeuvre node, **Then** the trajectory preview and Δv update from the core.
3. **Given** a vehicle with finite Δv, **When** the required Δv exceeds it, **Then** the planner flags the plan as infeasible against that vehicle.
4. **Given** a saved plan, **When** the player queues it, **Then** the burns are scheduled with auto-pause at the nodes (via plan → preview → commit).

---

### User Story 5 - Research & Development (Priority: P2)

A research lead views the science portfolio — Domain Understanding-Level bars with the world-tide ghost and
breakthrough "insight-pressure" hints, active programs, RP allocation — and the engineering programs — TRL
ladders with test-campaign and risk overlays, DE allocation, cost/schedule P50/P80 vs actual, dead-end
warnings — assigns lead personnel, and browses the tech-tree graph with prerequisites and source tags.

**Why this priority**: R&D is where the "research is a process" pillar becomes legible. P2 — a rich
read/allocate dashboard over FA-05 built on the P1 spine.

**Independent Test**: The UL bars, TRL ladders, P50/P80 figures, insight-pressure hints and dead-end
warnings all render from FA-05 queries; an allocation change submits a command and the displayed allocation
reflects the new core state; every node carries its source tag; the tech-tree graph shows prerequisites.

**Acceptance Scenarios**:

1. **Given** the science portfolio, **When** it renders, **Then** each Domain shows its UL bar, the world-tide ghost, and any breakthrough insight-pressure hint from FA-05.
2. **Given** an engineering program, **When** it renders, **Then** its TRL ladder, test-campaign status, P50/P80 vs actual and risk index are shown, with dead-end warnings where present.
3. **Given** RP/DE allocation sliders, **When** the player reallocates, **Then** the change is submitted to the core and the new allocation is reflected.
4. **Given** the tech-tree graph, **When** the player inspects a node, **Then** its prerequisites and `source` tag are shown.

---

### User Story 6 - Vehicle Designer (Priority: P2)

An engineer composes a vehicle from a component picker showing **only researched technologies**, watches
live computed mass/Δv/power/thermal/reliability/cost update with each change, sees **realism red-flags**
when the design violates physics or plausibility, saves it as a reusable class/template, iterates versions,
and compares designs side by side.

**Why this priority**: The designer is the spreadsheet-grade composer that makes the tyranny of mass/Δv
tangible. P2 — a focused tool over FA-04/FA-06 on the P1 spine, with traceability (US2) on every number.

**Independent Test**: The component picker lists only FA-05-researched components; every derived figure
comes from FA-04 and is traceable; a deliberately bad design surfaces the core's realism red-flag; saving
produces a reusable class; two designs render side by side with their differences.

**Acceptance Scenarios**:

1. **Given** the component picker, **When** it renders, **Then** only researched/available components are selectable.
2. **Given** a design change, **When** the player makes it, **Then** mass/Δv/power/thermal/reliability/cost recompute from the core and update live.
3. **Given** an implausible/over-constrained design, **When** it is evaluated, **Then** the core's realism red-flag is surfaced with its reason.
4. **Given** two saved designs, **When** the player compares them, **Then** their derived figures are shown side by side.

---

### User Story 7 - Operations / Fleet (Priority: P2)

A controller opens the "what is everything doing" screen: a filterable/sortable/groupable table of all craft
and assets with status, location, fuel, health, current task, ops-capacity and comms load; mission timelines
(Gantt-like); the launch manifest & cadence planner; and crew assignments with health/dose readouts.

**Why this priority**: Operations is the daily-loop cockpit. P2 — a virtualised table + timelines over many
slices (FA-04/06/07/08) on the P1 spine.

**Independent Test**: With a large fleet from the core, the table virtualises and remains responsive while
filtering/sorting/grouping; each row's figures come from the core; the timeline reflects scheduled tasks;
crew health/dose readouts come from FA-08; a launch-manifest change goes through plan → preview → commit.

**Acceptance Scenarios**:

1. **Given** hundreds of craft/assets, **When** the table renders, **Then** it virtualises and stays responsive to filter/sort/group.
2. **Given** a craft row, **When** the player inspects it, **Then** its location/fuel/health/task/ops/comms come from the core.
3. **Given** crewed assets, **When** the player views them, **Then** crew assignments and health/dose readouts from FA-08 are shown.
4. **Given** the manifest, **When** the player schedules a launch, **Then** it follows plan → preview → commit.

---

### User Story 8 - Economy & Contracts (Priority: P2)

A finance lead views budget/cash-flow dashboards (P&L for companies, appropriation tracker for agencies),
the **resource-by-location ledger** (inventory addressed by Δv-location), market prices & trends, the
contract/RFP board (post/bid), partnerships & licensing, facilities, and learning-curve tracking — and sees
the **logistics-graph view** (nodes = dynamical locations, edges priced in Δv/TOF).

**Why this priority**: Economy makes "money is a proxy for mass-to-orbit" legible and is where the
delta-v-addressed inventory and logistics graph live. P2 — dashboards over FA-06 on the P1 spine.

**Independent Test**: Budgets/ledgers/markets render from FA-06; the resource ledger groups inventory by
location; the logistics graph shows nodes/edges with Δv/TOF prices; posting/bidding a contract goes through
the command path; mood-driven budget modifiers (FA-09) are reflected and traceable.

**Acceptance Scenarios**:

1. **Given** a faction's finances, **When** the dashboard renders, **Then** the appropriate P&L or appropriation view shows the core's figures.
2. **Given** distributed inventory, **When** the resource-by-location ledger renders, **Then** stock is grouped by Δv-location.
3. **Given** the logistics graph, **When** it renders, **Then** nodes are dynamical locations and edges show Δv/TOF.
4. **Given** the RFP board, **When** the player posts or bids, **Then** the command is submitted via plan → preview → commit.

---

### User Story 9 - Bases & Construction (Priority: P2)

A settlement planner browses sites (surveyed properties, resources, hazards, planetary-protection category),
lays out a base/station from modules, and watches the **base schematic with live emergent-property gauges**
(power margin, ECLSS closure, population cap, sustainability index); tracks construction projects (delivery
→ assembly → commissioning) and ISRU plant status.

**Why this priority**: Bases is where FA-07's emergent properties become a readable schematic. P2 — a builder
+ schematic over FA-07/FA-08 on the P1 spine.

**Independent Test**: The site browser shows FA-03 site properties incl. PP category; the base schematic
renders FA-07 emergent-property gauges that update as modules change; construction projects show their
stage; ISRU status comes from FA-06/07; a build action follows plan → preview → commit.

**Acceptance Scenarios**:

1. **Given** surveyed sites, **When** the browser renders, **Then** resources/hazards/illumination/PP-category are shown from the core.
2. **Given** a base layout, **When** a module is added/removed, **Then** the power/closure/population/sustainability gauges recompute from FA-07.
3. **Given** a construction project, **When** it renders, **Then** its delivery/assembly/commissioning stage is shown.
4. **Given** a build/deliver action, **When** the player initiates it, **Then** it follows plan → preview → commit.

---

### User Story 10 - Alerts / Event Log & configurable interrupts (Priority: P2)

A player works under time-warp and the world interrupts them only when it matters. The Alerts/Event Log is a
chronological, filterable feed; each event links to the relevant screen; and the player configures **which
event classes pause the game** (the "increment until something matters" loop). Auto-pause fires on the
configured classes; log-only events accrue without interrupting.

**Why this priority**: This is the surface of the FA-01 interrupt-and-pause loop — the heartbeat of the play
session. P2, but it is what makes high time-warp usable.

**Independent Test**: Events from the core's feed render chronologically and filter by class; clicking an
event navigates to the relevant screen/object; setting an event class to "pause" causes the core to
interrupt on that class while log-only classes do not; acknowledging an interrupt resumes.

**Acceptance Scenarios**:

1. **Given** the event feed, **When** events arrive, **Then** they render chronologically and can be filtered by class.
2. **Given** an event, **When** the player clicks it, **Then** the UI navigates to the related screen/object.
3. **Given** a pause-policy setting, **When** the player marks a class as interrupting, **Then** the core pauses on that class and not on log-only classes.
4. **Given** a pending interrupt, **When** the player acknowledges it, **Then** the game resumes.

---

### User Story 11 - World/Politics & Personnel (Priority: P3)

A player reviews the strategic dashboards: faction relationships & prestige, public/political mood, policy/
treaty state & lobbying, and the **milestone race board** (world-first vs faction-first tracker) with a rival
activity feed (World/Politics); and the personnel pools — scientists/engineers/PMs/astronauts/controllers/
diplomats with skills, traits, assignments, recruitment/training, morale and crew health careers (Personnel).

**Why this priority**: These are read-heavy strategic dashboards over FA-09/FA-05/FA-08. P3 — valuable but
they layer cleanly on the established spine and inspector patterns.

**Independent Test**: The milestone board shows claimed world-/faction-firsts and the unclaimed race from
FA-09; mood/prestige/policy render from FA-09 and are traceable; the personnel pools show FA-05/FA-08 skills/
traits/assignments/health; an assignment or lobby action follows the command path.

**Acceptance Scenarios**:

1. **Given** the milestone board, **When** it renders, **Then** world-first and faction-first claims and the unclaimed race are shown from FA-09.
2. **Given** faction state, **When** it renders, **Then** relationships, prestige, mood and policy/treaty levels are shown and traceable.
3. **Given** the personnel pools, **When** they render, **Then** skills/traits/assignments/morale and crew health careers are shown.
4. **Given** a lobby or assignment, **When** the player acts, **Then** the command is submitted to the core.

---

### User Story 12 - Science Returns & Astrobiology (Priority: P3)

A scientist watches the belief-state per body (Geoscience UL), incoming data, and — the centrepiece — the
**staged astrobiology evidence meter**: a probabilistic, multi-stage, per-candidate-world tracker showing
the community consensus, the per-faction posteriors that can publicly disagree, the abiotic-competitor
hypotheses, and the confidence band — never a binary "life found" popup. The discoveries log records firsts.

**Why this priority**: This is where "are we alone here?" slowly resolves — the honest centrepiece of the
late game. P3 — it surfaces FA-09's astrobiology state through the bespoke evidence meter; valuable but
built on everything before it. It MUST present the question honestly (Principle VIII): probabilistic stages,
no false certainty, the ground truth never shown until conclusive.

**Independent Test**: The evidence meter renders the per-candidate consensus + per-faction posteriors +
conclusive status from FA-09; staged evidence advances the meter; disagreement is shown when factions
differ; the meter never displays a conclusive-positive that FA-09 has not set, and never exposes the hidden
ground truth.

**Acceptance Scenarios**:

1. **Given** a candidate world, **When** the evidence meter renders, **Then** the staged community consensus and the confidence band from FA-09 are shown — not a binary verdict.
2. **Given** divergent per-faction posteriors, **When** they render, **Then** the public disagreement is shown.
3. **Given** accruing evidence, **When** a stage completes, **Then** the meter advances per the core's posterior, with abiotic-competitor context.
4. **Given** an unresolved candidate, **When** the player inspects it, **Then** no ground-truth verdict is shown until the core marks it conclusive.

---

### User Story 13 - The Sojournal encyclopedia (Priority: P3)

A curious player opens the Sojournal: searchable, cross-linked, **source-cited** entries for every body,
technology, resource process, manoeuvre type, mission archetype and discovered result. It updates as the
player's belief-state and the world advance, doubles as a soft tutorial, and is reachable as a hover/click
from any term anywhere in the UI.

**Why this priority**: The Sojournal is the educational-honesty backbone and the deep reference behind
progressive disclosure. P3 — it enriches every screen but the screens function without it; it ships on the
FA-03 Sojournal surface already in the core.

**Independent Test**: Entries render from the FA-03 Sojournal surface with their citations; search and
cross-links resolve; an entry reflects the current belief-state; a term hovered anywhere opens its entry.

**Acceptance Scenarios**:

1. **Given** the Sojournal, **When** the player searches, **Then** matching cross-linked entries are returned.
2. **Given** an entry, **When** it renders, **Then** its real-science explanation and `source` citations are shown.
3. **Given** advancing belief-state, **When** the player reopens an entry (e.g. a partly-explored body), **Then** it reflects what is now known.
4. **Given** any term in any screen, **When** the player hovers/clicks it, **Then** the relevant Sojournal entry opens.

---

### User Story 14 - Accessibility, SI units & performance (Priority: P2)

Every screen uses **SI units only** with consistent formatting; colour is **colour-blind-safe** and never the
sole carrier of meaning; **text scales** and the layout stays readable from 1280×720 to 4K; **full keyboard
navigation** and hotkeys (time-warp, screen switching, common actions) make it power-user friendly; and dense
tables **virtualise** so thousands of catalogued bodies and large fleets/ledgers stay responsive even at high
time-warp.

**Why this priority**: These are non-negotiable constitutional constraints (SI units, accessibility,
performance) that apply to *every* screen, so they are validated as a cross-cutting story rather than buried.
P2 — they must hold as the screens land.

**Independent Test**: A unit audit finds no non-SI units; a colour-blind simulation confirms no
meaning-by-colour-alone; text scaling and 1280×720↔4K layouts remain legible; every primary action has a
keyboard path; a thousands-row table scrolls/filters within the responsiveness budget at high warp.

**Acceptance Scenarios**:

1. **Given** any screen, **When** values are displayed, **Then** they are in SI units with consistent formatting.
2. **Given** a colour-blind-simulation, **When** the UI is reviewed, **Then** no information is conveyed by colour alone.
3. **Given** text-scaling and a 1280×720 to 4K range, **When** applied, **Then** the layout remains readable and usable.
4. **Given** a thousands-row catalogue/fleet/ledger, **When** scrolled/filtered at high time-warp, **Then** it stays within the responsiveness budget via virtualisation.
5. **Given** any primary action, **When** the player uses the keyboard, **Then** there is a keyboard path/hotkey for it.

---

### Edge Cases

- **Core paused vs running**: the UI must be fully usable while the core is paused (plan, inspect, compare) and reflect live changes while running; it must never block the core's stepping.
- **Stale read during a step**: a snapshot read mid-advance must present a consistent core state, never a half-updated mix; the displayed clock/date always matches the snapshot shown.
- **Command rejected by the core**: a submitted command the core rejects (e.g. infeasible burn, insufficient funds) surfaces the rejection reason without the UI pretending it succeeded.
- **Interrupt arrives while on an unrelated screen**: a configured auto-pause interrupt surfaces in the shell regardless of the active screen and offers navigation to the relevant context.
- **Very large catalogues/fleets**: thousands of bodies and large fleets must not degrade the shell; virtualisation and level-of-detail keep the map and tables responsive.
- **Missing/partial belief-state**: an unsurveyed body or unresearched component shows "unknown"/unavailable rather than a fabricated value; the UI never invents data the core has not provided.
- **Derived value with no traceability tree**: if the core exposes a number without a tree, the inspector shows the value and indicates the derivation is unavailable rather than fabricating one.
- **Irreversible action with an out-of-date preview**: if state changed since the preview was generated, committing re-validates against current state (re-preview) rather than applying a stale plan.
- **Accessibility under time pressure**: even at maximum time-warp, the interrupt-and-pause loop gives the player time to read and decide; the UI never requires twitch input.

## Requirements *(mandatory)*

### Functional Requirements

#### The shell & System Map (US1) — FR-UI-1xx

- **FR-UI-101**: The UI MUST present a persistent shell: a top bar (date, time-warp controls, funds/political-capital, alert summary), a left nav to all screens, a central work area, a right inspector panel, and a bottom event/log ticker.
- **FR-UI-102**: The System Map MUST render, in 2D with logarithmic zoom and multi-focus (heliocentric ↔ planetocentric ↔ local-ops), the catalogued bodies and their **real orbits**, the player's craft and their trajectories, SOIs, Lagrange regions and transport-graph nodes — all from core queries.
- **FR-UI-103**: The map MUST support toggleable layers (resources, comms/relay coverage, traffic, planetary-protection zones, science) and a frame selector (inertial/rotating).
- **FR-UI-104**: Clicking a body/craft/site MUST pin its inspector (summary from the core); right-clicking MUST offer context actions (inspect, plan, survey, assign, compare) appropriate to the object.
- **FR-UI-105**: The time-warp controls MUST drive the core's stepping (1 s/s up to high warp) and pause; the displayed date/clock MUST always match the core's clock exactly.
- **FR-UI-106**: Screen/focus/zoom/layer state MUST persist across navigation within a session.

#### Traceability & progressive disclosure (US2) — FR-UI-2xx

- **FR-UI-201**: Any derived value the core exposes with a **traceability tree** MUST be expandable in an inspector into that tree, rendering its operation nodes and **sourced leaves** exactly as the core provides them (the UI MUST NOT compute the value itself).
- **FR-UI-202**: Each sourced leaf MUST display its provenance (the `source` citation); a leaf lacking a source MUST be visibly flagged (it is a core data defect, surfaced not hidden).
- **FR-UI-203**: Every screen MUST offer **progressive disclosure**: a newcomer summary level and a full-detail level, switchable without changing the underlying values.
- **FR-UI-204**: Any technical term MUST be linkable to its Sojournal entry via hover/click.

#### Plan → preview → commit (US3) — FR-UI-3xx

- **FR-UI-301**: Every **irreversible/destructive** action (burn, launch, cancellation, scrap, manoeuvre approval, build commit) MUST present a **preview of consequences computed by the core** and require an explicit confirm before submitting any command.
- **FR-UI-302**: Cancelling a preview MUST submit nothing and leave the game state unchanged.
- **FR-UI-303**: Confirming MUST submit exactly the previewed command; **reversible** actions MUST NOT impose the confirmation gate.
- **FR-UI-304**: If state changed since a preview was generated, committing MUST re-validate/re-preview against current state rather than apply a stale plan.
- **FR-UI-305**: A command the core **rejects** MUST surface the rejection reason; the UI MUST NOT present a rejected command as having succeeded.

#### Trajectory / Manoeuvre Planner (US4) — FR-UI-4xx

- **FR-UI-401**: The planner MUST render an interactive **porkchop plot** (Δv/TOF/C3 contours from the core) for a selected window and let the player pick a departure/arrival.
- **FR-UI-402**: A **manoeuvre-node editor**, a **low-thrust arc planner**, and a **flyby/gravity-assist designer** MUST update the previewed trajectory and Δv from the core as the player edits.
- **FR-UI-403**: The planner MUST show the **required Δv against the selected vehicle's available Δv** and flag infeasibility.
- **FR-UI-404**: A saved plan MUST be queueable as burns with **auto-pause at nodes**, via plan → preview → commit.

#### Research & Development (US5) — FR-UI-5xx

- **FR-UI-501**: The science portfolio MUST render **Domain Understanding-Level bars** with the world-tide ghost and breakthrough insight-pressure hints, active programs and RP allocation, from FA-05.
- **FR-UI-502**: The engineering view MUST render **TRL ladders** with test-campaign and risk overlays, DE allocation, cost/schedule **P50/P80 vs actual**, and dead-end warnings.
- **FR-UI-503**: RP/DE allocation changes and lead-personnel assignments MUST submit commands and reflect the new core state.
- **FR-UI-504**: A **tech-tree graph** MUST show nodes, prerequisites and `source` tags.

#### Vehicle Designer (US6) — FR-UI-6xx

- **FR-UI-601**: The component picker MUST list **only researched/available** components (from FA-05).
- **FR-UI-602**: Mass/Δv/power/thermal/reliability/cost MUST recompute from FA-04 and update live as the design changes, each figure traceable (US2).
- **FR-UI-603**: The core's **realism red-flags** MUST be surfaced with their reasons when a design violates physics/plausibility.
- **FR-UI-604**: Designs MUST be saveable as reusable classes/templates, versionable, and **comparable side by side**.

#### Operations / Fleet (US7) — FR-UI-7xx

- **FR-UI-701**: A **virtualised** table of all craft/assets MUST support filter/sort/group and show status, location, fuel, health, current task, ops-capacity and comms load from the core.
- **FR-UI-702**: Mission **timelines** (Gantt-like) and the **launch manifest & cadence planner** MUST render from the core; manifest changes follow plan → preview → commit.
- **FR-UI-703**: Crew assignments and **health/dose readouts** MUST render from FA-08.

#### Economy & Contracts (US8) — FR-UI-8xx

- **FR-UI-801**: Budget/cash-flow dashboards MUST render the appropriate **P&L (companies)** or **appropriation tracker (agencies)** from FA-06, with mood-driven modifiers (FA-09) reflected and traceable.
- **FR-UI-802**: The **resource-by-location ledger** MUST group inventory by Δv-addressed location; market prices/trends MUST render from FA-06.
- **FR-UI-803**: The **logistics-graph view** MUST show nodes (dynamical locations) and edges priced in Δv/TOF.
- **FR-UI-804**: The contract/RFP board MUST support post/bid, facilities and learning-curve tracking; contract actions follow plan → preview → commit.

#### Bases & Construction (US9) — FR-UI-9xx

- **FR-UI-901**: The **site browser** MUST show surveyed properties, resources, hazards and planetary-protection category from FA-03.
- **FR-UI-902**: The **base schematic** MUST render live emergent-property gauges (power margin, ECLSS closure, population cap, sustainability index) from FA-07 that recompute as modules change.
- **FR-UI-903**: Construction projects MUST show their delivery → assembly → commissioning stage and ISRU plant status; build/deliver actions follow plan → preview → commit.

#### Alerts / Event Log & interrupts (US10) — FR-UI-10xx

- **FR-UI-1001**: The Alerts/Event Log MUST render the core's event feed chronologically with filter-by-class; each event MUST link to the relevant screen/object.
- **FR-UI-1002**: The player MUST be able to configure **which event classes pause the game** via the journalled core pause-policy command (FR-UI-1505); the core MUST auto-pause on configured classes and not on log-only classes.
- **FR-UI-1003**: A pending interrupt MUST surface in the shell regardless of the active screen; acknowledging it MUST resume the game.

#### World/Politics & Personnel (US11) — FR-UI-11xx

- **FR-UI-1101**: The **milestone race board** MUST show world-first vs faction-first claims and the unclaimed race from FA-09, plus a rival activity feed.
- **FR-UI-1102**: Faction relationships, prestige, public/political mood and policy/treaty state MUST render from FA-09 and be traceable; lobbying actions follow the command path.
- **FR-UI-1103**: The **personnel pools** (scientists/engineers/PMs/astronauts/controllers/diplomats) MUST show skills, traits, assignments, recruitment/training, morale and crew health careers from FA-05/FA-08.

#### Science Returns & Astrobiology (US12) — FR-UI-12xx

- **FR-UI-1201**: The **astrobiology evidence meter** MUST render the per-candidate community consensus, per-faction posteriors (incl. public disagreement) and confidence band from FA-09 — **never a binary "life found" popup**.
- **FR-UI-1202**: The meter MUST advance with staged evidence and show abiotic-competitor context; it MUST **never display a conclusive-positive the core has not set** and **never expose the hidden ground truth** before conclusion (Principle VIII).
- **FR-UI-1203**: The belief-state per body (Geoscience UL), incoming data and a discoveries log MUST render from the core.

#### Sojournal (US13) — FR-UI-13xx

- **FR-UI-1301**: The Sojournal MUST present searchable, cross-linked, **source-cited** entries from the FA-03 Sojournal surface that update with the belief-state/world.
- **FR-UI-1302**: Any term anywhere in the UI MUST be able to open its Sojournal entry.

#### Accessibility, units & performance (US14) — FR-UI-14xx

- **FR-UI-1401**: All displayed quantities MUST be in **SI units** with consistent formatting; no imperial units anywhere.
- **FR-UI-1402**: Colour MUST be **colour-blind-safe** and MUST NOT be the sole carrier of any information.
- **FR-UI-1403**: Text MUST be scalable and the layout legible/usable from **1280×720 to 4K**.
- **FR-UI-1404**: Every primary action MUST have a **keyboard path/hotkey**; the UI MUST be fully keyboard-navigable and MUST NOT require twitch input.
- **FR-UI-1405**: Dense tables and the map MUST **virtualise / level-of-detail** so thousands of bodies and large fleets/ledgers stay interactive at high time-warp — the **responsiveness budget is a UI frame time ≤ 16 ms (≈ 60 fps) with no input-to-redraw stall > 100 ms** (SC-006).

#### Cross-cutting architecture — FR-UI-15xx

- **FR-UI-1501**: The UI MUST contain **no game logic and no authoritative simulation state**; every displayed value MUST originate from a core query/snapshot/event, and every state change MUST be a command submitted to the core (Constitution IV).
- **FR-UI-1502**: The UI MUST read the core through its **defined boundary** — the slices' **read-only typed query/snapshot/traceability surfaces** (consumed **in-process**, e.g. `WorldSnapshot`/`CrewSnapshot`) plus the FA-01 event/interrupt feed — and submit **typed journalled commands**; it MUST NOT depend on slice internals, and the **same core MUST run headless without the UI** for tests. (No serialized view-DTO layer this slice; the public query API is the seam.)
- **FR-UI-1503**: The UI MUST stay in sync via **events + pulled snapshots**: it subscribes to the FA-01 event/interrupt feed and pulls fresh read-only snapshots **on demand** (on event, on navigation, on a throttled tick), MUST present a **consistent core state** (a coherent snapshot + matching clock) never a half-updated mix, and MUST never block or alter the core's deterministic stepping.
- **FR-UI-1504**: The UI MUST NOT fabricate data the core has not provided: missing/unknown/unsurveyed values MUST show as unknown/unavailable, not invented.
- **FR-UI-1505**: All **state-changing configuration** (e.g. the pause-policy) MUST be a **journalled core command** (FA-01 `SetPausePolicy`), never UI-only authoritative state. UI-only state (active screen, zoom, pinned inspectors, **pre-commit draft plans**, layout) is **ephemeral/local** and MUST NOT be part of the deterministic save.
- **FR-UI-1506**: The UI's display logic — what to show, traceability-tree expansion, plan→preview→commit gating, SI-unit formatting — MUST be **testable headlessly against stub core snapshots** (no renderer); the view layer MUST stay off the determinism/headless gameplay path.

### Key Entities

- **Shell**: the persistent frame — top bar (clock/time-warp/funds/alerts), left nav, central work area, right inspector, bottom ticker — and the active-screen/focus/layer session state (UI-only, not authoritative).
- **Screen**: one of the twelve work areas (System Map, Trajectory Planner, R&D, Vehicle Designer, Operations, Economy, Bases, Personnel, World/Politics, Science-Returns/Astrobiology, Sojournal, Alerts).
- **Inspector**: a pinnable detail panel that renders a core value and (when available) its **traceability tree** to sourced leaves, at a summary or full-detail level.
- **Bespoke widget**: porkchop plot, Δv ladder, TRL ladder, Understanding bars, resource-by-location ledger, logistics-graph view, base schematic, astrobiology evidence meter — each a renderer of a specific core data shape.
- **Plan/Preview**: a UI-side draft of an irreversible action plus the core-computed consequence preview, pending an explicit commit (UI-only until committed).
- **Command**: the journalled action the UI submits to the core (the same envelope the headless harness uses).
- **View read**: a read-only snapshot/query result (per slice) + the FA-01 event/interrupt feed the UI subscribes to; never owned, never mutated by the UI.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A player can, from a cold open, identify a body on the System Map, inspect it, and read its key properties **within 30 seconds**, with every displayed value traceable to a core query.
- **SC-002**: **100% of derived numbers** shown anywhere in the UI either expand to the core's traceability tree (with every leaf sourced) or are explicitly flagged as having no available derivation — none are computed by the UI.
- **SC-003**: **100% of irreversible actions** present a core-computed consequence preview and require explicit confirmation before any command is submitted; cancelling submits nothing.
- **SC-004**: The same simulation core runs **headless with no renderer** in the test suite (the UI is not on the determinism/headless path), and the UI's display logic is covered by **view-model tests over stub core snapshots** — proving the decoupling.
- **SC-005**: A player can plan a transfer on the porkchop plot, check it against a vehicle's available Δv, and queue the burns with auto-pause — entirely through plan → preview → commit.
- **SC-006**: A **thousands-row** catalogue/fleet/ledger and a thousands-body map remain interactive **at the highest time-warp** via virtualisation/level-of-detail — **interactive = a UI frame time ≤ 16 ms (≈ 60 fps) on commodity hardware, with no input-to-redraw stall > 100 ms** during scroll/filter/pan/zoom.
- **SC-007**: A **unit audit finds zero non-SI units**, and a colour-blind simulation finds **no information conveyed by colour alone**, across every screen.
- **SC-008**: Every primary action has a **keyboard path**; the UI is fully navigable from the keyboard with no twitch-input requirement.
- **SC-009**: The astrobiology evidence meter **never** displays a conclusive-positive the core has not set and **never** exposes a candidate's hidden ground truth before conclusion (an honesty audit passes).
- **SC-010**: Every screen renders **only** from the slices' published read-only surfaces + the event feed and submits **only** journalled commands — verified by an architecture audit showing no game logic and no authoritative state in the view layer.
- **SC-011**: The UI remains **fully usable while the core is paused** (plan/inspect/compare) and reflects live changes while running, never blocking the core's stepping.
- **SC-012**: A newcomer can complete the core loop (inspect → plan a transfer → commit a launch → respond to an interrupt) using on-screen progressive-disclosure summaries and the Sojournal, without external documentation.

## Assumptions

- **Pure-consumer architecture (the defining constraint)**: the UI depends on the slices' read-only query/snapshot/traceability surfaces and the FA-01 event/interrupt feed, and submits journalled commands; it holds no authoritative state and no game logic (Constitution IV). This is assumed, not re-litigated.
- **Traceability is core-provided**: the slices already expose `TraceTree`-style derivations and `source`-cited data; the UI renders them rather than reconstructing them. Where a core surface does not yet expose a needed read, that read is added to the **core's** query surface (a small, headless, tested addition), never recomputed in the UI.
- **The Sojournal is FA-03's surface**: the encyclopedia content/citations live in the core; the UI is a reader.
- **SI units, accessibility, determinism-decoupling** are constitutional and non-negotiable (Engineering Constraints + Principle IV).
- **Platform target (confirmed)**: a **desktop-class native application** — one window legible from 1280×720 to 4K (the Aurora 4x / EVE Online desktop feel). The exact UI tech stack remains a `/speckit-plan` decision (the constitution leaves it open); the spec stays stack-agnostic.
- **Slice depth (confirmed)**: **all twelve screens are fully implemented** this slice — read + command + the bespoke widgets — not a scaffolded subset. This is, by design, the largest slice by surface.
- **Onboarding (confirmed)**: **no onboarding affordances** this slice — no contextual tips, no guided on-rails "first agency" tutorial, no historical-scenario tutorials. **Progressive disclosure** (newcomer summary vs full detail) and the **Sojournal-as-reference** remain in scope as core legibility features, not as tutorials.
- **No new gameplay**: this slice adds presentation only; if a screen needs a value the core does not expose, the gap is closed by extending the relevant slice's **headless** query surface (with tests), not by adding logic to the UI.

## Dependencies

- **FA-01 (core)**: the deterministic kernel's status/clock, the event/interrupt-and-pause feed, command submission, and save/load — the boundary the UI subscribes to and commands.
- **FA-02…FA-09 (all gameplay slices)**: each slice's read-only query/snapshot surfaces and traceability trees are the data the corresponding screens render (astro/world → map & planner & Sojournal; research → R&D; vehicle → designer; economy → economy; base → bases; crew → personnel/ops; polity → world/politics & astrobiology & milestones).
