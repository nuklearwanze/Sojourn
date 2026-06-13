# Phase 1 Data Model: World Data & Belief State (FA-03)

Entities, fields, relationships, validation and state transitions for `sojourn-world`. Types are
Rust-flavoured but the contract is the *shape and rules*, not the syntax. Ordered stores are
`BTreeMap` (determinism). Everything quantitative is sourced data (Principle I). "Slice state"
means it is serialized in the world module's slice and covered by save/roundtrip/replay gates.

---

## 1. Catalogue layer (data; loaded by `sojourn-astro::Catalog`)

### Body (catalogue) — *extends FA-02 `BodyDef`*
The real catalogue ships in the existing `BodyDef` shape (`contracts/body-catalog.md`) plus the
display/science-neutral metadata FA-03 adds. Fields beyond FA-02's `BodyDef`:

| Field | Type | Rules |
|---|---|---|
| `composition_class` | enum (sourced taxonomy) | C/S/M/V… for asteroids; ice/rock/gas classes for bodies; from `resources.ron` taxonomy |
| `designation` | string | IAU/MPC designation; drives the build-tool id assignment (R12) |
| `discovery` | `{year?, source}` | provenance metadata; optional year |
| `source`, `snapshot_date` | string | per-entry provenance + file-level retrieval date (CI-enforced non-empty) |

- **Relationships**: `parent: Option<BodyId>` (tree rooted at the Sun) — FA-02 rule unchanged.
- **Validation** (build tool + `validate-data`): unique id; non-empty `source`; `mu,radius > 0`;
  elliptic rails `e ∈ [0,1)`, `sma > 0`; parent resolves + acyclic; `divertible ⇒ !gravitating`
  (flag-driven; the FA-02 radius heuristic is dropped — R11); epoch normalised to the game epoch.
- **Counts** (SC-001): Sun + 8 planets + Pluto + ≥4 dwarfs; ≥140 moons; ≥2,800 small bodies.

### Generated Body — *slice state*
A prospecting product. Same `BodyDef` shape; `id ≥ 2³¹` (R12); `divertible` per eligibility;
`source = "generated: field <id>, seed-derived"`. Permanent once created.
- **Published** to astro as the `generated-bodies` view (`Vec<BodyDef>`), consumed for
  rails/targeting (R5/R11).

### Catalogue Indexes — *derived at load, not serialized*
`by_type`, `by_parent/region`, `by_flag` (`BTreeMap`s) over base ∪ generated, for <50 ms queries.

---

## 2. Dynamical locations (data + resolve)

### Location
| Field | Type | Rules |
|---|---|---|
| `id` | string key | stable across saves/catalogue versions |
| `kind` | enum (below) | tagged |
| `source` | string | non-empty |

`kind ∈ { OrbitBand{body, alt_lo_m, alt_hi_m, inclination_class}, LagrangePoint{primary, secondary,
point: L1|L2}, StagingOrbit{primary, kind: Halo|NRHO|DRO, params}, SurfaceAnchor{body, lat, lon} }`.

- **resolve_at(id, t)** → `Point(Vec3)` (L-points via astro L1/L2 solver; surface anchor via rail +
  global-Z spin + `rotation_period_s`) or `Region(shell|characterised)` (bands, staging orbits).
- **Validation**: referenced bodies exist; L-pairs are flagged/gravitating pairs; bands `alt_lo <
  alt_hi`. **Coverage** (FR-WORLD-201): orbit bands of major bodies, L1/L2 of documented pairs,
  named staging orbits, site surface anchors.

---

## 3. Sites (data truth + per-faction belief)

### Site — *ground-truth definition in data; truth values in the engine-private store*
| Field | Type | Rules |
|---|---|---|
| `id` | string key | unique |
| `body` | `BodyId` | **anchor by id** (follows diversion) |
| `placement` | `Surface{lat,lon}` \| `Orbital{elements}` | one of |
| `pp_category` | enum (COSPAR I–V) | catalogue-level knowledge (cheap) |
| properties | `PropertySet` (below) | each surveyable |
| `source` | string | non-empty |

### PropertySet (per site; each entry is a surveyable property)
`resource_type` (taxonomy), `grade` (scalar, log-space), `illumination` (scalar/profile), `slope`
(scalar), `thermal` (scalar), `comms_class` (ordinal), `hazard_level` (ordinal). Each property
declares **which observation classes sense it** and the per-class `σ(c,q)`/floor (from `priors.ron`).
- **Knowledge rules** (FR-WORLD-403): existence + `pp_category` near-certain at catalogue level;
  `grade`/`hazard_level` start with **wide priors** (real survey required).
- **Starter set**: ~30–40 sourced sites (lunar PSRs/peaks of eternal light, mare/highland, lava
  tubes; Jezero-class + mid-latitude ice Mars; representative NEA/Ceres/outer-system).

---

## 4. Truth / belief / observation (the honesty core)

### Ground Truth Store — *ENGINE-PRIVATE slice state (never in any snapshot — R4)*
Per `(target, property)`: the resolved truth value. **Fixed** where reality fixes it (orbits,
radii, documented compositions); **seeded** where the design calls for per-game variation (grades,
hazard details, astrobiology) — drawn at world creation from **sourced plausibility distributions**
via the `world-seed` named stream.
- **Astrobiology truth** (FR-WORLD-306): per candidate world (Mars subsurface, Europa, Enceladus,
  Titan, Ceres brines, Venus clouds) a seeded presence/absence (+ tier) from sourced distributions —
  **mostly negative**, rarely >1–2 positives/game. Held here; leaks through no query; the staged
  evidence process is deferred (mission slices).
- **Access**: module `step`/`on_command` resolution + `#[cfg(any(test, feature="privileged"))]` only.

### Belief State — *per-faction slice state*
Keyed `(faction_id, target, property)` → `Estimate`:

| Field | Type | Rules |
|---|---|---|
| `mean` | f64 (transformed space) | toward truth in expectation under observation |
| `variance` | f64 | **monotonically non-increasing**; clamped `≥ floor_var(class)` |
| `last_obs` | `{tick, class, quality}` | metadata for deltas/UI |

- Categorical/ordinal properties: a probability vector instead of `(mean,var)`, updated by a
  confusion-matrix likelihood; same monotone-information guarantee in entropy terms.
- **Init**: from documented priors (`priors.ron`) — tight for well-known major bodies, wide for
  unsurveyed sites/small bodies. Default prior MUST exist for any queried `(target,property)`
  (edge case: belief before any prior → wide honest default, never an error or truth leak).

### Observation Class — *data*
`remote-sensing | in-situ | sample-grade`, each with documented uncertainty **floor** (`floor(c)`)
and the `σ(c,q)` model. Ordering: remote ≥ in-situ ≥ sample-grade floors. (`priors.ron`.)

### Observation — *journaled command → refinement + event*
`(faction, target, property-or-"all", class, quality)`. Effect: draw `ε ~ N(0,σ²)` from the
`obs-noise` stream keyed by `(faction,target,property,seq)`; apply the R3 precision-add update;
clamp to floor; emit `survey-milestone` on threshold crossings. **Validation = trust-the-caller**
(FR-WORLD-304): structural only (target exists; class/quality valid); rejected deterministically if
invalid. Entitlement is a later mission-slice concern.

### Belief Change Log — *per-faction, tick-stamped slice state*
Append `(tick, faction, target, property)` on each refinement; deterministically trimmed; powers
`belief_delta_since(faction, tick)` (R9).

---

## 5. Prospecting (statistics → reality)

### Prospecting Field — *data*
| Field | Type | Rules |
|---|---|---|
| `id` | string key | unique |
| `region` | orbital region spec | belt / NEA / Kuiper |
| `size_frequency` | sourced dist (H / diameter) | |
| `type_mix` | sourced categorical dist | taxonomy weights |
| `element_dists` | sourced dists for a,e,i,… | |
| `detection_model` | `{effort → count}` params | |
| `source` | string | non-empty |

### Prospecting Command → Generated Bodies
Draw count from `detection_model(effort)` via the field's `prospect` stream; sample each body's
elements/type from the field dists; allocate `id` (R12); record as Generated Body (per-world fact);
narrow the **discoverer's** belief only; publish the updated `generated-bodies` view; emit
`body-catalogued`. **Determinism** (SC-004): identical seed+commands ⇒ identical ids/orbits/props;
aggregate over ≥100 seeds matches field dists within documented tolerance.

---

## 6. Sojournal (encyclopedia data)

### Sojournal Entry — *data*
`{id, kind: Body|BodyClass|LocationType|SiteClass|Concept, subject_ref?: Ref, title_id, body_text,
citations: [{source}] (≥1), links: [Ref]}`.
- **Validation** (FR-WORLD-601/602): ≥1 citation; every link resolves; every major body has an
  entry; **no entry references the truth store / states a seeded per-game value**.

---

## 7. Resource taxonomy (data)

`resources.ron`: sourced list (water ice, regolith O₂ feedstocks, metals/silicates,
volatiles/organics, rare isotopes) with `source`. Referenced by `composition_class` and site
`resource_type`. FA-06 prices it later.

---

## 8. Module slice & manifest (kernel contract)

### WorldSlice — *the owned, serialized state*
`ground_truth` (private), `beliefs` (per-faction), `belief_change_log`, `generated_bodies`,
`generated_id_counter`, `survey_progress`/milestones. (Catalogue base, sites defs, locations,
fields, sojournal, priors are immutable **data** loaded at init, hashed + pinned — not duplicated
in the slice beyond what mutates.)

### WorldModule manifest
| Field | Value |
|---|---|
| `id` | `world` |
| `owned_slice` | `WorldSlice` |
| `publishes` | `generated-bodies` (Vec<BodyDef>) view for astro |
| `reads` | — (consumes astro positions via snapshot composition at query time) |
| `streams` | `world-seed` (creation), `obs-noise` (observation), `prospect` (generation) |
| `emits` | `body-catalogued`, `survey-milestone` |
| `subscribes` | — (v1) |
| `cadence` | event-driven; cheap per-tick (no per-body work absent commands) |

- **Pinning**: world-data content hash pinned in saves (extends FA-02 catalogue-hash; R10).
- **Conformance/determinism**: ordered stores, libm-only, declared streams, no wall-clock — passes
  `conformance --module world` and the harness double-run/roundtrip/replay gates.

---

## Entity relationship summary

```text
Sun ──parent*── Bodies (real) ──anchor── Sites ──property── PropertySet
                   │                         │
                   ├── composition_class → Resource taxonomy
                   │                         │
ProspectingField ──generates──▶ Generated Bodies (id ≥ 2³¹)   (truth)   (belief: per faction)
                                     │                            │            │
                                     └── published view ─▶ sojourn-astro    GroundTruthStore   BeliefState
Location {band|L-point|staging|anchor} ──resolve_at(t)──▶ Vec3/Region        (private)        (per faction)
Sojournal Entry ──links──▶ {Body | Site | Location | Entry}                        ▲   refine   │
Observation cmd (faction,target,class,quality) ───────────────────────────────────┴── seeded ───┘
```
