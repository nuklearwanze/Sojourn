# Quickstart: Astrodynamics & Flight (FA-02)

Developer on-ramp for `sojourn-astro` — the first game-system module on the kernel.

## Build & test

```powershell
cargo build --workspace
cargo test -p sojourn-astro                 # unit + integration incl. the analytic validation suite
cargo test --workspace                      # everything (kernel suites still green)
cargo clippy --workspace --all-targets -- -D warnings   # determinism lints apply here too
```

## The physics gates (what CI runs for this slice)

```powershell
# Analytic validation suite (Hohmann, periods, J2 regression, flyby, spiral, synodic):
cargo test -p sojourn-astro --test validation

# Kernel conformance for the astro module (double-run, serde round-trip, cadence):
cargo run -p sojourn-harness -- conformance --module astro

# Determinism gates with astro scenarios (burns, flybys, exec error):
cargo run -p sojourn-harness -- verify scenarios/astro_transfer.ron
cargo run -p sojourn-harness -- roundtrip scenarios/astro_transfer.ron --save-at-ticks <mid-burn tick>
cargo run -p sojourn-harness -- replay scenarios/astro_transfer.ron --journal runs/<id>/journal.sjl --verify

# Data validation (catalogue, engines, config, validation cases — source fields enforced):
cargo run -p sojourn-harness -- validate-data data/astro
```

## Driving the module headlessly

Astro scenarios extend the RON scenario format: registering the astro module, spawning fixture
craft, and issuing astro commands through the kernel's `ModulePayload` envelope. Planning
queries are plain function calls on a snapshot — see `contracts/planning-queries.md`:

```rust
let snap = AstroModule::snapshot(&slice, &catalog, tick);
let grid = planner::porkchop(&snap, &query);          // pure; never journaled
let nodes = planner::cell_to_plan(&snap, &query, best_cell, craft);
// committing the plan is a command (journaled):
core.submit(astro_command("commit-plan", &CommitPlan { … }))?;
```

## Rules you will hit (in addition to the kernel's)

| You did | What happens | Why |
|---|---|---|
| Added a body constant in code | review rejection | constants live in `data/astro/*.ron` with sources (Principle II) |
| Made step size depend on warp/host pacing | divergence in `verify` | tiering is state-driven only (FR-CORE-204) |
| Let a planner query mutate or draw randomness | purity test fails | queries are pure (FR-ASTRO-409) |
| Applied thrust without `consume()` | conservation test fails | no thrust without mass flow (SC-005) |
| Returned an unsolvable Lambert cell as numbers | porkchop suite fails | honesty: unsolvable cells are explicit |

## Where things are

| Artifact | Path |
|---|---|
| Spec / plan / research / data model / contracts | `specs/003-astrodynamics/` |
| Module crate | `crates/sojourn-astro/` |
| Astro data (catalogue, engines, config, validation) | `data/astro/` |
| New event classes | `data/kernel/event-classes.ron` |
| Astro scenarios | `scenarios/astro_*.ron` |
| Kernel amendment | `crates/sojourn-core` (`Command::ModulePayload`, `SimModule::on_command`) + FA-01 contract docs |
