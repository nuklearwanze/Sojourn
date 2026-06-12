# Phase 0 Research: Astrodynamics & Flight (FA-02)

All Technical Context unknowns resolved. Decisions follow the FA-01 stack (Rust, libm-only
transcendentals, per-platform determinism) and the licence constraint (MIT/Apache-compatible —
trivially satisfied: no new third-party dependencies are added).

## R1 — Integrator

- **Decision**: Fixed-step classical **RK4** over the craft state (position, velocity, mass), with state-tiered step sizes (R2). No adaptive error control at runtime — accuracy is guaranteed by tier selection plus CI tolerance gates (energy drift, validation cases). Diverted small bodies integrate with the same scheme.
- **Rationale**: The force model includes non-conservative terms (thrust, drag), which forfeits the structural advantage of symplectic integrators; RK4 at the chosen tiers comfortably meets the documented tolerances (two-body energy drift ≤1e-8/yr at coast tier — verified by the validation suite, which is the binding contract), is branch-free per step (deterministic), trivially testable, and simple enough to audit. The constitution's "symplectic/high-order with deterministic step control" is satisfied via the high-order + deterministic-tiering reading; if validation ever demands better long-coast behaviour, a Störmer–Verlet coast path can be added behind the same tier mechanism without contract changes.
- **Alternatives considered**: Störmer–Verlet/leapfrog (symplectic, lovely for pure gravity, but thrust/drag break the symplectic property and force a hybrid anyway); Gauss–Jackson 8th-order (the professional choice for orbit propagation, but multi-step start-up state complicates save/load and SOI rebasing for marginal gain at game tolerances); Dormand–Prince adaptive (adaptive step control from local error estimates is runtime-state-dependent in ways that make warp-invariance audits harder; fixed tiers are the deterministic equivalent).

## R2 — Step tiering (state-driven, warp-invariant)

- **Decision**: Three data-configured tiers, selected per craft per kernel step from simulation state only: **coast** (default; module cadence, e.g. 60 s steps), **encounter/periapsis** (within a data-defined radius factor of any gravitating body, or inside an atmosphere's interface altitude: 1 s steps), **burn** (engine thrust non-zero: 1 s kernel escalation with internal substeps, e.g. 0.25 s, fixed subdivision). Tier boundaries are deterministic functions of state; the kernel's cadence+escalation declarations (FR-CORE-104) carry the coarse/fine split, and within one module step the integrator subdivides by a fixed integer factor per tier.
- **Rationale**: Satisfies FR-ASTRO-105 (step control from state alone), keeps quiet-span jumping effective (coasting craft step at cadence), concentrates cost where physics demands it (deep periapsis, burns), and is exactly reproducible regardless of warp (FR-CORE-204 invariance — tier selection never sees the host's pacing).
- **Alternatives considered**: per-craft adaptive step from local error (R1 note — harder determinism story); global fine step always (kills the performance envelope).

## R3 — Rails: analytic ephemerides

- **Decision**: Railed bodies expose `state_at(SimTimeNs) → (position, velocity)` computed on demand from **Keplerian elements** via Kepler-equation solution (Newton iteration with libm, universal-variable formulation for near-parabolic safety). Elements come from the catalogue (test fixtures now; FA-03's real data later — the interface admits future ephemeris representations, e.g. piecewise elements per epoch range, behind the same `state_at`). Railed positions are **never stepped or stored** — pure functions of time, cached per (body, tick) within a step for reuse across craft.
- **Rationale**: Exactly the clarified architecture (rails faithful to published elements); O(1) per query; cache makes 200 craft × few dozen gravitating bodies cheap; the function-of-time property is what keeps planner queries pure (FR-ASTRO-409).
- **Alternatives considered**: tabulated/interpolated ephemerides (better for FA-03's real data later — the interface allows it; unnecessary for fixtures); stepping bodies numerically (rejected in clarification).

## R4 — Lambert solver (porkchop)

- **Decision**: In-crate universal-variables Lambert solver (Battin/Curtis-style iteration, libm-only), supporting prograde transfers with 0..N revolution options, returning departure/arrival velocity vectors or an explicit no-convergence result. Porkchop = grid of Lambert solves between railed states; cells report Δv (vs departure/arrival circular or craft state), C3, TOF, solvability.
- **Rationale**: A bounded, classical algorithm; in-crate keeps the determinism/licence story trivial and the iteration caps explicit (fixed max iterations + tolerance from data ⇒ deterministic).
- **Alternatives considered**: Izzo's algorithm (faster convergence, more implementation subtlety; revisit if grid latency demands), external crates (none mature/maintained enough to outweigh a ~300-line classical solver we fully control).

## R5 — Math primitives

- **Decision**: In-crate `math.rs`: `Vec3` (f64), dot/cross/norm, orbital-element ↔ state-vector conversions, frame rotations — all transcendentals via `libm`. No nalgebra/glam.
- **Rationale**: ~200 lines we fully control; avoids dependency SIMD/codegen variance concerns entirely; nothing here needs matrix machinery beyond 3×3 rotations.
- **Alternatives considered**: `nalgebra` (large, generic, overkill), `glam` (f32-centric, SIMD paths complicate the float-policy audit).

## R6 — State representation & rebasing

- **Decision**: Craft state is stored **relative to its dominant body** (the SOI owner): position/velocity in body-centred inertial frame + dominant-body id. SOI crossing rebases the state into the new dominant frame by exact vector arithmetic at the crossing step boundary (continuity by construction, FR-ASTRO-202), emitting the `soi-crossing` event. Heliocentric absolute state is derived on demand (craft-relative + railed body state).
- **Rationale**: Preserves precision where it matters (mm-scale near bodies rather than f64 ulps at 4×10¹² m), aligns bookkeeping with patched-conic planning, and makes "which body's J2/drag applies" trivial.
- **Alternatives considered**: single barycentric frame for everything (simpler bookkeeping, worse local precision, rebasing still needed for planning); per-craft floating origin (game-engine pattern, unnecessary given body-centred frames already solve it).

## R7 — Force model assembly

- **Decision**: Per craft per substep, acceleration = Σ point-mass gravity over the **gravitating set** (clarified rule: catalogue-flagged bodies + the dominant body always) + J2 of the dominant body (data-flagged) + SRP (cannonball model, area/mass from the propulsion endpoint's craft properties, with cylindrical-shadow occultation by the dominant body) + drag (exponential-atmosphere model below the body's interface altitude: ρ(h)=ρ₀·exp(−h/H), data per body) + thrust (R9). Each term independently enableable per config (FR-ASTRO-102 testability).
- **Rationale**: Exactly the "perturbations that matter" list from design 04 §1.1, each in its standard textbook form, each sourced in data, each unit-testable in isolation.
- **Alternatives considered**: higher-order gravity harmonics, third-body indirect terms, conical shadows — all deferred; tolerance gates would not notice them at game scale, and data-driven constants let FA-03 raise fidelity later without code change.

## R8 — Two-body planning tier

- **Decision**: Planner predictions use conic propagation (universal variables) patched at SOIs: orbit from state, state at future time, transfer legs from Lambert, encounters as hyperbolic two-body within SOI. Every prediction carries a regime tag (well-conditioned vs low-confidence: multi-body regions, near-boundary legs) per FR-ASTRO-402.
- **Rationale**: Classical patched conics — instant, well-understood error behaviour, honest tagging where it lies.

## R9 — Propulsion interface

- **Decision**: `PropulsionEndpoint` trait (defined in `sojourn-astro`, implemented by FA-04 later; fixture impls now): `total_mass()`, `dry_mass()`, `propellant()`, `exhaust_velocity()`, `max_thrust()`, `throttle()`, `available_power()`, `drag_area()`, `srp_area()`, plus `consume(propellant_kg)` invoked by the burn executor. Thrust each substep = min(max_thrust × throttle, power-limited thrust where the endpoint declares power-limited mode); mass flow = thrust / v_e. Fixture engines: a hydrolox-class impulsive stand-in (Isp 450 s class) and an ion-class stand-in (Isp 3000 s class, power-limited) with sourced parameter values (real-engine textbook figures).
- **Rationale**: Exactly the spec's interface list; mass-flow coupling enforces "no thrust without mass flow" at the lowest level.

## R10 — Execution error & TCM

- **Decision**: Per-burn error drawn from the module's declared stream (`astro/exec-error`): magnitude factor ~ N(1, σ_mag) and pointing cone ~ N(0, σ_point), Box–Muller via libm, σ values from `data/astro/config.ron` (sourced from typical bi-prop execution dispersions), zero when disabled. TCM solver: differential-correction targeting — finite-difference sensitivity of the aim-point miss to a small burn at the chosen epoch, solved for the correction Δv (bounded iterations, deterministic).
- **Rationale**: Standard dispersion modelling; the TCM targeter is the minimal honest version of how real navigation teams do it.

## R11 — Kernel amendment: typed module commands

- **Decision**: Add to `sojourn-core` (additive, domain-agnostic): `Command::ModulePayload { module: String, kind: String, payload: Vec<u8> }`; routing: kernel validates the module exists, then calls new trait hook `SimModule::on_command(&self, slice, kind, payload, ctx) -> CommandOutcome` at command-application time (step 1 of the tick order; outcome journaled like any command). Astro defines its command structs (CreateNode, EditNode, DeleteNode, CommitPlan, SetThrottle/GuidanceArc, ScheduleStationKeeping, DivertBody, ReRailBody, SetResearchGate) serialized with postcard into the payload; malformed payloads are deterministic `Rejected` outcomes.
- **Rationale**: FA-01's `ModuleCommand{key, value: i64}` cannot express vector-valued domain commands; baking astro types into the kernel enum would violate FR-CORE-505. An opaque-payload variant keeps the kernel domain-free, the journal complete, and replay exact. The existing `ModuleCommand` stays (synthetic module uses it).
- **Alternatives considered**: stringly-typed key/value multiplexing (unauditable, error-prone); per-module command enums registered with the kernel via generics (object-safety/serde complexity far beyond the need).

## R12 — Defaults recorded (Outstanding items from clarify)

- **Craft–craft interaction**: craft exert no gravity on anything and do not collide with each other in this slice; rendezvous/proximity arrives with later slices. Recorded as data-model invariants.
- **Diversion budget**: default 16 simultaneously diverted bodies (`data/astro/config.ron`, tunable); exceeding it is a deterministic command rejection.
- **Atmosphere interface altitude**: per-body data field; crossing it while not flying a planned aero pass raises `atmosphere-entry` (handoff event per the EDL boundary clarification).

## R13 — Validation cases & sources

- **Decision**: `data/astro/validation.ron` defines the CI cases with tolerances and sources: (1) two-body circular/elliptic periods vs Kepler's third law (Curtis, *Orbital Mechanics for Engineering Students*, eq. 2.83) — 0.01%; (2) Hohmann LEO→GEO Δv vs analytic (Curtis ch. 6 worked example ≈ 3.935 km/s total) — 0.1% impulsive; (3) J2 nodal regression for an 800 km / 51.6° orbit vs the standard secular-rate formula (Vallado, *Fundamentals of Astrodynamics and Applications*, eq. 9-37) — 0.5%; (4) hyperbolic flyby turn angle δ = 2·asin(1/e) for a documented v∞/periapsis case (Curtis ch. 8 interplanetary example) — 0.5%; (5) low-thrust circular-to-circular spiral Δv ≈ v₁−v₂ (Edelbaum) — 5%; (6) synodic recurrence of porkchop minima vs 1/(1/T₁−1/T₂) — 2%. Test-catalogue body constants cite the textbook/standard values they are modelled on (e.g. Earth-like μ = 3.986004418×10¹⁴ m³/s², IERS Conventions 2010).
- **Rationale**: Each case is classical, closed-form, and cited — the constitution's physics-validation gate made concrete.

## R14 — Performance approach

- **Decision**: Rail-state caching per (body, time) inside a step; gravitating-set precomputed per dominant-body region; coast tier dominates wall time (60 s steps ⇒ ~0.5 M force evaluations per craft-century at coast, trivial); porkchop grids bounded (≤40×40 per query, parallel-free, <100 ms budget verified by bench); planner queries operate on cheap snapshots (copy of craft state + catalogue handle), never on live module borrows.
- **Rationale**: Meets SC-007 with margin on the same single-threaded discipline as the kernel.
