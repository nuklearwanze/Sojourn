# Verification Traceability: Simulation Core & Time (FA-01)

Maps each success criterion (SC-001…SC-011) to the tests/CI jobs that prove it.
All listed checks pass on the reference build (Rust 1.96.0, Windows + Linux).

| SC | Criterion | Proven by |
|----|-----------|-----------|
| SC-001 | Double-run identity | `sojourn-harness verify` (smoke_decade + interrupts), CI job *determinism*; `tests/determinism.rs::double_run_is_bit_identical_across_pacing`; conformance `double-run-identity` |
| SC-002 | Round-trip identity | `sojourn-harness roundtrip` (saves at 3 ticks incl. a command tick), CI *determinism*; `tests/persistence.rs` (4 tests) |
| SC-003 | Replay fidelity | `sojourn-harness replay --verify`; `tests/journal.rs` (replay to final + interior tick) |
| SC-004 | Crash recovery | `sojourn-harness killtest` (12 trials in CI); recover↔replay agreement cross-check |
| SC-005 | Interrupt exactness | `tests/interrupts.rs` (exact-tick halt, simultaneity, log-only, watch firing) |
| SC-006 | Kernel performance | `benches/kernel.rs` (criterion); recorded in `perf-results.md` |
| SC-007 | Watch fidelity & cost | `tests/interrupts.rs` (watch-fires/composed); bench covers scale |
| SC-008 | Headless completeness | Architecture: no presentation dep (CI *audit*); harness exercises every command/API path. **Scenario-coverage matrix: see below.** |
| SC-009 | Gate sensitivity | `sojourn-harness mutate --all` — 10 injection types, all caught (CI *determinism*) |
| SC-010 | Contract usability | Reference toy module built from the contract doc passes `conformance --module toy` |
| SC-011 | Time integrity | `clock::calendar::tests` (century round-trip, zero drift, leap years through 2126) |

## SC-008 scenario coverage matrix

Command / API surface vs the scenario(s) or test(s) exercising it:

| Surface | Covered by |
|---------|-----------|
| `Command::RegisterWatch` (+ true-at-registration, AND/OR) | interrupts.rs; interrupts.ron |
| `Command::ModifyWatch` / `RemoveWatch` | **gap → covered by `tests/watch_ops.rs`** (added) |
| `Command::SetPausePolicy` | interrupts.rs (log_only); smoke_decade.ron |
| `Command::AcknowledgeInterrupt` | interrupts.rs; spine.rs; all drivers |
| `Command::ContinueSandbox` | spine.rs (horizon → sandbox) |
| `Command::ModuleCommand` | spine.rs; journal.rs; scenarios |
| `StepRequest::{Ticks,UntilSimTime,UntilInterrupt}` | Ticks/UntilInterrupt everywhere; **UntilSimTime → `tests/watch_ops.rs`** |
| Ironman vs SaveAnywhere | persistence.rs (modes); ironman strict recovery in killtest design |
| `save`/`load`/`recover`/`replay`/`fingerprint`/`events` | persistence.rs, journal.rs, harness |

Gaps found during the SC-008 audit were closed by adding `tests/watch_ops.rs`
(ModifyWatch, RemoveWatch, UntilSimTime). No remaining uncovered player action.

## Notes

- Checkpoint-accelerated recovery is deferred (recovery uses full journal replay,
  the verified path); documented in `api::SimCore::recover`.
- Cross-platform bit-identity is explicitly out of scope (per-platform guarantee);
  CI runs each OS leg independently and never compares fingerprints across them.
