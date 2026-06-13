# Verification Traceability: Astrodynamics & Flight (FA-02)

Maps each success criterion to the proving tests/gates. All listed checks pass on the
reference build (Rust 1.96.0). 51 astro tests + all FA-01 kernel gates green.

| SC | Criterion | Proven by |
|----|-----------|-----------|
| SC-001 | Analytic fidelity | `tests/validation.rs`: two-body period (0.01%), Hohmann Δv (0.1%), J2 regression (0.5%), flyby turn angle (exact formula + flown half-turn ≤5%), Lambert self-consistency (1e-6) — tolerances from sourced `data/astro/validation.ron` |
| SC-002 | Conservation honesty | `validation.rs::energy_drift_within_bound` (≤1e-8/yr annualized); `propagation.rs::low_thrust_energy_equals_work` (ΔE = thrust work ≤5%); integrator unit tests |
| SC-003 | Determinism | `conformance.rs` (kernel suite + fuller double-run with burns/SOI); harness `verify` on astro_coast/transfer/lowthrust; `roundtrip` with mid-burn save; `divert.rs::diversion_is_deterministic` |
| SC-004 | Planner honesty | `maneuvers.rs::lowthrust_spiral` (≤5% Edelbaum); `planner.rs::flyby_flown` (≤5% + exact formula); regime tags asserted in `planner.rs::lowenergy` (LowConfidence); unsolvable Lambert cells explicit (`lambert.rs` unit + porkchop honesty) |
| SC-005 | Felt constraints | `maneuvers.rs::hohmann_flown` (propellant vs rocket equation ≤0.1%); `propagation.rs` (mass flow = T·t/ve ≤0.1%; no thrust without flow); `maneuvers.rs::unaffordable_plan` (infeasibility flag + exhaustion + invalidation); `power_limited_thrust_scales` |
| SC-006 | Windows emerge | `validation.rs::synodic_recurrence_emerges` (minima recur within 2% of analytic synodic period; nothing scripted) |
| SC-007 | Performance | Sanity: 1 sim-year ≈ 2.0 s wall (1 craft, full perturbations, release) ⇒ ~30 sim-yr/min; formal full-load bench (3,000-body catalogue + 200 craft) pending T048 on reference hardware (perf-results.md) |
| SC-008 | Verb coverage | node/chain (maneuvers, astro_transfer.ron), porkchop (validation + planner + divert), flyby+chain (planner + chain unit), low-thrust arc (maneuvers + astro_lowthrust.ron), low-energy (planner gating + dwell), aero corridor (planner flown triad), TCM (maneuvers), station-keeping (maneuvers L1), divert/re-rail (divert.rs) — every verb exercised headlessly |

## FR spot-notes

- FR-ASTRO-104 (no reactionless motion): structurally enforced (thrust ⇒ `consume`,
  spawn = journaled command); conservation tests catch regressions.
- FR-ASTRO-107: full lifecycle tested incl. budget/eligibility rejections and
  planning-target equivalence (`tests/divert.rs`).
- FR-ASTRO-409 purity: queries are pure functions over owned `AstroSnapshot` clones —
  mutation is structurally impossible (no `&mut` anywhere on the surface); extraction
  uses the kernel's read-only `with_slice`.
- Kernel amendment: `ModulePayload`/`on_command`/`with_slice` are additive; **all 38
  FA-01 kernel tests and harness gates still pass unmodified**.

## Known deferrals

- T048/SC-007 formal: synthetic 3,000-body catalogue + 200-craft bench on reference
  hardware (the harness bench group exists; the big catalogue generator comes with it).
- Automated assist-sequence search: deferred by clarification (chain representation is
  the contract).
- Surface EDL: split to the post-FA-04 slice by clarification.
