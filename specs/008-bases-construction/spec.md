# Feature Specification: Bases & Construction (FA-07)

**Feature Branch**: `008-bases-construction`
**Created**: 2026-06-14
**Status**: Draft
**Input**: User description: "Build Sojourn's bases and construction: orbital stations and surface bases assembled from modules with emergent properties. Authoritative design: design/03-ECONOMY.md (construction, facilities), design/05-WORLD.md (sites, planetary protection) and design/02-TECH-TREE.md (regolith construction, ISRU plants); also .specify/memory/constitution.md (Principles I, VII, VIII). Build on Slice 1 core, Slice 3 sites, Slice 5 technology, and Slice 6 economy/logistics."

## Overview

Settlement is the game's destination, and a base is **the sum of physical truths** — power margin,
life-support closure, shielding, population capacity, sustainability — not an abstract level bar.
This slice builds orbital stations and surface bases by **composing modules** (habitat, power, ECLSS,
ISRU host, science, storage, manufacturing, shielding) sited at the world's Sites and dynamical
locations, and **derives the base's properties from physics** — never from hand-set values. Building
far from Earth is a genuine logistics and self-sufficiency problem: a base is assembled by a
construction project whose modules become operational only as delivered mass and crew-time land, and
on-site ISRU/fabrication progressively replaces imports until a settlement can stand on its own.

The slice consumes the world model's Sites (resource, planetary-protection category, illumination,
slope, hazards — on the surveyed belief-state), the research layer's technology maturity (module
gating), and the economy/logistics layer's delivery accounting, location-addressed stocks, crew-time
and ops capacity, through their existing interfaces; it owns none of their logic.

**Scope boundary**: this slice computes a base's **static** emergent properties and construction state.
The **dynamic in-mission simulation** — consumables consumed over time, radiation dose accumulation,
crew physiology/psychology, ECLSS hardware failure — is **Slice 8 (Life Support & Crew)**, which
consumes this base state. Resource-extraction ISRU (ice→propellant, regolith→O₂/metals) is owned by
**Slice 6**; this slice owns the **construction use** of local materials (regolith-built shielding and
structures, base manufacturing) that reduces imported mass. Per-faction AI base-building is **Slice 9**.

## Clarifications

### Session 2026-06-14

- Q: Does this slice simulate dynamic in-mission life support/crew over time, or compute static base properties and sizing, leaving the dynamic simulation to Slice 8? → A: Static base properties + sizing here (power margin, ECLSS closure fraction, population capacity, shielding/dose attenuation, sustainability index, hazard exposure); the dynamic consumption/dose/physiology/failure simulation is Slice 8, which consumes this base state.
- Q: Who owns on-site production — does FA-07 model resource-extraction ISRU itself, or consume FA-06's ISRU output and own only the construction/manufacturing use of local materials? → A: FA-06 owns resource-extraction ISRU (ice→propellant, regolith→O₂/metals); FA-07 owns the **construction use** of local materials (regolith-built shielding/structures, base manufacturing) that reduces imported mass. A base *hosts* FA-06 ISRU plants whose output feeds construction and sustainment.
- Q: How does FA-07 couple to the other slices — hard crate dependencies, or composed values? → A: FA-07 depends only on the kernel core; Site properties (PP category, illumination, hazards, resource grade), module-tech maturity, and construction-delivery status flow in as **composed values** the host assembles (the FA-04/FA-06 decoupling). The host bridges FA-06's delivery accounting → base construction progress.
- Q: How should the self-sufficiency / sustainability index aggregate its per-loop closure ratios? → A: **Limiting factor (minimum)** — index = min over loops (ECLSS air/water, food, materials, power, spares) of (local supply ÷ demand), capped at 1. A base is only as self-sufficient as its weakest closed loop; improving the binding loop raises the index (monotonic).
- Q: How should the resupply-embargo stress test be computed, given dynamics are Slice 8's? → A: **Analytic rate + buffer check** — per loop, survive iff local production rate ≥ demand rate, OR stored buffer ≥ (demand − production) × N years; the base survives iff every loop survives. Static (no time-stepping), deterministic, reuses the limiting-factor index plus storage-module buffers.
- Q: How should shielding convert shield mass into a dose-attenuation factor? → A: **Mass-attenuation (exponential)** — attenuation factor = exp(−Σᵢ ρxᵢ ÷ λᵢ), summing per material in the exponent (so mixed materials compose correctly), with each module's areal density ρxᵢ (kg/m²) and a sourced mass-attenuation length λᵢ per material (regolith, water, polyethylene); transmitted dose = site dose × factor. Standard radiation-shielding physics, data-driven per material.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Compose a base from modules with emergent properties (Priority: P1)

A player composes a base or orbital station as a set of modules sited at a Site (surface) or a
dynamical location (orbit). The base's properties **emerge from the modules and the site** — power
margin, ECLSS closure fraction, population capacity, radiation shielding, sustainability index, hazard
exposure — every one derived from physics and fully traceable, never a hand-set level.

**Why this priority**: this is the heart — a base as the sum of physical truths. Without emergent,
derived properties there is no honest settlement. It is independently demonstrable.

**Independent Test**: compose a base from sourced modules at a sourced site; assert each emergent
property matches the analytic composition of its modules (add a power module → margin rises by its
sourced generation); assert every property traces to sourced module/site inputs; double-run identical.

**Acceptance Scenarios**:

1. **Given** a base with power, habitat and ECLSS modules, **When** its properties are derived, **Then**
   power margin = Σ generation − Σ demand, population capacity follows habitat accommodation gated by
   closure/power, and ECLSS closure fraction follows the best/composed ECLSS modules.
2. **Given** a base, **When** a sourced power module is added, **Then** the power margin rises by that
   module's generation; removing it lowers it — the property is emergent, not stored.
3. **Given** any emergent property, **When** the player drills into it, **Then** it resolves to sourced
   module/site leaves (the honesty contract).
4. **Given** a base with demand exceeding generation, **When** derived, **Then** the negative power
   margin is red-flagged.

---

### User Story 2 - Construction projects routed through logistics (Priority: P1)

A base is built by a **construction project** with a schedule and per-module delivered-mass and
crew-time demands. Modules become operational only when their materials and crew-time have landed at
the base's location (consuming the logistics delivery accounting). A half-built base has half a base's
(emergent) properties.

**Why this priority**: building far from Earth must be a logistics problem, not instant placement —
the felt tyranny of mass and the path to settlement. It makes US1's properties earned over time.

**Independent Test**: open a construction project requiring delivered mass + crew-time; deliver part →
assert only commissioned modules contribute to emergent properties; complete the deliveries + crew-time
→ assert the base is fully operational; assert no module is operational before its inputs land.

**Acceptance Scenarios**:

1. **Given** a construction project for a 3-module base, **When** one module's mass + crew-time land,
   **Then** that module commissions and contributes to the base's properties; the others do not yet.
2. **Given** a project mid-build, **When** the base's properties are queried, **Then** they reflect only
   the operational modules (partial base ⇒ partial properties).
3. **Given** insufficient crew-time/construction capacity, **When** commissioning is attempted, **Then**
   it is slowed or blocked, not silently completed.
4. **Given** all deliveries + crew-time complete, **When** the project finishes, **Then** the base
   reaches its full composed properties.

---

### User Story 3 - Siting respects planetary protection & site suitability (Priority: P2)

Siting and building respect the Site's planetary-protection category and physical suitability. Building
in a Special Region (Mars liquid-water, ocean worlds) without the required sterilization/containment is
red-flagged with the violated rule; a physically unsuitable siting (a solar-only base in permanent
shadow, no shielding in a high-radiation site, an unbuildable slope) is flagged. Cutting corners has
modelled consequences — the system never silently permits a violation.

**Why this priority**: planetary protection is a real, modelled constraint (Principle I); honest siting
is core to the game's credibility, but it layers on the compose/construct core.

**Independent Test**: site a base in a PP Special Region without containment → assert a hard red-flag
citing the rule; site a solar-only base in a permanently-shadowed region → assert a negative-power /
illumination flag; assert the checks read the world model's surveyed site state, not hidden truth.

**Acceptance Scenarios**:

1. **Given** a Special-Region Site, **When** a non-sterile base is sited there, **Then** the system
   red-flags the forward-contamination / PP-category violation with the specific rule.
2. **Given** a permanently-shadowed Site, **When** a solar-only base is sited, **Then** the negative
   power margin / illumination unsuitability is flagged.
3. **Given** a high-radiation Site, **When** a base lacks shielding for the dose, **Then** the shielding
   shortfall is flagged.
4. **Given** PP rules are violated, **When** the consequence is evaluated, **Then** a science/PP-value
   loss is representable (not silently ignored).

---

### User Story 4 - On-site ISRU & fabrication reduce imports (Priority: P2)

A base hosts on-site production — ISRU output (from the economy layer) plus manufacturing and
**regolith construction** — that yields local materials and shielding, reducing the imported mass
required to build and sustain it over time. Regolith-built shielding is protection you did not launch.

**Why this priority**: relaxing the mass/Δv constraint with local production is the mechanism of
settlement (Principle VII), but it builds on a working base + construction (US1/US2).

**Independent Test**: build a base needing a shielding target; supply on-site regolith construction →
assert the imported mass required to reach the target falls (local material substitutes for launched
mass); assert local production rates derive from sourced process params + the base's power/feedstock.

**Acceptance Scenarios**:

1. **Given** a base with a regolith-construction capability, **When** it builds shielding from local
   regolith, **Then** the shielding mass is added without a corresponding imported (launched) mass.
2. **Given** a base hosting ISRU/manufacturing, **When** it produces local materials, **Then** the
   future delivered-mass demand of its construction/sustainment falls measurably.
3. **Given** on-site production, **When** rates are derived, **Then** they follow sourced process params
   and the base's available power/feedstock, not a hand-set value.

---

### User Story 5 - Sustainability, self-sufficiency & the embargo stress test (Priority: P2)

A base computes a **self-sufficiency / sustainability index** from its closure fractions (ECLSS
air/water, food, materials, power, spares) and its local-production-vs-import ratio, and can be
evaluated against a **resupply-embargo stress test**: it survives N years without Earth resupply iff
its closure and local production meet its crew/operations demand over that span — the *Homestead* goal.

**Why this priority**: the settlement destination and the Homestead win condition; it composes the
closure/production from US1–US4 into a single honest measure.

**Independent Test**: compute a base's self-sufficiency index from its closure fractions; run a
5-year embargo stress test on a base above threshold (survives) and one below (fails); assert both are
deterministic and the index rises continuously with improved closure/production.

**Acceptance Scenarios**:

1. **Given** a base's closure fractions and production/import ratio, **When** the index is computed,
   **Then** it is a continuous derived measure, not a binary unlock.
2. **Given** a base above the closure/production threshold, **When** a 5-year embargo is simulated,
   **Then** it survives (demand met from closure + local production).
3. **Given** a base below threshold, **When** the embargo is simulated, **Then** it fails (a deficit
   accumulates), deterministically.
4. **Given** improving closure/production, **When** re-evaluated, **Then** the self-sufficiency index
   rises monotonically.

---

### User Story 6 - Base state exposed to economy, life support & politics (Priority: P3)

The base system exposes, through a read-only query surface, each base's emergent properties,
production/consumption (for the economy's location-addressed stocks), habitat/closure/shielding/
population state (for life support, Slice 8), and settlement milestones (for politics, Slice 9). Bases
are versioned, durable records; two bases can be compared.

**Why this priority**: the integration seam other slices consume, but the base mechanics stand alone
first.

**Independent Test**: query a base's production/consumption, habitat state and milestones; assert a
base consuming inputs and producing outputs integrates with the economy's stocks at its location;
assert `compare()` diffs two bases.

**Acceptance Scenarios**:

1. **Given** an operating base, **When** the economy queries it, **Then** it reports production and
   consumption at its location (feeding location-addressed stocks).
2. **Given** a base, **When** life support (Slice 8) queries it, **Then** it reports habitat capacity,
   closure fraction, shielding and population state.
3. **Given** a settlement milestone (e.g., first base to survive an embargo), **When** reached, **Then**
   it is exposed for politics/scoring.
4. **Given** two bases, **When** compared, **Then** the system diffs their emergent properties.

---

### Edge Cases

- **Negative power / unclosed loop**: a base whose demand exceeds generation, or whose ECLSS cannot
  support its population, is flagged (not silently "leveled up").
- **PP Special-Region violation**: siting/operating without required containment is red-flagged; the
  forward-contamination consequence is representable.
- **Unsuitable site**: permanent shadow for a solar base, unbuildable slope, no comms visibility, high
  radiation without shielding — each flagged.
- **Partial build**: a base mid-construction exposes only operational modules' contributions.
- **Embargo deficit**: an under-closed base accumulates a survival deficit during the stress test.
- **Local-production starvation**: ISRU/manufacturing with no power or feedstock produces nothing;
  import needs do not fall.
- **Data version mismatch on load**: a save referencing changed module/construction data is detected.
- **Module-tech not matured**: composing a module whose technology a faction has not matured is gated
  (consuming the research maturity input).

## Requirements *(mandatory)*

### Functional Requirements

#### Base composition & emergent properties (US1)

- **FR-BC-101**: System MUST let a player compose a base or orbital station as a set of **modules**
  (habitat, power, ECLSS, ISRU host, science, storage, manufacturing, shielding) from a sourced module
  catalogue, sited at a world **Site** (surface) or **dynamical location** (orbit).
- **FR-BC-102**: A base's properties MUST be **emergent** — computed from its modules and site, never
  hand-set: power margin, ECLSS closure fraction, population capacity, radiation shielding (dose
  attenuation), sustainability/self-sufficiency index, and hazard exposure.
- **FR-BC-103**: **Power margin** MUST be Σ(module generation, solar-distance-scaled for PV) −
  Σ(module demand); a negative power margin MUST be flagged.
- **FR-BC-104**: **Population capacity** MUST derive from habitat accommodation **gated by** ECLSS
  closure/consumables and power, not a fixed number.
- **FR-BC-105**: **Radiation shielding** MUST derive from passive shield areal density via a
  **mass-attenuation (exponential) model**, summing per material in the exponent so mixed materials
  compose correctly: attenuation factor = exp(−Σᵢ ρxᵢ ÷ λᵢ) over each shielding module's areal density
  ρxᵢ (kg/m²) and its **sourced mass-attenuation length λᵢ per material** (regolith, water,
  polyethylene); transmitted dose = site radiation environment × factor. A shortfall against the site's
  dose MUST be flagged. (Single-material special case: doubling ρx squares the attenuation factor.)
- **FR-BC-106**: Every emergent property MUST be **traceable** to its sourced module/site inputs
  (Principle VIII).
- **FR-BC-107**: Module/site parameters MUST be data-driven with sources; the engine MUST contain **no
  per-base/per-module magic numbers** (Principle II).

#### Construction projects & logistics (US2)

- **FR-BC-201**: Building a base MUST be a **construction project** with a schedule and per-module
  **delivered-mass + crew-time** demands; modules become operational only when their inputs have landed.
- **FR-BC-202**: Construction MUST consume **delivery accounting** from the logistics layer (delivered
  mass at the base's location) — no instant placement.
- **FR-BC-203**: A partially-built base MUST expose **partial** emergent properties (only operational
  modules contribute).
- **FR-BC-204**: Commissioning MUST require crew-time and/or construction robotics; insufficient
  crew-time/construction capacity MUST slow or block it.
- **FR-BC-205**: The construction schedule and progress (delivered vs remaining, estimated completion)
  MUST be queryable.

#### Siting & planetary protection (US3)

- **FR-BC-301**: Siting MUST respect the Site's **planetary-protection category**; siting/operating in
  a Special Region without required sterilization/containment MUST be red-flagged with the violated rule.
- **FR-BC-302**: Siting MUST account for **physical suitability** — illumination (solar viability),
  thermal, slope/roughness, comms visibility, hazard level — flagging unsuitable sitings.
- **FR-BC-303**: Forward-contamination **consequences** (science/PP-value loss) MUST be representable
  when PP rules are violated; the system MUST NOT silently permit a violation.
- **FR-BC-304**: PP categories and site properties MUST come from the **world model** (surveyed
  belief-state), consumed as inputs; the base system MUST act on the known site state, never hidden truth.

#### On-site ISRU & fabrication (US4)

- **FR-BC-401**: A base MUST be able to **host on-site production** — ISRU output (from the economy
  layer) plus manufacturing and **regolith construction** — yielding local materials/shielding that
  reduces the imported mass required to build and sustain it over time.
- **FR-BC-402**: **Regolith-built shielding/structures** MUST add shielding/structure mass that is
  **not launched** (local material), relaxing the mass/Δv constraint (Principle VII).
- **FR-BC-403**: On-site production rates MUST derive from sourced process params and the base's
  available power/feedstock, and feed back into construction (reducing future delivered-mass demand).
- **FR-BC-404**: The reduction in import needs as the base matures MUST be **measurable** (more local
  production ⇒ less import).

#### Sustainability & self-sufficiency (US5)

- **FR-BC-501**: A base MUST compute a **self-sufficiency / sustainability index** as the
  **limiting-factor (minimum)** over its per-loop closure ratios — ECLSS air/water, food, materials,
  power, spares — each ratio = (local supply ÷ demand) capped at 1. The weakest closed loop bounds the
  index, so improving the binding loop raises it (monotonic, SC-006).
- **FR-BC-502**: The system MUST support an **embargo stress test** (the Homestead condition) as an
  **analytic rate-plus-buffer check**: for each loop, the base survives iff local production rate ≥
  demand rate **or** its stored buffer ≥ (demand − production) × the embargo span; the base survives iff
  every loop survives. No time-stepping (the dynamic simulation is Slice 8) — a deterministic derivation
  over the closure ratios and storage-module buffers.
- **FR-BC-503**: Progress toward self-sufficiency MUST be a **continuous, derived** measure (not a
  binary unlock), rising as closure and local production improve.

#### Base-state exposure & queries (US6)

- **FR-BC-601**: The base system MUST expose, through a **read-only query surface**, each base's
  emergent properties, production/consumption (for the economy), habitat/closure/shielding/population
  state (for life support, Slice 8), and settlement milestones (for politics, Slice 9).
- **FR-BC-602**: Bases MUST be **versioned, durable records**; the system MUST support **comparing** two
  bases.
- **FR-BC-603**: Base production/consumption MUST integrate with the economy's **location-addressed
  stocks** (a base consumes inputs and produces outputs at its location).

#### Cross-cutting

- **FR-BC-701**: All module/construction parameters MUST live in versioned, schema-validated data files
  with non-empty `source`; CI MUST reject missing sources.
- **FR-BC-702**: The base system MUST be **deterministic** (identical seed + decisions ⇒ identical
  state); any stochastic outcome seeded; no wall-clock.
- **FR-BC-703**: Base state MUST **round-trip** through save/load with identical reload and pin its data
  version.
- **FR-BC-704**: The base system MUST run **headless** and integrate via the established module
  boundary, consuming Site/tech/delivery inputs **as composed values** without embedding those systems'
  logic or reaching hidden ground truth.
- **FR-BC-705**: The slice MUST contain **no combat/weapons** (Principle IX); a base is infrastructure,
  never a fortress.
- **FR-BC-706**: Emergent base properties MUST derive from **physics/sourced data, never hand-set
  levels** (Principles II/VII) — the core promise.

### Key Entities

- **Base / Station**: a sited composition of modules with emergent, derived properties.
- **Module**: a habitat / power / ECLSS / ISRU-host / science / storage / manufacturing / shielding
  unit with sourced physical parameters (mass, generation/demand, closure, accommodation, areal density).
- **ModuleCatalogue**: the sourced, schema-validated set of module types.
- **ConstructionProject**: a schedule + per-module delivered-mass and crew-time demands; modules
  commission as inputs land (consuming the logistics delivery accounting).
- **EmergentProperties**: power margin, ECLSS closure fraction, population capacity, shielding/dose
  attenuation, sustainability index, hazard exposure — all derived, never stored as authoritative.
- **SiteRef**: the world Site's PP category, illumination, thermal, slope, comms visibility, hazard,
  resource grade — a composed input from the world belief-state.
- **OnSiteProduction**: ISRU output + manufacturing + regolith construction that yields local materials.
- **SelfSufficiencyIndex / EmbargoStressTest**: the settlement measure + the Homestead survival test.
- **BaseSnapshot**: the read-only query surface (properties, production/consumption, habitat state,
  milestones, comparison).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Adding a sourced power module raises a base's power margin by that module's generation;
  removing it lowers it — the property is emergent and traceable to sourced inputs.
- **SC-002**: A base with negative power, insufficient shielding for its site's radiation, or a PP
  Special-Region violation is **red-flagged with the specific constraint**.
- **SC-003**: A construction project completes only when delivered-mass + crew-time land; a partially
  built base exposes **partial** properties; no module operates before its inputs land.
- **SC-004**: On-site regolith construction/ISRU **measurably reduces** the imported mass needed to
  reach a target shielding/closure (local material substitutes for launched mass).
- **SC-005**: A base above the closure/production threshold **survives** a 5-year resupply-embargo stress
  test; one below **fails** — both deterministic from identical inputs.
- **SC-006**: The self-sufficiency index **rises monotonically** as closure fractions and local
  production improve.
- **SC-007**: A double-run of the same seed + decisions produces **bit-identical** base state; state
  round-trips through save/load; a data-version mismatch is detected on load.
- **SC-008**: Every base/module/construction data file passes **schema + source-presence** validation in
  CI; no plausibility-bearing parameter lacks a `source`.
- **SC-009**: Base state (emergent properties, production/consumption, habitat state, settlement
  milestones) is exposed **read-only** for the economy, life support and politics.
- **SC-010**: The base system sustains the target scale (dozens of bases, hundreds of modules) at high
  time-warp within the core's tick-time budget.

## Assumptions

- **Scope — dynamic life support deferred** *(confirmed 2026-06-14)*: FA-07 computes static base
  properties + sizing; the dynamic in-mission consumption/dose/physiology/ECLSS-failure simulation is
  Slice 8, which consumes this base state.
- **On-site production split** *(confirmed 2026-06-14)*: FA-06 owns resource-extraction ISRU; FA-07 owns
  the construction use of local materials (regolith shielding/structures, base manufacturing). A base
  hosts FA-06 ISRU plants whose output feeds construction/sustainment.
- **Composed-value coupling** *(confirmed 2026-06-14)*: FA-07 depends only on the kernel core; Site
  properties, module-tech maturity, and construction-delivery status flow in as composed values the host
  assembles; the host bridges FA-06's delivery accounting → base construction progress.
- Site PP categories, illumination/thermal/slope/hazard/resource grades arrive through the **world
  model's surveyed belief-state**; the base system never reads hidden ground truth.
- **SI units** throughout; areal density (kg/m²) for shielding, closure as a fraction ∈ [0,1], power in
  watts, population as a crew count.
- Settlement milestones (e.g., first embargo-survivor) are **exposed** for the politics/scoring slice;
  their scoring weight is Slice 9's.
- The per-game **seed** fixes any stochastic construction/production outcomes; the base core is otherwise
  deterministic derivations.

## Dependencies

- **FA-01 (sim-core)**: module/slice contract, command/event routing, seeded streams, save/load,
  interrupt-and-pause for construction milestones / PP violations.
- **FA-03 (world)**: Sites (PP category, illumination, thermal, slope, comms visibility, hazard,
  resource grade) on the per-faction surveyed belief-state, and dynamical locations for orbital stations.
- **FA-05 (research)**: technology maturity/understanding gating modules (D3 inflatables, D4/D6 shielding
  & regolith construction, F2–F5 ECLSS closure, G ISRU, I10/I11 station/base modules).
- **FA-06 (economy/logistics)**: the project/resource-delivery accounting primitive, location-addressed
  stocks, crew-time and ops-capacity currencies, and ISRU plant output a base hosts.
