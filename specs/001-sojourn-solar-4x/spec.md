# Feature Specification: Sojourn — Hard-Science Solar-System 4X (v1.0 Umbrella)

**Feature Branch**: `001-sojourn-solar-4x`
**Created**: 2026-06-12
**Status**: Draft
**Input**: User description: Build "Sojourn", a single-player, pausable real-time hard-science 4X strategy game about the exploration, industrialisation and settlement of the Solar System, January 2026 to 2126. No combat, no weapons, no aliens; competition is a race for historic "firsts", economics and politics. Authoritative sources: `design/00-OVERVIEW.md` … `design/06-UI-UX.md` and `.specify/memory/constitution.md` (v1.0.0).

> **Role of this document.** This is the **umbrella specification** for Sojourn v1.0. It fixes the
> whole-game scope, the decomposition into ten bounded feature areas, the cross-cutting
> requirements every area must obey, and the game-level success criteria. Each feature area is
> sized to be specified and planned separately (`/speckit.specify` per area) under this umbrella;
> per-area specs refine — never contradict — what is written here and in the constitution.
> This specification deliberately chooses **no technology stack**.

---

## Feature Area Decomposition

The scope is partitioned into ten bounded feature areas. Each has a stable identifier used to
prefix its functional requirements, an explicit boundary, and declared dependencies so areas can
be specified, planned and implemented as separate features.

| ID | Feature area | One-line scope | Depends on |
|----|--------------|----------------|------------|
| FA-01 | **Simulation Core & Time** | Deterministic, seeded, headless simulation kernel: fixed-timestep continuous time, event-driven time-warp with interrupt-and-pause, event log, replay, save/load | — |
| FA-02 | **Astrodynamics & Flight** | N-body propagator (source of truth) + patched-conic planning aids; manoeuvre nodes, porkchop plots, flybys, low-thrust, low-energy transfers, aerocapture/EDL flight execution | FA-01, FA-03 |
| FA-03 | **World Data & Belief State** | Solar-System catalogue (bodies, moons, small bodies, dynamical locations), sites, ground-truth vs belief-state, planetary protection, seeded astrobiology question | FA-01 |
| FA-04 | **Vehicle Designer & Propulsion** | Component-composition designer with computed, traceable performance; propulsion physical models across all families; realism guards | FA-01, FA-05 |
| FA-05 | **Research & Personnel** | Two-track research (Understanding Levels → TRL-gated engineering programs), dead ends, breakthroughs, leapfrogging, global science tide, publish/patent, personnel management | FA-01 |
| FA-06 | **Economy, Markets & Logistics** | Six currencies, agency vs private funding, resources with delta-v addresses, ISRU economics, launch market, contracts/RFPs, partnerships, IP, transport graph, ops/comms capacity, facilities | FA-01, FA-03 |
| FA-07 | **Bases, Construction & ISRU Operations** | Construction projects, station/base modules with emergent properties, regolith construction, ISRU plants, self-sufficiency | FA-03, FA-06 |
| FA-08 | **Life Support & Crew** | Consumables/ECLSS closure, radiation, physiology, psychology, spares, crew pipeline; the crewed-difficulty multiplier | FA-01, FA-05 |
| FA-09 | **Politics, Events, Milestones & AI World** | Relationships, prestige, mood, policy/treaties, seeded+state-driven events, ~120 firsts, Grand Goals, scoring, AI factions, soft-fail states | FA-01, FA-06 |
| FA-10 | **UI Shell, Screens & Sojournal** | The data-dense 2D presentation layer: all twelve screens, traceability inspectors, plan→preview→commit, accessibility, onboarding, the source-cited encyclopedia | All (read-only consumer) |

Cross-cutting requirements that bind every area carry the prefix **FR-XCU** (see Requirements).

**Recommended specification order** (driven by dependencies and by the user-story priorities
below): FA-01 → FA-03 → FA-02 → FA-05 → FA-04 → FA-06 → FA-10 (incremental, alongside others) →
FA-07 → FA-08 → FA-09.

---

## User Scenarios & Testing *(mandatory)*

The actor in all scenarios is **the player**, who runs one of ten organisations (five real
national agencies, five fictional private companies) from January 2026. Stories are ordered so
that the P1 set alone is a viable, demonstrable kernel of the game (a seeded, deterministic world
you can watch, pause, and fly missions in), with each later story adding an independently
testable layer.

### User Story 1 - Run the World: Time, Interrupts, Saves (Priority: P1)

The player starts a new game with a chosen seed and faction in January 2026, sets time
acceleration, and the simulation advances continuously until something that matters happens — a
manoeuvre node, an anomaly, a budget vote, a discovery — at which point the game interrupts and
pauses. The player reviews the event, acts, and resumes. At any point they save; loading the save
reproduces the exact same world; replaying the same seed with the same decisions produces the
identical game.

**Why this priority**: This is the kernel everything else runs inside. Without deterministic
time, events, and persistence there is no game to attach any other system to. It is also the
constitution's hardest guarantee (Principles III, IV).

**Independent Test**: Can be fully tested headlessly — run a seed plus a scripted decision list
twice and compare final state for identity; save mid-run and verify the loaded state matches the
uninterrupted run; verify warp interrupts fire on each configured event class.

**Acceptance Scenarios**:

1. **Given** a new game with seed S and faction F, **When** the same seed, faction and recorded decision sequence are run twice (headless), **Then** the resulting game states are identical at every comparison checkpoint.
2. **Given** the game running at any warp factor with an upcoming manoeuvre node, **When** simulated time reaches the node, **Then** the simulation interrupts, pauses, and surfaces the node before any of its effects are applied.
3. **Given** a save file created mid-mission, **When** it is loaded, **Then** the restored state is identical to the state at save time and the subsequent simulation evolves identically to an unbroken run.
4. **Given** the player has configured "rival milestone" events not to pause, **When** a rival achieves a first, **Then** the event is logged and surfaced without interrupting warp.
5. **Given** warp at the maximum rate (order of 1 year/minute), **When** burns, EDL or docking occur, **Then** those phases resolve at fine timestep with no loss of fidelity relative to running them at 1× speed.

---

### User Story 2 - Plan and Fly a Mission (Priority: P1)

The player opens the trajectory planner for a probe in Earth orbit, examines a porkchop plot for
the next Mars window, places manoeuvre nodes, previews the trajectory with delta-v and
time-of-flight labels against the vehicle's actual propellant budget, commits the plan, and
warps. The simulation flies the plan with real physics: finite burns, execution error, and
perturbations create small divergences the player corrects with mid-course burns. Arrival
conditions feed an aerocapture or landing the player has to have designed for.

**Why this priority**: Physics is the gameplay (Principle II). Trajectory planning is the
player's core spaceflight verb and the most distinctive mechanical promise of the product.

**Independent Test**: With a stub vehicle from data files (no designer needed), plan and execute
a Hohmann-class transfer headlessly; verify planned vs flown reconciliation, propellant
accounting, and that analytic validation cases pass within tolerance.

**Acceptance Scenarios**:

1. **Given** a vehicle in LEO with known mass and propulsion, **When** the player plans a transfer on the porkchop plot and places nodes, **Then** the displayed delta-v, time-of-flight and arrival conditions are consistent with the authoritative propagator within the documented planning-aid tolerance, and any propellant shortfall is flagged before commit.
2. **Given** a committed multi-burn plan, **When** the simulation flies it, **Then** each burn debits propellant from the specified tanks, finite-burn and steering losses are applied, and accumulated divergence is surfaced with a correction-burn (TCM) suggestion.
3. **Given** a vehicle with electric propulsion, **When** the player plans a spiral from LEO outward, **Then** the planner produces a months-long continuous-thrust arc whose duration and propellant use reflect the power-limited thrust model.
4. **Given** a planned flyby of Venus, **When** the player uses the assist designer, **Then** the resulting post-flyby trajectory matches the propagated truth within planning tolerance, and the test suite's analytic flyby case passes.
5. **Given** an arrival at Mars with an aerocapture plan, **When** the vehicle's heat shield or entry corridor is inadequate, **Then** the risk is quantified in the preview and a failed capture produces a physically consistent outcome (heating loss or fly-through), never an arbitrary one.

---

### User Story 3 - Know the World Before You Bet On It (Priority: P1)

The player examines the Solar System map: every planet, ~150 moons and ~3,000 catalogued
asteroids move on real orbits. Candidate mining and base sites show estimated — not true —
properties with error bars. The player sends a prospector to a lunar polar site; the survey
narrows the ice-grade estimate, which turns out worse than hoped. A rival lands a non-sterile
craft in a Mars Special Region and takes a public reputational and scientific hit.

**Why this priority**: The world is the board. The belief-state-vs-ground-truth distinction, real
ephemerides and the planetary-protection regime are load-bearing for astrodynamics, economy,
and the astrobiology endgame alike.

**Independent Test**: Headlessly query the world model: verify ephemeris positions against
reference data for sample epochs across 2026–2126; verify that survey actions monotonically
narrow site uncertainty toward seeded ground truth; verify contamination consequences fire.

**Acceptance Scenarios**:

1. **Given** any catalogued body and any date in 2026–2126, **When** its position is queried, **Then** it matches the real-data-derived ephemeris within the documented accuracy, and launch windows/assist opportunities derived from it are physically correct.
2. **Given** an unsurveyed site with seeded ground truth G, **When** the player runs successive survey missions with appropriate instruments, **Then** each survey narrows the estimate's error bars and estimates converge toward G, while the unsurveyed display never reveals G directly.
3. **Given** a body region with planetary-protection category IV/Special Region status, **When** a mission that does not meet the bioburden requirement lands (or crashes) there, **Then** science value for the affected astrobiology question degrades, the responsible faction pays reputation/political costs, and the event is recorded permanently.
4. **Given** the statistical prospecting layer for the main belt, **When** the player runs a survey campaign, **Then** statistical fields convert into newly catalogued, individually addressable targets consistent with the seeded distribution.

---

### User Story 4 - Research Is a Process, Not a Purchase (Priority: P2)

The player funds plasma-physics science for years, raising the domain's Understanding Level.
That UL unlocks the option to start a high-power Hall thruster engineering program. The program
advances through TRL gates via a test campaign: one test fails ("failure-that-teaches" — schedule
slips, understanding rises), costs overrun realistically, and a stalled error bar hints the
chosen approach may be a seeded dead end. The player pivots to a parallel approach, reaches a
flight demo (TRL 7), and the technology becomes usable — initially unreliable, maturing with
flight heritage. Meanwhile a sustained basic-science bet pays off with a rare seeded breakthrough,
and the global tide means a trailing rival catches up cheaply on published results.

**Why this priority**: The two-track research model is a defining pillar (Principle VI) and gates
the entire tech tree, designer and progression arc. It depends only on the sim core.

**Independent Test**: Headless script: allocate research over simulated years and assert UL
growth with diminishing returns; run an engineering program through TRL gates asserting cost/
schedule variance, failure-UL injection, dead-end hinting; verify across many seeds that every
capability category retains ≥1 viable path.

**Acceptance Scenarios**:

1. **Given** a domain below a program's UL floor, **When** the player attempts to start that program, **Then** it is unavailable, with the gating UL requirement visible.
2. **Given** an active engineering program, **When** a test fails, **Then** funds/schedule are lost, domain UL increases, and the program's risk index updates — and repeated failures without UL growth raise an explicit dead-end warning.
3. **Given** a technology at TRL 6, **When** it is placed on a flying vehicle, **Then** the vehicle's reliability reflects a steep immaturity penalty that shrinks as TRL and flight heritage grow.
4. **Given** sustained heavy investment in a basic-science domain, **When** the seeded insight-pressure threshold is crossed, **Then** a breakthrough event fires with one of the defined effects (cluster discount, early unlock, or dead-end bypass) and a sourced Sojournal entry.
5. **Given** any game seed, **When** all candidate approaches to one capability category are examined, **Then** at least one is viable (seeding never bricks a category).
6. **Given** a faction trailing the World UL in a domain, **When** it invests there, **Then** its research efficiency in that domain is higher than the frontier leader's (catch-up effect), and publishing/patenting choices shift World UL and licensing income as designed.

---

### User Story 5 - Design a Vehicle You Can Trust (or Knowingly Gamble On) (Priority: P2)

The player composes a reusable lunar lander in the vehicle designer from researched components:
tanks, a deep-throttle methalox engine, avionics, landing gear. The designer live-computes mass,
per-stage delta-v, thrust-to-weight at the Moon, power and thermal margins, reliability, cost and
build time. Every derived number expands to its inputs. The designer red-flags a configuration
whose radiators are too small and refuses one whose thrust cannot lift its own mass — and lets
the player fly a marginal-but-possible design with honest risk numbers.

**Why this priority**: The Aurora-style designer is the bridge from research to operations and
the main expression of the rocket equation as a felt constraint (Principle VII).

**Independent Test**: Headless designer API: compose reference vehicles and assert computed
performance against hand-calculated values; assert realism guards trigger on each defined
impossible-configuration class; assert traceability returns a complete input tree for any output.

**Acceptance Scenarios**:

1. **Given** a set of researched components, **When** the player composes them, **Then** computed wet/dry mass, per-stage delta-v, T/W per gravity field of interest, power/thermal balance, life-support endurance (if crewed), reliability, cost and build time update live and match the physical models.
2. **Given** any computed number in the designer, **When** the player inspects it, **Then** the full derivation down to sourced data inputs is shown.
3. **Given** a design with negative power margin, undersized radiators, lander T/W below local gravity, delta-v short of the declared mission, or crewed endurance below mission duration, **Then** the designer red-flags it (and refuses physically impossible ones), while marginal designs remain flyable with truthful risk numbers.
4. **Given** a component not yet researched to usable maturity, **When** the player browses the picker, **Then** it is absent or clearly locked.
5. **Given** repeated production of the same design, **Then** unit cost falls along the configured learning curve and reliability rises with accumulated flight heritage.

---

### User Story 6 - Make Payroll in Space: Budgets, Markets, Contracts (Priority: P2)

As NASA, the player navigates an annual appropriation shaped by prestige and political mood,
including a directed program they didn't ask for. As Helion (a private company), the player
manages cash runway, bids on an agency RFP to deliver cargo to the lunar surface, wins it,
slips schedule, pays penalties — and survives to raise a round on the strength of a milestone.
Launch is bought and sold on a market whose $/kg responds to world capacity. Every dollar figure
can be traced to the mass and delta-v it is proxying.

**Why this priority**: The six-currency economy is the strategic substrate connecting research,
operations and politics, and funding-model asymmetry is the factions' identity.

**Independent Test**: Headless economy: simulate fiscal cycles for an agency and a company from
data-driven baselines; assert appropriation modifiers, runway/bankruptcy mechanics, RFP
post→bid→award→penalty flow, and market price response to capacity shocks.

**Acceptance Scenarios**:

1. **Given** an agency with recent failures and low political mood, **When** the next budget cycle resolves, **Then** the appropriation falls within the modifier model, and sustained collapse can trigger the "gutted" soft-fail state.
2. **Given** a private faction whose cash reaches zero with no financing available, **When** the tick resolves, **Then** the faction goes bankrupt (game over for a player faction; market exit for an AI one).
3. **Given** an open RFP, **When** factions bid, **Then** award follows the published evaluation terms, and the winner accrues revenue plus heritage on success or penalties and reputation loss on failure.
4. **Given** a doubling of world launch capacity, **When** the market tick resolves, **Then** $/kg by orbit class falls per the supply/demand model — and ISRU propellant's relative value shifts accordingly.
5. **Given** any cost shown to the player, **When** inspected, **Then** the underlying mass/delta-v basis is visible (money stays a proxy).
6. **Given** one tonne of water in LEO, at EML-1 and on the lunar surface, **Then** the ledger treats them as distinct location-addressed goods with different values.

---

### User Story 7 - Build the Supply Chain: Depots, Tugs, ISRU Break-Even (Priority: P2)

The player builds a propellant depot at EML-1, fields a reusable tug, and commissions a lunar
polar ice mine. The transport graph prices every movement in delta-v, time and window
availability. The ISRU plant's economics are emergent: plant mass delivered, power through lunar
night, surveyed ice grade and the market price of Earth-launched propellant determine whether the
mine ever pays back. Ops capacity and comms passes are finite; over-subscribing the fleet
degrades data return and raises anomaly risk.

**Why this priority**: Logistics-as-delta-v is the economic heart of the game; ISRU break-even is
the single mechanism that makes expansion strategy meaningful rather than decorative.

**Independent Test**: Headless: construct a transport graph scenario, schedule shipments through
a depot with window constraints, and assert flows, boil-off, and amortisation; run the lunar-ice
reference case and assert break-even responds correctly to each input (grade, power, plant mass,
launch price).

**Acceptance Scenarios**:

1. **Given** cargo at LEO bound for the lunar surface, **When** the player routes it, **Then** available edges show delta-v cost, time-of-flight and window constraints, and the assignment debits propellant and vehicle time accordingly.
2. **Given** a depot with stored cryogenic propellant, **When** time passes, **Then** boil-off losses accrue per the thermal model unless zero-boil-off technology is operating.
3. **Given** the lunar-ice reference scenario at surveyed grade X and Earth-launch price Y, **When** the plant operates for its design life, **Then** cumulative cost vs value matches the documented break-even model, and a worse grade or cheaper Earth launch can make the venture unprofitable.
4. **Given** more active craft than ops/comms capacity supports, **When** the fleet operates, **Then** data return degrades and anomaly probability rises for under-attended craft, visibly attributed to over-subscription.
5. **Given** a reusable tug flying repeated cargo legs, **Then** its per-trip cost falls as capital amortises, verifiably cheaper than expendable equivalents over the same manifest.

---

### User Story 8 - Found Something Permanent: Bases and Settlement (Priority: P3)

The player selects a surveyed site at the lunar south pole, sequences a construction project —
power module, ECLSS, habitat, ISRU plant, regolith-sintered radiation shielding — through
delivery, assembly and commissioning. The base's emergent properties (power margin, ECLSS
closure, population capacity, sustainability index) classify it from beachhead toward settlement.
Years later, a Mars base pursuing the Homestead goal is stress-tested by a five-year resupply
embargo.

**Why this priority**: Settlement is the long-game payoff and the integrating consumer of ISRU,
construction, life support and logistics — it needs them all in place first.

**Independent Test**: Headless: run a scripted construction project and assert phase gating on
deliveries/power/labour; compose module sets and assert emergent properties match the documented
aggregation; run the embargo stress test against a reference settlement and assert
survive/fail outcomes track the sustainability index.

**Acceptance Scenarios**:

1. **Given** a construction project with a delivery sequence, **When** a required module delivery fails or slips, **Then** assembly halts at the dependent step and the schedule/cost impact is surfaced.
2. **Given** a base's installed modules, **Then** power margin, ECLSS closure, population capacity and sustainability index are computed from module properties and local site conditions (illumination, resources), and adding/removing modules updates them.
3. **Given** a base with regolith-construction capability, **When** shielding is built locally, **Then** required Earth-launched mass for equivalent radiation protection is displaced, visible in the logistics ledger.
4. **Given** a settlement with sustainability index above the Homestead threshold, **When** a five-year Earth-resupply embargo is simulated, **Then** it survives; below threshold, it degrades realistically (consumable exhaustion, ECLSS failures, population decline) rather than failing abruptly.

---

### User Story 9 - Send People, Not Just Probes (Priority: P3)

The player mounts a crewed Mars mission. Selection and training take years of pipeline time.
The mission integrates transfer delta-v, ECLSS closure against a ~2.5-year round trip, radiation
career limits with a solar-storm shelter, deconditioning countermeasures, psychology under
comms lag, spares for the inevitable ECLSS breakdowns, abort options, and Mars EDL plus
ISRU-fuelled ascent. It is materially harder and more expensive than the equivalent robotic
mission — and a loss-of-crew accident has heavy human and political consequences.

**Why this priority**: Crewed difficulty is a constitutional mandate (Principle VII) and the
emotional core of the late game, but it consumes nearly every other system, so it lands after
they exist.

**Independent Test**: Headless reference pair: equivalent robotic vs crewed mission to the same
destination; assert the crewed variant requires multiplied launched mass/cost/ops and tracks
dose, health, psychology and consumables; script a solar-storm event and assert shelter
mechanics; script ECLSS failure and assert spares/crew-time response.

**Acceptance Scenarios**:

1. **Given** equivalent robotic and crewed missions to the same destination, **Then** the crewed mission requires materially greater launched mass, cost, ops capacity and approval steps, with the difference traceable to life support, crew systems and abort provisions.
2. **Given** a crew in deep space when a solar particle event fires, **When** they reach the storm shelter in time, **Then** dose accrual is reduced per the shielding model; otherwise career dose advances and may disqualify crew members from future flight.
3. **Given** a long micro-gravity transit without artificial gravity, **Then** crew deconditioning accrues and degrades performance on arrival unless countermeasures were provisioned.
4. **Given** an ECLSS component failure beyond spares coverage, **Then** an escalating consumable crisis follows with survival determined by distance, abort options and remaining margins — never an instant arbitrary death.
5. **Given** a loss-of-crew accident, **Then** political mood, budgets/valuation and crewed-program approvals suffer severe, long-lasting effects per the politics model.

---

### User Story 10 - Race the World and Answer the Question (Priority: P3)

Nine AI-run organisations research, build, fly, bid, partner and race the same ~120 historic
firsts under the same physics. The player watches a rival take "first crewed lunar return of the
new era", lobbies for nuclear-launch approval, weathers a public-mood collapse after a televised
failure, and pursues their selected Grand Goal. On the Seeker path, a staged, probabilistic,
mission-driven evidence process — orbital hints, in-situ chemistry, sample return — slowly
resolves the seeded astrobiology question for Enceladus, with false positives possible and
discovered life remaining a science object, never an actor. At the configured horizon the game
scores.

**Why this priority**: Politics, milestones and the AI world supply the competitive pressure that
replaces combat — the last integrating layer over a functioning game.

**Independent Test**: Headless 100-year AI-only runs: assert AI factions achieve milestones at a
plausible cadence without violating physics/plausibility constraints; assert milestone
world-first/faction-first scoring; script policy and mood scenarios and assert budget/approval
effects; run the astrobiology evidence pipeline against seeded truths across many seeds and
assert calibrated convergence (including false-positive/negative behaviour).

**Acceptance Scenarios**:

1. **Given** a 100-year run with no player input, **When** it completes, **Then** AI factions have researched, flown missions, won contracts and claimed milestones — all within the same physical and plausibility rules as the player (no AI-only capabilities).
2. **Given** a milestone achieved first in the world by any faction, **Then** the world-first award (prestige plus any funding/market effects) goes to that faction once, and later factions receive only the smaller faction-first credit.
3. **Given** a pending NTP flight needing nuclear-launch approval under restrictive policy, **When** the player lobbies (spending political capital), **Then** approval probability/lead time shift per the policy model; without approval the launch cannot proceed.
4. **Given** a candidate world whose seeded truth is "no life", **When** the player runs the full staged evidence process, **Then** evidence converges toward a conclusive negative, with individual stages still able to yield misleading intermediate results.
5. **Given** a confirmed positive astrobiology result, **Then** it scores as a top-tier milestone, hardens planetary-protection policy, updates the Sojournal — and adds no agent-like behaviour of any kind to the game.
6. **Given** the configured scoring horizon arrives, **Then** the run is scored from milestones and Grand Goal completion, and soft-failed factions (gutted/bankrupt) are scored accordingly.

---

### User Story 11 - See Everything, Trust Every Number (Priority: P2)

The player lives in a data-dense, mostly-2D interface: a logarithmically zoomable multi-focus
system map, twelve purpose-built screens, tables with thousands of virtualised rows, hotkeys
everywhere. Any derived number expands to its derivation. Irreversible actions always go
plan → preview → commit. The Sojournal explains the real science behind every mechanic with
citations and doubles as the soft tutorial. Units are SI only; palettes are colour-blind-safe;
the whole game is playable from the keyboard at 1280×720 through 4K.

**Why this priority**: The UI is how every other area becomes playable; it grows incrementally
alongside P1/P2 systems (map and planner early, late-game screens later), so it is prioritised
with the P2 wave while spanning all waves.

**Independent Test**: UI reads state and issues commands only through the sim core's public
boundary (verifiable by architecture test); screen-by-screen acceptance against the S1–S12
definitions; accessibility audit (keyboard completeness, contrast/palette, scaling); table
virtualisation benchmarked at 3,000+ rows.

**Acceptance Scenarios**:

1. **Given** the system map, **When** the player zooms from heliocentric to local-ops focus and switches inertial/rotating frames, **Then** bodies, trajectories, SOIs and overlay layers render correctly and stay interactive at the performance budget.
2. **Given** any derived number on any screen, **When** the player drills in, **Then** a complete derivation with sourced inputs is displayed.
3. **Given** an irreversible action (burn commit, launch, program cancellation), **When** the player triggers it, **Then** a consequence preview and explicit confirm step precede execution — the player can never be surprised by something the UI couldn't show.
4. **Given** the asteroid catalogue table with 3,000+ rows, **When** scrolled, filtered and sorted, **Then** interaction stays responsive via virtualisation.
5. **Given** a Sojournal entry for any body, technology or mechanic, **Then** it explains the real science with cited sources, stays consistent with the simulation, and updates as the player's belief state advances.
6. **Given** keyboard-only play, **Then** all gameplay actions are reachable; **and given** any screen at 1280×720 and 4K with scaled text and colour-blind-safe palette, **Then** it remains readable and operable.

---

### Edge Cases

- **Simultaneous interrupts at high warp**: multiple pause-worthy events land on the same tick (node + solar storm + budget vote). The event queue must order deterministically, present all, and lose none.
- **Save/load during fine-timestep phases**: saving mid-burn, mid-EDL or mid-docking must round-trip exactly, including integrator and event-queue state.
- **Seeding pathology**: a seed must never render a capability category unreachable (all paths dead-ended); the seeding procedure must enforce the ≥1-viable-path guarantee, verified across a large seed sample.
- **Plan invalidation**: a committed manoeuvre plan becomes infeasible before execution (propellant leaked, engine failed, vehicle redirected). The system must detect, interrupt and require re-planning rather than executing an impossible burn.
- **Missed windows**: the player or a delayed program misses a transfer window; plans must gracefully re-target the next window with updated costs, not corrupt.
- **Player-faction death mid-flight**: bankruptcy or gutting occurs while crewed missions are in flight; the observer/rebuild mode must keep simulating those missions safely.
- **End-of-horizon with active missions**: the scoring date arrives while missions are en route; scoring must close out deterministically and any continue-past-end play must be clearly unscored.
- **Contamination of the evidence**: a faction (player or AI) contaminates a candidate world before its astrobiology question resolves; the evidence pipeline must permanently account for degraded site value for everyone.
- **Light-lag autonomy gap**: an anomaly strikes a craft whose one-way light time exceeds the real-time response threshold and whose autonomy tier is insufficient; outcome must follow the autonomy model (possible loss), not silently allow ground intervention.
- **Ops starvation**: ops/comms capacity collapses (facility loss, budget cut) while a large fleet is active; degradation must be gradual, attributed, and recoverable.
- **Personnel cliff**: loss of key personnel (death, poaching, gutting-driven brain drain) drops effective UL in a niche domain; dependent programs must reflect the regression rather than ignore it.
- **Market collapse**: a rival's breakthrough collapses a price the player's business depended on (e.g., launch $/kg) mid-contract; contracts, runway and valuation must re-mark without breaking the economy sim.
- **Depot dry-out**: a tanker chain fails and a depot empties while missions depend on it; dependent plans must flag infeasibility at planning, not at execution.
- **Zero-production learning curve**: cost models must behave (no division-by-zero, no negative cost) for first units, restarted lines, and single-unit bespoke builds.
- **Astrobiology false positive at high stakes**: a stage-2 positive triggers political/PP tightening, then sample return refutes it; the model must walk back consensus honestly and reputation effects must follow the published process.

## Requirements *(mandatory)*

Requirement IDs are prefixed by feature area (e.g., FR-SIM-### for FA-01). Cross-cutting
requirements are FR-XCU-###. Per-area child specifications must trace their requirements back to
these IDs.

### Cross-Cutting Requirements (bind every feature area)

- **FR-XCU-001**: Every quantitative entry in game data files that bears on plausibility (propulsion performance, body data, ISRU yields, life-support closure, costs, budgets, milestone definitions, event parameters) MUST carry a non-empty `source` citation, and continuous-integration checks MUST fail on missing or empty sources. *(Constitution I)*
- **FR-XCU-002**: All game content (bodies, sites, tech nodes, components, economy constants, milestones, events, faction definitions) MUST live in versioned, schema-validated data files; mechanics live in code; rebalancing or realism corrections MUST NOT require logic changes. Schemas MUST be validated in CI. *(Constitution V)*
- **FR-XCU-003**: The simulation core MUST be deterministic: a given seed plus an ordered decision sequence MUST reproduce identical state. All stochastic outcomes MUST derive from explicitly threaded, seeded PRNG streams; the core MUST NOT read wall-clock time or any unseeded entropy. A CI test MUST run a seed+decision script twice and assert state identity. Bit-identical reproducibility is guaranteed **per platform and build**: the same seed, decisions and build on the same platform always reproduce identical state. Saves MUST remain loadable across supported platforms, but replay/state identity across *different* platforms or CPU architectures is not a v1.0 requirement (clarified 2026-06-12). *(Constitution III)*
- **FR-XCU-004**: The simulation core MUST run fully headless with no dependency on UI, rendering or input layers; the UI MUST interact only through a defined state-read/command boundary and MUST contain no game logic. Every gameplay system MUST be testable headlessly. *(Constitution IV)*
- **FR-XCU-005**: The physics engine MUST contain no per-technology magic numbers; all constants are read from sourced data files. Physics MUST be validated in CI against analytic cases — Hohmann transfer delta-v, two-body orbital periods, simple flyby geometry, rocket-equation identities — within tolerances defined in the test data. *(Constitution II)*
- **FR-XCU-006**: Saves MUST be deterministic, versioned and forward-migratable; loading a save MUST reproduce identical state (round-trip tested in CI), including mid-phase saves (burn/EDL/docking) and full event-queue/integrator state. Each save MUST pin the exact content-data version its run started with: in-progress runs continue on their original data even after patches (preserving determinism and replay identity for the whole run), while new runs use the latest data. Save-format migration (FR-XCU-006 "forward-migratable") concerns the save *structure*, never silently swapping a run's pinned content values.
- **FR-XCU-007**: All quantities in simulation, data files and player-facing UI MUST use SI units exclusively.
- **FR-XCU-008**: v1.0 MUST contain no weapons, combat, sabotage mechanics, or alien actors. Reserved future features MUST exist only as clearly marked extension points with no implemented logic. An automated scope audit (data + feature flags) MUST verify this. *(Constitution IX)*
- **FR-XCU-009**: The full simulation (3,000+ catalogued bodies, large fleets, complete economy/logistics graph, AI factions) MUST sustain the maximum time-warp rate on the reference machine — a high-end consumer desktop (8+ performance cores, discrete GPU, 32 GB RAM) — within explicit, tracked tick-time budgets; UI tables MUST virtualise at thousands of rows within explicit frame-time budgets. Budgets are defined per area in child specs and tracked in CI benchmarks.
- **FR-XCU-010**: Difficulty settings MUST alter only political/economic harshness and event base rates — never physics, plausibility data, or AI access to capabilities.
- **FR-XCU-011**: Every event, decision and state transition in the core MUST be captured in an append-only event log sufficient to replay the run from seed to current state.
- **FR-XCU-012**: Headless integration tests MUST cover each major system (research, economy, spaceflight, world/events, crew) independently of the UI.

### FA-01 — Simulation Core & Time

- **FR-SIM-001**: System MUST simulate continuous time from 1 January 2026 over a configurable horizon (25/50/100 years; default 100, ending 2126) on a fixed-timestep deterministic kernel.
- **FR-SIM-002**: System MUST provide time-warp from 1 s/s up to approximately 1 year/min, with sub-stepping such that burns, EDL, docking and other fine-grained phases always resolve at fine timestep regardless of warp.
- **FR-SIM-003**: System MUST implement event-driven interrupt-and-pause: configurable event classes (at minimum: manoeuvre nodes, anomalies, program reviews, budget votes, discoveries, rival milestones, solar storms, contract events, personnel events) interrupt warp and pause before their effects require player input; per-class pause behaviour MUST be player-configurable.
- **FR-SIM-004**: System MUST remain fully playable while paused: all planning, design, allocation and review actions available.
- **FR-SIM-005**: System MUST maintain a deterministic event queue in which simultaneous events are ordered by a stable, documented rule.
- **FR-SIM-006**: System MUST model the calendar's gameplay-relevant structure: fiscal years, election cycles, transfer windows, eclipse seasons, Mars dust-storm seasons, and lunar day/night (~354 h) for surface power.
- **FR-SIM-007**: System MUST support scripted decision input in headless mode (for tests, replays and AI-only runs).
- **FR-SIM-008**: System MUST score and formally end the run at the configured horizon, supporting continued unscored sandbox play afterwards, and MUST support observer/rebuild mode after player soft-fail states.
- **FR-SIM-009**: System MUST support both ironman and save-anywhere modes.
- **FR-SIM-010**: System MUST journal the append-only event/decision log (FR-XCU-011) continuously to durable storage so that, after a crash or power loss, the run recovers by replaying seed + journal to within moments of the interruption; autosaves MUST additionally be written at key events. Recovery MUST work identically in ironman (a crash is never a free reroll and never destroys the campaign) and in save-anywhere mode.

### FA-02 — Astrodynamics & Flight

- **FR-AST-001**: The authoritative vehicle/body state MUST come from a numerical n-body propagator with the perturbations that matter: third-body gravity, oblateness (J2), solar radiation pressure, atmospheric drag, and continuous low-thrust acceleration.
- **FR-AST-002**: System MUST provide a fast planning layer (patched-conic/analytic) that is explicitly approximate, is reconciled against the propagator, and surfaces plan-vs-truth divergence to the player.
- **FR-AST-003**: System MUST support heliocentric, planetocentric and rotating (CR3BP) reference frames, sphere-of-influence regions, Lagrange points and their operational orbits (halo/NRHO).
- **FR-AST-004**: Players MUST be able to plan manoeuvre nodes (prograde/normal/radial components) on a timeline, chain multi-burn plans, preview resulting trajectories, and have warp auto-pause at nodes.
- **FR-AST-005**: System MUST generate interactive porkchop plots (departure × arrival → C3/delta-v/time-of-flight contours) for ballistic transfers between any two catalogued locations.
- **FR-AST-006**: System MUST provide gravity-assist/flyby planning, including geometry design for single flybys and solver assistance for multi-assist sequences.
- **FR-AST-007**: System MUST plan and fly low-thrust trajectories (spirals, continuous-thrust arcs) under guidance laws, with durations and propellant use consistent with power-limited thrust.
- **FR-AST-008**: System MUST offer low-energy transfers (weak-stability-boundary/ballistic capture) as planning options gated by Astrodynamics Understanding Level.
- **FR-AST-009**: System MUST support aerocapture and aerobraking as plannable manoeuvres at bodies with atmospheres, trading delta-v savings against quantified thermal/structural risk.
- **FR-AST-010**: Flown trajectories MUST include finite-burn, gravity and steering losses and seeded execution error; the system MUST detect divergence and support correction burns (TCMs).
- **FR-AST-011**: Every planned burn MUST debit propellant from specific tanks on a specific vehicle per the rocket equation; infeasible plans (insufficient delta-v, violated engine limits) MUST be flagged at planning time and re-flagged if state changes invalidate a committed plan.
- **FR-AST-012**: Cryogenic propellant boil-off MUST accrue over time per the thermal model and visibly erode available delta-v unless mitigated by researched technology.
- **FR-AST-013**: Station-keeping (L-point upkeep, drag make-up) MUST consume propellant/resources over time for assets that require it.
- **FR-AST-014**: EDL MUST be simulated as its own phase: atmospheric entry (heating, ballistic coefficient, parachutes, supersonic retropropulsion — including the Mars heavy-landing difficulty), propulsive descent at airless bodies with precision/hazard-relative landing, and rendezvous/anchoring at microgravity bodies; outcomes MUST depend on vehicle suitability, site hazards and reliability — never on arbitrary rolls disconnected from physics.
- **FR-AST-015**: There MUST be no top speed, no reactionless propulsion, and no motion not produced by modelled forces.

### FA-03 — World Data & Belief State

- **FR-WLD-001**: The world MUST contain the Sun, the 8 planets, Pluto and major dwarf planets, approximately 150 significant moons, and a curated catalogue of approximately 3,000 asteroids/comets with real orbital elements from authoritative small-body data, with provenance recorded per FR-XCU-001.
- **FR-WLD-002**: Ephemerides propagated from these elements MUST be accurate enough over 2026–2126 that transfer windows, alignments and assist opportunities are physically correct (accuracy bounds defined in the FA-03 child spec).
- **FR-WLD-003**: The remainder of the belt and Kuiper region MUST be modelled as statistical prospecting fields that survey campaigns convert into newly catalogued targets consistent with the seeded distribution.
- **FR-WLD-004**: Each body MUST carry sourced physical data — mass, radius, gravity, rotation, atmosphere (composition/pressure/scale height), thermal and radiation environment, hazards — sufficient to drive EDL, power, thermal and crew-dose models.
- **FR-WLD-005**: Dynamical locations (orbit bands such as LEO/MEO/GEO/GTO, Lagrange points, NRHO, low orbits of bodies, cycler trajectories) MUST be first-class addressable nodes shared with the logistics graph.
- **FR-WLD-006**: Per body and site, the system MUST maintain a belief state (what each faction knows, with uncertainty) distinct from seeded ground truth; missions and surveys refine belief toward truth; per-body Geoscience understanding MUST grow primarily from missions.
- **FR-WLD-007**: Bodies MUST expose surveyable Sites with progressively revealed properties: resource type and grade, illumination/thermal profile, slope/roughness, comms visibility, hazard level, science value, and planetary-protection category — all with modelled survey uncertainty (acting on a poorly surveyed site MUST be possible and risky).
- **FR-WLD-008**: A COSPAR-style planetary-protection regime MUST assign categories (I–V, including Special Regions) imposing sterilisation/bioburden requirements with real cost/mass consequences; forward contamination MUST permanently degrade affected astrobiology science value and cost the responsible faction politically; back contamination rules MUST require containment/receiving chains and restricted-return trajectories for samples from potentially habitable worlds; stringency MUST be adjustable by the policy layer.
- **FR-WLD-009**: Astrobiology ground truth (presence/absence of microbial/chemical life per candidate world: Mars subsurface, Europa, Enceladus, Titan, Ceres brines, Venus clouds) MUST be seeded per game within plausibility-bounded distributions (most games mostly negative; rarely more than one or two positives).
- **FR-WLD-010**: Astrobiology resolution MUST be a staged, probabilistic, mission-driven evidence process (orbital biosignature hints → in-situ chemistry → microscopy/metabolism → sample return and independent confirmation) in which stages yield calibrated evidence — never a binary popup — with false positives/negatives and competing abiotic explanations; consensus forms over time and missions.
- **FR-WLD-011**: Discovered life MUST be a science object only: it MUST trigger milestone, prestige, policy and Sojournal effects, and MUST NOT introduce any agent-like behaviour.
- **FR-WLD-012**: The world baseline at January 2026 MUST reflect real capabilities: ISS and Tiangong as the only stations, current real launch capability in service, and no established off-Earth bases. The commercial launch sector is represented **entirely by the five fictional companies**, whose vehicles stand in for real 2026 commercial lift with realistic, sourced performance (e.g., Helion's partially reusable medium-lift vehicle proxies real reusable medium-lift capability and pricing); national agencies fly real-named government vehicles (SLS/Orion, Ariane 6, Soyuz/Proton/Angara, H3). Every launch supplier in the market is therefore a simulated faction (clarified 2026-06-12).

### FA-04 — Vehicle Designer & Propulsion

- **FR-VEH-001**: Players MUST compose vehicles from researched component technologies only (structures, tanks, propulsion, power, thermal, avionics/GNC, comms, life support and accommodation, payloads, EDL kit, docking, RCS); unresearched components MUST be absent or visibly locked.
- **FR-VEH-002**: The designer MUST live-compute: dry/wet mass, per-stage and per-mode delta-v, thrust and T/W at each gravity field of interest, power and thermal balance with margins, life-support closure and endurance (crewed), payload capacity, composed reliability (from component TRL/heritage), and unit cost and build time with learning-curve effects.
- **FR-VEH-003**: Every computed output MUST be traceable: the player can expand any derived number to its complete input tree terminating in sourced data values.
- **FR-VEH-004**: The designer MUST enforce realism guards: physically impossible configurations (negative power margin unresolvable in any mode, radiators insufficient for heat load, lander T/W below local gravity, structural limits exceeded) are refused or hard-red-flagged; marginal-but-possible designs remain buildable with truthful reliability/risk figures.
- **FR-VEH-005**: Each propulsion technology MUST expose a physical model (Isp, thrust, input power, propellant type/density, throttle range, restart/duty limits, mass model including feed/power/radiators, reliability curve) spanning the design families: chemical (storable, kerolox, hydrolox, methalox, deep-throttle landing engines, long-duration cryo stages), electric (gridded ion, Hall, high-power Hall, MPD, VASIMR-class, electrospray), nuclear-thermal (solid-core through advanced concepts), nuclear-electric (reactor + conversion + PMAD + radiators + integrated tug), propellant logistics (transfer, zero-boil-off, depots), and gated frontier concepts (fission-fragment, fusion) reachable only via breakthrough/endgame paths and possibly seeded as dead ends.
- **FR-VEH-006**: The simulation MUST enforce honest physical couplings: electric propulsion thrust is power-limited (power source and its thermal rejection are carried mass); nuclear systems carry reactor, shielding and radiator mass plus political/approval costs; waste-heat rejection is a first-class, frequently binding constraint.
- **FR-VEH-007**: Designs MUST be savable as versioned classes/templates supporting iteration, derivative designs (inheriting heritage discounts), and side-by-side comparison.
- **FR-VEH-008**: All vehicle archetypes MUST be designer-built from the same system: launch vehicles, capsules, cargo craft, tugs, landers, ascent vehicles, transit habitats/cyclers/spin-habitats, rovers, surface mobility, station and base modules, ISRU plants, relay satellites, science probes and body-specific explorers.
- **FR-VEH-009**: Produced units MUST accumulate flight heritage that raises reliability toward tech-specific ceilings and discounts derivative engineering programs.

### FA-05 — Research & Personnel

- **FR-RES-001**: The research model MUST implement two linked tracks: continuous Understanding Levels (0–100) per Knowledge Domain (the science track), and Engineering Programs that advance named Technologies through TRL 1–9 (the implementation track). Research MUST NOT be reducible to spend-points-and-unlock.
- **FR-RES-002**: Domain UL MUST gate engineering-program availability and set risk floors; UL growth MUST show diminishing returns at high levels and synergy bonuses across coupled domains, per the domain graph in the design (A1–A17).
- **FR-RES-003**: Research Points MUST be generated by scientists, facilities and instruments; Design Effort by engineers and facilities; a portfolio allocation interface MUST let players split RP/DE across programs with efficiency multipliers for staffing quality, domain mismatch and facility bottlenecks.
- **FR-RES-004**: Missions MUST inject UL directly into relevant domains (flight and surface data advance geoscience/astrobiology/flight-science domains beyond what labs can).
- **FR-RES-005**: Each TRL step MUST carry a cost, a minimum duration that cannot be bought down past a floor (schedule-compression penalties apply), and facility requirements; technologies MUST be flyable only at TRL ≥ 6 with steep reliability penalties that ease through TRL 7–9.
- **FR-RES-006**: Program cost and schedule MUST be estimated with uncertainty (e.g., P50/P80) and realised with variance influenced by TRL jump size, domain UL margin, staffing, facility adequacy and political interference.
- **FR-RES-007**: Test campaigns MUST be simulated: failures cost money/schedule but inject UL ("failure-that-teaches"); spectacular flight-test failures MUST also carry political/PR consequences.
- **FR-RES-008**: Per-game seeding MUST designate some approaches as dead ends within TRL bands, with advance hints (rising risk index, stalled error bars, repeated failures without UL growth); every capability category MUST retain at least one viable path in every seed, and parallel-approach pursuit MUST be supported as a de-risking strategy.
- **FR-RES-009**: Breakthroughs MUST accrue from sustained basic-science investment via hidden insight pressure with seeded thresholds, delivering one of: a tech-cluster discount, an early branch unlock, or revelation of a hidden path past a dead end; cadence MUST be rare (order of once per 8–15 years for a heavily invested domain) and MUST be announced with a sourced Sojournal reference.
- **FR-RES-010**: Leapfrogging MUST be possible: satisfying a higher tier's prerequisites through UL investment rather than intermediate products, at higher cost and risk (no inherited heritage).
- **FR-RES-011**: A global science tide MUST advance World UL per domain from all factions' activity plus an exogenous baseline; factions MUST choose between publishing (prestige + faster World UL) and holding/patenting (lead + licensing income); licensing, purchase and co-development partnerships (sharing cost, TRL credit and IP) MUST be available; trailing factions MUST research known ground more cheaply than the frontier.
- **FR-RES-012**: Personnel MUST be a managed resource: scientists, engineers, program managers, astronauts, mission controllers and diplomats/lobbyists with skills, discipline tags and traits affecting outcomes; recruitment, poaching (with relations cost), training, morale, retention and aging MUST be modelled; losing key people MUST reduce effective UL in niche domains (tacit knowledge).
- **FR-RES-013**: Faction research heritage discounts MUST position 2026 starting maturity per real-world strengths (e.g., reuse/methalox, NTP/NEP expertise, precision landing/sample return, aerocapture/deep-space science, ISRU/depots) as encoded in faction data files.

### FA-06 — Economy, Markets & Logistics

- **FR-ECO-001**: The system MUST track and exchange six currencies — funds, delta-v/propellant, mass-to-orbit, crew-time, ops capacity, political/reputation capital — and gameplay MUST surface conversions between them.
- **FR-ECO-002**: Agencies MUST be funded by a political appropriation process: baselines modified by prestige, public/political mood, economic cycle and election timing; directed funds the player did not request; faction-specific carry-over rules; and fiscal-cliff events. Agencies cannot go bankrupt but MUST be guttable to a caretaker soft-fail state.
- **FR-ECO-003**: Private factions MUST manage cash, burn, revenue and financing (equity rounds priced by milestone-driven valuation, debt, finite owner injections); cash exhaustion without financing MUST mean bankruptcy — game over for the player, market exit for AI.
- **FR-ECO-004**: Resources MUST include bulk commodities (propellants, water, metals, regolith, silicon, polymers, consumables, spares) and constrained strategic materials (RTG isotopes with capped global supply, HEU-vs-LEU policy trade-offs, restricted electronics/rare-earths), each lot addressed by location and delta-v; identical goods at different nodes MUST be distinct in value and the ledger.
- **FR-ECO-005**: ISRU economics MUST be emergent, never subsidised by hidden bonuses: break-even for lunar polar ice → propellant, Mars Sabatier CH₄/O₂, regolith oxygen/metals, and asteroid/comet volatiles MUST follow from delivered plant mass, capex/opex, power availability, surveyed resource grade and the local price of Earth-delivered alternatives; pilot-to-production scale-up MUST have its own learning and reliability ramp.
- **FR-ECO-006**: A global launch market MUST set $/kg by orbit class from world supply and demand; factions MUST be able to buy launch or sell their own capacity; market prices MUST respond to capacity changes (e.g., widespread reuse collapsing prices and shifting ISRU's relative value).
- **FR-ECO-007**: A service-contract system MUST let agencies post RFPs (delivery, hosting, crew transport, data purchase) with published evaluation terms; companies bid; awards yield revenue and heritage; failures yield penalties and reputation loss; agencies MUST be able to choose in-house versus commercial procurement as a strategic axis.
- **FR-ECO-008**: Partnerships and consortia MUST support co-funding, shared TRL credit, IP sharing, crew-seat and data barter; per-faction trust state MUST persist and betrayal MUST carry lasting reputation costs.
- **FR-ECO-009**: A data/IP market MUST support selling science data, licensing matured technologies, and patents that earn royalties while slowing the global tide in the holder's favour.
- **FR-ECO-010**: Tourism (suborbital → orbital → lunar) and in-space manufacturing products MUST exist as niche revenue streams with realistic price ceilings and market sizes.
- **FR-ECO-011**: Logistics MUST run on a directed transport graph: nodes are dynamical locations (shared with FA-03); edges are transfers priced in delta-v and time with launch-window availability; moving goods assigns vehicles to edges and pays propellant and time.
- **FR-ECO-012**: Depots MUST act as buffer nodes decoupling production from transport (storage, transfer, station-keeping costs, boil-off); reusable tugs and cyclers MUST amortise capital across repeated legs and be verifiably cheaper than expendable equivalents on suitable routes.
- **FR-ECO-013**: Ops capacity (controllers, mission control) and comms capacity (antenna-network/relay passes under light-time delay) MUST be finite shared pools; over-subscription MUST degrade data return and raise anomaly risk, attributably.
- **FR-ECO-014**: Facilities MUST be capital assets with capex, opex, capacity and upgrade paths spanning R&D (labs, test stands, chambers, analog sites), manufacturing, launch/recovery, ground segment, and space-side infrastructure built via missions.
- **FR-ECO-015**: Production MUST follow learning curves (unit cost declining with cumulative count) such that reuse and standardisation are emergently economical; estimates MUST carry uncertainty and realise with overruns consistent with FA-05's model.
- **FR-ECO-016**: Every player-facing cost MUST be traceable to its mass/delta-v physical basis (money stays a proxy).

### FA-07 — Bases, Construction & ISRU Operations

- **FR-BAS-001**: Large builds MUST be projects with explicit phases: site selection → module manifest → delivery sequence → assembly → commissioning; missing deliveries, power, robotics or labour MUST halt dependent phases with surfaced schedule/cost impact.
- **FR-BAS-002**: Stations and surface bases MUST be composed from module types including habitat, power, ECLSS, ISRU, greenhouse, workshop, medical, storage, comms and radiation shelter.
- **FR-BAS-003**: Base/station properties MUST be emergent from installed modules and local site conditions: power margin (including night/eclipse survival), ECLSS closure fraction, population capacity, and a sustainability index classifying beachhead → outpost → settlement.
- **FR-BAS-004**: Construction MUST consume delivered or locally produced (ISRU) materials, power, construction robotics and/or crew-time, and time; regolith construction (sintering, printed structures, sulfur/geopolymer concrete) MUST displace Earth-launched mass, most prominently for radiation shielding.
- **FR-BAS-005**: ISRU plants MUST operate as ongoing industrial assets integrated with the logistics graph and economy: ramp-up reliability, maintenance demands, output feeding propellant supply and construction feedstock.
- **FR-BAS-006**: Settlements MUST be able to pursue self-sufficiency (rising closure, local food fraction, local manufacturing); the Homestead stress test (surviving a 5-year Earth-resupply embargo) MUST be simulatable with survive/degrade outcomes driven by the sustainability model, with gradual realistic failure modes.

### FA-08 — Life Support & Crew

- **FR-CRW-001**: For every crewed vehicle and base, the system MUST track per crew member over time: consumables (O₂, water, food, N₂ buffer, CO₂ removal) against ECLSS closure fraction, with closure directly setting resupply mass and mission feasibility.
- **FR-CRW-002**: Radiation MUST be modelled as accumulated career and mission dose versus limits, from continuous GCR plus solar particle events; storm shelters MUST mitigate SPE dose when reached in time; dose limits MUST bound mission duration and end careers.
- **FR-CRW-003**: Physiological deconditioning (bone, muscle, cardiovascular) MUST accrue in micro-gravity and degrade crew capability, countered by exercise, pharmacological measures and artificial gravity (tethered spin or rotating habitats) when researched and provisioned.
- **FR-CRW-004**: Psychological load MUST grow with isolation, confinement, mission duration and comms lag, affecting error rates, anomaly risk and morale.
- **FR-CRW-005**: Life-support and vehicle systems MUST fail per reliability models; spares mass and crew maintenance time MUST be the mitigation; unrepairable ECLSS failure far from Earth MUST create an escalating survival crisis governed by remaining margins and abort options — never an instant unexplained loss.
- **FR-CRW-006**: Crewed EDL, ascent and abort risk MUST be explicit: crewed missions MUST declare abort options and the system MUST evaluate them when contingencies strike.
- **FR-CRW-007**: The astronaut pipeline MUST cover selection, multi-year training (requiring facilities/analog missions), assignment, in-mission management and career aging; loss-of-crew MUST carry heavy, lasting human and political consequences per FA-09.
- **FR-CRW-008**: Crewed missions MUST be materially harder than equivalent robotic missions — in launched mass, cost, ops burden and approvals — with the difference fully traceable to modelled causes (no arbitrary multipliers), verified by reference-mission comparisons.

### FA-09 — Politics, Events, Milestones & AI World

- **FR-POL-001**: Per-faction relationships (cooperation ↔ rivalry), partnership/consortium state and prestige MUST be tracked; prestige MUST feed agency budgets and company valuation/contract access.
- **FR-POL-002**: Public and political mood MUST respond to successes, failures (loss-of-crew most severely and durably), spectacular firsts, accidents and economic cycles, and MUST drive appropriation modifiers, approval delays and the probability of directed or cancelled programs.
- **FR-POL-003**: A policy/treaty layer MUST model launch licensing and range access, nuclear-launch approval (gating NTP/NEP/RTG flights), planetary-protection stringency, export controls (constraining specific factions' supply chains and partnerships), and debris/sustainability rules; world policy MUST drift over time and respond to events; factions MUST be able to lobby at political-capital cost.
- **FR-POL-004**: The event system MUST be seeded plus state-driven — anomaly probability earned from TRL immaturity, low heritage, poor maintenance and ops over-subscription, never pure RNG — and MUST feed the FA-01 interrupt system. Event classes MUST include at minimum: launch outcomes, vehicle anomalies, solar storms, funding crises/booms, political shake-ups, rival milestones, discoveries, supply shocks and personnel events.
- **FR-POL-005**: Approximately 120 historic firsts MUST be defined in data with world-first and faction-first tiers; world-first MUST award once globally with prestige and possible funding/market effects; a milestone race board MUST show progress of all factions.
- **FR-POL-006**: Four selectable Grand Goals (Pathfinder, Homestead, Prospector, Seeker) MUST shape scoring, be selectable at start and changeable mid-run at a penalty, and be evaluated at the scoring horizon.
- **FR-POL-007**: The nine non-player organisations plus always-AI CNSA and minor agencies MUST run functional versions of the same systems — researching (feeding the world tide), building, flying, bidding, partnering, hitting milestones and suffering accidents — under identical physics and plausibility rules; difficulty MUST tune their funding/competence only.
- **FR-POL-008**: Soft-fail states MUST be implemented: agency gutting, private bankruptcy, and loss-of-crew spiral, each transitioning the player to observer/rebuild mode rather than deleting the world.
- **FR-POL-009**: No event, policy or AI behaviour may introduce combat, sabotage or alien actors (FR-XCU-008).

### FA-10 — UI Shell, Screens & Sojournal

- **FR-UI-001**: The UI MUST present a persistent shell — top bar (date, warp controls, key currencies, alerts), navigation, central work area, context/inspector panel, event ticker — hosting twelve screens: System Map, Trajectory/Manoeuvre Planner, Research & Development, Vehicle Designer, Operations/Fleet, Economy & Contracts, Bases & Construction, Personnel, World/Politics, Science Returns & Astrobiology, Sojournal, and Alerts/Event Log, each per its design definition (design/06-UI-UX.md S1–S12).
- **FR-UI-002**: The System Map MUST be 2D, logarithmically zoomable and multi-focus (heliocentric ↔ planetocentric ↔ local ops) with selectable inertial/rotating frames, trajectory overlays labelled with delta-v/time-of-flight, SOI/Lagrange regions, and toggleable layers (resources, comms coverage, traffic, planetary-protection zones, science).
- **FR-UI-003**: The planner UI MUST integrate porkchop plots, the manoeuvre-node editor, low-thrust arc planning, flyby design and aerocapture planning with live delta-v-versus-available checks against the selected vehicle, plan saving, burn queueing and node auto-pause.
- **FR-UI-004**: The R&D screens MUST show domain UL bars with world-tide comparison and breakthrough insight-pressure hints; TRL ladders with test-campaign status, P50/P80-versus-actual, risk indices and dead-end warnings; personnel assignment; and a tech-graph view with prerequisites and source tags.
- **FR-UI-005**: Every derived number on every screen MUST be drillable to its derivation (the FR-VEH-003 traceability generalised game-wide).
- **FR-UI-006**: All irreversible actions MUST follow plan → preview → commit with consequences shown before confirmation.
- **FR-UI-007**: Operations, economy, base, personnel, politics and science screens MUST present their areas' state as filterable, sortable, virtualised tables and purpose-built widgets (delta-v ladders, resource-by-location ledger, logistics-graph view, base schematic gauges, astrobiology evidence meter, milestone race board).
- **FR-UI-008**: The Sojournal MUST provide searchable, cross-linked, source-cited entries for every body, technology, resource process, manoeuvre type and discovered result, staying consistent with the simulation, updating with the player's belief state, and serving as the soft-tutorial layer; it MUST never present misinformation as fact. *(Constitution VIII)*
- **FR-UI-009**: The event log MUST be chronological and filterable, with per-class pause configuration (driving FR-SIM-003) and links from each event to its relevant screen.
- **FR-UI-010**: Interaction MUST be mouse-first and keyboard-rich: hotkeys for warp, screens and common verbs; context menus on every object; hover summaries with pinnable detail panels; full keyboard operability.
- **FR-UI-011**: Standing orders and queues (auto-resupply, depot level keeping, station-keeping automation) MUST curb late-game micromanagement while preserving manual override.
- **FR-UI-012**: Accessibility MUST include colour-blind-safe palettes, scalable text, full keyboard navigation and readable layouts from 1280×720 to 4K; dense tables MUST virtualise (FR-XCU-009).
- **FR-UI-013**: Onboarding MUST include a guided first-agency scenario, contextual first-use tips, optional historical scenarios, and assist toggles (e.g., trajectory-solver help) that can be disabled.
- **FR-UI-014**: The UI MUST contain no game logic and communicate with the core only via the FR-XCU-004 boundary.

### Key Entities

- **Game / Run**: a seeded instance — seed, faction choice, difficulty, horizon, decision/event log; replayable and saveable.
- **Faction**: one of ten playable organisations (plus always-AI CNSA and minor agencies); funding model, starting assets, political constraints, heritage discounts, relationships, prestige, trust.
- **Body**: Sun/planet/moon/dwarf/asteroid/comet — sourced orbital elements and physical/environmental data; per-body ground truth and per-faction belief state; Geoscience UL sub-track.
- **Site**: a characterised surface location on a body — true and believed properties (resources, grade, illumination, slope, hazards, comms, science value, planetary-protection category).
- **Dynamical Location**: non-body logistics node (orbit band, Lagrange point/NRHO, low orbit, cycler) shared between world model and transport graph.
- **Knowledge Domain**: a science track with faction UL and World UL, synergy links, insight pressure and breakthrough state.
- **Engineering Program**: a project advancing one Technology through TRL gates — scope, budget, schedule (estimate vs actual), test campaign, staff, facilities, risk index, dead-end status.
- **Technology**: a concrete capability at a maturity (TRL + flight heritage + reliability curve) with a sourced physical parameter set consumed by the designer.
- **Vehicle Design / Vehicle Unit**: a versioned class composed of component technologies with computed performance; physical instances with state (location, propellant by tank, health, heritage, crew).
- **Flight Plan / Manoeuvre Node**: planned trajectory artefacts — burn components, windows, predicted vs flown state, TCMs.
- **Resource Lot**: a quantity of a commodity or strategic material at a delta-v address.
- **Transport Edge**: window-constrained transfer between nodes priced in delta-v and time.
- **Facility**: capital asset (lab, test stand, pad, ground segment, factory, space-side infrastructure) with capex/opex/capacity/upgrades.
- **Station / Base**: module composition at a location/site with emergent properties (power margin, closure, population capacity, sustainability index) and construction-project lineage.
- **Person**: scientist/engineer/PM/astronaut/controller/diplomat — skills, traits, assignment, morale; astronauts add training, health, career dose, psychology.
- **Contract / RFP**: posted requirement, terms, bids, award, performance state, penalties.
- **Partnership / IP Asset**: co-development agreements, licences, patents with royalty and tide effects.
- **Event**: seeded/state-driven occurrence with class, pause behaviour, effects and log entry.
- **Milestone ("First")**: scored achievement with world-first/faction-first tiers and effects.
- **Policy**: world/faction regulatory state (licensing, nuclear approval, planetary-protection stringency, export controls) with lobbying hooks.
- **Astrobiology Candidate**: a world's seeded truth plus staged public evidence state and consensus level.
- **Sojournal Entry**: source-cited encyclopedia article bound to game objects, updating with belief state.
- **Save / Event Log**: versioned, migratable persisted state; append-only decision/event record sufficient for replay.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (Determinism)**: 100% of CI determinism runs — the same seed and scripted decisions executed twice, including saves/loads at arbitrary points — produce identical end states across a suite of at least 50 seed/script combinations covering all ten feature areas.
- **SC-002 (Physics validity)**: All analytic validation cases (Hohmann delta-v, two-body periods, flyby geometry, rocket-equation identities) pass within the tolerances declared in the validation data; planning-aid outputs reconcile against the authoritative propagator within their declared approximation bounds in 100% of test scenarios.
- **SC-003 (Performance at scale)**: A full world — 3,000+ catalogued bodies, at least 200 active player+AI craft, complete economy and AI factions — sustains the maximum warp rate (≈1 year/min) on the reference machine (high-end consumer desktop: 8+ performance cores, discrete GPU, 32 GB RAM), so that an uninterrupted simulated century completes in under 2.5 hours of wall-clock warp time; UI tables remain responsive (no interaction stall perceptible to the user) at 3,000+ rows.
- **SC-004 (Persistence)**: 100% of save/load round-trip tests, including saves taken during burns, EDL and docking, restore identical state; saves from older v1.x versions load via migration in 100% of migration tests.
- **SC-005 (Source coverage)**: 100% of plausibility-bearing data entries carry non-empty source citations, enforced by CI; a release build contains zero unsourced entries.
- **SC-006 (Seeding fairness)**: Across an automated sample of at least 1,000 seeds, every capability category (cheap launch, lunar propellant, Mars ascent, MW-class deep-space tug, closed life support, ocean-world access) retains at least one viable technology path in 100% of seeds.
- **SC-007 (Crewed difficulty)**: For each reference destination pair (lunar surface, Mars surface), the minimal crewed mission requires at least 3× the launched mass and at least 3× the total cost of the minimal robotic mission delivering equivalent payload, with the entire difference attributable in the traceability view to modelled causes.
- **SC-008 (Living world)**: In headless AI-only century runs, AI factions collectively claim at least 60% of the ~120 milestones by the 2126 horizon in the median run, with zero physics or plausibility violations detected by the audit suite.
- **SC-009 (Traceability)**: For a random sample of 100 derived numbers across all screens, 100% expand to complete derivations terminating in sourced inputs.
- **SC-010 (Educational honesty)**: 100% of Sojournal entries carry at least one citation; an expert review pass finds zero entries presenting misinformation as fact; every gameplay mechanic in the ten areas has at least one corresponding entry.
- **SC-011 (Learnability)**: At least 80% of first-time players (playtest cohort) complete the guided onboarding scenario and reach their first milestone within 2 hours of play without external help.
- **SC-012 (Accessibility)**: All gameplay actions are achievable keyboard-only; all screens pass colour-blind-safe palette checks and remain readable and operable at 1280×720 and 4K with scaled text.
- **SC-013 (Scope purity)**: The automated scope audit finds zero combat, weapon, sabotage or alien-actor logic in v1.0 releases; reserved extension points contain no executable behaviour.
- **SC-014 (Pause integrity)**: In 100% of tested event-class configurations, every event of a pause-enabled class interrupts before its effects require player input, and no configured event is ever silently dropped — including when multiple events coincide on one tick.

## Assumptions

- **Single-player, fully offline, desktop-class experience.** No multiplayer and no networked features of any kind in v1.0 — including no telemetry and no automated crash reporting. Field diagnostics rely on players manually sharing saves and event journals, which (by FR-XCU-003/FR-XCU-011) make reported bugs exactly reproducible. (Platform/OS targets are a plan-level decision; this spec is stack-agnostic.)
- **MVP composition**: the P1 user stories (sim core & time, astrodynamics, world data, plus the map/planner slices of the UI) constitute the minimum demonstrable game; P2 adds the strategic layer (research, designer, economy, logistics); P3 completes the 4X experience (bases, crew, politics/AI, astrobiology). Sequencing within that frame is a plan-level decision.
- **English-only at v1.0**, with player-facing text externalised so localisation is possible later without rework.
- **Audio is minimal and non-gameplay-bearing** (ambient/alert sounds at most) and is not specified by this document; no design source defines audio scope.
- **Content counts are targets, not exact contracts**: ~150 moons, ~3,000 small bodies, ~120 milestones, ten faction definitions; ±10% during content production is acceptable where data quality demands it.
- **The January 2026 baseline is a fixed historical snapshot**: real capabilities as of that date are frozen into faction/world data and do not chase later real-world developments.
- **National agencies use real names and real government vehicles** (e.g., SLS/Orion, Ariane 6, Soyuz, H3); the five private companies are fictional archetypes that fully replace real commercial firms in the world and launch market (see FR-WLD-012).
- **Difficulty presets and assist toggles** are data-driven configurations within FR-XCU-010's bounds; their count and naming are a design-tuning decision.
- **Frontier technologies (fusion, fission-fragment, He-3) are optional endgame content**: a run is fully completable without them, and some seeds will never produce them.
- **Modding is an architectural property only in v1.0**: data-driven, schema-validated content makes the game mod-friendly by construction, and data files remain human-editable, but documented schemas, third-party content loading and modder-facing validation tooling are not v1.0 deliverables; official mod support is deferred post-1.0, and schema stability is not yet a public contract (clarified 2026-06-12).
- **The constitution (v1.0.0) and design/ documents are authoritative**; where this spec summarises them, the source documents govern detail-level questions, and child specs must cite both.
- **Numeric design-intent values** quoted from design docs (UL thresholds, breakthrough cadence, warp ceiling, milestone counts) are defaults to be encoded in sourced data files, tunable without code changes per FR-XCU-002.

## Out of Scope (v1.0)

- Weapons, combat, sabotage, espionage-beyond-OSINT, alien civilisations or any agent-like life (Constitution IX; extension points only, no logic).
- Interstellar travel; terraforming beyond paraterraforming-scale habitats.
- Multiplayer of any form; real-money or online services; any phone-home functionality (telemetry, automated crash reporting, online updates checks).
- 3D spectacle rendering; the presentation is data-dense 2D.
- Tile-based surface simulation (surface play is site-based).
- Nuclear-pulse (Orion-type) propulsion as gameplay (Sojournal historical entry only).
- Tech-stack, engine and language choices (deferred to planning per constitution).

## Clarifications

### Session 2026-06-12

- Q: Must bit-identical determinism hold across platforms/CPUs, or per platform/build only? → A: Per platform and build only; saves remain loadable cross-platform, cross-platform replay identity is not a v1.0 requirement (FR-XCU-003).
- Q: How is the real 2026 commercial launch sector represented? → A: The five fictional companies fully replace real commercial firms; their vehicles stand in for real commercial lift with sourced realistic performance; agencies fly real-named government vehicles (FR-WLD-012).
- Q: Is end-user modding a v1.0 deliverable? → A: No — architectural property only; documented schemas, mod loading and modder tooling deferred post-1.0 (Assumptions).
- Q: What hardware baseline anchors the performance targets? → A: A high-end consumer desktop (8+ performance cores, discrete GPU, 32 GB RAM) is the reference machine for all tick-time/frame-time budgets (FR-XCU-009, SC-003).
- Q: When content data changes in a patch, what do existing saves run against? → A: Saves pin the content-data version they started with; in-progress runs keep their original data, new runs use the latest data (FR-XCU-006).
- Q: What is the autosave/crash-recovery policy? → A: Event-log journaling — after a crash the game recovers by replaying seed + journal to the moment of interruption, plus autosaves at key events; applies in both save-anywhere and ironman modes (FR-SIM-010).
- Q: What telemetry/crash-reporting does the shipped game include? → A: None — the game is fully offline with no telemetry and no automated crash reporting; players share saves/event journals manually for bug reports (Assumptions, Out of Scope).

No open [NEEDS CLARIFICATION] markers remain; all other potential ambiguities are resolved with documented assumptions above.
