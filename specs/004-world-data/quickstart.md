# Quickstart: World Data & Belief State (FA-03)

Headless scenarios that exercise the slice end-to-end and double as the independent tests for each
user story. All run through `sojourn-harness` with the `world` flag (points astro's catalogue at
`data/world` and installs Astro + World modules). No UI anywhere.

## Build & gate

```pwsh
cargo test -p sojourn-world                     # unit + integration (incl. tests/audit.rs truth-leak)
cargo run -p sojourn-harness -- validate-data world   # schema + sources + counts + ephemeris + links
cargo run -p sojourn-harness -- conformance --module world
cargo run -p sojourn-harness -- verify  scenarios/survey_refine.ron   # double-run bit-identity
cargo run -p sojourn-harness -- roundtrip scenarios/prospect.ron      # save/load state identity
cargo bench -p sojourn-harness --bench world                          # query latency + tick budget
```

## US1 — The real Solar System loads
1. `validate-data world` passes: counts (≥140 moons, ≥2,800 small bodies, all majors), 100%
   schema + non-empty `source`, epoch normalisation, ephemeris reference checks.
2. `world_load.ron` loads the full catalogue headlessly; major-body rail positions match
   `reference-ephemeris.ron` at ≥10 epochs within documented bounds.
3. The **FA-02 analytic suite re-runs against the real catalogue** and passes unchanged
   (`cargo test -p sojourn-astro` with the real catalogue substituted).

## US2 — Truth is hidden, belief is played
1. `believed(faction, site, grade)` pre-survey returns a wide prior-based `Estimate`, **never** the
   seeded truth.
2. `tests/audit.rs` enumerates the whole query surface → no truth path (SC-002).
3. Two factions hold independent beliefs; a `verify` double-run is bit-identical (belief + truth in
   slice state).

## US3 — Surveys make knowledge
1. `survey_refine.ron`: a sequence of `Observe` commands of increasing class/quality against a
   seeded site. Assert variance **monotonically non-increasing**, mean converging toward truth, and
   a poor instrument never reaching sample-grade certainty (clamped to the class floor).
2. Repeated max-class observations are **stable** at the floor (no oscillation/underflow).
3. `verify` double-run: the entire belief evolution is bit-identical.

## US4 — Sites: places worth going
1. Starter sites load schema-valid with sourced properties + PP category, anchored to catalogued
   bodies.
2. Each property class refines independently per the observation classes that sense it.
3. Site queries filter by body and by believed-property thresholds within faction Y's knowledge.

## US5 — Dynamical locations
1. Enumerate `locations()`; each `resolve_location(id, t)` returns a point (L-points/anchors) or
   region (bands/staging) via FA-02 frames.
2. Identity is stable across a save/load and a catalogue-version bump (string keys).

## US6 — Prospecting the unknown
1. `prospect.ron`: `Prospect` against a field generates new small bodies — deterministic per seed
   (`verify` + `roundtrip`), collision-free permanent ids (`≥ 2³¹`).
2. A generated body is a full FA-02 target: run a porkchop/encounter query against it.
3. Aggregate over ≥100 seeds matches the field's sourced distributions within tolerance.

## US7 — The Sojournal knows its sources
1. `validate-data world` enforces: ≥1 citation/entry, all links resolve, every major body has an
   entry, no entry states a seeded per-game truth.

## Performance (SC-006)
- Indexed world queries < 50 ms; catalogue load < 5 s; full catalogue + belief holds the FA-02
  ≥1 sim-year/min envelope — all checked by the `world` bench on the reference machine.
