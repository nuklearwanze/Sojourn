# Feature Specification: Life Support & Crew (FA-08)

**Feature Branch**: `009-life-support-crew`
**Created**: 2026-06-14
**Status**: Draft
**Input**: User description: "Build Sojourn's life-support and crew model, the system that makes crewed missions materially harder than robotic ones. Authoritative design: design/04-SPACEFLIGHT.md (life support, crewed difficulty, EDL), design/01-RESEARCH.md (crew pipeline), design/02-TECH-TREE.md (life support & crew branch); also .specify/memory/constitution.md (Principles I, VII, VIII). Build on Slice 1 core, Slice 4 vehicles, Slice 5 personnel/research, and Slice 7 bases."

## Overview

Humans in space are the hardest part of spaceflight, and the difference between a robotic probe and
a crewed expedition must be **felt in mass, risk, time and consequence** — not flavour text. This slice
is the **dynamic, time-evolving** model of crewed life that Slice 7 deferred to it: a crewed asset (a
vehicle in transit, or an occupied base) consumes O₂/water/food over time against its ECLSS closure
fraction; each crew member accumulates radiation dose, micro-gravity deconditioning and psychological
load; ECLSS hardware degrades and can fail; and entry/descent/landing carries a real crew-risk roll.
Crewed mission and base viability depend on these physical states, and **loss-of-crew is a real,
modelled consequence** — never silently absorbed.

The slice consumes the vehicle's and base's **static life-support sizing** (closure capability, shield
mass/attenuation, population capacity — from Slices 4 and 7), the **crew roster, traits and ECLSS
technology maturity** (from Slice 5), and **ops capacity, crew-time and light-time delay** (from Slice
6), all through their existing interfaces; it owns none of their logic. It is the first slice with
genuinely **dynamic per-entity state stepped over time on seeded streams** (solar-particle-event storms,
ECLSS failures, EDL risk rolls, anomalies) — all *seeded and state-driven*: a low-maturity,
under-maintained, over-subscribed-ops, high-psych-load craft **earns** its failure probability.

**Scope boundary**: this slice owns the **dynamic time-evolution** of crew/ECLSS state; the **static
sizing** (closure fraction capability, shield mass, population capacity, endurance design figure,
EDL suitability) is owned by Slices 4/7 and consumed here. The **astronaut roster** (who exists, traits,
training) is Slice 5; this slice owns the per-astronaut **dynamic health/dose record** that evolves as
they fly. Loss-of-crew's **political/prestige/flight-freeze fallout** is **Slice 9 (Politics)**, which
consumes the loss-of-crew event this slice emits.

## Clarifications

### Session 2026-06-14

- Q: Does FA-08 track crew per individual astronaut, or as a per-mission aggregate cohort? → A: **Per-individual** — each crew member has a dynamic health record (career radiation dose, deconditioning indices, psychological load, alive/grounded status) keyed to the Slice 5 astronaut identity, so career dose and health follow the person across missions (design/01-RESEARCH §7).
- Q: Who owns the loss-of-crew consequence? → A: FA-08 owns the **physical/medical model + emits the loss-of-crew event and physical consequence** (crew member(s) lost, mission failed); the **political/prestige/flight-freeze fallout is Slice 9**, which consumes the event downstream.
- Q: How does FA-08 couple to the other slices? → A: FA-08 depends **only on the kernel core**; vehicle/base static sizing, crew roster + ECLSS-tech maturity, and ops/light-time flow in as **composed values** the host assembles (the FA-04/06/07 decoupling). FA-08 owns the dynamic time-evolution with seeded streams.
- Q: How should seeded event probabilities (ECLSS failure, anomaly, EDL crew-risk) compose from their contributing state factors? → A: **Multiplicative hazard** — probability = base_rate × ∏(factor multipliers), each factor (tech maturity, maintenance deficit, psych load, ops oversubscription, EDL suitability, body difficulty) a sourced multiplier, then clamped to [0,1]. Each factor is sourced and testable in isolation; a maxed factor can dominate.
- Q: How should the radiation dose limit be modelled? → A: **REID-based, age/sex-adjusted** — compute a **Risk of Exposure-Induced Death (REID)** from accumulated dose and the astronaut's age/sex (composed from the Slice 5 roster) via a **sourced dose→risk curve**; ground the astronaut at the **3% REID threshold** and flag a mission that would push REID past it. (NASA's actual radiation-limit model.)
- Q: How should the composite crew-capability metric be derived from the per-member states? → A: **Multiplicative product of per-state factors** — each state (deconditioning, psych load, radiation/health) maps to a [0,1] capability factor from a sourced curve; overall capability = ∏ factors. Stressors compound; each factor is independently sourced and testable.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Consumables versus ECLSS closure (Priority: P1)

A crewed asset consumes O₂/water/food/N₂ over time. Its ECLSS closure fraction (from the vehicle/base
sizing) sets the resupply make-up mass: closing the loop trades launched mass for ECLSS technology and
risk. A mission whose consumables stock plus scheduled resupply cannot cover its duration is non-viable;
if it runs out, the crew is at loss-of-crew risk. A robotic asset incurs none of this.

**Why this priority**: this is the felt core — *crewed = mass*. Closing the loop is the central
life-support trade, and it is independently demonstrable.

**Independent Test**: occupy an asset with a crew for a mission duration; assert consumables deplete at
the sourced rate; assert the resupply make-up mass = gross consumption × (1 − closure) and falls as
closure rises; assert a mission that cannot cover its duration is flagged non-viable; assert a robotic
asset of the same profile carries no consumable constraint.

**Acceptance Scenarios**:

1. **Given** a crew of N over D days at closure C, **When** consumption is computed, **Then** the
   **air/water** resupply make-up = air/water per-crew-day rate × N × D × (1 − C) and the **food** make-up
   = food per-crew-day rate × N × D (open-loop; ECLSS closure does not recycle food).
2. **Given** two identical missions differing only in closure, **When** compared, **Then** the
   higher-closure mission requires measurably less resupply mass.
3. **Given** a mission whose stock + resupply cannot cover its duration, **When** evaluated, **Then** it
   is flagged non-viable (and running out puts the crew at risk).
4. **Given** a robotic asset, **When** evaluated, **Then** it has no consumables/ECLSS constraint.

---

### User Story 2 - Radiation dose accumulation & limits (Priority: P1)

Each crew member accumulates radiation dose over time — a continuous galactic-cosmic-ray (GCR) rate
plus seeded solar-particle-event (SPE) storms — attenuated by the asset's shielding, with a storm
shelter mitigating acute SPE dose. Career dose is tracked per astronaut across missions; exceeding the
limit grounds the astronaut and bounds deep-space mission duration.

**Why this priority**: radiation is the hard ceiling on deep-space crewed duration and a defining
crewed constraint; career dose following the person is core to the crew pipeline.

**Independent Test**: accrue dose over time for a crew member from GCR + a seeded SPE; assert shielding
attenuates it and a storm shelter cuts the SPE dose; assert career dose accumulates across two missions;
assert exceeding the limit grounds the astronaut; assert SPE storms are deterministic per seed.

**Acceptance Scenarios**:

1. **Given** a shielded asset over time, **When** dose is accrued, **Then** career dose rises by the
   GCR rate × attenuation × time, plus any seeded SPE.
2. **Given** an SPE storm, **When** the crew shelters, **Then** the acute SPE dose is reduced versus not
   sheltering.
3. **Given** a crew member who flew a prior mission, **When** a new mission begins, **Then** their
   career dose carries over.
4. **Given** accumulated dose exceeding the limit, **When** evaluated, **Then** the astronaut is grounded
   and a mission that would exceed the limit is flagged.

---

### User Story 3 - Physiological deconditioning & countermeasures (Priority: P2)

Crew accumulate micro-gravity deconditioning over time across bone, muscle, cardiovascular and vision
indices. Countermeasures — exercise, pharmacology, and especially **artificial gravity** (spin-hab) —
slow it. Deconditioning degrades crew capability during the mission and lengthens post-mission recovery.

**Why this priority**: deconditioning makes long micro-g missions costly and makes artificial gravity a
real unlock, but it builds on the crewed-asset/crew-member core.

**Independent Test**: accrue deconditioning over a long micro-g mission; assert the indices rise at
sourced rates; assert countermeasures (and artificial gravity most of all) measurably slow them; assert
deconditioning reduces a crew-capability metric.

**Acceptance Scenarios**:

1. **Given** a micro-g mission over time, **When** evaluated, **Then** bone/muscle/cardio/vision indices
   degrade at sourced rates.
2. **Given** identical missions with and without artificial gravity, **When** compared, **Then** the
   spin-hab mission shows materially less deconditioning.
3. **Given** advanced deconditioning, **When** crew capability is evaluated, **Then** it is reduced.

---

### User Story 4 - Psychology under isolation, confinement & comms-lag (Priority: P2)

Crew accumulate psychological load over time, rising with mission duration, confinement (habitat volume
per crew) and comms-lag (light-time delay to Earth). High load raises crew error rates and anomaly
probability and erodes morale.

**Why this priority**: psychology is the human cost of distance and isolation and a seeded driver of
anomalies, but it layers on the crew-state core.

**Independent Test**: accrue psychological load over a long, distant, confined mission; assert it rises
with duration/comms-lag/confinement; assert higher load measurably raises the anomaly probability; assert
the anomaly draw is seeded and reproducible.

**Acceptance Scenarios**:

1. **Given** a long mission at high comms-lag in a cramped habitat, **When** load is accrued, **Then** it
   rises faster than a short, near-Earth, roomy mission.
2. **Given** high psychological load, **When** the anomaly probability is evaluated, **Then** it is
   higher than at low load.
3. **Given** a fixed seed, **When** anomaly outcomes are drawn, **Then** they reproduce.

---

### User Story 5 - ECLSS spares, maintenance & failure (Priority: P2)

ECLSS hardware has a reliability derived from its technology maturity and flight heritage; it degrades
over time and is subject to seeded failures. Crew-time and spares maintain it; insufficient maintenance
raises the failure probability. A failed ECLSS far from Earth — beyond resupply or abort reach — is a
loss-of-crew risk that the system never silently absorbs.

**Why this priority**: a failed life-support loop far from home is the existential crewed risk; it
makes maturity, spares and abort reach matter.

**Independent Test**: run an ECLSS of given maturity over time; assert lower maturity/heritage raises the
seeded failure probability; assert maintenance (crew-time + spares) lowers it; assert a critical failure
beyond abort reach produces a loss-of-crew risk and is not silently absorbed.

**Acceptance Scenarios**:

1. **Given** two ECLSS units differing only in maturity/heritage, **When** failure is evaluated, **Then**
   the less mature one fails more often (seeded).
2. **Given** an ECLSS maintained with crew-time + spares versus unmaintained, **When** compared, **Then**
   the maintained one has a lower failure probability.
3. **Given** a critical ECLSS failure beyond abort/resupply reach, **When** resolved, **Then** it is a
   loss-of-crew risk surfaced as an interrupt, never silently absorbed.

---

### User Story 6 - EDL & aerocapture crew risk (Priority: P2)

An entry/descent/landing or aerocapture attempt produces a seeded crew-risk outcome gated by the
vehicle's EDL suitability (from the designer), the target body's atmosphere and site hazards (from the
world), and the crew's state. The hard Mars EDL case is materially riskier for a given mass than an
airless-body propulsive landing — the modelled "Mars EDL gap." Failure is a loss-of-vehicle / loss-of-crew
consequence.

**Why this priority**: EDL is where missions die; the crew-risk of landing is a defining crewed
constraint, but it composes the vehicle and world states.

**Independent Test**: attempt an EDL with a sourced vehicle suitability at a body; assert the crew-risk
outcome depends on suitability + body + crew state; assert the Mars case is materially harder than an
airless landing; assert a failure is representable as loss-of-crew.

**Acceptance Scenarios**:

1. **Given** an EDL attempt, **When** resolved, **Then** the crew-risk probability reflects vehicle
   suitability, body atmosphere/hazards and crew state.
2. **Given** the same vehicle mass, **When** landing on Mars versus an airless body, **Then** the Mars
   attempt carries a materially higher loss probability.
3. **Given** an EDL failure, **When** resolved, **Then** it is a loss-of-vehicle / loss-of-crew
   consequence (seeded, deterministic).

---

### User Story 7 - Crew-state exposure & loss-of-crew consequence (Priority: P3)

The system exposes, read-only, each crew member's dose/deconditioning/psych/career state and each crewed
asset's consumables/ECLSS/viability state (for the economy, politics and UI). Crewed viability depends on
the composite state. Loss-of-crew is a real, modelled consequence — a physical loss plus an emitted
event — with the political fallout deferred to the politics slice.

**Why this priority**: the integration seam other slices consume; the mechanics stand alone first.

**Independent Test**: query a crew member's full state and an asset's viability; assert a mission becomes
non-viable when consumables/dose/ECLSS/capability cross thresholds; assert loss-of-crew emits an event +
physical consequence the politics slice can consume.

**Acceptance Scenarios**:

1. **Given** an operating crewed asset, **When** queried, **Then** it reports each crew member's
   dose/deconditioning/psych/career state and the asset's consumables/ECLSS viability.
2. **Given** a state crossing a viability threshold, **When** evaluated, **Then** the mission/base is
   flagged non-viable.
3. **Given** a loss-of-crew event, **When** it occurs, **Then** the crew is physically lost, the mission
   fails, and an event is emitted for the politics slice (the political fallout is FA-09's).

---

### Edge Cases

- **Consumables exhaustion**: a mission that runs out of consumables before resupply puts the crew at
  loss-of-crew risk (flagged, not silently survived).
- **Dose-limit breach**: an astronaut at career-dose limit is grounded; a planned mission that would
  exceed it is flagged non-viable.
- **SPE during EVA / no shelter**: an SPE storm with the crew unable to shelter delivers the full acute
  dose.
- **ECLSS failure beyond abort reach**: surfaced as an interrupt loss-of-crew risk; near-Earth, an abort
  is an option.
- **Maxed deconditioning**: crew capability floored; post-mission recovery extended (or career-ending).
- **Mars EDL**: the hard case — a large-mass crewed Mars landing carries the highest modelled loss
  probability.
- **Robotic mission**: carries none of the crew constraints — the crewed-difficulty multiplier is real.
- **Data version mismatch on load**: a save referencing changed physiology/ECLSS data is detected.

## Requirements *(mandatory)*

### Functional Requirements

#### Consumables & ECLSS closure (US1)

- **FR-LSC-101**: A crewed asset MUST consume O₂/water/food/N₂ over time at a **sourced per-crew-day
  rate**; the consumables stock depletes as the crew occupies it.
- **FR-LSC-102**: ECLSS **closure fraction** (composed from the vehicle/base sizing) MUST set the
  resupply make-up for the **air/water loop only** (O₂/water/N₂/CO₂ — what ECLSS recycles): air/water
  make-up = air/water gross × (1 − closure). **Food is open-loop** (carried as mass) in this slice —
  ECLSS closure does **not** recycle food (food is bioregenerative, the base greenhouse loop of FA-07,
  optionally composed in as a food-supply reduction). Total make-up = air/water make-up + food gross.
  Higher closure ⇒ less air/water resupply, traded for ECLSS mass/technology/risk.
- **FR-LSC-103**: A crewed mission whose consumables stock + scheduled resupply cannot cover its duration
  MUST be flagged **non-viable**; running out MUST put the crew at loss-of-crew risk.
- **FR-LSC-104**: The closure tiers (open-loop → physico-chemical → high-closure → bioregenerative) MUST
  be data-driven and **gated by ECLSS technology maturity** (composed from research).
- **FR-LSC-105**: A **robotic** asset MUST incur none of these consumable/ECLSS constraints — the
  crewed-difficulty multiplier is real (Principle VII).

#### Radiation dose (US2)

- **FR-LSC-201**: Each crew member MUST accumulate radiation dose over time from a continuous **GCR**
  rate (per environment) plus seeded **SPE** storms.
- **FR-LSC-202**: **Shielding** (composed from the vehicle/base) MUST attenuate the dose; an **SPE storm
  shelter** MUST mitigate the acute SPE dose when the crew shelters.
- **FR-LSC-203**: **Career dose** MUST be tracked per astronaut (tied to the research crew pipeline)
  across missions; the limit MUST be modelled as a **Risk of Exposure-Induced Death (REID)** computed
  from accumulated dose and the astronaut's **age/sex** (composed from the Slice 5 roster) via a
  **sourced dose→risk curve**; reaching the **3% REID threshold** MUST ground the astronaut.
- **FR-LSC-204**: Accumulated dose / REID MUST **bound deep-space mission duration** — a mission that
  would push an astronaut's REID past the 3% threshold MUST be flagged non-viable.
- **FR-LSC-205**: SPE storms MUST be **seeded + state-driven** (deterministic per seed), not pure RNG.

#### Physiological deconditioning (US3)

- **FR-LSC-301**: Crew MUST accumulate **micro-gravity deconditioning** over time across
  bone/muscle/cardiovascular/vision indices at sourced rates.
- **FR-LSC-302**: **Countermeasures** (exercise, pharmacology, **artificial gravity**) MUST slow
  deconditioning at sourced effectiveness; artificial gravity MUST be the strongest mitigation.
- **FR-LSC-303**: Deconditioning MUST affect **crew capability** during the mission and **post-mission
  recovery**. The composite **crew-capability** metric MUST be the **multiplicative product of per-state
  capability factors** — deconditioning, psychological load, and radiation/health — each a sourced
  [0,1] curve, so stressors compound (used by the EDL crew-state gate FR-LSC-601 and viability FR-LSC-703).

#### Psychology (US4)

- **FR-LSC-401**: Crew MUST accumulate **psychological load** over time, rising with mission duration,
  confinement (habitat volume per crew, composed from the base/vehicle) and comms-lag (light-time delay).
- **FR-LSC-402**: Psychological load MUST raise crew **error rates / anomaly probability** and reduce
  morale at sourced sensitivities.
- **FR-LSC-403**: Psychological state MUST be a **seeded, state-driven** contributor to anomaly events (a
  high-load crew earns its anomaly probability).

#### ECLSS spares, maintenance & failure (US5)

- **FR-LSC-501**: ECLSS hardware MUST have a **reliability derived from technology maturity + flight
  heritage** (composed from research); it MUST degrade over time and be subject to seeded failures.
- **FR-LSC-502**: **Crew-time + spares** MUST maintain ECLSS; insufficient maintenance MUST raise the
  failure probability.
- **FR-LSC-503**: A **failed ECLSS far from Earth** (beyond resupply/abort reach) MUST be a loss-of-crew
  risk surfaced as an interrupt; the system MUST NOT silently absorb a critical failure.
- **FR-LSC-504**: ECLSS failures MUST be **seeded + state-driven** (maturity/maintenance/heritage),
  deterministic per seed.

#### EDL & aerocapture crew risk (US6)

- **FR-LSC-601**: An EDL/aerocapture attempt MUST produce a **seeded crew-risk outcome** gated by the
  vehicle's EDL suitability (composed from the designer), the target body's atmosphere/site hazards
  (composed from the world), and crew state.
- **FR-LSC-602**: The **Mars EDL case** MUST be materially harder (higher loss probability for a given
  mass) than airless-body propulsive landing — the modelled "Mars EDL gap."
- **FR-LSC-603**: An EDL failure MUST be representable as a **loss-of-vehicle / loss-of-crew** consequence
  (seeded, deterministic).

#### Crew-state exposure & loss-of-crew (US7)

- **FR-LSC-701**: The system MUST expose, through a **read-only query surface**, each crew member's
  dose/deconditioning/psych/career state and each crewed asset's consumables/ECLSS/viability state (for
  the economy, politics and UI).
- **FR-LSC-702**: **Loss-of-crew** MUST be a real, modelled consequence — a physical loss (crew member(s)
  lost, mission failed) + an emitted event; the political/prestige/flight-freeze fallout is **deferred to
  the politics slice (FA-09)**, consumed downstream.
- **FR-LSC-703**: Crewed mission/base **viability MUST depend on the composite** crew/ECLSS state (a
  mission becomes non-viable when consumables, dose, ECLSS or crew capability cross thresholds).

#### Cross-cutting

- **FR-LSC-801**: All physiological, ECLSS, radiation and EDL parameters MUST live in versioned,
  schema-validated data files with non-empty `source`; CI MUST reject missing sources.
- **FR-LSC-802**: The model MUST be **deterministic** — identical seed + decisions ⇒ identical crew
  state; all stochastic outcomes (SPE storms, ECLSS failures, EDL rolls, anomalies) MUST derive from
  **named seeded streams**; no wall-clock.
- **FR-LSC-803**: Crew/asset state MUST **round-trip** through save/load with identical reload and pin its
  data version.
- **FR-LSC-804**: The model MUST run **headless** and integrate via the established module boundary,
  consuming vehicle sizing, base state, crew roster/maturity and ops/light-time **as composed values**
  without embedding those systems' logic.
- **FR-LSC-805**: Operations MUST occur under **light-time delay and finite ops capacity** (composed from
  the economy/ops layer); over-subscribed ops MUST raise anomaly risk (consistent with FA-06).
- **FR-LSC-806**: The slice MUST contain **no combat/weapons** (Principle IX); crew loss is a modelled
  safety consequence, never combat.
- **FR-LSC-807**: The **crewed-difficulty premise** MUST hold — a crewed mission of a given profile MUST
  be materially harder (mass, risk, time, consequence) than the equivalent robotic mission (Principle VII).
- **FR-LSC-808**: Every seeded event probability (ECLSS failure FR-LSC-504, anomaly FR-LSC-403, EDL
  crew-risk FR-LSC-601, SPE-driven outcomes) MUST be composed as a **multiplicative hazard** —
  `probability = base_rate × ∏(factor multipliers)` over the contributing factors (tech maturity,
  maintenance deficit, psych load, ops oversubscription, EDL suitability, body difficulty), each a
  **sourced** multiplier, the result **clamped to [0,1]**. Each factor MUST be independently sourced and
  testable in isolation (raising one factor monotonically changes the probability).

### Key Entities

- **CrewedAsset**: a vehicle in transit or an occupied base hosting crew — with a consumables stock, an
  ECLSS state, shielding/spin-gravity flags (composed from the vehicle/base), comms-lag (distance) and an
  abort/resupply reach.
- **CrewMember**: a per-astronaut **dynamic health record** (Slice 5 astronaut id + traits; career
  radiation dose, deconditioning indices, psychological load, capability, alive/grounded) — career state
  follows the person across missions.
- **ConsumablesState**: O₂/water/food/N₂ stock + gross consumption + ECLSS-closure make-up rate.
- **EclssState**: closure capability + reliability (from maturity/heritage) + degradation + maintenance
  (crew-time/spares) + failure state.
- **RadiationState**: GCR rate, seeded SPE storms, shielding attenuation, storm-shelter mitigation, and
  the accumulated career dose → **REID** (via a sourced dose→risk curve + the astronaut's age/sex).
- **DeconditioningState**: bone/muscle/cardio/vision indices + countermeasure effectiveness (incl.
  artificial gravity).
- **PsychState**: psychological load + comms-lag + confinement + anomaly contribution.
- **EdlRisk**: a per-attempt seeded crew-risk outcome (vehicle suitability × body × crew state).
- **LossOfCrew**: the modelled consequence — physical loss + emitted event (political fallout → FA-09).
- **CrewSnapshot**: the read-only query surface (crew state, asset viability, loss-of-crew, exposure).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For a crew of N over D days at closure C, the **air/water** resupply make-up equals
  air/water per-crew-day rate × N × D × (1 − C) and **food** make-up equals food per-crew-day rate × N × D
  (open-loop); raising closure measurably reduces the air/water make-up (not the food); a mission that
  cannot cover its duration is flagged non-viable.
- **SC-002**: A crew member's career dose accumulates from GCR + seeded SPE, attenuated by shielding and
  reduced by sheltering; its **REID** (from the sourced dose→risk curve + the astronaut's age/sex) reaching
  the 3% threshold grounds the astronaut — deterministic per seed.
- **SC-003**: Micro-g deconditioning rises over time; an artificial-gravity mission shows materially less
  than an equivalent micro-g mission; deconditioning reduces a crew-capability metric.
- **SC-004**: Psychological load rises with duration/comms-lag/confinement and measurably raises the
  anomaly probability; the draw is reproducible per seed.
- **SC-005**: A lower-maturity/under-maintained ECLSS has a higher seeded failure probability; a critical
  failure beyond abort reach surfaces a loss-of-crew risk and is never silently absorbed.
- **SC-006**: An EDL crew-risk outcome reflects vehicle suitability + body + crew state; the Mars EDL case
  is materially harder than an airless-body landing for the same mass.
- **SC-007**: A **robotic** mission of a given profile incurs none of the crew constraints, while the
  equivalent **crewed** mission is materially harder (more mass, more risk, more constraints) — the
  crewed-difficulty multiplier is demonstrable.
- **SC-008**: A double-run of the same seed + decisions produces **bit-identical** crew/asset state; state
  round-trips through save/load; a data-version mismatch is detected on load.
- **SC-009**: Every physiology/ECLSS/radiation/EDL data file passes **schema + source-presence**
  validation in CI; no plausibility-bearing parameter lacks a `source`.
- **SC-010**: Loss-of-crew is exposed as a real event + physical consequence the politics slice can
  consume; crew/asset state is queryable read-only for the economy, politics and UI.
- **SC-011**: The model sustains the target scale (dozens of crewed assets, hundreds of crew members) at
  high time-warp within the core's tick-time budget.

## Assumptions

- **Per-individual crew** *(confirmed 2026-06-14)*: each crew member is a dynamic health record keyed to
  the Slice 5 astronaut identity; career dose/health follow the person across missions.
- **Loss-of-crew boundary** *(confirmed 2026-06-14)*: FA-08 owns the physical/medical model + the
  loss-of-crew event/consequence; the political/prestige/flight-freeze fallout is Slice 9.
- **Composed-value coupling** *(confirmed 2026-06-14)*: FA-08 depends only on the kernel core; vehicle/base
  static sizing, crew roster + ECLSS-tech maturity, and ops/light-time flow in as composed values; FA-08
  owns the dynamic time-evolution with seeded streams.
- The model steps on a **daily** cadence (consistent with the other slices), accumulating
  consumption/dose/deconditioning/psych and resolving seeded daily events.
- The **astronaut roster + traits + age/sex** come from Slice 5 (the age/sex feed the REID dose-limit
  model); FA-08 owns the per-astronaut dynamic health/dose record and computes REID from a sourced
  dose→risk curve. The host bridges the career dose/REID back to the Slice 5 pipeline record.
- **SI units** throughout (dose in sieverts, mass in kg, time in days/seconds, volume in m³).
- The **per-game seed** fixes SPE storms, ECLSS failures, EDL rolls and anomalies; the model is otherwise
  deterministic.

## Dependencies

- **FA-01 (sim-core)**: module/slice contract, seeded streams, command/event routing, save/load, and the
  interrupt-and-pause loop for loss-of-crew, SPE storms, ECLSS failures and anomalies.
- **FA-04 (vehicle)**: vehicle life-support sizing (closure capability, shield mass, endurance) and EDL
  suitability — composed in.
- **FA-05 (research/personnel)**: the astronaut roster, traits, training and **age/sex** (for the REID
  dose-limit model); ECLSS technology maturity + flight heritage; the crew-pipeline career-dose record.
- **FA-06 (economy/logistics)**: ops capacity, crew-time currency, light-time delay, and resupply/abort
  reach over the logistics graph.
- **FA-07 (bases)**: the static base habitat/closure/shielding/population state for occupied bases.
