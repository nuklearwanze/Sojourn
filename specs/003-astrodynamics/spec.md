# Feature Specification: Astrodynamics & Flight (FA-02)

**Feature Branch**: `003-astrodynamics`
**Created**: 2026-06-12
**Status**: Draft
**Input**: User description: Build Sojourn's astrodynamics — the orbital-mechanics layer that is the physical source of truth for where everything is and how it moves: an authoritative deterministic n-body propagator (third-body, J2, solar radiation pressure, drag, continuous low-thrust) reconciled with a fast patched-conic planning tier; reference frames and spheres of influence; manoeuvre nodes with delta-v budgeting, porkchop/launch-window solving, gravity-assist chaining, low-thrust spiral arcs, research-gated low-energy transfers, aerocapture/aerobraking geometry; a propulsion interface (mass, Isp, thrust, throttle, power) consumed here and implemented in the vehicle slice; TCMs and execution error; validated headlessly against analytic cases. No top speed, no reactionless motion. No tech stack.

> **Position in the programme.** Child specification for feature area **FA-02** of the umbrella
> spec (`specs/001-sojourn-solar-4x/spec.md`), refining FR-AST-001…013/015 (see Scope Boundary
> for FR-AST-014/EDL). Built as a `SimModule` on the FA-01 kernel
> (`specs/002-sim-core/contracts/module-contract.md`): it inherits the integer-tick clock,
> kernel-managed cadence with fine-step escalation, named random streams, single-writer slice
> ownership, and the determinism obligations (ordered iteration, libm-only transcendentals,
> seed-derived randomness, warp invariance). Authoritative sources:
> `design/04-SPACEFLIGHT.md` §1–3, §8; `.specify/memory/constitution.md` v1.0.0
> (Principles I, II, III). Where this spec is silent, those documents govern.

---

## Scope Boundary

**This slice delivers** the physics of motion: the authoritative numerical propagator and its
perturbation models; reference frames, sphere-of-influence structure and Lagrange-point regions;
the analytic planning tier and its reconciliation against the truth; manoeuvre nodes, finite-burn
execution, delta-v budgeting and the rocket equation; the planning solvers (porkchop, flyby/
assist, low-thrust arcs, low-energy transfers, aerocapture/aerobraking corridors); the propulsion
*interface* (consumed here, implemented by FA-04); execution error and trajectory-correction
manoeuvres; and the analytic validation suite (Hohmann, periods, flyby) as CI gates.

**This slice does NOT deliver**: the real Solar-System catalogue and ephemerides (FA-03 — this
slice consumes a celestial-body interface and ships a small sourced test catalogue as fixtures);
the propulsion implementations, tanks, staging and boil-off (FA-04 — this slice consumes the
propulsion interface; boil-off arrives as FA-04 mass-state changes that this slice's propagation
simply reflects); research gating levels (FA-05 supplies the Astrodynamics-understanding gate
value; this slice consumes a boolean/threshold input); all UI (FA-10 renders the planner over
this slice's computed results); and atmospheric **entry-descent-landing to a surface** —
aerocapture and aerobraking (flight through the upper atmosphere that exits back to orbit) are
in scope; surface EDL is not. **Clarified 2026-06-13**: this slice ends at aerocapture/
aerobraking plus a deterministic impact/entry handoff event; the umbrella's FR-AST-014
surface-EDL phase (entry-descent-landing to touchdown, propulsive descent, landing risk) is
split out to its own slice after FA-04 exists, because a meaningful EDL simulation needs the
vehicle properties (heat shield, T/W, throttle, guidance) that slice owns. This umbrella
traceability split is recorded here deliberately.

## User Scenarios & Testing *(mandatory)*

Actors: **the player** (plans and flies trajectories — through the harness now, FA-10 later)
and **the integrator/validator** (builds dependent slices, verifies physics against reality).

### User Story 1 - The Truth: Propagate Everything Honestly (Priority: P1)

A craft coasting in orbit moves because gravity moves it. The propagator advances every craft
under the gravitational influence of the relevant bodies plus the perturbations that matter —
third-body pull, oblateness (J2), solar radiation pressure, atmospheric drag where an atmosphere
exists, and continuous low-thrust when engines run. Orbits precess, halo orbits need
station-keeping, LEO orbits decay. Nothing has a top speed; nothing moves without a modelled
force; the same seed and decisions always produce the identical trajectory.

**Why this priority**: This is the source of truth everything else reconciles against
(Constitution Principle II). Every other story consumes it.

**Independent Test**: Headless validation suite: two-body propagation matches analytic periods
and conserves energy within stated tolerance over long spans; J2 produces the textbook nodal
regression for a reference LEO orbit; drag decays a low orbit monotonically; SRP displaces a
high-area craft measurably; all bit-identical across double runs.

**Acceptance Scenarios**:

1. **Given** a craft in a two-body orbit with no perturbations enabled, **When** it is propagated for 100 orbital periods, **Then** the period matches the analytic value within the documented tolerance and orbital energy drift stays within the documented bound.
2. **Given** a reference LEO orbit around an oblate body, **When** propagated with J2 enabled, **Then** the ascending node regresses at the analytic J2 rate within tolerance.
3. **Given** a craft below the documented drag-relevant altitude of a body with an atmosphere, **When** propagated with drag enabled, **Then** its orbit decays monotonically, and an identical craft above that altitude is unaffected.
4. **Given** a craft with continuous low thrust applied via the propulsion interface, **When** propagated, **Then** its energy change equals the work done by thrust (within integration tolerance) and its mass decreases per the exhaust-velocity relation — never thrust without mass flow.
5. **Given** identical seeds and command logs, **When** the same scenario is run twice with different stepping patterns, **Then** every craft's state is bit-identical at every checkpoint (kernel double-run gate, with this module installed).

---

### User Story 2 - Plan a Burn, Fly It, Feel the Equation (Priority: P1)

The player places a manoeuvre node on a craft's trajectory: a burn defined in prograde/radial/
normal components at a chosen time. The planner instantly shows the predicted resulting orbit
(patched-conic tier), the delta-v cost, and the propellant it will consume against the craft's
actual budget via the rocket equation — flagging shortfalls before commitment. Committed nodes
auto-pause the game when due (kernel `maneuver-node` event). The flown burn is a finite burn:
thrust-limited duration, gravity and steering losses included, so the flown result differs
slightly from the impulsive plan — and the system shows that reconciliation honestly.

**Why this priority**: Manoeuvre planning is the player's core spaceflight verb
(design 04 §1.2) and the rocket equation as a felt constraint is constitutional (Principle VII).

**Independent Test**: Headless: place a node for a Hohmann transfer; verify the planned
impulsive delta-v matches the analytic value; fly it finite-burn and verify the achieved orbit
within documented finite-burn tolerance; verify propellant consumed matches the rocket equation;
verify a node exceeding available delta-v is flagged at planning time.

**Acceptance Scenarios**:

1. **Given** a craft in a circular orbit, **When** the player plans a Hohmann transfer to a higher circular orbit via two nodes, **Then** the planner's impulsive delta-v matches the analytic Hohmann value within the validation tolerance.
2. **Given** a committed node, **When** simulated time reaches it, **Then** the kernel pauses (maneuver-node event) before the burn, and on approval the burn executes as a finite burn whose losses are computed, not ignored.
3. **Given** a planned burn costing more delta-v than the craft's remaining budget, **Then** the plan is flagged infeasible at planning time and re-flagged if state changes invalidate a committed plan.
4. **Given** an executed burn, **Then** propellant consumed equals the rocket-equation prediction for the achieved delta-v within tolerance, debited through the propulsion interface.
5. **Given** a chain of nodes (multi-burn plan), **Then** each node's prediction builds on the previous node's predicted outcome, and the whole chain's total delta-v is budgeted against the craft.
6. **Given** seeded execution error enabled, **When** the same burn is flown, **Then** the achieved delta-v deviates per the seeded error model (deterministically reproducible), and the divergence from plan is quantified for TCM planning.

---

### User Story 3 - Windows: When to Go (Priority: P2)

The player asks "when should I depart for that body?" and receives a porkchop result: a grid of
departure × arrival dates with computed delta-v / C3 / time-of-flight, solved from the actual
ephemerides — revealing the synodic rhythm (the ~26-month Mars window emerges; it is nowhere
scripted). Selecting a grid point produces a concrete transfer plan (departure burn node) ready
to refine and commit.

**Why this priority**: Transfer windows are the campaign heartbeat of the whole game; they must
emerge from real geometry.

**Independent Test**: Headless: porkchop between two test-catalogue planets on known orbits;
verify the optimal grid point's delta-v matches the analytic Hohmann window for the
near-circular coplanar case within tolerance, and that successive optima recur at the synodic
period.

**Acceptance Scenarios**:

1. **Given** two bodies on near-circular coplanar orbits, **When** a porkchop grid is solved across a span covering two synodic periods, **Then** minimum-delta-v points recur at the synodic period and their value matches the analytic transfer within tolerance.
2. **Given** any grid point, **When** the player converts it to a plan, **Then** the resulting departure node reproduces the grid's stated delta-v/TOF when propagated (within the planner-tier tolerance).
3. **Given** grid points where the transfer geometry has no solution at the requested revolution count, **Then** those cells are reported as unsolvable rather than fabricated.

---

### User Story 4 - Steal Speed: Flybys and Assist Chains (Priority: P2)

The player plans a flyby: approach a body with some v-infinity, choose a periapsis, and the
geometry tool computes the turning angle and the heliocentric velocity change — energy honestly
conserved in the body's frame, gained in the Sun's. Flybys chain: the outbound leg of one
becomes the inbound constraint of the next, so multi-assist routes can be composed and their
total delta-v savings quantified against a direct transfer.

**Why this priority**: Gravity assists are a defining real-physics capability (design 04 §1.2);
they must fall out of the propagation, not a lookup table.

**Independent Test**: Headless: reproduce a documented textbook flyby (known v∞, periapsis →
known turn angle and post-flyby heliocentric state) within the validation tolerance; verify the
propagated flyby matches the planner's predicted geometry; verify a two-assist chain's composed
prediction against propagation.

**Acceptance Scenarios**:

1. **Given** a craft approaching a body with known v-infinity and target periapsis, **When** the flyby tool computes the encounter, **Then** turning angle and outbound v-infinity match the analytic hyperbolic-encounter values within tolerance, and the flown (propagated) flyby matches the prediction within the planner-reconciliation tolerance.
2. **Given** a planned periapsis below the body's surface or atmosphere-interface altitude, **Then** the plan is flagged invalid (impact/entry), never silently accepted.
3. **Given** a chained two-flyby plan, **Then** the composed prediction (leg → encounter → leg → encounter) is checked end-to-end against propagation and the cumulative divergence is surfaced.

---

### User Story 5 - Patience as Propellant: Low-Thrust Arcs (Priority: P2)

For a craft whose propulsion interface reports low thrust and high exhaust velocity (an
electric-propulsion stand-in), the player plans a continuous-thrust arc under a guidance law —
e.g. a months-long spiral out of a gravity well. The planner estimates duration and propellant;
the propagator flies the arc applying thrust each step, power-limited per the interface. The
characteristic trade is felt: superb propellant efficiency, terrible patience.

**Why this priority**: Low-thrust is a pillar of the tech tree's strategy space (NEP cargo);
the propagation and planning must support it natively, not as teleporting approximations.

**Independent Test**: Headless: spiral from a low test orbit to a target radius under tangential
thrust; verify duration and propellant against the analytic low-thrust spiral estimate within
documented tolerance; verify thrust ceases when the interface reports zero available power or
propellant.

**Acceptance Scenarios**:

1. **Given** a craft with thrust-to-weight far below impulsive regimes, **When** a tangential-steering spiral to a higher orbit is planned, **Then** the planner's duration/propellant estimate matches the flown result within the documented low-thrust tolerance.
2. **Given** the propulsion interface reports reduced available power, **Then** delivered thrust scales accordingly during propagation (power-limited thrust, FR-VEH-006 coupling) — never free thrust.
3. **Given** propellant exhausts mid-arc, **Then** thrust ends at that instant, the event is surfaced, and the trajectory continues ballistically.

---

### User Story 6 - Brake with the Sky: Aerocapture and Aerobraking (Priority: P3)

At a body with an atmosphere, the player plans an aerocapture: target an entry corridor, fly
through the upper atmosphere where drag sheds speed, and exit into a capture orbit — saving
propellant at the price of risk. Aerobraking does the same gently over many passes. The corridor
geometry (too shallow = skip out, too steep = too deep/structural-thermal limit exceeded) is
computed from the atmosphere model and surfaced before commitment; the flown pass uses the same
drag physics as everything else.

**Why this priority**: Delta-v-for-risk trades are core strategy; depends on drag modelling
(US1) and planning machinery (US2), so it lands after them.

**Independent Test**: Headless: for a test body with a documented exponential atmosphere,
verify a corridor-centre entry produces a bound orbit with the predicted apoapsis (within
tolerance), a too-shallow entry exits hyperbolic, and a too-steep entry crosses the
depth/load limit and is reported as such.

**Acceptance Scenarios**:

1. **Given** an arrival hyperbola and a chosen entry-corridor target, **When** the pass is flown, **Then** the exit orbit matches the corridor prediction within the documented aerocapture tolerance.
2. **Given** an entry shallower than the corridor, **Then** the craft skips out (remains hyperbolic) — predicted and flown consistently.
3. **Given** an entry steeper than the documented limit, **Then** the plan is flagged as exceeding the limit at planning time; if flown anyway, the violation event is raised (consequences belong to the vehicle/EDL slice).
4. **Given** repeated aerobraking passes, **Then** apoapsis reduces monotonically per pass consistent with the drag model.

---

### User Story 7 - See Further: Low-Energy Transfers (Priority: P3)

Once the research gate (supplied by the research slice; a configuration input here) is open, the
planner offers low-energy/weak-stability-boundary routes: slower transfers exploiting multi-body
dynamics that cost meaningfully less delta-v — e.g. a ballistic lunar capture. Until the gate
opens, the option is absent. The routes are real: the propagator (which models multi-body
gravity always) confirms them.

**Why this priority**: A research-gated planning unlock (design 04 §1.2) — strategic richness,
not core mechanics; multi-body truth (US1) already supports it.

**Independent Test**: Headless: with the gate open, plan the catalogue's reference low-energy
route; verify capture occurs under propagation with the advertised delta-v saving versus the
direct transfer; with the gate closed, verify the planner does not offer the route.

**Acceptance Scenarios**:

1. **Given** the research gate closed, **Then** low-energy route planning is unavailable while all other planning remains unaffected.
2. **Given** the gate open, **When** the reference low-energy transfer is planned and flown, **Then** ballistic capture occurs as predicted and total delta-v undercuts the direct-transfer baseline by the documented margin.

---

### User Story 8 - Stay on Course: Drift, Error and Correction (Priority: P2)

Plans are approximations; flight is truth. As a mission flies, the system tracks divergence
between the committed plan's prediction and the propagated reality (perturbations the planner
tier ignores, finite-burn losses, seeded execution error). When divergence exceeds a threshold,
it is surfaced, and the player plans a trajectory-correction manoeuvre — a small node computed
to re-target the original aim point. Station-keeping (halo upkeep, drag make-up) appears as the
same mechanism on a schedule: ongoing small burns that consume real propellant.

**Why this priority**: Reconciliation is the constitutional contract between the two tiers
(Principle II); execution error and TCMs are what make plans feel real rather than scripted.

**Independent Test**: Headless: commit an interplanetary-class plan in the test catalogue; let
perturbations and seeded execution error accumulate divergence; verify divergence is quantified
and monotonically tracked; compute a TCM and verify the re-targeted arrival within tolerance;
verify a halo-class orbit decays without station-keeping and persists with it.

**Acceptance Scenarios**:

1. **Given** a committed plan and ongoing propagation, **Then** the predicted-vs-actual divergence is computable at any boundary and is surfaced when it exceeds the documented threshold.
2. **Given** seeded execution error on a departure burn, **When** a TCM is computed mid-transfer, **Then** flying the TCM brings the arrival-point error within the documented re-target tolerance — and the whole sequence is bit-identically reproducible from the seed.
3. **Given** an unstable reference orbit (Lagrange-region class), **Then** without station-keeping the craft departs the region within the predicted timescale, and with scheduled station-keeping burns it remains, consuming the predicted propellant.

---

### Edge Cases

- **SOI handoff continuity**: crossing a sphere-of-influence boundary must not discontinuously change the craft's physical state — only the bookkeeping frame; position/velocity remain continuous to integration tolerance, and the crossing emits a deterministic event.
- **Burn spanning an SOI crossing or node chain boundary**: finite burns that straddle reference changes must integrate correctly through them.
- **Propellant exhaustion mid-burn**: thrust ends exactly at exhaustion (sub-step accuracy), remaining delta-v shortfall is quantified, plan invalidation fires.
- **Node in the past / invalidated node**: a committed node whose craft state has changed beyond tolerance (or that coincides with another interrupt on the same tick — resolved by the kernel's documented intra-tick order) must flag for re-planning, never execute blindly.
- **Surface/atmosphere intersection while coasting**: a trajectory that intersects a body's surface (or the atmosphere-interface altitude when not flying a planned aero pass) raises an impact/entry event deterministically; what happens to the craft is the vehicle slice's concern — this slice stops propagating it and marks the state.
- **Degenerate Lambert geometry**: porkchop cells with near-180° transfer angles or impossible revolution counts must report unsolvable, not produce garbage.
- **Deep gravity-well precision**: periapsis passes deep in a gravity well (flybys, aerobraking) must hold accuracy — fine-step escalation must engage by state, not by luck.
- **Eclipse/shadow for SRP**: SRP must switch off (or scale) deterministically when the body is occulted; the boundary crossing must not introduce integrator instability.
- **Very long coasts**: decades-long outer-system cruises must neither drift unphysically (energy bound) nor stall the kernel's quiet-span jumping (cadence/escalation declared correctly).
- **Mass edge cases**: zero-propellant burns, near-empty tanks, mass approaching dry mass — no divide-by-zero, no negative mass, ever.
- **Simultaneous nodes**: two craft with nodes at the same tick, or one craft with a node and an SOI crossing at the same tick — deterministic ordering per the kernel's documented intra-tick order.
- **Planner-truth divergence blowup**: when the patched-conic tier is meaningfully wrong (multi-body regions), the reconciliation must say so rather than letting the player trust a fiction (honesty requirement, Principle VIII).
- **Diverted-body transitions**: taking a small body off its rail (and re-railing it) must preserve state continuity exactly at the transition instant, must not retroactively alter history, and a diverted body must interact with craft planning (porkchop targets, encounters) identically to a railed one.

## Requirements *(mandatory)*

IDs are FR-ASTRO-###, grouped by concern. Umbrella traceability (FR-AST-###) and kernel-contract
obligations noted inline. All quantitative model constants (gravitational parameters, J2,
atmosphere scale heights, tolerances) live in schema-validated data files with `source` fields
(FR-XCU-001/002) — the physics engine contains no per-body magic numbers (Principle II).

### Propagation — the source of truth (FR-AST-001, 015)

- **FR-ASTRO-101**: The module MUST propagate every craft numerically with a deterministic fixed-step integration scheme under the gravitational acceleration of the relevant celestial bodies, as the authoritative state — planning tiers never overwrite it. *(FR-AST-001)* The gravitating set is rule-defined (clarified 2026-06-13): bodies carry a catalogue **gravitating flag** (set from sourced masses — planets, major moons, large small-bodies); flagged bodies exert far-field gravity on all craft, unflagged bodies exert none — except that **any** body is gravitating for craft within its own sphere of influence (you can orbit the asteroid you are mining). The rule is deterministic, data-driven, and the validation suite states which bodies participate in every case.
- **FR-ASTRO-102**: Perturbations MUST be modelled and individually enableable per configuration: third-body gravity, primary-body oblateness (J2), solar radiation pressure (with occultation handling), atmospheric drag below each body's documented drag-relevant altitude, and continuous low-thrust acceleration from the propulsion interface. *(FR-AST-001)*
- **FR-ASTRO-103**: Celestial bodies MUST be supplied through a consumed catalogue interface (gravitational parameter, radius, J2, rotation, atmosphere model, ephemeris/trajectory) — fed by FA-03 in production and by a small, sourced test catalogue shipped as fixtures in this slice. Celestial bodies are **on rails** by default: positions come from analytic ephemerides derived from their orbital elements, exactly faithful to the published catalogue data, while craft are propagated numerically under multi-body gravity from the railed bodies (clarified 2026-06-13).
- **FR-ASTRO-107**: **Small bodies MUST be divertible**: a body-class-gated mechanism (small bodies only — asteroids/comets, never planets or major moons) by which an external action (e.g. a deflection campaign acting through the propulsion interface, arriving with later slices) transitions a small body off its rail into craft-grade numerical propagation, and — once its new path is stable — optionally re-rails it on updated orbital elements. Diverted-body motion obeys the same physics as craft (forces only, no teleportation); the rail→propagated→re-railed transitions are deterministic, journaled consequences of commands, and the number of simultaneously diverted bodies is bounded by a documented performance budget (clarified 2026-06-13: enables late-game asteroid-orbit modification for resource harvesting).
- **FR-ASTRO-104**: There MUST be no top speed, no reactionless acceleration, and no positional teleportation: every state change follows from integrated forces or explicitly journaled load/creation operations. *(FR-AST-015)*
- **FR-ASTRO-105**: Integration MUST hold documented accuracy bounds: two-body energy drift, long-coast stability, and deep-periapsis accuracy each carry explicit tolerances in the validation data, and the integrator's step control MUST derive only from simulation state (kernel fine-step escalation during burns, encounters and atmosphere passes). *(Kernel FR-CORE-104/204)*
- **FR-ASTRO-106**: As a kernel module, the slice MUST declare its cadence and escalation conditions, own exactly its trajectory-state slice, draw randomness only from its declared streams, and pass the kernel conformance suite (double-run identity with varied stepping). *(specs/002-sim-core contracts)*

### Frames, SOIs and regions (FR-AST-003)

- **FR-ASTRO-201**: The module MUST support heliocentric-inertial, body-centred-inertial and rotating (co-orbiting/CR3BP-style) reference frames, with exact, invertible conversions between them at any boundary.
- **FR-ASTRO-202**: Sphere-of-influence structure MUST be derived from the body catalogue and used for planner patching and dominant-body bookkeeping; SOI crossings MUST be detected deterministically, emit events, and preserve physical state continuity.
- **FR-ASTRO-203**: Lagrange-point regions of relevant body pairs MUST be representable as named dynamical locations (the logistics graph's nodes, umbrella FR-WLD-005) with the dynamics that make them meaningful (halo-class orbits exist in propagation; their instability timescales are physical, not scripted).

### Manoeuvres and burns (FR-AST-004, 010, 011)

- **FR-ASTRO-301**: Players MUST be able to create, edit, chain and delete manoeuvre nodes (burn epoch + prograde/radial/normal (`dv_prn`) delta-v components, or a guidance-law arc reference); committed nodes MUST schedule kernel `maneuver-node` events (auto-pause per pause policy). *(FR-AST-004)*
- **FR-ASTRO-302**: Every planned burn MUST be budgeted by the rocket equation against the craft's mass state and the propulsion interface: required propellant, achievable delta-v, and infeasibility flags computed at planning time and re-validated when state changes (plan invalidation events). *(FR-AST-011)*
- **FR-ASTRO-303**: Flown burns MUST be finite burns: thrust-limited duration computed from the interface, with gravity losses and steering losses emerging from integration (impulsive plans get honest corrections, never silent grace). *(FR-AST-010)*
- **FR-ASTRO-304**: Execution error MUST be representable: a seeded, data-parameterised error model (magnitude/pointing) applied per burn from the module's declared random streams — deterministic per seed, zero when configured off. *(FR-AST-010)*
- **FR-ASTRO-305**: Trajectory-correction manoeuvres MUST be plannable: given a committed plan's aim point and the current divergence, the module computes a correction node re-targeting the aim point, with its cost budgeted like any burn. *(FR-AST-010)*
- **FR-ASTRO-306**: Recurring station-keeping MUST be expressible as scheduled small burns (data-defined cadence/budget per orbit class) consuming real propellant. *(FR-AST-013)*
- **FR-ASTRO-307**: Propellant exhaustion mid-burn MUST cut thrust at the exhaustion instant, quantify the shortfall, raise an event, and invalidate dependent plan stages.

### Planning tier and reconciliation (FR-AST-002, 005…009)

- **FR-ASTRO-401**: A fast analytic planning tier (patched-conic + closed-form models) MUST produce instant predictions for orbits, transfers and encounters, explicitly labelled approximate, never mutating authoritative state. *(FR-AST-002)*
- **FR-ASTRO-402**: Reconciliation MUST be continuous: for any committed plan the module computes predicted-vs-propagated divergence at tick boundaries, surfaces it past a documented threshold, and the planner's known blind spots (multi-body regions) are flagged as low-confidence rather than presented as truth. *(FR-AST-002, Principle VIII)*
- **FR-ASTRO-403**: Porkchop solving MUST compute departure×arrival grids (delta-v, C3, time-of-flight) between any two catalogue locations from their ephemerides, including multi-revolution options, reporting unsolvable cells honestly; a selected cell converts to a concrete node plan. *(FR-AST-005)*
- **FR-ASTRO-404**: Flyby planning MUST solve single hyperbolic encounters (v∞, periapsis → turn angle, outbound state) and support chaining encounters into multi-assist routes with end-to-end divergence checking; periapsides below surface/atmosphere-interface MUST be flagged invalid. *(FR-AST-006)*
- **FR-ASTRO-405**: Low-thrust planning MUST produce continuous-thrust arcs under named guidance laws (at minimum: tangential steering; the set is data-extensible) with duration/propellant estimates reconciled against propagation. *(FR-AST-007)*
- **FR-ASTRO-406**: Low-energy/weak-stability-boundary transfer planning MUST exist behind a research-gate input (supplied externally; default closed): when open, the planner offers catalogue-defined low-energy route families whose savings are verified by propagation. *(FR-AST-008)*
- **FR-ASTRO-407**: Aerocapture/aerobraking planning MUST compute entry corridors (shallow/steep bounds, predicted exit orbit, per-pass apoapsis reduction) from each body's atmosphere model, with violations (skip-out, depth/load limit) predicted at planning time and raised as events when flown. *(FR-AST-009)*
- **FR-ASTRO-408**: Automated assist-sequence *search* (VEEGA-style route discovery beyond manual chaining) is **deferred to a later enhancement** (clarified 2026-06-13): this slice ships manual chaining with single-encounter solving and end-to-end chain verification (FR-ASTRO-404); the design promise of solver-assisted sequence discovery (design 04 §1.2) remains on the roadmap and MUST be buildable on FR-ASTRO-404's chain machinery without rework (the chain representation is the contract).
- **FR-ASTRO-409**: Structured planner results (porkchop grids, sampled trajectories, encounter solutions, corridors, reconciliation reports) MUST be delivered through a **read-only planning-query surface** on the module: pure functions over a state snapshot plus query parameters, invoked by the host between kernel steps (clarified 2026-06-13). Planning queries never mutate state, never draw from random streams, never enter the journal or the fingerprint; kernel published views stay flat scalars for status and watch bindings. This surface is the contract FA-10's planner screens consume.

### Propulsion interface (consumed; FA-04 implements)

- **FR-ASTRO-501**: The module MUST define and consume a propulsion interface per craft exposing: current total mass and dry mass, available propellant, effective exhaust velocity (or Isp), maximum thrust, throttle range/setting, and available power (for power-limited thrust); thrust application MUST debit propellant via the exhaust-velocity relation and respect throttle and power limits each step. *(FR-VEH-006 coupling; design 04 §3)*
- **FR-ASTRO-502**: Until FA-04 exists, a fixtures-grade implementation of the interface (constant-property test engines, sourced parameter values) MUST ship for tests and scenarios; the interface contract is the deliverable FA-04 implements.

### Validation and performance (FR-XCU-005, SC-003)

- **FR-ASTRO-601**: A headless analytic validation suite MUST run in CI: Hohmann transfer delta-v, two-body orbital periods, J2 nodal regression, and a documented hyperbolic flyby, each within tolerances stated in sourced validation data; failures block merge. *(FR-XCU-005; Constitution Testing gates)*
- **FR-ASTRO-602**: The module MUST meet its share of the performance envelope: with the synthetic full-load profile (3,000+ railed bodies, ≥200 propagated craft, plus the documented diverted-body budget of FR-ASTRO-107), the kernel's ≥1 simulated year per wall-minute target holds on the reference machine. *(Umbrella SC-003; kernel SC-006)*
- **FR-ASTRO-603**: All plausibility-bearing constants (test-catalogue body data, engine fixture parameters, tolerances, atmosphere models, error-model magnitudes) MUST carry `source` citations and pass `validate-data` in CI. *(FR-XCU-001)*

### Key Entities

- **Celestial Body (consumed)**: catalogue-supplied gravitating body — μ, radius, J2, rotation, atmosphere model, ephemeris; production data from FA-03, fixtures here. Motion-state: Railed (default, ephemeris-positioned) | Diverted (small bodies only: craft-grade propagation after an external action) | Re-railed (new elements after a stable diversion) — FR-ASTRO-107.
- **Craft**: a propagated object owned by this slice — position/velocity state, mass state, propulsion endpoint reference, dominant-body/SOI bookkeeping, flight status (propagating / impacted / entry-handoff).
- **Propulsion Endpoint (interface)**: per-craft contract reporting mass, propellant, exhaust velocity, thrust, throttle, power; debited by burns.
- **Trajectory**: the propagated path (authoritative) and its sampled representation for queries/rendering.
- **Manoeuvre Node**: planned burn — epoch, frame-relative delta-v components or guidance-arc reference, predicted outcome, feasibility state, kernel event linkage.
- **Plan**: an ordered chain of nodes/arcs with composed predictions, total budget, aim point(s), divergence state, validity flags.
- **Guidance Law**: named steering rule for continuous-thrust arcs (data-extensible set).
- **Porkchop Grid**: solved departure×arrival lattice with per-cell delta-v/C3/TOF/solvability.
- **Encounter Solution**: hyperbolic flyby geometry — v∞ in/out, periapsis, turn angle, validity.
- **Aero Corridor**: entry-interface geometry — shallow/steep bounds, predicted exit, limits.
- **Reference Frame / SOI Node**: frame definitions and the SOI hierarchy; Lagrange-region named locations.
- **Execution Error Model**: data-parameterised, seeded per-burn error description.
- **Validation Case**: sourced analytic scenario + tolerance consumed by the CI suite.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (Analytic fidelity)**: 100% of CI validation cases pass: two-body periods within 0.01%, Hohmann delta-v within 0.1% (impulsive plan) , J2 nodal-regression rate within 0.5%, reference flyby turn angle and outbound v∞ within 0.5% — all tolerances recorded in sourced validation data.
- **SC-002 (Conservation honesty)**: In unperturbed two-body propagation, relative energy drift stays below the documented bound (≤ 1×10⁻⁸ per simulated year) over a 10-year coast; with perturbations enabled, energy change matches the work done by modelled forces within integration tolerance.
- **SC-003 (Determinism)**: With this module installed, 100% of kernel double-run, round-trip, replay and conformance checks pass, including scenarios with burns, flybys, aero passes and seeded execution error.
- **SC-004 (Planner honesty)**: For the documented well-conditioned regimes, planner-tier predictions reconcile against propagation within the stated per-tool tolerances (Hohmann-class transfers ≤ 1% delta-v; flyby geometry ≤ 0.5°; low-thrust spiral duration ≤ 5%); outside those regimes the result is explicitly flagged low-confidence — zero unflagged silent divergence beyond threshold in the test suite.
- **SC-005 (Felt constraints)**: In the reference scenarios, propellant accounting matches the rocket equation within 0.1%; no test can produce motion without force, thrust without mass flow, or delta-v beyond the budget without an infeasibility flag.
- **SC-006 (Windows emerge)**: The porkchop suite finds the synodic recurrence of transfer windows for the test catalogue within 2% of the analytic synodic period, with no window data scripted anywhere.
- **SC-007 (Performance)**: Under the synthetic full-load profile (3,000+ catalogue bodies, 200 propagated craft, mixed coast/burn activity), the workspace sustains ≥ 1 simulated year per wall-clock minute on the reference machine, and planner queries (single porkchop grid ≤ 40×40, node prediction, flyby solve) each return in under 100 ms.
- **SC-008 (Coverage of the verbs)**: Every planning verb (node, chain, porkchop, flyby, low-thrust arc, low-energy route, aero corridor, TCM, station-keeping) is exercised by at least one headless scenario in the suite.

## Assumptions

- **Kernel contracts hold as shipped** (specs/002-sim-core): this module conforms to `SimModule`, declares cadence + escalation (burns, encounters, atmosphere passes ⇒ fine-step), owns its slice, and uses named streams for execution error.
- **Bodies and ephemerides are consumed, not owned**: FA-03 owns the real catalogue; this slice ships a small sourced **test catalogue** (an idealised Sun + 2–3 planets + 1 moon + 1 atmosphere-bearing body with documented parameters) sufficient for every validation case and scenario.
- **The propulsion interface is the deliverable to FA-04**: fixture engines here use sourced real-engine-class parameters (e.g. a hydrolox-class impulsive stand-in and an ion-class low-thrust stand-in) without modelling engine internals.
- **Research gating is an input**: a configuration/command-supplied gate state (default closed) stands in for FA-05's Astrodynamics understanding level.
- **Surface interaction is a handoff**: impact/entry events mark craft and stop propagation; vehicle/EDL consequences arrive with the vehicle slice (pending the EDL clarification above).
- **Numeric tolerances are data**: every tolerance cited in Success Criteria lives in the sourced validation-data files; the values stated here are the defaults under change control, not hard-coded constants.
- **Planner availability is not gated by craft existence**: what-if planning on hypothetical states is allowed (the designer/UI slices will rely on it); only committed nodes bind to real craft.
- **No relativistic effects**: Newtonian gravity + named perturbations; relativistic corrections are documented as out of scope at game scale (a Sojournal-honesty note, not a simulation feature).

## Out of Scope (this slice)

- The real Solar-System catalogue, ephemerides from JPL/MPC data, sites, belief state (FA-03).
- Propulsion implementations, tanks, staging, boil-off, vehicle design (FA-04) — the interface defined here is their contract.
- Surface EDL (entry-descent-landing to touchdown) and its vehicle-dependent risk model — pending the EDL-boundary clarification; aerocapture/aerobraking are in scope.
- Research model and understanding levels (FA-05) — only the gate input is consumed.
- All UI/rendering (FA-10): porkchop plots and node editors are *computed results* here, delivered through the read-only planning-query surface (FR-ASTRO-409); node creation/commitment flows through kernel commands.
- Launch-to-orbit modelling (ascent from a surface) — arrives with vehicles/launch (FA-04+).
- Relativistic dynamics; interstellar trajectories (constitutionally reserved).

## Clarifications

### Session 2026-06-13

- Q: Bodies on rails or mutual n-body? → A: **On rails** (analytic ephemerides faithful to catalogue elements; craft propagated numerically under multi-body gravity from railed bodies) — **with divertible small bodies**: late-game technology may push an asteroid/comet off its rail into craft-grade propagation and re-rail it on new elements (e.g. moving an asteroid for easier resource harvesting). Never planets or major moons. (FR-ASTRO-103, FR-ASTRO-107)
- Q: Where does FA-02 end — aerocapture or full EDL? → A: Ends at aerocapture/aerobraking + deterministic impact/entry handoff; surface EDL (umbrella FR-AST-014) splits into its own slice after FA-04 supplies vehicle properties. (Scope Boundary)
- Q: Automated assist-sequence search now? → A: Deferred — manual chaining + single-encounter solving ship now; the chain representation is the contract automated search builds on later. (FR-ASTRO-408)
- Q: Which bodies exert gravity on craft? → A: Catalogue-flagged gravitating set (majors + large small-bodies, flagged from sourced masses) exert far-field gravity; unflagged bodies none — except every body gravitates within its own SOI. Deterministic, data-driven (FR-ASTRO-101).
- Q: How do structured planner results reach consumers? → A: A read-only planning-query surface on the module — pure functions over a state snapshot, called by the host between steps; never journaled, never fingerprinted; kernel views stay flat scalars (FR-ASTRO-409).

No open [NEEDS CLARIFICATION] markers remain.
