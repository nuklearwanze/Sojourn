# Implementation Plan: Simulation Core & Time (FA-01)

**Branch**: `002-sim-core` | **Date**: 2026-06-12 | **Spec**: `specs/002-sim-core/spec.md`
**Input**: Feature specification from `/specs/002-sim-core/spec.md`

## Summary

Build the deterministic, headless simulation kernel for Sojourn as a pure Rust library crate
(`sojourn-core`) plus a headless test/CLI harness crate (`sojourn-harness`) in a Cargo workspace,
with no UI, Tauri, webview or rendering dependency anywhere in the core. The kernel delivers:
an integer-tick fixed-timestep loop over one authoritative world state; a seeded, splittable PRNG
with named hierarchical sub-streams; a kernel-managed multi-rate scheduler with event-driven
time-warp and interrupt-and-pause (event classes + data-driven, composable watch conditions);
an append-only, integrity-checked, replayable command/event journal with crash recovery; exact
serde-based versioned save/load that pins the content-data version; the module contract
(ownership, views, cadence, streams, deterministic ordering) all later slices build on; and a
public API boundary designed so a future Tauri host drives it in-process or over serialized IPC.
Quality gates ship in CI: double-run determinism (state hash + event log identity, varied
stepping patterns), save→load→continue identity, journal replay, kill-test recovery, and
nondeterminism mutation tests that must fail the gate.

## Technical Context

**Language/Version**: Rust, stable toolchain pinned via `rust-toolchain.toml` (1.88 or later at adoption; pin exact version), edition 2024. No nightly features.
**Primary Dependencies** (all MIT/Apache-2.0-compatible; see research.md R1–R12):
- `sojourn-core`: `serde` (+derive), `postcard` (canonical binary codec), `rand_core` + `rand_chacha` (seeded ChaCha12 streams), `blake3` (fingerprints, stream derivation, integrity), `slotmap` (generational arenas / stable handles), `libm` (fixed transcendental path policy), `thiserror` (errors). **Forbidden in core**: `rand::thread_rng`, `std::time` reads in sim logic, `HashMap`/`HashSet` iteration in sim logic (clippy `disallowed-types`/`disallowed-methods` enforced), any Tauri/webview/rendering crate, result-affecting parallelism.
- `sojourn-harness`: `clap` (CLI), `ron` (scenario scripts), `serde_json` (interchange), `criterion` (benches, dev), `anyhow` (CLI-level errors).
**Storage**: Files only — versioned binary saves, append-only framed journal, RON scenario scripts, RON/JSON kernel data files (event-class registry, watch-condition template catalogue, kernel config). No database, no network.
**Testing**: `cargo test` (unit + integration), harness binary as CI determinism suite (double-run, round-trip, replay, kill-test, mutation gate), `cargo clippy` with determinism lint config, `cargo fmt --check`, `criterion` benches with tracked budgets.
**Target Platform**: Desktop Windows/Linux/macOS (x86-64 + ARM64). Determinism guarantee is per platform+build (umbrella clarification); the dependency choices (ChaCha, postcard, blake3, libm, integer clock) make cross-platform identity *likely* but it is not promised or tested as a gate.
**Project Type**: Library crate + headless CLI harness in a Cargo workspace (future `sojourn-app` Tauri crate and per-slice module crates will join the workspace).
**Performance Goals**: SC-006: ≥ 1 simulated year per wall-clock minute under the synthetic full-game load profile (3,000+ propagated-entity stand-ins, ~200 active craft stand-ins, ~10k events/sim-year, 500 watch conditions) on the reference machine; kernel overhead (scheduling, eventing, journaling, hashing cadence) ≤ 20% of tick budget; step path allocation-light (no per-tick heap allocation in steady state); large-scale tuning beyond the synthetic profile deferred to content slices.
**Constraints**: Determinism is load-bearing — integer tick clock (no float accumulation); single master seed → named sub-streams; ordered/indexed iteration only in sim logic; no wall-clock, no unseeded entropy, no result-affecting parallelism; no fast-math flags; libm-only transcendentals in core; warp is pure playback speed (never journaled); queries answered only between step calls at tick boundaries; journal durability bound ≤ 5 s wall-clock loss; fully offline; saves pin content-data version.
**Scale/Scope**: Century runs (~3.16×10⁹ s simulated) with zero time drift; full event history queryable (disk-backed tiering allowed); journal + saves for 100-year campaigns; this slice ships kernel + harness only (no game content, no UI).

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1 design.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Scientific Plausibility / sourced data | PASS (n/a-leaning) | Kernel contains no plausibility-bearing values. Kernel data files (event classes, condition templates, config) carry `source` fields where values are plausibility-bearing (none expected in this slice); the data-version identity + validation hooks this slice ships are what enable CI source enforcement for content slices. |
| II. Physics is authoritative / no magic numbers in engine | PASS | No physics in this slice by design. The kernel reads all tunables (tick size, cadences, autosave/flush cadence) from explicit config data, never hard-coded constants — establishing the pattern Principle II requires of later slices. |
| III. Deterministic, reproducible core | PASS (this slice *is* the enforcement) | Integer tick clock; seeded ChaCha sub-streams threaded explicitly; ordered iteration enforced by clippy config; no wall-clock in sim logic; warp invariance (pure playback speed); double-run + replay + mutation CI gates; event-log replay reconstructs state. |
| IV. Core decoupled from presentation | PASS | `sojourn-core` has zero UI/Tauri/render dependencies (enforced by dependency audit in CI); public API is serializable-DTO based so it works in-process or over IPC; everything testable headlessly via `sojourn-harness`. |
| V. Data-driven content, code-driven mechanics | PASS | Event-class registry, watch-condition template catalogue and kernel config live in versioned, schema-validated (strict serde + harness `validate-data`) data files; mechanics in code. |
| VI–VIII. Research model / mass & delta-v / educational honesty | N/A | No research, economy or encyclopedia content in this slice; module contract leaves their state ownership to their slices. |
| IX. No combat/aliens | PASS | Kernel is domain-agnostic; no reserved-feature logic. |
| Engineering constraints (SI, determinism tooling, saves, performance, accessibility) | PASS | SI units (time in integer seconds/nanoseconds); fixed-timestep + seeded PRNG per constraint; round-trip-tested versioned migratable saves; explicit tick-time budgets tracked by criterion benches in CI; accessibility n/a (no UI). |
| Workflow gates (testing required) | PASS | Determinism double-run test, save/load round-trip test, headless integration tests, and data schema validation are all CI gates delivered by this slice. Physics-validation cases attach when FA-02 lands (hook provided in harness design). |

**Post-Phase-1 re-check (2026-06-12)**: design artifacts (data-model.md, contracts/) introduce no
violations — no UI coupling appeared in the API contract, no unsourced plausibility data, no
nondeterministic structures. Gate remains PASS.

## Project Structure

### Documentation (this feature)

```text
specs/002-sim-core/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── core-api.md      # Public boundary: operations + DTOs (the Tauri/IPC seam)
│   ├── module-contract.md  # SimModule trait, manifest, kernel context, conformance
│   └── persistence-format.md  # Save container, journal framing, versioning/migration
└── tasks.md             # Phase 2 output (/speckit-tasks — NOT created by /speckit-plan)
```

### Source Code (repository root)

```text
Cargo.toml                    # workspace root (members: crates/*)
rust-toolchain.toml           # pinned stable toolchain
clippy.toml                   # disallowed-types/methods (determinism lints)
.github/workflows/ci.yml      # fmt, clippy, test, determinism suite, benches, data validation

crates/
├── sojourn-core/             # THE KERNEL — pure library, no UI/Tauri/render deps
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # public API facade (re-exports the boundary, nothing else)
│       ├── api/              # SimCore handle: create/step/query/command/save/load/replay
│       ├── clock/            # integer tick clock, civil calendar (2026–2126), horizon
│       ├── rng/              # master seed, named hierarchical sub-streams (ChaCha12 + blake3 derivation)
│       ├── state/            # WorldState, StateSlice trait, slotmap arenas, stable handles
│       ├── module/           # ModuleManifest, registry, dependency ordering, single-writer enforcement
│       ├── sched/            # kernel-managed cadences, fine-step escalation, due-event advance
│       ├── command/          # command envelopes, validation, application, rejection outcomes
│       ├── event/            # event records, classes, pause policies, deterministic queue, history store
│       ├── watch/            # condition template catalogue, instances, AND/OR composition, evaluation
│       ├── journal/          # append-only framed log, integrity, durability, replay
│       ├── save/             # versioned save container, content-data pinning, migration
│       ├── hash/             # canonical state fingerprint (blake3 over canonical encoding), diff mode
│       ├── data/             # kernel data-file loading/validation hooks, DataVersionId
│       └── error.rs          # thiserror error taxonomy
│   └── tests/                # core integration tests (contract conformance, edge cases)
│
└── sojourn-harness/          # HEADLESS HARNESS — CLI bin + test scaffolding, no UI
    ├── Cargo.toml
    └── src/
        ├── main.rs           # clap CLI: run|verify|replay|hash|killtest|bench|validate-data
        ├── scenario.rs       # RON scenario scripts (seed, config, command script, checkpoints)
        ├── synthetic/        # synthetic load modules (entity churn, event storms, watch load)
        ├── doublerun.rs      # double-run determinism check (varied stepping patterns)
        ├── roundtrip.rs      # save→load→continue identity check
        ├── replay.rs         # journal replay fidelity check
        ├── killtest.rs       # subprocess kill + recovery verification
        └── mutation/         # injected-nondeterminism builds that MUST fail the gate
    └── benches/              # criterion: step-loop, scheduler, journal, hashing budgets

# Future workspace members (NOT this slice): crates/sojourn-app (Tauri host),
# crates/sojourn-astro, crates/sojourn-world, … (SimModule implementors per slice)
```

**Structure Decision**: Cargo workspace with two members in this slice. `sojourn-core` is the
authoritative seam between simulation and presentation: its public API (contracts/core-api.md)
uses only owned, serde-serializable types so a future `sojourn-app` (Tauri) can call it
in-process today and the identical surface can be bridged over serialized IPC without redesign.
`sojourn-harness` is both the CLI developers use and the body of the CI determinism suite.
Game-domain slices will join as sibling crates implementing the `SimModule` contract — the
workspace layout makes the kernel's no-game-logic rule (FR-CORE-505) a crate boundary, not a
convention.

## Complexity Tracking

No constitution violations to justify — table intentionally empty.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
