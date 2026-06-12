# Quickstart: Simulation Core & Time (FA-01)

Developer on-ramp for the `sojourn-core` + `sojourn-harness` workspace slice.

## Prerequisites

- Rust via `rustup` — the workspace pins its toolchain in `rust-toolchain.toml` (auto-installed
  on first `cargo` invocation). No nightly, no other tools required.
- OS: Windows, Linux or macOS. Remember: determinism guarantees hold **per platform + build** —
  never compare fingerprints across machines of different platforms or builds.

## Build & test

```powershell
cargo build --workspace                 # builds kernel + harness, no UI deps anywhere
cargo test  --workspace                 # unit + integration tests (incl. contract conformance)
cargo fmt --all -- --check              # formatting gate
cargo clippy --workspace -- -D warnings # includes determinism lints (disallowed types/methods)
```

## The determinism suite (what CI runs)

```powershell
# Double-run: same seed+script twice, varied stepping patterns, hashes + event logs must match
cargo run -p sojourn-harness -- verify scenarios/smoke_decade.ron

# Save → load → continue equals never-saved run
cargo run -p sojourn-harness -- roundtrip scenarios/smoke_decade.ron --save-at-ticks 1000,50000

# Journal replay reconstructs identical history (and verifies recorded events/hashes)
cargo run -p sojourn-harness -- replay   runs/<run-id>/journal.sjl --verify

# Crash recovery: spawn child, kill at random points, verify recovered state + bounded loss
cargo run -p sojourn-harness -- killtest scenarios/smoke_decade.ron --trials 100

# Prove the gate has teeth: nondeterminism-injected builds MUST fail verification
cargo run -p sojourn-harness -- mutate --all

# Validate kernel data files (event classes, watch templates, config) against strict schemas
cargo run -p sojourn-harness -- validate-data data/kernel/
```

## Run a scenario interactively (headless)

```powershell
cargo run -p sojourn-harness -- run scenarios/smoke_decade.ron --until-interrupt --print-status
# steps until the next interrupt-and-pause, prints tick/date/pending interrupts/fingerprint
```

Scenario scripts are RON files: seed, run config, tick-stamped commands, checkpoints, optional
golden fingerprints (see `contracts/persistence-format.md` §4, `data-model.md` §10). They double
as bug-report reproductions: seed + journal = exact repro on the originating build.

## Benchmarks (performance budgets)

```powershell
cargo bench -p sojourn-harness          # criterion: step loop, scheduler, journal, hashing
# Budget assertions (SC-006): ≥1 sim-year/min under synthetic full-game load on the reference
# machine; kernel overhead ≤20% of tick budget; no steady-state per-tick allocation
# (checked with an allocation-counting test allocator in debug runs).
```

## Rules you will hit (by design)

| You did | What happens | Why |
|---|---|---|
| Used `HashMap` in sim logic | clippy gate fails | hash-order iteration is nondeterministic (R8) |
| Read `Instant::now()`/`SystemTime` in core | clippy gate fails | wall-clock is forbidden in sim logic (FR-CORE-105) |
| Drew randomness without a declared stream | loud runtime defect | streams are named & declared (FR-CORE-202) |
| Mutated another module's slice | structural rejection | single-writer ownership (FR-CORE-502) |
| Called `f64::sin` in a sim crate | clippy gate fails | use `libm` — fixed transcendental path (R7) |
| Added a UI/Tauri dependency to core | CI dependency audit fails | the core never knows presentation exists (FR-CORE-702) |

## Where things are

| Artifact | Path |
|---|---|
| Spec / plan / research / data model | `specs/002-sim-core/` |
| API & module & persistence contracts | `specs/002-sim-core/contracts/` |
| Kernel crate | `crates/sojourn-core/` |
| Harness crate (CLI + suite + benches) | `crates/sojourn-harness/` |
| Kernel data files | `data/kernel/` (event classes, watch templates, config) |
| Scenario fixtures | `scenarios/` |
| CI workflow | `.github/workflows/ci.yml` |

## What NOT to build here

No UI, no Tauri, no rendering, no astrodynamics math, no game content. Those arrive as sibling
crates implementing `contracts/module-contract.md` against this kernel. If your change makes the
core aware of presentation or game domains, it belongs in a different slice.
