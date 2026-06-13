# Feature Specification: World Data & Belief State (FA-03)

**Feature Branch**: `004-world-data`
**Created**: 2026-06-13
**Status**: Draft
**Input**: User description: Build Sojourn's world data model — the real Solar System the game is played in (Sun, planets, ~150 significant moons, ~3,000 catalogued small bodies from real orbital elements, plus statistical prospecting fields), dynamical locations as first-class nodes, surveyable Sites, ground-truth physical properties held separately from per-faction belief states with modelled uncertainty that missions refine, and the source-cited Sojournal encyclopedia data. All data-driven with provenance, schema-validated, with query interfaces for the UI and other systems. No tech stack.

> **Position in the programme.** Child specification for feature area **FA-03** of the umbrella
> spec (`specs/001-sojourn-solar-4x/spec.md`), refining FR-WLD-001…007 and FR-WLD-012 in full
> (FR-WLD-008…011 — planetary-protection mechanics and the astrobiology process — see the open
> scope clarification below). Built on the FA-01 kernel contracts (SimModule, ModulePayload
> commands, with_slice queries, seeded streams) and as the production implementer of FA-02's
> **body-catalogue contract** (`specs/003-astrodynamics/contracts/body-catalog.md`): the real
> catalogue replaces the five-body test fixture behind the same interface, so the propagator,
> rails, SOI machinery and planners work unchanged. Authoritative sources:
> `design/05-WORLD.md`, `design/00-OVERVIEW.md` §6, `.specify/memory/constitution.md` v1.0.0
> (Principles I, V, VIII). Where this spec is silent, those documents govern.

---

## Scope Boundary

**This slice delivers**: the real Solar-System catalogue (bodies + physical data + provenance)
and its validation; dynamical locations as named, queryable nodes; surveyable Sites with
ground-truth properties; the per-faction belief-state layer with modelled uncertainty and the
observation/refinement machinery; statistical prospecting fields and their conversion into
newly catalogued targets; the Sojournal encyclopedia data set and its query surface; and the
read-only world-query interfaces other slices and the future UI consume ("what is here, what
do we believe is here, how certain are we").

**This slice does NOT deliver**: missions, instruments or vehicles (FA-04+ fly the surveys —
this slice defines the observation interface they will drive); the economy's use of resources
(FA-06 prices what this slice describes); faction definitions (FA-09 — belief states are keyed
by opaque faction identity now); UI rendering of any of it (FA-10); and — pending the open
clarification — the planetary-protection *consequence* mechanics and the staged astrobiology
*evidence process* of umbrella FR-WLD-008…011 (the data fields and seeded ground truth those
mechanics need are in scope either way). [NEEDS CLARIFICATION: Where does FA-03 end on
planetary protection and astrobiology? The umbrella places FR-WLD-008 (COSPAR categories with
sterilisation costs, forward/back contamination consequences), FR-WLD-009 (seeded astrobiology
ground truth), FR-WLD-010 (staged probabilistic evidence process) and FR-WLD-011 (life as
science object) inside FA-03. Your slice brief covers the planetary-protection *category* as a
site property and the truth/belief machinery, but contamination consequences touch politics
(FA-09) and the evidence process is a mission-driven loop. Options: (a) this slice ships the
**data and seeding**: PP categories on bodies/sites, seeded astrobiology ground truth per
candidate world (from the kernel's seeded streams), and belief fields for both — while the
evidence-staging process and contamination-consequence mechanics arrive with the
mission/politics slices that can actually drive them; (b) this slice also implements the full
staged evidence pipeline and contamination consequence logic now, driven through the
observation interface.]

## User Scenarios & Testing *(mandatory)*

Actors: **the player** (explores a real Solar System through imperfect knowledge) and **the
integrator** (builds mission/economy/UI slices on the catalogue, belief and query contracts).

### User Story 1 - The Real Solar System Loads (Priority: P1)

The game world is the actual Solar System: the Sun, the eight planets, Pluto and the major
dwarfs, ~150 significant moons, and ~3,000 catalogued asteroids and comets — each on its real
orbit from published orbital elements, each carrying sourced physical data. The FA-02
propagator flies craft through this world unchanged: transfer windows to the real Mars, flybys
of the real Jupiter, rendezvous with real near-Earth asteroids. Every quantitative entry traces
to its provenance, and the data set validates in CI.

**Why this priority**: Everything else anchors to this. It is also the production fulfilment of
the body-catalogue contract FA-02 already consumes.

**Independent Test**: Load the full catalogue headlessly; validate counts, schema, source
fields; check ephemeris positions of major bodies at reference epochs against published
positions within documented accuracy; run the FA-02 validation suite against the real catalogue
(it must still pass — same physics, real world).

**Acceptance Scenarios**:

1. **Given** the shipped catalogue, **When** it loads, **Then** it contains the Sun, 8 planets, Pluto + major dwarf planets, ≥140 moons and ≥2,800 small bodies, every entry schema-valid with a non-empty source.
2. **Given** any major body and a set of documented reference epochs across 2026–2126, **When** its rail position is computed, **Then** it matches the published ephemeris-derived check values within the documented per-body accuracy bound.
3. **Given** the real catalogue substituted for the test fixture, **When** the FA-02 kernel gates run (conformance, double-run on a coast scenario), **Then** they pass unchanged.
4. **Given** the gravitating-flag rule (FA-02 clarification), **Then** planets, major moons and the most massive small bodies are flagged from their sourced masses, and the flagged set is documented.

---

### User Story 2 - Truth Is Hidden, Belief Is Played (Priority: P1)

For everything surveyable — a lunar cold trap's ice grade, an asteroid's composition, a Mars
site's slope — the world holds ground truth, but the player's faction holds only a belief:
an estimate with explicit uncertainty. Every query the player (or UI, or AI faction) can make
returns belief, never truth. Acting on a poorly-surveyed site is possible — and risky — by
design: the mine built on a hopeful estimate can hit a worse seam than advertised.

**Why this priority**: The truth/belief separation is the game's honesty contract (the WHY of
this slice) and must be structural — nothing downstream should even be able to read truth.

**Independent Test**: Headless: query a site's believed grade pre-survey (wide uncertainty,
prior-based); verify no query surface exposes the seeded truth; verify two factions hold
independent beliefs; verify belief state is part of the deterministic world state
(save/round-trip, double-run).

**Acceptance Scenarios**:

1. **Given** an unsurveyed site with seeded ground truth G, **When** any faction-facing query runs, **Then** it returns a prior-based estimate with uncertainty and never G itself.
2. **Given** two factions, **When** one surveys a site, **Then** only that faction's belief narrows; the other's is unchanged (per-faction knowledge).
3. **Given** a save/load round-trip or a double run, **Then** all belief states (and ground truths) are bit-identical (kernel determinism gates).
4. **Given** the full query surface, **Then** an audit finds no path from faction-facing queries to ground-truth values (truth is reachable only by the engine's own resolution mechanics and the test suite's privileged access).

---

### User Story 3 - Surveys Make Knowledge (Priority: P1)

Knowledge improves only through observation. An observation — parameterised by instrument
quality, observation class (remote sensing, in-situ, sample-grade) and target — refines the
acting faction's belief toward ground truth: error bars narrow, estimates converge, and
successive observations with better instruments converge further. Observations are commands
(journaled, deterministic); their noise is seeded. Later slices supply the missions that earn
the right to observe; this slice defines what observing does.

**Why this priority**: The refinement loop is the mechanism that makes belief playable; P1
because Sites and prospecting both depend on it.

**Independent Test**: Headless: issue observation commands of increasing quality against a
seeded site; verify uncertainty decreases monotonically per class, estimates converge toward
truth within documented bounds, results are seed-deterministic, and a poor instrument cannot
reach sample-grade certainty.

**Acceptance Scenarios**:

1. **Given** a site with truth G and a faction prior, **When** a remote-sensing observation applies, **Then** the faction's estimate moves toward G and its uncertainty shrinks by the class/quality-determined amount — never below the class's floor.
2. **Given** repeated identical observations, **Then** uncertainty converges to the class floor rather than collapsing to zero (you cannot remote-sense your way to ground truth).
3. **Given** the same seed and command script, **Then** the entire belief evolution is bit-identical (kernel double-run with the world module installed).
4. **Given** an observation command for an unknown target or invalid class, **Then** it is deterministically rejected.

---

### User Story 4 - Sites: Places Worth Going (Priority: P2)

Bodies expose Sites — specific named locations seeded from real exploration targets (lunar
polar cold traps, Jezero-class deltas, mid-latitude ice, NEA surfaces) — each carrying
ground-truth properties the game's economics and operations will live on: resource type and
grade, illumination profile, slope/roughness, thermal environment, comms geometry, hazards,
and planetary-protection category. All of it surveyable: revealed progressively through the
belief layer, with the PP category and existence of a site being knowable early while grades
and hazards demand real survey work.

**Why this priority**: Sites are where FA-06 (ISRU economics) and FA-07 (bases) will anchor;
they need the belief plumbing first.

**Independent Test**: Headless: load the starter site set (schema + sources); verify per-site
truth/belief separation and survey refinement for each property class; verify site queries by
body and by property filters.

**Acceptance Scenarios**:

1. **Given** the shipped starter sites, **Then** every site is schema-valid with sourced
   properties and a planetary-protection category, anchored to a catalogued body.
2. **Given** a site survey campaign, **Then** each property class (grade, illumination, slope, hazards) refines independently per the observation classes that can sense it.
3. **Given** a site query for "believed water-ice grade ≥ X within faction Y's knowledge", **Then** results reflect only Y's beliefs and uncertainties.

---

### User Story 5 - Dynamical Locations: The Map's Nodes (Priority: P2)

The map has places that aren't bodies: orbit bands (LEO/MEO/GEO class), Lagrange points of
relevant pairs, halo/NRHO-class staging orbits, and named surface anchors. These are
first-class, queryable locations with positions over time — the nodes the logistics graph
(FA-06) will price edges between, and the staging points mission planning will reference.

**Why this priority**: The logistics layer needs these as stable identities; FA-02 already
computes L-point positions — this slice names and catalogues them.

**Independent Test**: Headless: enumerate locations; verify each resolves to a position/region
at any time via the FA-02 machinery; verify identity stability across saves and catalogue
versions.

**Acceptance Scenarios**:

1. **Given** the location catalogue, **Then** it includes the documented orbit bands of major bodies, L1/L2 of the flagged pairs (Sun–planet, planet–moon), and named staging orbits — each with a sourced definition.
2. **Given** any location id and time, **Then** a query returns its position (point) or region (band) in the FA-02 frame conventions.

---

### User Story 6 - Prospecting the Unknown (Priority: P2)

Beyond the catalogued ~3,000, the belt and the NEA population exist statistically: prospecting
fields with sourced population models (size-frequency, type mix, orbital distribution). A
prospecting survey against a field converts statistics into reality: newly catalogued small
bodies are generated — seeded, plausible, permanent — and join the world as real targets
(surveyable, divertible per FA-02's rules, eventually minable).

**Why this priority**: The Prospector strategy depends on it; it builds directly on the
observation machinery (US3).

**Independent Test**: Headless: run prospecting campaigns against a field; verify generated
bodies are deterministic per seed, statistically consistent with the field's sourced model
(over many seeds), permanently catalogued (save/load), and fully functional as FA-02 targets.

**Acceptance Scenarios**:

1. **Given** a prospecting field and a survey command, **Then** zero or more new small bodies are catalogued with orbits/types drawn from the field's documented distributions via the kernel's seeded streams.
2. **Given** the same seed and commands, **Then** identical bodies (ids, orbits, properties) are generated; **and given** a porkchop/encounter query against a generated body, **Then** it behaves like any catalogued target.
3. **Given** many independent seeds, **Then** the aggregate of generated populations matches the field's distribution parameters within documented statistical tolerance.

---

### User Story 7 - The Sojournal Knows Its Sources (Priority: P3)

Every body, site class, location type and world concept has an encyclopedia entry: real
science, cited sources, written to teach (Principle VIII). Entries are data; their factual
claims carry citations; body entries link to the live catalogue so the UI can later show "what
we know" (belief-aware framing comes from querying belief alongside the entry — the entry text
itself never leaks truth).

**Why this priority**: Educational honesty is constitutional but consumes the other layers;
its UI arrives with FA-10.

**Independent Test**: Headless: validate the Sojournal data set (every entry has ≥1 citation;
every catalogued major body has an entry; links resolve); query entries by id/body/topic.

**Acceptance Scenarios**:

1. **Given** the Sojournal data set, **Then** every entry carries at least one citation, every link resolves to a catalogue object or another entry, and CI validation enforces both.
2. **Given** any major body, **Then** an entry exists describing the real object with sources.

---

### Edge Cases

- **Belief before any prior**: a faction queries a body/site no one has defined priors for — there must be a documented default prior (wide, honest), never an error or a truth leak.
- **Observation of an already-converged belief**: repeated max-quality observations must be stable (no oscillation, no uncertainty underflow below the class floor).
- **Conflicting observations**: a later, better observation must dominate gracefully; a worse one must not *degrade* an already-tight belief (information never decreases).
- **Truth-leak regression**: adding a new query later must not expose truth — the audit (US2-4) must be a standing test pattern, not a one-off.
- **Generated-body identity collisions**: prospecting-generated ids must never collide with catalogued ids or each other, across saves and replays.
- **Catalogue version vs belief state**: belief states reference catalogue objects; loading a save against a different catalogue version must fail actionably (the FA-01 pinning rules + FA-02's catalogue-hash guard extend to world data).
- **Site on a diverted body**: FA-02 can divert a small body; its sites and beliefs must follow the body (identity by body id, not position).
- **Epoch drift in elements**: real elements published at different epochs must all evaluate correctly at game time (epoch normalisation at data-build time, validated).
- **Massive catalogue, light queries**: enumerating 3,000+ bodies must not be the only access path — filtered/indexed queries (by type, region, flag) must stay within latency budgets.
- **Two factions prospecting the same field**: discoveries are per-world facts (a generated body exists for everyone once generated) but *knowledge* of it is per-faction — the discoverer knows; others learn by their own observation or (later slices) data trade.

## Requirements *(mandatory)*

IDs are FR-WORLD-###. Umbrella traceability (FR-WLD-###) inline. All content lives in
schema-validated data files with `source` provenance (Principles I, V); the engine reads it.

### Catalogue (FR-WLD-001/002/004/012)

- **FR-WORLD-101**: The shipped catalogue MUST contain the Sun, the 8 planets, Pluto and the major dwarf planets (at minimum Ceres, Eris, Haumea, Makemake), ≥140 significant moons and ≥2,800 catalogued small bodies, each with real orbital elements derived from published authoritative data (JPL/MPC-class sources) and provenance recorded per entry. *(FR-WLD-001)*
- **FR-WORLD-102**: Each body MUST carry sourced physical data sufficient for FA-02 and later slices: gravitational parameter (or mass), radius, rotation, J2 where significant, atmosphere model where present, gravitating/divertible flags per the FA-02 rules, and bulk composition class. *(FR-WLD-004)*
- **FR-WORLD-103**: Rail ephemerides from the shipped elements MUST reproduce published reference positions for major bodies within documented per-body accuracy bounds across 2026–2126, validated by CI checks against committed reference values. *(FR-WLD-002)* [NEEDS CLARIFICATION: How is the real catalogue produced and maintained? Options: (a) an **offline data-build tool** (part of the repo, run by developers) that queries public datasets (JPL SBDB, planetary fact sheets, MPC) and emits the committed, schema-validated data files with per-entry provenance and a recorded snapshot date — the game ships only committed data, fully offline; (b) hand-curated data files without a generator (lighter now, painful at 3,000 bodies and for updates); (c) a runtime downloader (violates the fully-offline posture). This decides repo tooling, data refresh workflow, and how the ~3,000-body file stays honest.]
- **FR-WORLD-104**: The catalogue MUST implement FA-02's body-catalogue contract so the propagator, SOI machinery and planners run unchanged against the real world; the FA-02 test fixture remains for physics tests.
- **FR-WORLD-105**: The 2026 baseline MUST match the umbrella's clarified world: real government assets context, fictional commercial sector (no real-company small-body naming conflicts; real IAU body names are used for natural bodies). *(FR-WLD-012)*

### Dynamical locations (FR-WLD-005)

- **FR-WORLD-201**: A location catalogue MUST define first-class nodes: orbit bands of major bodies (LEO/MEO/GEO-class, low orbits of other bodies), Lagrange points L1/L2 of documented pairs, named staging orbits (halo/NRHO-class as region definitions), and surface anchors at sites — each with stable identity, a sourced definition, and a resolvable position/region over time via FA-02's frames.
- **FR-WORLD-202**: Locations MUST be queryable for the logistics layer's future use (enumerate, resolve at time, classify) without FA-06 existing yet.

### Truth, belief and observation (FR-WLD-006)

- **FR-WORLD-301**: Ground truth for all surveyable properties MUST be held separately from belief: seeded where the design calls for per-game variation (resource grades, hazard details — drawn from sourced plausibility distributions via kernel streams at world creation), fixed where reality fixes it (orbits, radii, documented compositions).
- **FR-WORLD-302**: Belief state MUST be per-faction (opaque faction ids now; FA-09 binds them later): per surveyable property, an estimate + uncertainty, initialised from documented priors (real 2026 knowledge: well-known for major bodies, wide for unsurveyed sites/small bodies).
- **FR-WORLD-303**: No faction-facing query may return ground truth for an unsurveyed property; truth access is restricted to engine-internal resolution and explicitly privileged test interfaces. A standing audit test MUST enforce this for the whole query surface. *(Principle VIII)*
- **FR-WORLD-304**: Observations MUST be journaled commands parameterised by (faction, target, observation class, instrument quality): remote-sensing, in-situ and sample-grade classes with documented uncertainty floors per class; each observation moves the faction's estimate toward truth with seeded noise and narrows uncertainty per a documented refinement model; information never decreases; repeated observations converge to the class floor. [NEEDS CLARIFICATION: In this slice, what validates an observation command? Options: (a) **trust-the-caller**: any faction may observe any target (mission slices later enforce "you must actually be there with an instrument" before issuing the command) — simplest honest seam, matching how the research-gate stand-in worked in FA-02; (b) proximity-validated now: the world module checks a craft of that faction is within range via FA-02 state (requires new cross-module data flows — craft positions aren't in published views — and craft have no faction identity until FA-09); (c) trust-the-caller plus a declared validation hook interface that mission slices will implement. This decides cross-module coupling now versus later.]
- **FR-WORLD-305**: Mission-derived knowledge MUST also flow per the umbrella: per-body Geoscience understanding grows primarily from observation events (the FA-05 research slice will consume these; this slice emits them as kernel events with class/quality payloads).

### Sites (FR-WLD-007)

- **FR-WORLD-401**: Bodies MUST expose Sites defined in data: identity, body anchor, location (surface coordinates or orbital), and ground-truth properties — resource type and grade, illumination profile, slope/roughness, thermal environment, comms geometry class, hazard level, planetary-protection category — each property surveyable through the belief layer with per-property observation-class sensitivity.
- **FR-WORLD-402**: A starter site set MUST ship covering the design's named real targets (lunar PSRs and peaks of eternal light, mare/highland references, lava tubes; Jezero-class and mid-latitude-ice Mars sites; representative NEA/Ceres/outer-system sites) — every property sourced (measured values where humanity has them; documented plausibility distributions where it does not).
- **FR-WORLD-403**: Site existence and PP category MUST be knowable cheaply (catalogue-level knowledge); grades and hazards MUST require survey work (wide priors).

### Prospecting fields (FR-WLD-003)

- **FR-WORLD-501**: Statistical prospecting fields MUST be defined in data with sourced population models (region, size-frequency distribution, type mix, orbital-element distributions) for the uncatalogued belt/NEA/Kuiper populations.
- **FR-WORLD-502**: Prospecting observations against a field MUST generate newly catalogued small bodies deterministically from kernel streams, consistent with the field's distributions, with collision-free permanent identities; generated bodies are full catalogue citizens (surveyable, FA-02-targetable, divertible when eligible) and per-world facts whose *knowledge* is per-faction.

### Sojournal (FR-WLD + Principle VIII)

- **FR-WORLD-601**: The Sojournal data set MUST ship entries for every major body, body class, location type, site class and world concept in this slice — each entry carrying citation(s), resolvable links, and identifier-keyed text (rendering is FA-10's); CI MUST enforce citation presence and link resolution.
- **FR-WORLD-602**: Sojournal entries MUST never state seeded per-game truths (they describe the real world and the game's honest mechanics, not the current game's hidden values).

### Queries & integration

- **FR-WORLD-701**: A read-only world-query surface (the FA-10/inter-slice seam, kernel `with_slice` + pure functions per the FA-02 pattern) MUST answer: what is catalogued here (bodies, sites, locations, filtered/indexed); what does faction F believe about X (estimates + uncertainties); how certain is F about X; what changed since tick T (belief deltas for UI refresh).
- **FR-WORLD-702**: The world module MUST conform to the kernel module contract (owned belief/generated-body slice, declared streams for seeding and observation noise, journaled commands, conformance + determinism gates) and integrate with FA-02 (catalogue supply; generated/diverted body interop).
- **FR-WORLD-703**: Events MUST be emitted for discovery-significant changes (new body catalogued, survey milestone reached) through the kernel event system (classes in data).

### Validation & performance

- **FR-WORLD-801**: All world data MUST pass schema + source-presence validation in CI (`validate-data` extended); counts, link resolution, epoch normalisation and ephemeris reference checks included.
- **FR-WORLD-802**: With the full catalogue loaded, the kernel performance envelope MUST hold (umbrella SC-003 / FA-02 SC-007: ≥1 sim-year/min under load on the reference machine) and indexed world queries MUST return within 50 ms on the reference machine.

### Key Entities

- **Body (catalogue)**: as FA-02's contract + composition class, name/IAU designation, discovery metadata; ~3,000 shipped + generated additions.
- **Prospecting Field**: region + sourced population model; consumed by prospecting observations.
- **Generated Body**: a prospecting product — permanent, seeded, collision-free identity.
- **Dynamical Location**: orbit band | L-point | staging orbit | surface anchor; stable id; resolvable at time.
- **Site**: body-anchored surveyable place with the property set (resource/grade, illumination, slope, thermal, comms, hazards, PP category).
- **Ground Truth Store**: fixed real values + seeded per-game values (engine-private).
- **Belief State**: per (faction, target, property): estimate, uncertainty, last-observation metadata.
- **Observation**: journaled command (faction, target, class, quality) → belief refinement + events.
- **Observation Class**: remote-sensing | in-situ | sample-grade, with documented uncertainty floors.
- **Sojournal Entry**: cited, linked, identifier-keyed encyclopedia content.
- **World Query Surface**: pure read-only functions over a snapshot (catalogue + faction-filtered belief).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (The world is real)**: The shipped catalogue meets the counts (≥140 moons, ≥2,800 small bodies, all majors), 100% schema+source validity, and major-body rail positions match committed published reference values within the documented bounds at ≥10 epochs spanning 2026–2126.
- **SC-002 (Honesty is structural)**: The truth-leak audit passes over 100% of faction-facing queries; no test can obtain an unsurveyed seeded truth through any public query path.
- **SC-003 (Knowledge converges honestly)**: In the refinement suite, belief error decreases monotonically in expectation with observation count and quality, never tightens below class floors, and 100% of runs are bit-identical per seed (kernel gates with the world module installed).
- **SC-004 (Prospecting is statistics made real)**: Generated populations across ≥100 seeds match field distribution parameters within documented tolerances; identities are collision-free across save/replay; generated bodies pass FA-02 targetability checks.
- **SC-005 (Integration holds)**: FA-02's full test suite and all kernel gates pass with the real catalogue substituted; the world module passes conformance; saves pin and verify world data versions.
- **SC-006 (Performance)**: Full catalogue + belief layer holds the ≥1 sim-year/min envelope under the FA-02 load profile on the reference machine; indexed world queries < 50 ms; catalogue load < 5 s.
- **SC-007 (Sojournal completeness)**: 100% of entries cited and link-resolved; 100% of major bodies covered; zero entries leaking seeded truths (audited).
- **SC-008 (Sites ready for economics)**: Every starter site exposes the full property set through both truth (engine) and belief (queries) layers, with survey refinement demonstrated per property class.

## Assumptions

- **Faction identity is opaque**: belief states key on faction ids supplied by callers (a default player faction in scenarios); FA-09 binds real factions later without data migration.
- **Element fidelity**: bodies ride Keplerian mean elements per the FA-02 rails contract; per-body accuracy bounds are documented honestly (planets tight, small bodies looser), and the contract's pure-`state_at` semantics allow later fidelity upgrades (piecewise elements) without interface change.
- **Spin axes and higher-fidelity geometry**: v1 keeps FA-02's global-Z spin idealisation except where a site's illumination model documents otherwise; refinement is data-driven later.
- **Site count**: the starter set targets ~30–40 fully-sourced sites (the design's named targets); breadth grows as data work, not code work.
- **Resource taxonomy**: a documented, sourced resource-type list (water ice, regolith O₂ feedstocks, metals/silicates, volatiles/organics, rare isotopes) ships with this slice; FA-06 prices it later.
- **Observation noise & priors are data**: refinement-model parameters, class floors, priors and seeding distributions live in sourced data files; tuning is data, not code.
- **Public-data licensing**: orbital/physical data derive from public-domain or attribution-permitted sources (NASA/JPL, MPC, IAU); attribution is carried in the provenance fields and a data-credits document.
- **The FA-02 catalogue-hash pinning pattern extends** to all world data (saves refuse mismatched world data, actionably).
- **Kernel/event additions** (e.g. `body-catalogued`, `survey-milestone` classes) follow the established data-registry mechanism; no kernel code changes are anticipated for this slice.

## Out of Scope (this slice)

- Missions, instruments, vehicles, launch (FA-04+) — observation commands are the seam they will drive.
- Resource extraction, pricing, ISRU economics (FA-06); base building (FA-07).
- Faction definitions, politics, AI behaviour (FA-09); data trade between factions.
- All UI (FA-10) — including Sojournal rendering and map display.
- Planetary-protection consequence mechanics and the staged astrobiology evidence process — pending the scope clarification above (data + seeding are in scope under option (a)).
- Real-time data updates from live astronomy services (offline posture; data refresh is a development-time act).

## Open Clarifications Summary

Three items are marked [NEEDS CLARIFICATION], in priority order:

1. **Scope Boundary — planetary protection & astrobiology**: data + seeded ground truth here with mechanics later (a), or the full evidence/consequence machinery now (b)? (Umbrella FR-WLD-008…011 traceability.)
2. **FR-WORLD-304 — observation validation**: trust-the-caller (a), proximity-validated now (b), or trust + declared validation hook (c)? (Cross-module coupling now vs later.)
3. **FR-WORLD-103 — catalogue production pipeline**: offline data-build tool emitting committed files (a), hand-curated (b), or runtime download (c)? (Tooling, refresh workflow, data honesty at 3,000 bodies.)
