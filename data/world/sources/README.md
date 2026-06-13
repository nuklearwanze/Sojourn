# World-data source snapshots (build-tool inputs)

This directory holds **local snapshots** of public astronomical datasets — the raw
inputs the offline build tool (`crates/sojourn-worldbuild`) transforms into the
committed catalogue under `data/world/catalog/`.

## Why local snapshots (and not a network fetch)

The shipped game is **fully offline** (Constitution, Principle/Engineering
constraints) and the Cargo workspace carries **no network dependencies**
(`cargo-deny`-clean). Fetching is therefore a **separate, documented developer
step** outside the Rust workspace; the build tool only reads what is already here.

## The pipeline (FR-WORLD-106)

1. **Fetch** (manual / scripted, outside Rust): query the public datasets and save
   the responses here, each with a provenance header (URL/query + retrieval date):
   - **JPL SBDB** (small-body orbital elements + physical params) → `sbdb-*.json`
   - **JPL planetary fact sheets** (planet/moon physical + orbital data) → transcribed `factsheet-*.json`
   - **MPC** (designations, discovery metadata) → `mpc-*.json`
2. **Build**: `cargo run -p sojourn-worldbuild -- data/world/sources data/world/catalog`
   epoch-normalises elements to the game epoch, assigns deterministic body ids
   (`< 2^31`) from designation, stamps per-entry `source` + file `snapshot_date`,
   and writes `planets.ron`, `moons.ron`, `small-bodies.ron`.
3. **Validate**: `cargo run -p sojourn-harness -- validate-data world` checks schema,
   non-empty sources, counts, epoch normalisation and ephemeris references — over
   the **committed** output (never the network).

Re-running the tool on the same committed inputs yields byte-identical output
(CI may assert this). The tool is **not** a dependency of the shipped game
(`sojourn-core` / `-astro` / `-world` / `-harness`).

## Provenance expectations

Every snapshot file MUST carry, in a `provenance` field: the dataset, the exact
query/URL, and the retrieval date. The build tool propagates this into each
catalogue entry's `source` and the file-level `snapshot_date`, satisfying
Principle I (every quantitative datum is sourced) end-to-end.
