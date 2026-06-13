# Contract: Real-Catalogue Data + Build Tool + Astro Touchpoints (FR-WORLD-101..106)

How the real Solar System enters the game as *data*, how it is produced and kept honest, and the
two additive `sojourn-astro` changes that let it (and prospecting products) run through the
propagator unchanged.

## Catalogue data format

Ships under `data/world/catalog/` split by class (`planets.ron`, `moons.ron`, `small-bodies.ron`)
in the FA-02 `BodyDef` shape (`specs/003-astrodynamics/contracts/body-catalog.md`) plus FA-03's
metadata (`composition_class`, `designation`, `discovery`, `snapshot_date`). Every entry carries a
non-empty `source`; every file a `snapshot_date`. Loaded by `sojourn-astro::Catalog` (generalised
loader, below); content-hashed and **pinned in saves** (extends FA-02's catalogue-hash guard).

- **Counts (SC-001)**: Sun + 8 planets + Pluto + ≥4 dwarfs; ≥140 moons; ≥2,800 small bodies.
- **Physical data (FR-WORLD-102)**: μ/mass, radius, rotation, J2 where significant, atmosphere
  where present, gravitating/divertible flags, composition class — enough for FA-02 + later slices.
- **Rails accuracy (FR-WORLD-103)**: major-body rail positions match committed published reference
  values (`reference-ephemeris.ron`, ≥10 epochs across 2026–2126) within documented per-body
  bounds; checked in CI. Elements are epoch-normalised to the game epoch at build time.
- **Naming (FR-WORLD-105)**: real IAU names for natural bodies; fictional commercial sector (no
  real-company small-body naming).

## Build tool (`crates/sojourn-worldbuild`, dev-only)

- **Inputs**: local snapshots under `data/world/sources/` — developer-fetched SBDB/MPC JSON
  exports and transcribed fact-sheet tables, each committed with its retrieval provenance. Fetching
  is a **separate documented step** outside the Rust workspace → the workspace has **no network
  deps** (reuses `serde_json` + `ron` only).
- **Outputs**: the committed, schema-valid `data/world/catalog/*.ron` with per-entry `source`,
  file `snapshot_date`, deterministic id assignment (R12: real ids `< 2³¹` from designation), and
  epoch-normalised elements.
- **Reproducibility**: re-running the tool on the same committed inputs yields byte-identical
  output; CI may assert this. The tool is **not** in the shipped game's dependency tree.

## Validation (`validate-data world`, CI)

Schema + non-empty `source` for every entry; counts; parent resolution + acyclicity; elliptic-rail
bounds; `divertible ⇒ !gravitating`; epoch normalisation; ephemeris reference checks; location
referential integrity; Sojournal citation presence + link resolution + major-body coverage;
no-truth-leak in Sojournal entries.

## Additive astro changes (NOTED; FA-02 gates stay green)

1. **Generalised `Catalog` loader** — reads the real **multi-file** catalogue from a directory and
   derives divertibility from the explicit `divertible`/`gravitating` **data flags** (the
   body-catalogue contract's actual rule). The fixture-only `radius > 100 km` guard is removed from
   the loader and enforced instead by the build tool/validation. The existing `test-catalog.ron`
   still loads identically.
2. **Generated-bodies view consumption** — the propagator resolves rails/targeting over
   `base catalogue ∪ generated view`, where the view is a `Vec<BodyDef>` (astro's own DTO) the
   world module **publishes** and astro **reads** (kernel manifest seam). An **empty** view ⇒
   behaviour identical to today, so all FA-02 propagation/planner/divert tests pass unchanged.

Neither change touches the kernel. Astro never depends on `sojourn-world` (the exchanged type is
astro's own `BodyDef`).
