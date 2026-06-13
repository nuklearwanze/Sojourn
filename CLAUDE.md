# Sojourn Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-06-13

## Active Technologies
- Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01). + `sojourn-core` (kernel contracts), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `slotmap` (craft/node stores), `ron` (data fixtures), `thiserror`. **No new third-party math/physics dependencies**: vectors, Kepler solver, Lambert solver and integrator are in-crate (research R1–R5). Harness gains astro scenarios; no UI anywhere. (003-astrodynamics)
- Data files only — `data/astro/` (test catalogue, fixture engines, config: step tiers, thresholds, diversion budget, error model) and `data/astro/validation.ron` (analytic cases + tolerances), all with `source` fields, validated in CI. (003-astrodynamics)
- Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/FA-02). + `sojourn-core` (kernel contracts), `sojourn-astro` (`BodyId`, `BodyDef`, `Catalog`, `Elements`, frames, L-point solver, `state_at`), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`. The offline build tool reuses `serde_json` + `ron` only (no network deps in the workspace; see R2). **No new third-party math/statistics dependencies**: sampling, the Gaussian belief update and distribution draws are in-crate on the kernel's seeded streams. (004-world-data)
- Data files only — `data/world/` (catalogue split by class, sites, locations, prospecting fields, priors/floors/noise params, astrobiology distributions, resource taxonomy, Sojournal entries, committed reference ephemerides) all carrying `source` provenance and validated in CI; raw developer-fetched inputs under `data/world/sources/` feed the build tool. (004-world-data)
- Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/02/03). + `sojourn-core` (kernel contracts), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`, `rand_core` (the kernel stream trait). **No astro/world dependency** (research is independent of physics and world data). No new third-party math/stats deps: the UL curves, S-curve progress, P50/P80 draws, insight pressure and seeded outcomes are in-crate on the kernel's seeded streams. (005-research-personnel)
- Data files only — `data/research/` (domains + synergy, RP/DE generation, overrun/breakthrough/tide/reliability params, trait definitions, dead-end-seeding params) and `data/tech/` (the web-shaped tech-tree: A1–A17 domains + a representative sourced engineering-node subset + capability-category → candidate-path map), all carrying `source` provenance and validated in CI. (005-research-personnel)
- Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/02/03/05). + `sojourn-core` (kernel contracts), `sojourn-astro` (`PropulsionEndpoint`/`EngineDef` shape consumed by the propagator; body μ/radius for T/W), `sojourn-research` (the `maturity()`/`heritage()`/`understanding()` query contract), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`. No new third-party physics deps: the rocket equation, power/thermal balances, reliability-block-diagram and learning curve are in-crate. FA-06 will depend on `sojourn-vehicle` (cost estimate). (006-vehicle-designer)
- Data files only — `data/vehicle/` (component catalogue, propulsion family models, reliability/cost/life-support/EDL/solar-distance params, vehicle-class templates), all with `source` provenance and validated in CI. (006-vehicle-designer)

- Rust, stable toolchain pinned via `rust-toolchain.toml` (1.88 or later at adoption; pin exact version), edition 2024. No nightly features. (002-sim-core)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test; cargo clippy

## Code Style

Rust, stable toolchain pinned via `rust-toolchain.toml` (1.88 or later at adoption; pin exact version), edition 2024. No nightly features.: Follow standard conventions

## Recent Changes
- 006-vehicle-designer: Added Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/02/03/05). + `sojourn-core` (kernel contracts), `sojourn-astro` (`PropulsionEndpoint`/`EngineDef` shape consumed by the propagator; body μ/radius for T/W), `sojourn-research` (the `maturity()`/`heritage()`/`understanding()` query contract), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`. No new third-party physics deps: the rocket equation, power/thermal balances, reliability-block-diagram and learning curve are in-crate. FA-06 will depend on `sojourn-vehicle` (cost estimate).
- 005-research-personnel: Added Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/02/03). + `sojourn-core` (kernel contracts), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`, `rand_core` (the kernel stream trait). **No astro/world dependency** (research is independent of physics and world data). No new third-party math/stats deps: the UL curves, S-curve progress, P50/P80 draws, insight pressure and seeded outcomes are in-crate on the kernel's seeded streams.
- 004-world-data: Added Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/FA-02). + `sojourn-core` (kernel contracts), `sojourn-astro` (`BodyId`, `BodyDef`, `Catalog`, `Elements`, frames, L-point solver, `state_at`), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`. The offline build tool reuses `serde_json` + `ron` only (no network deps in the workspace; see R2). **No new third-party math/statistics dependencies**: sampling, the Gaussian belief update and distribution draws are in-crate on the kernel's seeded streams.


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
