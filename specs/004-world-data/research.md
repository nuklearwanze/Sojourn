# Phase 0 Research: World Data & Belief State (FA-03)

All decisions resolve the Technical Context and the spec's requirements against Constitution
v1.0.0 and the FA-01/FA-02 contracts already in the tree. Format per decision: **Decision /
Rationale / Alternatives rejected**. No `NEEDS CLARIFICATION` remain (spec clarified 2026-06-13).

---

## R1 — Crate topology: where the world lives relative to astro

**Decision.** A new module crate `crates/sojourn-world` depends on `sojourn-astro` (+ `sojourn-core`)
and implements the kernel `SimModule` contract. The **real catalogue is data** under `data/world/`,
loaded by astro's existing `Catalog` type (the body-catalogue contract owner). The world module
owns the *belief/truth/site/location/prospecting/Sojournal* layer keyed by astro `BodyId`. Astro
never depends on world. The harness `world` flag points astro's catalogue loader at `data/world`
and installs both modules.

**Rationale.** Mirrors the proven FA-02-on-FA-01 layering. The body-catalogue contract already
declares that "FA-03 implements [it] in production"; the cleanest implementation is *to ship the
data astro already knows how to read*, not to re-type the catalogue. Belief is a new concern that
nothing in astro needs, so it belongs in a sibling module above astro. Keeping the dependency
arrow one-way (world → astro → core) preserves the workspace's acyclic shape and lets FA-02's
physics tests run untouched against either catalogue.

**Alternatives rejected.** (a) Put belief inside `sojourn-astro` — pollutes the physics crate with
game-knowledge concerns and couples FA-02 gates to belief state. (b) A world crate that owns its
*own* catalogue type and converts to astro's — duplicates the contract and the validation, invites
drift. (c) Merge astro+world — abandons the module boundary the kernel was built to enforce.

---

## R2 — Catalogue production: the offline data-build tool (FR-WORLD-106), network-free workspace

**Decision.** A dev-only bin crate `crates/sojourn-worldbuild` reads **local source snapshots**
under `data/world/sources/` (developer-fetched SBDB/MPC JSON exports and transcribed fact-sheet
tables, each itself committed with provenance) and emits the committed, schema-valid
`data/world/catalog/*.ron` with per-entry `source` and a file-level `snapshot_date`. Fetching the
raw snapshots is a **separate, documented step** (a script / manual SBDB query), *outside* the
Rust workspace, so the workspace carries **zero network dependencies**. The tool reuses only
`serde_json` + `ron` (already workspace deps). It is **not** a member of the shipped game's
dependency tree (game = core + astro + world + harness); it is a workspace member only so CI can
build and `validate-data` can cross-check its output is reproducible from the committed inputs.

**Rationale.** Honours the clarified decision (offline tool → committed files) and the
constitution's fully-offline posture *and* the no-presentation-deps architecture: by ingesting
local snapshots rather than calling the network from Rust, the workspace stays `cargo-deny`-clean
and reproducible. Provenance is twofold — the raw snapshot (with its retrieval date) and the
per-entry `source` the tool stamps — so the 3,000-body file stays honest and auditable. Epoch
normalisation (R-elements at varied epochs → game-epoch mean elements) happens in the tool, once,
and is validated (edge case: epoch drift).

**Alternatives rejected.** (a) Runtime downloader — violates offline posture (and the spec's Out
of Scope). (b) Hand-curated files with no generator — unmaintainable at 3,000 bodies, no
reproducibility, easy source rot. (c) A Rust tool that fetches over HTTP — drags a TLS/HTTP stack
+ its licences into the workspace and breaks the network-free guarantee for marginal convenience.

---

## R3 — Belief representation and the refinement model (FR-WORLD-301/302/304)

**Decision.** Each surveyable **scalar** property's belief is a Gaussian `(mean, variance)` in an
appropriate transformed space (log-space for positive quantities like grade/density so the belief
can't imply negatives). An **observation** of class *c* and quality *q* yields a measurement
`m = truth + ε`, `ε ~ N(0, σ²(c,q))` drawn from a named stream, with `σ(c,q) ≥ floor(c)`. The
update is **Gaussian precision addition**:

```
post_var  = 1 / (1/prior_var + 1/σ²)            # precision adds — variance only ever shrinks
post_mean = post_var * (prior_mean/prior_var + m/σ²)
post_var  = max(post_var, floor_var(c))          # converge to the class floor, never to zero
```

**Ordinal/categorical** properties (hazard level, composition class, PP category) use a discrete
probability vector updated by a per-class confusion-matrix likelihood; PP category and site
existence start near-certain (catalogue-level knowledge, FR-WORLD-403). Priors, `σ(c,q)`, floors
and confusion matrices are all **data** (`priors.ron`).

**Rationale.** Precision addition gives the spec's guarantees *for free and structurally*:
variance is monotonically non-increasing, so **information never decreases** even when a worse
(larger-σ) observation arrives (it just adds little precision); the floor makes repeated
observations **converge to the class floor, not to truth** ("you cannot remote-sense your way to
ground truth"); the estimate moves toward truth in expectation. It is a closed form (no iteration,
trivially deterministic), commutative in precision so observation *order* doesn't change the final
tightness (only the seeded noise path does, which is itself deterministic). Log-space keeps grades
positive and makes multiplicative survey error natural.

**Alternatives rejected.** (a) Store raw samples and recompute — unbounded slice growth, order
sensitivity. (b) Ad-hoc "shrink uncertainty by k% per survey" — can't guarantee monotonicity vs
the truth or honest convergence, and isn't a real estimator. (c) Full particle filter — overkill,
nondeterministic-prone, and unjustified for unimodal grade beliefs.

---

## R4 — Truth/belief separation: structural, not just disciplined (FR-WORLD-303, SC-002)

**Decision.** Ground truth lives in an **engine-private** field of the world slice (`truth.rs`).
The **only** type the public query surface sees is `WorldSnapshot`, which is *constructed to carry
belief + public catalogue data and no truth fields at all*. Truth is therefore unreachable from
any `pub` query *by construction*, not by convention. Truth is read only by the module's own
`step`/`on_command` resolution and a `#[cfg(any(test, feature = "privileged"))]` accessor used by
the test suite. A **standing audit test** (`tests/audit.rs`) enumerates the entire public query API
and asserts no path yields an unsurveyed seeded truth; it is a permanent regression guard, run in
CI, so adding a new query later cannot silently open a leak.

**Rationale.** SC-002/Principle VIII demand that *nothing downstream can even read truth*. The
strongest enforcement is to make the leak unrepresentable: if `WorldSnapshot` has no truth, no pure
function over it can return truth. The audit test then covers the residual risk (a future snapshot
field) and codifies the edge case "truth-leak regression … a standing test pattern, not a
one-off."

**Alternatives rejected.** (a) Same struct holds truth + belief with "don't read truth" discipline
— one careless query leaks the game's honesty contract; unenforceable. (b) Separate process /
capability tokens — far heavier than the determinism budget or the threat model needs.

---

## R5 — Prospecting: deterministic generation + how generated bodies become propagatable (FR-WORLD-501/502)

**Decision.** A prospecting field carries a region + sourced distributions (size-frequency / H,
taxonomic type mix, orbital-element distributions). A prospecting command (faction, field, effort)
draws a detection count and then samples each new body's elements/type from those distributions via
the field's **named stream**; each gets a **collision-free permanent id** from the generated-id
allocator (R12) and is recorded in the world slice. The world module **publishes** a
`generated-bodies` view — a `Vec<BodyDef>` in astro's own DTO — and `sojourn-astro` **reads** it
(kernel `publishes`/`reads` manifest seam): the propagator resolves rails/targeting over
`base catalogue ∪ generated view`, so a generated body is a full FA-02 citizen (porkchop, encounter,
divert) with no astro→world dependency. Generated bodies are **per-world facts** (they exist for
everyone once generated); *knowledge* of them stays per-faction (the discoverer's belief narrows;
others must observe).

**Rationale.** Uses the kernel's intended cross-module mechanism rather than inventing a command or
breaking slice isolation; astro stays ignorant of world by exchanging only its own type. Seeded
streams + a persisted monotonic allocator make generation **bit-identical per seed** across
save/replay (SC-004) and collision-free by construction. Publishing astro's DTO (not a world type)
keeps the dependency arrow one-way.

**Alternatives rejected.** (a) A new `AstroCommand::RegisterBody` that mutates an astro-owned annex
— workable but puts world-driven content behind an astro command and an astro-owned store; the
publish/reads view is cleaner and keeps generation wholly in the world module. (b) Astro reaches
into the world slice directly — violates single-writer slice isolation. (c) Pre-generate the whole
field at world creation — defeats "prospecting converts statistics into reality" and bloats state.

---

## R6 — Dynamical locations resolvable over time (FR-WORLD-201/202)

**Decision.** A `Location` is a tagged enum: `OrbitBand{body, alt_lo, alt_hi, inclination_class}`
| `LagrangePoint{primary, secondary, point: L1|L2}` | `StagingOrbit{primary, kind: Halo|NRHO|DRO,
params}` | `SurfaceAnchor{body, lat, lon}`. `resolve_at(id, t)` returns a point (L-points via
astro's existing L1/L2 bisection solver; surface anchors via the body rail + FA-02's global-Z spin
idealisation and `rotation_period_s`) or a region (orbit bands as radial shells; staging orbits as
characterised regions in v1). Identities are **string-keyed in data**, stable across saves and
catalogue versions.

**Rationale.** FA-02 already computes L-point positions and frames; this slice *names and
catalogues* them as first-class nodes the logistics graph (FA-06) will key edges on. Treating
halo/NRHO/DRO as **region definitions** in v1 (not propagated trajectories) is the honest
approximation the spec's assumptions allow — refinable to real periodic orbits later behind the
same `resolve_at` seam without identity churn.

**Alternatives rejected.** (a) Store precomputed ephemerides for locations — stale vs the rails,
needless. (b) Full periodic-orbit propagation for staging orbits now — large effort outside this
slice's value; deferred behind the stable interface.

---

## R7 — Sites and per-property survey sensitivity (FR-WORLD-401/402/403)

**Decision.** A `Site` has identity, a body anchor (**identity by `BodyId`, not position**, so it
follows a diverted body — edge case), a location (surface coords or orbital), and a ground-truth
**property set**: resource type+grade, illumination profile, slope/roughness, thermal, comms
geometry class, hazard level, PP category. Each property declares which **observation classes** can
sense it and how well (the per-property `σ(c,q)`/floor mapping in data). Site existence + PP
category are catalogue-level (cheap knowledge); grades/hazards carry wide priors needing real
survey (FR-WORLD-403). The starter set (~30–40 sourced sites) ships in `sites.ron`.

**Rationale.** Sites are where FA-06/FA-07 anchor; they need the truth/belief plumbing and the
per-property class sensitivity now so economics can later price *believed* grades with honest
uncertainty. Anchoring by body id is the only choice consistent with FA-02 diversion.

**Alternatives rejected.** (a) One uncertainty per site — loses that you can know the slope while
the ice grade is still a guess. (b) Sites as positions — break under diversion.

---

## R8 — Sojournal data + validation (FR-WORLD-601/602, Principle VIII)

**Decision.** Sojournal entries are data: `{id, kind, subject_ref?, title_id, body_text,
citations:[{source}], links:[ref]}`. CI (`validate-data`) enforces ≥1 citation per entry, that
every link resolves (to a catalogue object or another entry), and that every major body has an
entry. Entries describe the **real world and the game's honest mechanics only** — they are static
and structurally cannot bind to a per-game seeded truth, which (with a check that no entry
references the truth store) satisfies FR-WORLD-602 "never state seeded per-game truths."

**Rationale.** Educational honesty is constitutional; making citations and link-resolution
mechanical CI checks is the cheapest durable enforcement. Keeping entries truth-free by
construction means belief-aware framing is the UI's job (query belief alongside the entry in FA-10).

**Alternatives rejected.** (a) Prose embedded in code — violates data-driven content. (b) Entries
that quote current grades — leaks truth; forbidden.

---

## R9 — The read-only world-query surface (FR-WORLD-701)

**Decision.** `WorldSnapshot::from_core(&core)` via kernel `with_slice` over the world slice,
composed with FA-02's `AstroSnapshot` for positions. Pure functions answer: catalogue queries
(bodies/sites/locations, **filtered/indexed** by type/region/flag); `believed(faction, target,
property) → Estimate{value, uncertainty}`; `certainty(faction, target, property)`;
`belief_delta_since(faction, tick) → [Change]` for UI refresh; `resolve_location(id, t)`;
`sojournal(id)`. Belief deltas come from a per-faction, tick-stamped **change log** in the slice
(deterministically trimmed). Indexes (`BTreeMap` by type/parent/flag) are built at load for the
<50 ms budget.

**Rationale.** Identical seam to FA-02's planning queries (`with_slice` + pure fns over a
snapshot) — already the pattern the harness/Tauri host calls between ticks, IPC-serializable.
Indexed access answers the "massive catalogue, light queries" edge case within budget.

**Alternatives rejected.** (a) Live mutable handles to the slice — breaks the read-only seam and
determinism (queries only between ticks). (b) Full-scan every query — blows the 50 ms budget at
3,000 bodies.

---

## R10 — Version pinning, saves, and performance (FR-WORLD-801/802, SC-005/006)

**Decision.** World data is content-hashed and **pinned in saves**, extending FA-02's
catalogue-hash guard: a save loaded against a different world-data version fails actionably (edge
case). Belief, truth, generated bodies and the change log are slice state (postcard, kernel-driven
roundtrip). Performance: indexes give <50 ms queries; the belief layer steps event-driven (cheap),
so the FA-02 ≥1 sim-yr/min envelope holds with the full catalogue; catalogue load targets <5 s for
~3,000 RON entries, with a **documented fallback** to a build-tool-produced `postcard` sidecar
(RON stays the sourced, validated form) if RON parse misses budget.

**Rationale.** Reuses the FA-01/FA-02 pinning and serde machinery wholesale; the only new cost is
belief stepping (bounded, event-driven) and query indexing (amortised at load).

**Alternatives rejected.** (a) No pinning — silent truth/belief corruption across versions. (b)
Recompute indexes per query — misses the latency budget.

---

## R11 — Kernel/astro touchpoints (what changes, what doesn't)

**Decision.** **Kernel: no code change.** World commands (observe, prospect) route through FA-02's
`Command::ModulePayload` → `SimModule::on_command`; new event classes (`body-catalogued`,
`survey-milestone`, both `LogOnly`) are **data-registry** additions to `event-classes.ron`.
**Astro: two additive changes**, both leaving FA-02 gates green: (a) `Catalog` loads the real
multi-file catalogue and derives divertibility from the explicit `divertible`/`gravitating` data
flags (the contract's actual rule), replacing the fixture-only `radius > 100 km` guard with
validation in the build tool; (b) the propagator consumes an optional `reads` generated-bodies view
(astro `BodyDef` DTO) for rails/targeting — empty view ⇒ identical behaviour to today.

**Rationale.** Keeps FA-01 frozen and confines change to additive, contract-consistent astro
extensions that the existing test fixture exercises as the unchanged baseline. No domain logic
enters the kernel.

**Alternatives rejected.** (a) A kernel "catalogue service" — over-generalises; ModulePayload +
publish/reads already suffice. (b) Forking the astro catalogue type for the real world — drift and
double validation (see R1).

---

## R12 — Body identity: catalogued vs generated, collision-free and permanent (edge case, SC-004)

**Decision.** `BodyId` (u32) space is **partitioned**: real catalogued bodies take ids `< 2³¹`
(assigned deterministically by the build tool from designation, recorded in provenance); generated
bodies take ids `≥ 2³¹` from a **persisted monotonic counter** in the world slice, advanced per
generation via the seeded prospecting stream. Disjoint ranges + monotonic allocation make
collisions impossible across saves and replays; the counter is slice state so replay reproduces
identical ids.

**Rationale.** Directly satisfies "generated ids must never collide with catalogued ids or each
other, across saves and replays" with no runtime lookup — a range check and a counter.

**Alternatives rejected.** (a) Hash-derived ids — birthday collisions at scale, and a u32 is tight.
(b) Reuse freed ids — generated bodies are permanent (no freeing); reuse would break replay
identity.

---

## Summary of decisions feeding Phase 1

| # | Decision | Primary artifacts |
|---|---|---|
| R1 | New `sojourn-world` module above astro; catalogue is data astro loads | plan structure, data-model |
| R2 | Dev-only, network-free `sojourn-worldbuild`; local snapshots → committed RON | contracts/catalogue-data |
| R3 | Gaussian precision-add belief, log-space, class floors; params in data | contracts/belief-model, data-model |
| R4 | Truth structurally absent from the query snapshot; standing audit test | contracts/belief-model, contracts/world-queries |
| R5 | Deterministic prospecting; generated bodies via publish/reads astro DTO | contracts/observation-commands, contracts/catalogue-data |
| R6 | Locations as tagged enum, `resolve_at` via FA-02 frames/L-points | contracts/world-queries, data-model |
| R7 | Sites anchored by BodyId; per-property class sensitivity | data-model, contracts/belief-model |
| R8 | Sojournal data + CI citation/link/coverage checks | data-model, quickstart |
| R9 | `with_slice` + pure fns over truth-free snapshot; indexed; belief deltas | contracts/world-queries |
| R10 | World-data version pinned in saves; perf via indexes; postcard fallback | contracts/catalogue-data, quickstart |
| R11 | No kernel change; two additive astro changes | contracts/catalogue-data, plan |
| R12 | Partitioned BodyId space; persisted monotonic generated-id counter | data-model, contracts/observation-commands |
