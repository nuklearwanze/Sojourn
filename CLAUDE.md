# Sojourn Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-06-13

## Active Technologies
- Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01). + `sojourn-core` (kernel contracts), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `slotmap` (craft/node stores), `ron` (data fixtures), `thiserror`. **No new third-party math/physics dependencies**: vectors, Kepler solver, Lambert solver and integrator are in-crate (research R1–R5). Harness gains astro scenarios; no UI anywhere. (003-astrodynamics)
- Data files only — `data/astro/` (test catalogue, fixture engines, config: step tiers, thresholds, diversion budget, error model) and `data/astro/validation.ron` (analytic cases + tolerances), all with `source` fields, validated in CI. (003-astrodynamics)

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
- 003-astrodynamics: Added Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01). + `sojourn-core` (kernel contracts), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `slotmap` (craft/node stores), `ron` (data fixtures), `thiserror`. **No new third-party math/physics dependencies**: vectors, Kepler solver, Lambert solver and integrator are in-crate (research R1–R5). Harness gains astro scenarios; no UI anywhere.

- 002-sim-core: Added Rust, stable toolchain pinned via `rust-toolchain.toml` (1.88 or later at adoption; pin exact version), edition 2024. No nightly features.

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
