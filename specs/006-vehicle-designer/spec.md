# Feature Specification: Vehicle Designer & Propulsion (FA-04)

**Feature Branch**: `006-vehicle-designer`
**Created**: 2026-06-13
**Status**: Draft
**Input**: User description: Build Sojourn's vehicle designer and the propulsion/vehicle physics behind it — a component-composition designer (Aurora-style, physics-checked) that builds vehicles from researched parts and computes, with full traceability, every derived number (mass, Δv, thrust, T/W, power, thermal/radiator balance, reliability, cost). Model propulsion families as physical parameter sets feeding the FA-02 propulsion interface (electric power-limited, nuclear-electric radiators as first-class mass); compose reliability from FA-05 technology maturity, flight heritage and domain understanding; define vehicle classes and surface realism red-flags. Data-driven with sources; no per-design magic numbers. No tech stack.

> **Position in the programme.** Child specification for feature area **FA-04** of the umbrella
> spec (`specs/001-sojourn-solar-4x/spec.md`), refining FR-VEH-001…009. It is the **production
> implementer of FA-02's propulsion-interface contract**
> (`specs/003-astrodynamics/contracts/propulsion-interface.md`): the designer's vehicles expose
> the `PropulsionEndpoint` shape the propagator already flies, replacing the FA-02 fixture
> engines. It is a **consumer of FA-05's maturity contract**
> (`specs/005-research-personnel/contracts/maturity-queries.md`): component reliability and
> availability come from `maturity()`/`heritage()`/`understanding()`. Built on the FA-01 kernel
> contracts. Authoritative sources: `design/04-SPACEFLIGHT.md` §2–4, `design/02-TECH-TREE.md`
> §B–F, `.specify/memory/constitution.md` v1.0.0 (Principles I, II, VII). Where this spec is
> silent, those documents govern.

---

## Scope Boundary

**This slice delivers**: the **component catalogue** (researched parts — propulsion, power,
thermal/radiators, structures/tanks, avionics/GNC, comms, life-support kit, payloads, EDL kit,
landing gear, RCS, docking — as sourced data with physical parameter sets); the **propulsion
physics models** per family (chemical; electric ion/Hall/MPD/VASIMR/electrospray; nuclear-thermal;
nuclear-electric; gated frontier), each a physical model (Isp, thrust, T/W, input power, propellant
type/density, throttle, restart/duty, mass model incl. feed/power/radiators, reliability curve)
that produces a **`PropulsionEndpoint`** FA-02 propagates; the **vehicle designer** that composes
components into a design and **derives** total/dry/propellant mass, per-stage/mode Δv, thrust and
T/W at gravity fields of interest, power generation/demand and margin, thermal/radiator balance,
composed **reliability**, and a physical cost/build-time estimate — **with full traceability** of
every number to sourced inputs; **vehicle classes** (templates for launchers, stages/tugs, crewed
vehicles, landers, rovers, station/base modules, probes, …) all built from the one system;
**realism guards** that red-flag the physically impossible; **flight-heritage accrual** that feeds
back to FA-05; and the read-only **design-query surface** other slices and the future UI consume.

**This slice does NOT deliver**: astrodynamics, trajectory propagation, manoeuvre planning,
porkchop/flyby/low-thrust/aerocapture planning (FA-02 — it *flies* the endpoints this slice
produces); the research model itself (FA-05 — this slice *reads* maturity/heritage/understanding);
the **EDL flight phase** dynamics, the **in-mission life-support** consumption/physiology, and the
**economic pricing/funding** of designs — see the clarified boundaries below (this slice computes
the design-time *suitability checks*, *static life-support sizing* and *physical cost estimate*
respectively; the dynamic/economic simulations are FA-02/FA-08/FA-06); politics/approval costs of
nuclear systems (FA-09 — this slice flags the requirement); and UI rendering of any of it (FA-10).

## Clarifications

### Session 2026-06-13

- Q: Where does FA-04 end on EDL/landing? → A: This slice computes **design-time landing/EDL
  *suitability checks*** — T/W vs local gravity, heat-shield adequacy and ballistic coefficient for
  atmospheric bodies, throttle/guidance fit — as designer red-flags; the actual EDL **flight-phase
  dynamics** (entry heating integration, the Mars EDL gap as a flown event) belong to FA-02's
  entry handoff and later slices. (Scope Boundary, FR-VD-401/Out of Scope)
- Q: How much life support does FA-04 model? → A: The designer computes the **static life-support
  sizing**: consumables mass (open-loop) or ECLSS closure-fraction mass + endurance, radiation-
  shield mass vs a dose target, and crew-accommodation mass — as design outputs and realism guards;
  the **dynamic in-mission** consumption, physiology, psychology and ECLSS failure are FA-08's,
  which consumes this sizing. (FR-VD-501/502)
- Q: Does FA-04 compute cost? → A: Yes — a **physical, mass-and-maturity-driven cost & build-time
  estimate** with learning-curve effects, as a designer output (FR-VEH-002); the **six-currency
  economic pricing, funding and market** of building it are FA-06's, which consumes the estimate.
  (FR-VD-701)
- Q: Where does FA-04's state ownership end vs FA-02's craft state? → A: FA-04 is the **design-time
  authority** (owns the design catalogue + heritage/cost state); **flight-time craft state (live
  mass/propellant) stays in FA-02**, populated from designer outputs (engine parameters conforming
  to `PropulsionEndpoint`, dry mass, propellant capacity, boil-off rate) at craft spawn; FA-02's
  burn-executor `consume` remains the sole live-mass mutation. Designer-built engines replace the
  FA-02 fixtures without changing the propagator. (FR-VD-302/802, Assumptions)
- Q: How is vehicle reliability composed from components? → A: A **reliability-block-diagram** —
  series chains multiply per-component reliabilities (∏ rᵢ); **declared redundancy blocks** evaluate
  as parallel (1 − ∏(1 − rᵢ)); redundancy is a **design-data declaration** carried in the design.
  (FR-VD-501)

No open [NEEDS CLARIFICATION] markers remain.

## User Scenarios & Testing *(mandatory)*

Actors: **the player** (a vehicle designer who composes spacecraft from researched parts and lives
under the rocket equation) and **the integrator** (builds the propagator/economy/politics/UI on the
endpoint, reliability, heritage and design-query contracts).

### User Story 1 - Compose & Derive (Priority: P1)

The player composes a vehicle from **researched component technologies** — a structure, tanks,
engines, power, thermal, avionics — and the designer **live-computes** the emergent performance:
dry and wet mass, Δv (per stage and per propulsion mode via the rocket equation), thrust and T/W at
each gravity field of interest. Unresearched components are absent or visibly locked. **Every
derived number is traceable**: the player can expand any output to its full input tree terminating
in sourced data values. Capability falls out of physics, never from stat points.

**Why this priority**: This is the WHY of the slice — the tyranny of mass and Δv felt at design
time. The whole feature is this loop; everything else refines or guards it.

**Independent Test**: Headless: compose a vehicle from a sourced component set; verify mass/Δv/
thrust/T-W match analytic rocket-equation values within tolerance; verify an unresearched component
is rejected; verify the traceability tree of any output resolves to sourced leaves; verify the
derivation is deterministic and identical on a double run.

**Acceptance Scenarios**:

1. **Given** a researched component set, **When** a vehicle is composed, **Then** its dry/wet mass, per-stage Δv (`Δv = v_e·ln(m0/m1)`), thrust and T/W are computed from the components' sourced parameters and match analytic values within documented tolerance.
2. **Given** an unresearched (or insufficiently mature) component, **When** it is added, **Then** the designer rejects it or marks it locked, reporting the gating technology.
3. **Given** any derived output, **When** its derivation is requested, **Then** a complete input tree is returned, every leaf a sourced data value (FR-VEH-003).
4. **Given** the same design and research state, **Then** all derived numbers are bit-identical across a double run.

---

### User Story 2 - Propulsion Is Physics, Not Stats (Priority: P1)

Each propulsion family — chemical, electric (ion/Hall/MPD/VASIMR/electrospray), nuclear-thermal,
nuclear-electric, gated frontier — is a **physical model**: Isp (exhaust velocity), thrust, T/W,
input power, propellant type/density, throttle range, restart/duty limits, and a **mass model**
(engine + feed + power + radiators). Honest couplings are enforced: **electric propulsion is
power-limited** (thrust ∝ power/Isp; high Isp ⇒ low thrust ⇒ you carry and cool the power source);
**nuclear-electric drags reactor + radiators as first-class mass**. Each engine produces a
**`PropulsionEndpoint`** the FA-02 propagator flies unchanged.

**Why this priority**: Propulsion is the heart of the mass/Δv constraint and the direct FA-02 seam;
without honest propulsion models the designer is a stat sheet.

**Independent Test**: Headless: instantiate each family from sourced data; verify the produced
`PropulsionEndpoint` exposes correct exhaust velocity, thrust, throttle range, power-limited flag
and mass; verify an EP endpoint's deliverable thrust scales with available power and that its
power source + radiator mass appears in the vehicle dry mass; substitute a designer-built engine
for the FA-02 fixture and confirm the propagator flies it (a coast + a burn) unchanged.

**Acceptance Scenarios**:

1. **Given** each propulsion family's sourced parameters, **Then** the designer produces a `PropulsionEndpoint` (exhaust velocity, max thrust, throttle range, available power, power-limited flag, masses) conformant with FA-02's contract.
2. **Given** an electric-propulsion engine, **Then** its delivered thrust is power-limited (∝ available power) and its power source and radiators are carried as dry mass that the rocket equation sees.
3. **Given** a nuclear-electric engine, **Then** reactor, shielding and radiator mass are first-class dry mass and the waste-heat rejection requirement is computed.
4. **Given** a designer-built engine substituted for the FA-02 fixture, **When** the propagator runs a coast and a finite burn, **Then** it flies unchanged and plan-vs-flown propellant agrees to the FA-02 tolerance.

---

### User Story 3 - Reliability Is Earned (Priority: P1)

A vehicle's **reliability is composed** from its components' technology maturity — each component's
per-use reliability comes from FA-05's `maturity()` (TRL + flight-units + relevant domain
understanding) — combined across the vehicle (series/redundancy per the design). Flying immature
(low-TRL) hardware is allowed but the reliability number tells the truth. Successful operational
use **accrues flight heritage**, which raises a technology's reliability toward its ceiling and
discounts derivative designs — fed back to FA-05.

**Why this priority**: Reliability is the honest risk signal that makes "informed gambles" real and
closes the FA-04↔FA-05 loop; P1 because it gates buildability/red-flags and mission risk.

**Independent Test**: Headless with a stubbed/real FA-05 maturity source: compose a vehicle from
components at varying TRL/heritage; verify component reliability tracks `maturity()`, that the
composed vehicle reliability follows the documented composition (lower with more low-TRL parts,
higher with redundancy), that sub-TRL-6 components are flagged, and that registering operational
use raises heritage and the next derivative starts higher.

**Acceptance Scenarios**:

1. **Given** components with maturities from FA-05, **Then** each component's per-use reliability equals its `maturity().reliability`, and the vehicle's composed reliability follows the documented series/redundancy model.
2. **Given** a component below TRL 6, **Then** the design is red-flagged (flyable, but the reliability/risk figure reflects the immaturity).
3. **Given** operational uses of a vehicle, **Then** flight heritage accrues (via FA-05's heritage interface), raising the technology's reliability ceiling and discounting a declared derivative design.

---

### User Story 4 - The Designer Refuses the Impossible (Priority: P2)

The designer **red-flags or refuses** physically impossible configurations: an unresolvable
negative power margin, radiators too small for the heat load, a lander whose T/W is below local
gravity, Δv short of a stated mission requirement, structure that can't take the engines' thrust,
crewed endurance below mission duration, radiation dose above the crew limit. **Marginal-but-
possible** designs remain buildable — you can fly a risky design, but the numbers don't lie (no
physics cheats, only informed gambles).

**Why this priority**: The guards are what make the physics *bind*; P2 because the core derivation
(US1/US2) must exist first for the guards to check.

**Independent Test**: Headless: construct designs that violate each guard (negative power margin,
radiator shortfall, T/W < local g, Δv short, over-thrust structure) and verify each is red-flagged
with the specific violated constraint; verify a marginal design is still buildable with truthful
risk; verify no guard can be bypassed by a magic number (all from data).

**Acceptance Scenarios**:

1. **Given** a design with a negative power margin in every mode, **Then** it is hard-red-flagged with the power deficit reported.
2. **Given** a lander with T/W below the target body's surface gravity, **Then** it is red-flagged as unable to land there.
3. **Given** a design whose Δv is below a stated mission requirement, **Then** the shortfall is flagged; **and** a marginal (small-positive-margin) design remains buildable with its reliability/risk shown.
4. **Given** radiators insufficient for the computed heat load, **Then** the thermal shortfall is flagged.

---

### User Story 5 - Power & Thermal Balance (Priority: P2)

For every design the designer computes the **power balance** (generation from PV/RTG/fission vs
demand from propulsion/avionics/payload/life-support, with a margin that must be ≥0 in some mode)
and the **thermal balance** (waste-heat load vs radiator rejection capacity, radiators carried as
real mass). Waste-heat rejection is a **first-class, frequently-binding constraint** — you cannot
ignore it on any nuclear/high-power craft.

**Why this priority**: Power/thermal is the constraint that makes EP/NEP honest (US2's couplings
made visible at the vehicle level); P2 alongside the other vehicle-level balances.

**Independent Test**: Headless: compose a high-power EP/NEP vehicle; verify power generation,
demand and margin are computed from sourced component data; verify the radiator mass required to
reject the computed waste heat is included in dry mass and that an undersized radiator is flagged;
verify the balance at a distant body (e.g. PV margin collapses far from the Sun).

**Acceptance Scenarios**:

1. **Given** a design, **Then** power generation, demand and margin are computed per mode from sourced component data, and a design that cannot close its power budget in any mode is flagged.
2. **Given** a nuclear/high-power design, **Then** the radiator mass needed to reject the computed waste heat is carried as dry mass and an undersized radiator is a thermal red-flag.
3. **Given** a solar-powered design evaluated at a distant body, **Then** its power generation falls with solar distance and the margin reflects it.

---

### User Story 6 - Every Vehicle From One System (Priority: P2)

All vehicle archetypes — launch vehicles, crew capsules, cargo craft, chemical/EP/nuclear tugs,
landers, ascent vehicles, transit habitats/cyclers/spin-habs, rovers, surface mobility, station
and base modules, ISRU plants, relay sats, science probes, body-specific explorers — are
**designer-built from the same component system** via class templates. Designs are **savable as
versioned classes**, support **derivatives** (inheriting heritage discounts) and **side-by-side
comparison**.

**Why this priority**: Breadth proves the one-system claim and feeds FA-06/FA-07; P2 because the
core compose/derive loop must work for one class before generalising.

**Independent Test**: Headless: build one design of each archetype from the shared system; verify
each validates and derives its class-appropriate outputs; save a design, create a derivative, and
verify the derivative inherits heritage discounts and that two designs can be compared field-by-
field.

**Acceptance Scenarios**:

1. **Given** the component system and class templates, **Then** every archetype is buildable from the same parts with class-appropriate derived outputs and guards.
2. **Given** a saved design, **Then** it is a versioned class; a derivative inherits its lineage's heritage discounts; two designs expose a field-by-field comparison.

---

### User Story 7 - Cost & Build-Time Estimate (Priority: P3)

The designer computes a **physical cost and build-time estimate** for a design — driven by mass,
component maturity and a **learning curve** (unit cost declining with cumulative production count)
— as a design output. This is the physical estimate; the six-currency economic pricing, funding
and market that turn it into an affordable (or not) program are FA-06's, which consumes it.

**Why this priority**: Cost closes the design picture and is the FA-06 seam; P3 because the economy
that prices it arrives later.

**Independent Test**: Headless: compute cost/build-time for a design; verify both derive from
sourced mass/maturity/learning-curve parameters and are traceable; verify the learning curve
lowers unit cost as cumulative count rises; verify determinism.

**Acceptance Scenarios**:

1. **Given** a design, **Then** a physical unit-cost and build-time estimate is computed from sourced mass/maturity parameters, fully traceable (FR-VEH-003).
2. **Given** rising cumulative production of a design, **Then** unit cost declines along the documented learning curve.

---

### Edge Cases

- **Zero-propellant / zero-Δv design**: a design with no propellant reports Δv 0 (not an error); the rocket equation is well-defined at the limit.
- **Power-limited thrust at zero power**: an EP engine with no available power delivers zero thrust (not negative, not infinite); the endpoint exposes this honestly.
- **Radiator/​power coupling loop**: more power ⇒ more waste heat ⇒ more radiator mass ⇒ lower Δv — the designer resolves the coupling to a consistent fixed point (or flags non-convergence) rather than oscillating.
- **Reliability of an unflown, low-TRL component**: composed reliability is defined (and low) with zero heritage; never undefined.
- **Mission-requirement absent**: Δv/endurance guards compare against a *stated* requirement; with none stated, the design is not flagged for shortfall (the requirement is an input, not invented).
- **Derivative of a retired/edited parent**: a derivative references its parent design version; editing the parent does not silently mutate existing derivatives (versioned lineage).
- **Component data vs saved design**: a saved design references component-technology + parameter versions; loading against a different component-data version fails actionably (FA-01 pinning + FA-02/FA-03 hash pattern extended to component data).
- **Staging with mixed propulsion modes**: per-stage and per-mode Δv are reported separately (a chemical first stage + an EP upper stage do not blend into one fictitious Isp).
- **Negative structural margin under thrust**: a structure rated below the engines' thrust load is a hard red-flag, not a silent acceptance.

## Requirements *(mandatory)*

IDs are FR-VD-###. Umbrella traceability (FR-VEH-###) inline. All component and physics constants
live in schema-validated data files with `source` provenance (Principles I, V); the **physics
engine contains no per-design/per-tech magic numbers** — it reads them (Principle II).

### Component catalogue & composition (FR-VEH-001)

- **FR-VD-101**: A **component catalogue** MUST define, as sourced data, every composable part class (structures/tanks, propulsion, power, thermal/radiators, avionics/GNC, comms, life-support & accommodation, payloads, EDL kit, landing gear, docking, RCS) with its physical parameter set; each component MUST reference the technology that researches it. *(FR-VEH-001)*
- **FR-VD-102**: A vehicle MUST be composed **only** from components whose researching technology is available to the faction (per FA-05); unresearched/insufficiently-mature components MUST be absent or visibly locked with the gating technology reported. *(FR-VEH-001)*

### Derived performance & traceability (FR-VEH-002/003)

- **FR-VD-201**: The designer MUST live-compute, from sourced component data: **dry & wet mass**, **per-stage and per-mode Δv** (rocket equation from exhaust velocity + mass fractions), **thrust and T/W at each gravity field of interest**, **payload capacity**. *(FR-VEH-002)*
- **FR-VD-202**: Every computed output MUST be **fully traceable**: a complete derivation tree from the output to sourced data leaves is retrievable for any number. *(FR-VEH-003)*
- **FR-VD-203**: All derivations MUST be **deterministic** pure functions of the design + component data + research state (no wall-clock, no hidden randomness); identical inputs yield bit-identical outputs.

### Propulsion physics & the FA-02 endpoint (FR-VEH-005/006)

- **FR-VD-301**: Each **propulsion family** (chemical; electric ion/Hall/MPD/VASIMR/electrospray; nuclear-thermal; nuclear-electric; gated frontier) MUST be a **physical model** exposing Isp/exhaust velocity, thrust, T/W, input power, propellant type/density, throttle range, restart/duty limits, a **mass model** (engine + feed + power + radiators), and a reliability curve — all from sourced data. *(FR-VEH-005)*
- **FR-VD-302**: Each engine MUST produce a **`PropulsionEndpoint`** conformant with FA-02's propulsion-interface contract (exhaust velocity, max thrust, throttle range, available power, power-limited flag, drag/SRP area, masses, `consume`), so the propagator flies designer-built vehicles unchanged. *(FR-VEH-005)*
- **FR-VD-303**: Honest couplings MUST be enforced: **electric propulsion is power-limited** (thrust ∝ power/Isp; the power source and its radiators are carried mass); **nuclear systems carry reactor + shielding + radiator mass** and a waste-heat rejection requirement; free thrust is structurally impossible. *(FR-VEH-006)*

### Power & thermal balance (FR-VEH-002/006)

- **FR-VD-401**: The designer MUST compute the **power balance** (generation vs demand, per mode, margin ≥ 0 in some mode) and the **thermal/radiator balance** (waste-heat load vs rejection, radiators as dry mass); waste-heat rejection MUST be a first-class constraint, and PV generation MUST scale with solar distance. Landing/EDL **suitability checks** (T/W vs local gravity, heat-shield adequacy, ballistic coefficient) MUST be computed as designer red-flags (the EDL flight phase is FA-02/later, clarified 2026-06-13). *(FR-VEH-002/004/006)*

### Reliability & heritage (FR-VEH-002/009)

- **FR-VD-501**: **Composed reliability** MUST derive from component maturity: each component's per-use reliability comes from FA-05's `maturity()` (TRL + flight-units + domain UL), combined across the vehicle by a documented **reliability-block-diagram** model — series chains multiply per-component reliabilities (∏ rᵢ); declared **redundancy blocks** evaluate as parallel (`1 − ∏(1 − rᵢ)`); redundancy is a design-data declaration carried in the design (clarified 2026-06-13). **Static life-support sizing** (consumables/closure-fraction mass, endurance, radiation-shield mass vs a dose target, crew-accommodation mass) MUST be computed as design outputs and guards (the dynamic in-mission simulation is FA-08, clarified 2026-06-13). *(FR-VEH-002)*
- **FR-VD-502**: Produced units MUST **accrue flight heritage** through FA-05's heritage interface, raising a technology's reliability toward its ceiling and discounting derivative designs. *(FR-VEH-009)*

### Realism guards (FR-VEH-004)

- **FR-VD-601**: The designer MUST enforce **realism guards** — hard-red-flag the physically impossible (unresolvable negative power margin, radiator shortfall, lander T/W below local gravity, structural limit exceeded, crewed endurance below mission duration, radiation dose above crew limit, Δv short of a stated requirement) — while leaving **marginal-but-possible** designs buildable with truthful reliability/risk figures. No guard may be bypassed by a magic number. *(FR-VEH-004)*

### Vehicle classes & designs (FR-VEH-007/008)

- **FR-VD-701**: All vehicle **archetypes** MUST be designer-built from the **same** component system via class templates (launchers, capsules, cargo, tugs, landers, ascent vehicles, transit habs/cyclers/spin-habs, rovers, surface mobility, station/base modules, ISRU plants, relay sats, probes, body-specific explorers). A **physical cost & build-time estimate** (mass + maturity + learning curve) MUST be a design output (economic pricing is FA-06, clarified 2026-06-13). *(FR-VEH-008/002)*
- **FR-VD-702**: Designs MUST be **savable as versioned classes** supporting iteration, **derivatives** (inheriting heritage discounts via lineage), and **side-by-side comparison**; editing a parent MUST NOT silently mutate existing derivatives. *(FR-VEH-007)*

### Queries, integration, determinism & validation

- **FR-VD-801**: A read-only **design-query surface** (the FA-10/inter-slice seam, kernel `with_slice` + pure functions per the FA-02/FA-03/FA-05 pattern) MUST answer: a design's derived outputs + their traceability trees, its red-flags, its `PropulsionEndpoint`(s), its composed reliability and heritage, and its cost/build estimate — faction-scoped where research state is involved.
- **FR-VD-802**: The module MUST conform to the kernel module contract (owned **design-time** slice — design catalogue, heritage, cost; journaled commands for compose/edit/save/derive/register-use; events for heritage-relevant operational use). **Flight-time craft state (live mass/propellant) stays in FA-02**; designer outputs populate FA-02's craft + engine state at spawn and `consume` remains FA-02's craft mutation (clarified 2026-06-13, Q1:A). The module integrates with FA-02 (endpoints) and FA-05 (maturity/heritage) with no kernel code change anticipated.
- **FR-VD-803**: All component/physics data MUST pass schema + source-presence validation in CI (`validate-data` extended); the designer's analytic outputs MUST be validated against **known cases** (rocket-equation Δv, T/W, power-limited-EP thrust, mass-fraction identities) as CI gates (constitution testing mandate); component-data version pinned in saves; the whole module passes conformance + determinism gates.

### Key Entities

- **Component**: a researched part (class, physical parameter set, researching technology, source); the designer's building block.
- **Propulsion Model**: a family-specific physical model → exhaust velocity, thrust, power, throttle, mass model, reliability curve.
- **Propulsion Endpoint**: the FA-02-contract object a built engine exposes (consumed by the propagator).
- **Vehicle Design**: a composition of components (staged, multi-mode, with optional **redundancy-block** declarations) + a class template; the unit that derives performance.
- **Derived Outputs**: mass, Δv, thrust/T-W, power/thermal balance, reliability, cost/build — each with a traceability tree.
- **Realism Flag**: a red-flag/refusal carrying the specific violated constraint.
- **Heritage Record**: operational-use accrual per technology (shared with FA-05).
- **Design-Query Surface**: pure read-only functions over a design snapshot (outputs, traces, flags, endpoints, reliability, cost).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (Capability falls out of physics)**: For a sourced design, dry/wet mass, per-stage/mode Δv, thrust and T/W match analytic rocket-equation/T-W values within documented tolerance; 100% of derived outputs resolve to sourced data leaves via their traceability trees; derivations are bit-identical on a double run.
- **SC-002 (Propulsion is honest)**: Every propulsion family produces an FA-02-conformant `PropulsionEndpoint`; EP thrust is power-limited and its power+radiator mass is in dry mass; a designer-built engine substituted for the FA-02 fixture flies unchanged with plan-vs-flown propellant agreement to the FA-02 tolerance.
- **SC-003 (Reliability is earned)**: Component reliability equals FA-05 `maturity().reliability`; composed vehicle reliability follows the documented model (lower with more low-TRL parts, higher with redundancy); operational use accrues heritage that raises reliability and discounts a derivative.
- **SC-004 (The impossible is refused)**: 100% of the defined impossible configurations are red-flagged with the specific violated constraint; marginal designs remain buildable; no guard is bypassable by a non-sourced value.
- **SC-005 (Power & thermal bind)**: Power and thermal balances are computed per mode from sourced data; undersized radiators and unclosable power budgets are flagged; PV generation falls with solar distance.
- **SC-006 (One system, all vehicles)**: Every archetype is buildable from the shared component system with class-appropriate outputs; designs save as versioned classes; derivatives inherit heritage; comparison works.
- **SC-007 (Cost is physical)**: Unit cost and build time derive from sourced mass/maturity/learning-curve data, are traceable, and the learning curve lowers unit cost with cumulative count.
- **SC-008 (Integration & determinism)**: The design-query surface is pure and faction-scoped; analytic-case CI gates pass; the module passes conformance and the kernel double-run/round-trip/replay gates; saves pin and verify component-data versions.

## Assumptions

- **Research state is an input**: component availability, maturity, reliability and heritage come from FA-05's contract; in tests/scenarios a stubbed maturity source stands in (the honest-seam pattern of prior slices). Faction identity is opaque (FA-09 binds it later).
- **FA-04 owns design-time state; FA-02 owns flight-time state** (clarified 2026-06-13, Q1:A): FA-04 owns the design catalogue, heritage and cost estimates; when a vehicle is spawned to fly, the designer's outputs (engine parameters conforming to `PropulsionEndpoint`, dry mass, propellant capacity, boil-off rate) populate FA-02's **existing** craft + engine state, and FA-02's burn-executor `consume` is the sole live-mass mutation (single-writer preserved). The FA-02 fixture engines are replaced by designer-built engines without changing the propagator; boil-off rates are FA-04 data that FA-02 applies during propagation. The exact registration/spawn mechanism is settled in this slice's **plan**, with `PropulsionEndpoint` as the fixed binding shape.
- **Cost is physical, not monetary**: the cost estimate is a mass/maturity/learning-curve quantity; FA-06 maps it to the six-currency economy later without data migration.
- **Reliability composition model**: a documented, sourced series-with-declared-redundancy model; tuning is data.
- **Boil-off & time-dependent propellant losses** are FA-04 state changes the propagator reads via `propellant()` (per the FA-02 contract); their rates are sourced data.
- **No magic numbers**: every propulsion/structure/power/thermal/cost constant lives in sourced `data/` files; the engine reads them (Principle II); analytic validation cases gate CI.
- **Reference hardware & performance envelope** inherit the FA-01/FA-02 definitions; the designer's derivations are sub-millisecond pure computations.

## Out of Scope (this slice)

- Astrodynamics, propagation, manoeuvre/porkchop/flyby/low-thrust/aerocapture planning (FA-02) — this slice produces the endpoints FA-02 flies.
- The research/personnel model (FA-05) — this slice consumes maturity/heritage/understanding.
- The **EDL flight phase** dynamics (entry-heating integration, the flown Mars EDL gap) — this slice computes design-time *suitability checks* only.
- **In-mission life support**: consumption, physiology, psychology, ECLSS failure, spares (FA-08) — this slice computes the *static sizing* only.
- **Economic pricing, funding, markets, learning-curve economics in currency** (FA-06) — this slice computes a *physical* cost/build estimate.
- Political/approval costs of nuclear-launch (FA-09) — this slice flags the requirement; politics prices it.
- All UI (FA-10), including the designer screen, traceability inspector and comparison view.
