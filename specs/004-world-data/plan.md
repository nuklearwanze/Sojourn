# Implementation Plan: World Data & Belief State (FA-03)

**Branch**: `004-world-data` | **Date**: 2026-06-13 | **Spec**: `specs/004-world-data/spec.md`
**Input**: Feature specification from `/specs/004-world-data/spec.md`

## Summary

Build `sojourn-world`, the module that turns FA-02's idealised five-body test fixture into the
**real Solar System** and layers the game's honesty contract on top of it. Three things ship
together: (1) the real catalogue — Sun, planets, dwarfs, ≥140 moons and ≥2,800 small bodies from
published elements, produced by an **offline data-build tool** and consumed by the FA-02
propagator *unchanged* through the existing body-catalogue contract; (2) the **truth/belief
layer** — engine-private ground truth (sourced + seeded) held strictly apart from per-faction
belief (estimate + uncertainty), refined only by journaled, seeded **observation** commands
whose Gaussian precision-addition model guarantees information never decreases and converges to
documented class floors, never to truth; (3) the supporting world structure — surveyable **Sites**
with the full property set, first-class **dynamical locations** (orbit bands, L-points, staging
orbits, surface anchors) resolvable over time through FA-02's frames, **statistical prospecting
fields** that deterministically generate permanent new small bodies, and the cited **Sojournal**
encyclopedia data. All of it reaches other slices and the future UI through a pure, read-only
**world-query surface** (`with_slice` + functions over a snapshot that *structurally cannot
contain truth*).

The slice needs **no kernel change** (module commands route through FA-02's `ModulePayload`;
new event classes are data-registry additions). It needs **two additive changes to
`sojourn-astro`**, each in the spirit of FA-02's divert capability: generalise the `Catalog`
loader to read the real multi-file catalogue (data-flag-driven divertibility instead of the
test fixture's radius heuristic), and let the propagator consume a `reads` **generated-bodies
view** the world module publishes, so prospecting products are railed and targetable like any
catalogued body. All FA-02 gates stay green (the test catalogue and an empty generated view are
the unchanged-behaviour baseline).

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/FA-02).
**Primary Dependencies**: `sojourn-core` (kernel contracts), `sojourn-astro` (`BodyId`, `BodyDef`, `Catalog`, `Elements`, frames, L-point solver, `state_at`), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`. The offline build tool reuses `serde_json` + `ron` only (no network deps in the workspace; see R2). **No new third-party math/statistics dependencies**: sampling, the Gaussian belief update and distribution draws are in-crate on the kernel's seeded streams.
**Storage**: Data files only — `data/world/` (catalogue split by class, sites, locations, prospecting fields, priors/floors/noise params, astrobiology distributions, resource taxonomy, Sojournal entries, committed reference ephemerides) all carrying `source` provenance and validated in CI; raw developer-fetched inputs under `data/world/sources/` feed the build tool.
**Testing**: `cargo test` (unit + integration: catalogue load/validation, belief refinement, the standing truth-leak audit, prospecting statistics, locations, sites, Sojournal); kernel conformance (`conformance --module world`); harness determinism gates (verify/roundtrip/replay with the real catalogue + world module installed); FA-02's full analytic suite re-run against the real catalogue; `validate-data` extended to world; criterion bench for indexed query latency and full-catalogue tick budget.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-world`) implementing the FA-01 `SimModule` contract + a public read-only world-query API; a dev-only build-tool bin crate (`crates/sojourn-worldbuild`); harness/bench/data extensions.
**Performance Goals**: SC-006 — full catalogue (≈3,000 railed bodies) + belief layer holds the FA-02 load envelope (≥1 sim-year/wall-minute on the reference machine); indexed world queries < 50 ms; catalogue load < 5 s.
**Constraints**: Full kernel determinism (ordered iteration, libm-only, declared streams for seeding + observation noise, no wall-clock); truth is engine-private and unreachable from any public query (structural, audited); belief uncertainty is monotonically non-increasing and floored per class; generated-body identities are collision-free and permanent across save/replay; SI units; offline (no runtime network); real IAU names for natural bodies, fictional commercial sector.
**Scale/Scope**: ≈3,000 catalogued bodies (+ generated additions), ≥140 moons, ~30–40 starter sites, the documented location set, the belt/NEA/Kuiper prospecting fields, Sojournal entries for every major body + class/type/concept, multiple opaque factions holding independent beliefs.

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS (this slice is the embodiment) | Every catalogue entry, site property, location definition, prospecting distribution, prior, class floor, noise sigma and Sojournal claim lives in `data/world/*` with a `source` field; `validate-data` (extended) fails CI on any missing/empty source; major-body rails are checked against committed published reference ephemerides (FR-WORLD-103). Seeded per-game truths draw from *sourced plausibility distributions*, not invented numbers. |
| II. Physics authoritative / no magic numbers | PASS | The real catalogue feeds FA-02's authoritative propagator unchanged; the planners' reconciliation guarantees carry over; no physics constants are introduced in world code — they are data the engine reads. |
| III. Deterministic core | PASS | World module conforms to the kernel contract: named streams for world-creation seeding, observation noise and prospecting draws; ordered (`BTreeMap`) stores; libm-only; belief/truth/generated bodies are slice state exercised by double-run / roundtrip / replay / conformance gates. |
| IV. Headless / decoupled | PASS | Pure library module + read-only query functions; zero UI deps; the Sojournal ships as *data* (rendering is FA-10). Everything is driven and audited through harness scenarios. |
| V. Data-driven content | PASS | Catalogue, sites, locations, prospecting models, priors/floors/refinement params, astrobiology distributions, resource taxonomy, event classes and encyclopedia are all schema-validated data; new event classes use the existing registry (no kernel code change). |
| VI. Research a modelled process | N/A (this slice) | Observation events are emitted for FA-05 to consume; the research model itself lands later. |
| VII. Tyranny of mass / Δv | N/A (this slice) | World data feeds the constraint; economics/logistics price it in FA-06. |
| VIII. Educational honesty | PASS (central) | Truth/belief separation is *structural* — the query snapshot cannot contain truth and a standing audit test enforces it over the whole public surface; the Sojournal carries citations (CI-enforced) and never states seeded per-game truths. |
| IX. No combat/aliens | PASS | Astrobiology ground truth is a seeded *scientific question* (a science object), never an actor; no weapons/combat/sabotage anywhere. |
| Engineering constraints | PASS | SI everywhere; performance budgets tracked by bench; world data version pinned in saves (extends FA-02's catalogue-hash guard); offline posture preserved (build tool is dev-only and network-free in the workspace). |
| **Astro contract amendments** | NOTED (additive) | Two additive `sojourn-astro` changes, no behavioural change to FA-02 with the test fixture: (a) the `Catalog` loader reads the real multi-file catalogue and derives divertibility from the explicit data flags (the body-catalogue contract already specifies flag-driven divertibility; the radius heuristic was a fixture-only guard); (b) the propagator consumes an optional `reads` generated-bodies view (astro's own `BodyDef` DTO) so prospecting products propagate/target like catalogued bodies. No kernel change. Contract doc `contracts/catalogue-data.md` records both. |

**Post-Phase-1 re-check (2026-06-13)**: the design artifacts introduce no new violations. Truth
remains structurally unreachable (the snapshot type carries no truth fields); the astro changes
are additive and leave FA-02 gates green; no kernel amendment is required. Gate remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/004-world-data/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R12)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── world-queries.md         # FR-WORLD-701: the read-only world-query surface (FA-10 / inter-slice seam)
│   ├── observation-commands.md  # FR-WORLD-304/502: observation + prospecting commands (via ModulePayload) + event classes
│   ├── catalogue-data.md        # FR-WORLD-101..106: real-catalogue data format, provenance, build-tool I/O, the two additive astro changes
│   └── belief-model.md          # FR-WORLD-301..304: truth/belief separation + the refinement model (estimate/uncertainty, class floors, update rule)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged this slice (new event classes are data only)
├── sojourn-astro/               # FA-02 — two ADDITIVE changes (FA-02 gates stay green):
│   └── src/bodies/mod.rs        #   • Catalog loader reads the real multi-file catalogue; flag-driven divertibility
│   └── src/module.rs            #   • propagator consumes a `reads` generated-bodies view (astro BodyDef DTO)
├── sojourn-world/               # THIS SLICE — pure library, SimModule implementor
│   ├── Cargo.toml               # deps: sojourn-core, sojourn-astro, serde, postcard, libm, ron, thiserror
│   └── src/
│       ├── lib.rs               # public surface: WorldModule, world queries, WorldCommand
│       ├── catalogue.rs         # real-catalogue indexes over astro::Catalog (by type/region/flag) for fast queries
│       ├── locations.rs         # dynamical-location catalogue + resolve_at (orbit band | L-point | staging orbit | surface anchor)
│       ├── sites.rs             # Site defs + the surveyable property set; site identity follows diverted bodies
│       ├── truth.rs             # ENGINE-PRIVATE ground-truth store + world-creation seeding (incl. astrobiology, FR-WORLD-306)
│       ├── belief.rs            # per-faction belief state; Gaussian precision-add + categorical update; class floors
│       ├── observe.rs           # observation command handling → refinement + events (trust-the-caller validation)
│       ├── prospect.rs          # prospecting fields + deterministic body generation; published generated-bodies view
│       ├── sojournal.rs         # encyclopedia data load + validation (citations, link resolution, no truth leak)
│       ├── ids.rs               # BodyId partitioning + collision-free generated-id allocator (seeded, persisted)
│       ├── query.rs             # WorldSnapshot (belief + catalogue only — NO truth) + pure read-only query fns (FR-WORLD-701)
│       └── module.rs            # SimModule impl: manifest (owned slice, streams, publishes generated-bodies), step, on_command, publish, save/load_slice
│   └── tests/                   # catalogue.rs, belief.rs, observe.rs, prospect.rs, locations.rs, sites.rs,
│                                # sojournal.rs, audit.rs (standing truth-leak audit), integration_fa02.rs, conformance.rs
├── sojourn-worldbuild/          # DEV-ONLY build tool (bin) — NOT in the shipped game's dep tree
│   ├── Cargo.toml               # deps: serde, serde_json, ron only (network-free workspace; R2)
│   └── src/main.rs              # reads data/world/sources/* → emits data/world/catalog/*.ron + provenance + snapshot date
└── sojourn-harness/             # + `world` scenario flag: point astro Catalog at data/world, install Astro+World modules; benches
data/
├── kernel/event-classes.ron     # + body-catalogued, survey-milestone (LogOnly) — data-registry addition
└── world/
    ├── catalog/
    │   ├── planets.ron          # Sun, 8 planets, Pluto + major dwarfs (sourced elements + physical data)
    │   ├── moons.ron            # ≥140 significant moons
    │   └── small-bodies.ron     # ≥2,800 asteroids/comets (build-tool output; per-entry provenance + snapshot date)
    ├── sites.ron                # ~30–40 starter sites (sourced properties + PP category)
    ├── locations.ron            # orbit bands, L1/L2 of documented pairs, staging orbits, surface anchors
    ├── prospecting-fields.ron   # belt/NEA/Kuiper population models (size-frequency, type mix, orbital dists)
    ├── priors.ron               # default priors, per-class uncertainty floors, refinement params, noise sigmas
    ├── astrobiology.ron         # candidate worlds + sourced plausibility distributions (FR-WORLD-306)
    ├── resources.ron            # sourced resource taxonomy
    ├── reference-ephemeris.ron  # committed published check values (epochs × major bodies) for FR-WORLD-103
    ├── sojournal/*.ron          # cited encyclopedia entries
    └── sources/                 # developer-fetched raw SBDB/MPC/fact-sheet snapshots (build-tool inputs; provenance)
scenarios/                       # + world_load.ron, survey_refine.ron, prospect.ron
```

**Structure Decision**: `sojourn-world` is the second game-system module crate, sitting above
`sojourn-astro` exactly as FA-02 sits above the kernel: it depends on the astro types and frames
but the astro crate never depends back on it — the one cross-module data flow (generated bodies)
travels through the kernel's designed `publishes`/`reads` manifest seam using astro's own DTO, so
there is no crate cycle and no kernel change. The catalogue itself is *data* loaded by astro, so
substituting the real world for the test fixture is a data + harness-wiring act, not a physics
change. The build tool is a separate, dev-only, network-free member kept out of the shipped
dependency tree, honouring the fully-offline posture while still being a real, runnable, auditable
pipeline. The world-query surface is plain public crate API (pure functions over a truth-free
snapshot) — the same IPC-serializable seam a Tauri host or the harness calls between steps.

## Complexity Tracking

No constitution violations to justify — table intentionally empty. The two astro changes are
additive contract extensions (recorded under Constitution Check), not principle deviations.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
