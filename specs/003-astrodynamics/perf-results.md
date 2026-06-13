# Performance Results: Astrodynamics & Flight (FA-02)

Tracks SC-007 (≥1 sim-year/wall-minute under the full synthetic load; planner queries
< 100 ms).

## Sanity measurements (development machine, release build)

| Measurement | Result | Target |
|-------------|--------|--------|
| 1 simulated year, 1 coasting craft, full perturbations (third-body, J2, SRP, drag rules) | ~2.0 s wall ⇒ ~30 sim-yr/min | ≥ 1 sim-yr/min at full load |
| Porkchop 64×12 grid (768 Lambert solves, synodic suite) | well under 1 s within a debug test run | < 100 ms/grid (release, ≤40×40) |
| Hohmann finite-burn day + encounter-tier passes (test suite) | seconds-scale per scenario | — |

Craft integration cost is linear in craft count; extrapolation puts 200 craft at roughly
3–8 sim-yr/min depending on tier mix — above the floor with margin, but **not yet a
formal measurement**.

## Status

- **Formal SC-007 capture pending (T048)**: requires the synthetic 3,000-body catalogue
  generator + 200-craft scenario and the criterion bench group run on the reference
  machine (high-end consumer desktop). To be recorded here when taken, alongside the
  FA-01 outstanding items (specs/002-sim-core/perf-results.md).
- Planner-query latency budgets to be asserted in the same bench group.
