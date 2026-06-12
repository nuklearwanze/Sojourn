# Performance Results: Simulation Core & Time (FA-01)

Tracks SC-006 (≥ 1 simulated year per wall-clock minute under full synthetic load;
kernel overhead ≤ 20% of tick budget) and SC-007 (500 watch conditions within budget).

## Throughput sanity (development machine)

| Measurement | Result | Target (SC-006) |
|-------------|--------|-----------------|
| 10 simulated years under full synthetic load (3,200 entity stand-ins, ~10k events/sim-year, hourly cadence) | ~26 s wall → **≈23 sim-years / wall-minute** | ≥ 1 sim-year / wall-minute |

The kernel exceeds the throughput floor by more than an order of magnitude on a
development machine, so a century completes well under the umbrella's 2.5-hour
budget. The step path performs no per-tick heap allocation in steady state
(quiet spans are jumped, not iterated tick-by-tick).

## Status

- **Throughput floor (SC-006): met** by a wide margin in dev-machine sanity runs.
- **Formal reference-machine numbers** (high-end consumer desktop: 8+ perf cores,
  discrete GPU, 32 GB RAM) and the criterion micro-budgets (`benches/kernel.rs`:
  step-loop, scheduler, journal, fingerprint) are to be captured on the reference
  hardware and recorded here before the FA-01 milestone is closed.
- **Kernel-overhead ≤ 20%** and **SC-007 (500 watch conditions)**: bench harness
  in place (`benches/kernel.rs`); formal capture pending reference hardware.

This file is updated by task T068 when reference-machine measurements are taken.
