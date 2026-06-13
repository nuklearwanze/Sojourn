# Phase 0 Research: Vehicle Designer & Propulsion (FA-04)

Decisions resolving the Technical Context against Constitution v1.0.0 (esp. Principles II/VII), the
FA-01/02/05 contracts in the tree, and the spec's clarified scope. Format: **Decision / Rationale /
Alternatives rejected**. No `NEEDS CLARIFICATION` remain (spec clarified 2026-06-13).

---

## R1 — Crate topology: a designer above astro + research

**Decision.** A new crate `crates/sojourn-vehicle` depends on `sojourn-core` (kernel),
`sojourn-astro` (the `PropulsionEndpoint`/`EngineDef` shape + body μ/radius for T/W) and
`sojourn-research` (the `maturity()` contract). FA-06 will later depend on `sojourn-vehicle` (cost).

**Rationale.** The designer *produces* what FA-02 flies and *consumes* what FA-05 matures, so it
sits above both; both already exist and are CI-verified, so the seams are real, not stubs. The
dependency arrow is acyclic (vehicle → {astro, research} → core) and points one way (FA-06 → vehicle).

**Alternatives rejected.** (a) Put the designer in astro — pollutes the propagator with design-time
concerns and game-knowledge. (b) Duplicate the maturity model — violates Principle VI/single-source.

---

## R2 — Design-time vs flight-time split + the FA-02 binding (clarified Q1:A)

**Decision.** FA-04 owns **design-time** state only: the design library (saved compositions, class
templates) and per-design **cumulative production counts** (learning curve). **Flight-time craft
state stays in FA-02.** A craft is spawned from a design by carrying the designer's **engine
parameters + dry mass + propellant capacity + boil-off rate inline** into an extended FA-02
`SpawnCraft` (the **one additive astro change**); FA-02 stores them on its `Craft` and its
burn-executor `consume` remains the sole live-mass mutation. The FA-02 fixture path (engine-by-id)
is unchanged.

**Rationale.** FA-02 *already* owns craft mass/propellant and mutates it correctly; splitting that
across two slices (option B) breaks single-writer and duplicates working state. Carrying engine
params inline at spawn (vs a runtime engine-catalogue view) avoids the kernel's scalar-only view
limitation and keeps the engine snapshot with the craft that flies it. This is the
"registration/spawn mechanism" the spec deferred to the plan.

**Alternatives rejected.** (a) FA-04 owns live mass/propellant, FA-02 mutates cross-module — breaks
single-writer. (b) A published engine-catalogue view — the kernel view system is scalar-only (FA-03
R5); serializing `EngineDef` into a string view is a hack vs inline-at-spawn. (c) A shared craft slice
— co-ownership is forbidden.

---

## R3 — Derived outputs are pure query-time computations (not stored)

**Decision.** The designer's derived outputs (mass, Δv, thrust/T-W, power/thermal, reliability,
life-support sizing, EDL suitability, cost) are **pure functions** computed at query time over a
`DesignSnapshot` that composes the design (FA-04 slice) with the **current FA-05 maturity** and
caller-supplied gravity. They are **not** stored in the slice.

**Rationale.** Reliability and cost depend on research state that changes over time; recomputing on
demand keeps them honest (a design's reliability rises as its components mature) without a stale
stored copy or invalidation logic. The slice stays tiny (compositions + counts), so saves/round-trips
are trivial and deterministic.

**Alternatives rejected.** (a) Store derived outputs in the slice — stale vs research; invalidation
complexity; bloats determinism surface. (b) Recompute in `step()` each tick — wasteful; derivations
are query-time, between-ticks (the FA-02/03/05 pattern).

---

## R4 — Component availability: trust-the-caller compose, query-time gating

**Decision.** The `ComposeDesign`/`EditDesign` commands are **trust-the-caller** for component
*availability*: structural validity is checked (component exists, design well-formed, staging
sane); the *maturity/research gating* ("is this component researched/mature enough for faction F?")
is surfaced by the **design-query surface** (which composes FA-05 maturity) and enforced by the host/
UI before composing — the same honest-seam pattern as FA-03's observation entitlement and FA-05's
funding.

**Rationale.** A module cannot richly read another module's slice during `on_command` (slice
isolation; views are scalar-only). Gating at query time (where the snapshot already composes FA-05
maturity) is clean and keeps the command deterministic and self-contained.

**Alternatives rejected.** (a) Read FA-05 maturity inside `on_command` — cross-slice read during
command application; not available. (b) A scalar maturity view — can't carry per-component maturity.

---

## R5 — Propulsion family models → endpoint params

**Decision.** Each family (chemical; electric ion/Hall/MPD/VASIMR/electrospray; nuclear-thermal;
nuclear-electric; gated frontier) is a **physical model** in data: exhaust velocity (Isp·g₀),
thrust, input power, propellant type/density, throttle range, restart/duty, and a **mass model**
(engine + feed + power + radiators). The designer derives a `PropulsionEndpoint`-shaped param set
(`EngineDef`) per engine. **Electric propulsion is power-limited** (delivered thrust ∝ available
power / rated power, the FA-02 contract coupling); **nuclear-electric reactor + shielding + radiator
mass are first-class dry mass** computed from the power level and the thermal model (R7).

**Rationale.** Directly implements design §3 and FR-VEH-005/006; producing FA-02's `EngineDef` shape
means the propagator flies designer engines unchanged (R2).

**Alternatives rejected.** (a) A flat engine stat list — violates "model, not list" (design §3) and
Principle II. (b) Ignore power/radiator mass for EP/NEP — the dishonest coupling the design forbids.

---

## R6 — Mass & Δv: rocket equation per stage/mode; T/W vs supplied gravity

**Decision.** Dry/wet mass from component masses + propellant; **per-stage and per-mode Δv** via
`Δv = v_e·ln(m0/m1)` (stages and modes reported separately — a chemical stage + an EP stage never
blend into one fictitious Isp, edge case). **Thrust and T/W** computed against **caller-supplied
surface gravity** per "gravity field of interest" (the host supplies `g = μ/r²` from the astro
catalogue), so the vehicle crate stays catalogue-agnostic.

**Rationale.** The rocket equation is the law (design §2); per-stage/mode separation is the honest
representation; supplied-gravity keeps coupling light and testable against analytic values.

**Alternatives rejected.** (a) Blended-Isp single Δv — dishonest for mixed propulsion. (b) Hard-depend
on a specific catalogue for gravity — needless coupling; pass `g` in.

---

## R7 — Power & thermal balance + the radiator coupling fixed-point

**Decision.** **Power balance** per mode: generation (PV scaled by solar distance, RTG, fission) vs
demand (propulsion/avionics/payload/life-support); margin must be ≥ 0 in some mode. **Thermal
balance**: waste-heat load → required radiator area/mass (carried as dry mass). The **power↔radiator
coupling** (more power ⇒ more heat ⇒ more radiator mass ⇒ lower Δv) is resolved to a **fixed point**
by bounded iteration (documented max iterations; non-convergence is a flag, not a hang).

**Rationale.** Implements FR-VEH-006's first-class waste-heat constraint and the design §3 NEP
bottleneck; the fixed-point makes the coupling consistent and deterministic.

**Alternatives rejected.** (a) Ignore the coupling (radiators as a fixed mass) — dishonest. (b)
Unbounded iteration — non-determinism/hang risk; bound it and flag.

---

## R8 — Reliability-block-diagram from FA-05 maturity (clarified Q2:A)

**Decision.** Each component's per-use reliability = FA-05 `maturity().reliability` (TRL +
flight-units + UL). Composed across the vehicle by a **reliability-block-diagram**: series chains
multiply (∏ rᵢ); **declared redundancy blocks** evaluate as parallel (1 − ∏(1 − rᵢ)). Redundancy is
a **design-data declaration** in the composition. Sub-TRL-6 components are flagged (R13).

**Rationale.** The clarified standard model — captures "more low-TRL parts ⇒ lower; redundancy ⇒
higher" with a clean analytic target for SC-003, sourced and simple.

**Alternatives rejected.** (a) Series-product only — can't model redundancy a real design needs. (b)
Load-sharing/common-cause — unjustified fidelity for this slice.

---

## R9 — Static life-support sizing (clarified scope Q2:A)

**Decision.** For crewed designs the designer computes **static sizing**: consumables mass
(open-loop) or ECLSS closure-fraction mass + endurance, radiation-shield mass vs a dose target, and
crew-accommodation mass — as design outputs and guards. The **dynamic** in-mission consumption,
physiology, psychology and ECLSS failure are FA-08's, which consumes this sizing.

**Rationale.** The clarified boundary: the designer must size crewed vehicles (endurance/dose guards)
but not simulate the mission; FA-08 does the latter.

**Alternatives rejected.** (a) No life support here — can't design crewed vehicles. (b) Full
in-mission sim — FA-08's scope.

---

## R10 — EDL/landing suitability checks (clarified scope Q1:A)

**Decision.** The designer computes **design-time suitability checks**: T/W vs target-body surface
gravity (can it land?), heat-shield adequacy + ballistic coefficient for atmospheric bodies, and
throttle/guidance fit — as red-flags. The **EDL flight phase** (entry-heating integration, the flown
Mars EDL gap) is FA-02's entry handoff and later slices.

**Rationale.** The clarified boundary: static "can it land here?" checks belong to the designer's
guards; the flown descent is FA-02/later.

**Alternatives rejected.** (a) Full EDL flight sim here — overlaps FA-02. (b) No landing checks —
loses a core realism guard.

---

## R11 — Cost + build-time + learning curve (clarified scope Q3:A)

**Decision.** A **physical** unit-cost and build-time estimate from sourced mass/maturity params,
with a **learning curve** (unit cost declines with cumulative production count). The slice stores
per-design cumulative production count (incremented by a `RegisterProduction` command); cost is
computed at query time from it. The **six-currency economic pricing/funding/market** is FA-06's.

**Rationale.** Implements FR-VEH-002's cost output and the clarified physical-vs-economic split; the
learning curve is the FA-06-relevant emergent economy of reuse/standardisation, computed physically
here.

**Alternatives rejected.** (a) Money/currency cost here — FA-06's scope. (b) No cost — drops the
design-time affordability signal.

---

## R12 — Traceability tree

**Decision.** Every derived output exposes a **traceability tree**: a recursive structure whose
leaves are **sourced data values** (component params, propulsion-model constants, tuning params) and
whose internal nodes are the named operations (rocket equation, sum, block-diagram). A query returns
the full tree for any output.

**Rationale.** FR-VEH-003 / Principle VIII — the honesty contract is that any number is drillable to
its sourced basis; building it as data (not prose) makes it CI-checkable and FA-10-renderable.

**Alternatives rejected.** (a) Opaque numbers — violates traceability. (b) Prose explanations —
not machine-checkable.

---

## R13 — Realism guards

**Decision.** A `guards` pass produces **red-flags** for the physically impossible: unresolvable
negative power margin, radiator shortfall, lander T/W below local gravity, structural limit
exceeded, crewed endurance below a stated mission duration, radiation dose above a crew limit, Δv
short of a stated requirement, sub-TRL-6 component. Each flag carries the specific violated
constraint + the offending value. Marginal-but-possible designs are **buildable** with truthful
reliability/risk; no guard is bypassable by a non-sourced value.

**Rationale.** FR-VEH-004 — the guards make the physics bind; "informed gambles, not cheats."

**Alternatives rejected.** (a) Hard-block marginal designs — removes the gamble. (b) Silent
acceptance — dishonest.

---

## R14 — Determinism, data-version pin, touchpoints

**Decision.** Derivations are pure (libm-only, no randomness/wall-clock). Component/propulsion/param
data is content-hashed and **pinned in saves** (extends the FA-02/03/05 pattern). **No kernel
change.** **One additive astro change** (R2: inline engine params on `SpawnCraft`). New events
(`vehicle-produced` for heritage-relevant production) via the data registry. Analytic validation
cases (rocket-equation Δv, T/W, power-limited thrust, mass fractions) gate CI (constitution).

**Rationale.** Keeps FA-01 frozen and confines change to one additive, contract-consistent astro
extension the fixture path doesn't exercise.

**Alternatives rejected.** (a) A kernel design service — over-generalises. (b) Mutate FA-02's engine
catalogue file at build time — designs are runtime state, not data.

---

## R15 — Design-query surface

**Decision.** `DesignSnapshot::from_core(&core, &vehicle_module, &research_module, gravity)` via
kernel `with_slice` over the vehicle slice, composing FA-05's `ResearchSnapshot` (maturity) and
caller-supplied gravity. Pure functions answer: a design's derived outputs + traceability trees, its
red-flags, its engine `EngineDef`(s), composed reliability, life-support sizing, EDL suitability, and
cost/build estimate — faction-scoped where research is involved.

**Rationale.** Identical seam to FA-02/03/05 — pure fns over a composed snapshot, between ticks,
IPC-serializable for FA-06/09/10 and the Tauri host.

**Alternatives rejected.** (a) Mutable handles — break read-only/determinism. (b) Store outputs (R3).

---

## Summary of decisions feeding Phase 1

| # | Decision | Primary artifacts |
|---|---|---|
| R1 | `sojourn-vehicle` above astro + research | plan structure |
| R2 | Design-time/flight-time split; inline-engine spawn (additive astro) | contracts/propulsion-binding |
| R3 | Derived outputs are pure query-time computations | contracts/design-queries, data-model |
| R4 | Trust-the-caller compose; query-time gating | contracts/vehicle-commands |
| R5 | Propulsion family models → EngineDef; power-limited EP; NEP radiators | data-model, contracts/component-data |
| R6 | Rocket equation per stage/mode; T/W vs supplied gravity | data-model |
| R7 | Power/thermal balance + radiator coupling fixed-point | data-model |
| R8 | Reliability-block-diagram from FA-05 maturity | contracts/design-queries, data-model |
| R9 | Static life-support sizing | data-model |
| R10 | EDL suitability checks | data-model |
| R11 | Cost + learning curve; production count in slice | contracts/vehicle-commands, data-model |
| R12 | Traceability tree | contracts/design-queries |
| R13 | Realism guards | data-model |
| R14 | Determinism; data-version pin; additive astro; analytic gates | contracts/component-data, contracts/propulsion-binding |
| R15 | Composed design-query surface | contracts/design-queries |
