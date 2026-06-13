# Feature Specification: Research & Personnel (FA-05)

**Feature Branch**: `005-research-personnel`
**Created**: 2026-06-13
**Status**: Draft
**Input**: User description: Build Sojourn's research system as a modelled process, not a purchase — the two-track model (Track A Science raising continuous Understanding Levels across knowledge domains; Track B Engineering advancing technology programs through TRL 1–9 via domain-gated test campaigns), with cost/schedule uncertainty, seeded dead ends, failures-that-teach, rare earned breakthroughs, leapfrogging, a global science tide, and personnel as managed assets including an astronaut career pipeline. Web-shaped sourced tech tree; capability categories always reachable; exposes the maturity/heritage/understanding query interface for vehicle design, economy and politics. No tech stack.

> **Position in the programme.** Child specification for feature area **FA-05** of the umbrella
> spec (`specs/001-sojourn-solar-4x/spec.md`), refining FR-RES-001…013 and the personnel/crew
> facets of FR-CRW-007. Built on the FA-01 kernel contracts (SimModule, ModulePayload commands,
> with_slice queries, seeded streams, watch/event system). It is the **producer** of the
> technology-maturity, flight-heritage and domain-understanding values that FA-04 (vehicle &
> propulsion) consumes for capability and reliability, that FA-06 (economy) prices, and that FA-09
> (politics) turns into prestige. Authoritative sources: `design/01-RESEARCH.md`,
> `design/02-TECH-TREE.md`, `design/00-OVERVIEW.md` §1–2, `.specify/memory/constitution.md` v1.0.0
> (Principles I, VI, VIII). Where this spec is silent, those documents govern.

---

## Scope Boundary

**This slice delivers**: the two-track research engine — Knowledge Domains with continuous
Understanding Levels (UL 0–100, diminishing returns + cross-domain synergy) and Engineering
Programs advancing Technologies through TRL 1–9 via domain-UL-gated test campaigns; the
generation of Research Points (RP) and Design Effort (DE) from funded staff, facilities and
mission injections, and their portfolio allocation; cost/schedule uncertainty with overruns;
seeded dead ends with parallel-approach mitigation; failures-that-teach; rare seeded-and-earned
breakthroughs; leapfrogging; the global science tide (World UL, publish-vs-hold, cheaper
catch-up); the Flight-Heritage and reliability model fed by operational-use events; the
**personnel roster** (scientists, engineers, programme managers, mission controllers,
diplomats/lobbyists, and astronauts as managed assets) with traits, recruitment/poaching/training,
morale/retention/aging and tacit-knowledge loss; the **astronaut career pipeline** (select →
train → assign → age) up to the mission boundary; the web-shaped, fully-sourced, schema-validated
tech-tree data; and the read-only maturity/heritage/understanding query surface other slices and
the future UI consume.

**This slice does NOT deliver**: vehicle/propulsion performance models (FA-04 *consumes* maturity
and heritage; this slice exposes them, it does not compute thrust/Isp); funding, budgets, markets,
facilities-as-economic-assets, and the monetary settlement of licensing/partnerships (FA-06 — this
slice takes funded inputs and facility capabilities as opaque caller-supplied descriptors and
exposes the licence/partner *interfaces*, but does not move money); missions, instruments and
flights that inject UL and accrue heritage (FA-04+ fly them — this slice defines the injection and
heritage interfaces they drive); in-mission life-support closure and acute crew physiology
(radiation dose, psychological load, bone/muscle budgets *during* flight) — see the clarified
FA-05↔FA-08 split below; politics, prestige and public mood (FA-09 consume publish/firsts events);
and UI rendering of any of it (FA-10).

## Clarifications

### Session 2026-06-13

- Q: Where is the FA-05 ↔ FA-08 crew boundary? → A: FA-05 owns the **personnel roster + career pipeline** for all staff including astronauts (selection, multi-year training, assignment readiness, morale, aging, tacit-knowledge), and exposes astronaut readiness/career state; FA-08 later owns **in-mission** ECLSS closure and acute physiological/radiation/psychological dynamics, feeding career effects (dose accumulated, health) back through the interface this slice declares. (Scope Boundary, FR-RESP-601/602)
- Q: How much of the global science tide lives here vs FA-06? → A: FA-05 owns the **knowledge dynamics** — World UL advancement, the publish-vs-hold choice, cheaper catch-up, and the declared licence/partner/buy-in *interfaces* and TRL/IP credit they grant; the **monetary settlement** (licensing income, purchase price, partnership cost-sharing) is FA-06. (FR-RESP-401/403)
- Q: How much of the ~150-node tech tree ships here? → A: The **full Knowledge-Domain set (A1–A17)** plus a **representative, sourced, schema-validated subset of engineering nodes** that exercises every mechanic (cross-branch gates, ≥2 paths per capability category, seeded dead ends, breakthroughs, leapfrog, heritage discounts); the complete node population is a documented data-authoring expansion behind the same schema (code complete; data grows). (FR-RESP-701/702)
- Q: What shape is the reliability value FA-04 consumes? → A: A **scalar per-use success probability ∈ [0,1]**, computed in FA-05 from TRL + accumulated flight-units + domain UL, exposed **alongside the raw inputs** (TRL, flight-units, UL margin); FA-04 may layer duration/phase effects but the maturity→reliability model lives here (design §2). (FR-RESP-202/801)
- Q: How is "every capability category reachable in every seed" enforced? → A: **Constructive + verified** — the dead-end seeding algorithm guarantees it **by construction** (it never closes a capability category's last viable path), and a CI sweep over a sampled seed set **verifies** the invariant (defense in depth), mirroring the structural-plus-standing-check pattern used in FA-03. (FR-RESP-301/901)

No open [NEEDS CLARIFICATION] markers remain.

## User Scenarios & Testing *(mandatory)*

Actors: **the player** (a research director who funds understanding and matures technologies under
uncertainty) and **the integrator** (builds vehicle/economy/politics/UI slices on the maturity,
heritage, understanding and personnel contracts).

### User Story 1 - Understanding Before Capability (Priority: P1)

You cannot buy a technology. The player funds **Research Programs** that raise continuous
**Understanding Levels** across **Knowledge Domains** (A1–A17). Understanding grows from funded
scientists, facilities and instruments, and is injected directly by missions (flight/surface data).
Understanding has diminishing returns at high levels and synergy bonuses across coupled domains, and
it **gates** which Engineering Programs are available and sets their risk floor. Nothing engineering
can start until the relevant domains are understood enough.

**Why this priority**: This is the WHY of the slice — the science-gates-engineering contract is the
identity of the whole game. Everything downstream (programs, the tree, vehicles) depends on it.

**Independent Test**: Headless: fund research into a domain; verify UL rises with diminishing
returns and synergy, that mission-injection events raise the named domains' UL beyond labs alone,
that an engineering program below its domain-UL floor is unavailable and becomes available once the
floor is crossed, and that the whole evolution is seed-deterministic (double-run).

**Acceptance Scenarios**:

1. **Given** a domain at low UL and a funded research allocation, **When** the simulation advances, **Then** that domain's UL rises along a diminishing-returns curve and coupled domains receive synergy, all from sourced data parameters.
2. **Given** an engineering program with a domain-UL availability floor, **When** the floor is not met, **Then** the program is unavailable (with the gating domain/threshold reported); **When** the floor is met, **Then** it becomes available.
3. **Given** a mission-injection command naming domains and a payload, **Then** those domains' UL increases per the documented injection model (more than equivalent lab time for geoscience/astrobiology domains).
4. **Given** the same seed and command script, **Then** all ULs are bit-identical across a double run.

---

### User Story 2 - Maturing a Technology Through TRL (Priority: P1)

An Engineering Program advances a **Technology** through **TRL 1–9** via a **test campaign**. Each
TRL step carries a cost, a minimum duration that cannot be bought below a floor (schedule-compression
penalty), and a facility requirement; progress is S-curved and gated by the relevant domains' UL. A
technology is flyable only at **TRL ≥ 6** with steep reliability penalties that ease through 7–9.
Cost and schedule are estimated with uncertainty (P50/P80) and realised with overruns. Successful
operational use accrues **Flight Heritage**, raising reliability toward a per-tech ceiling and
discounting derivative programs.

**Why this priority**: TRL is the spine of implementation (design §2); without it there are no usable
technologies for FA-04 and no reliability story.

**Independent Test**: Headless: start a program, allocate DE, advance it tier by tier; verify TRL
steps respect cost/min-time/facility gates and the UL gate, that estimates carry P50/P80 and realise
with seeded overrun variance, that reliability is a documented function of TRL + flight-units + domain
UL, that heritage events raise reliability and discount a derivative program, and that it is
seed-deterministic.

**Acceptance Scenarios**:

1. **Given** a program at TRL n with its UL/facility prerequisites met, **When** a TRL step is funded, **Then** it completes only after its minimum duration and cost, and compressing schedule raises overrun risk per the documented penalty.
2. **Given** a TRL step whose domain-UL prerequisite is unmet, **Then** the step cannot complete and the gating domain is reported.
3. **Given** a matured technology, **When** its TRL is queried, **Then** reliability is reported as a function of TRL, accumulated flight-units and domain UL; flying below TRL 6 is refused and TRL-6 hardware carries the documented reliability penalty.
4. **Given** operational-use (heritage) events for a technology, **Then** its reliability rises asymptotically toward its ceiling and a declared derivative program starts partway up the TRL ladder.

---

### User Story 3 - The Tree Is Alive (Priority: P1)

The same goal can be reached by more than one technology path, and a given game's **seed** fixes
which approaches are **dead ends** within TRL bands, when **breakthroughs** are possible, and which
clusters they favour. Programs hit cost/schedule **overruns**; test campaigns **fail** but inject
understanding ("failure-that-teaches"); recognising a dead end early (rising risk index, stalled
error bars, repeated failures without UL growth) and cutting it is a skill; **parallel approaches**
de-risk; sustained basic-science investment accrues hidden insight pressure that can trigger a rare
**breakthrough**; and **leapfrogging** lets a player skip a generation by over-investing basic
science. Every capability category remains reachable by *some* path in *every* seed.

**Why this priority**: Dead ends, failures, breakthroughs and leapfrogging are what make the tree feel
earned rather than purchased (design §4); P1 because they are the core research-as-process mechanics.

**Independent Test**: Headless across many seeds: verify some approaches are dead ends within TRL
bands (with rising risk hints before confirmation) while a parallel approach to the same category
remains viable; verify test failures inject UL; verify breakthroughs occur only with sustained
basic-science investment at roughly the documented rare cadence and are announced with a sourced
reference; verify leapfrogging reaches a higher tier via UL; verify every capability category has a
viable path in every seed; verify all of it is seed-deterministic.

**Acceptance Scenarios**:

1. **Given** a seed, **When** a seeded dead-end approach is pursued, **Then** its risk index rises and error bars stall before confirmation, and a parallel approach to the same capability category is reachable.
2. **Given** a failing test campaign, **Then** the failure costs money/schedule **and** injects UL into the relevant domain; repeated failure without UL growth is the documented dead-end signal.
3. **Given** sustained high basic-science investment in a domain, **Then** insight pressure accrues and a breakthrough may trigger at a seeded threshold (cluster discount, early branch unlock, or hidden-path reveal), at roughly once per ~8–15 years for a heavily-invested domain, announced with a Sojournal reference; rushing applied work alone almost never triggers one.
4. **Given** over-investment in basic science, **Then** a higher-tier branch becomes reachable by UL (leapfrog) at higher cost/risk and without the skipped product's heritage.
5. **Given** any seed, **Then** every capability category retains at least one viable technology path (no seed bricks a strategy), and the entire stochastic evolution is bit-identical on a double run.

---

### User Story 4 - The Global Tide (Priority: P2)

Science is not a private resource. Each domain has a **World Understanding Level** that advances from
all factions' activity plus an exogenous baseline; a faction's private UL is the world level plus its
lead/lag. Factions choose to **publish** (prestige + faster World UL, lose exclusivity) or **hold/
patent** (keep the lead, slower tide, licensing potential). Trailing a domain makes catch-up research
cheaper than the frontier. This slice owns the knowledge dynamics and the declared licence/partner/
buy-in interfaces; the money for them is FA-06's.

**Why this priority**: The tide bounds runaway leads, keeps AI factions relevant, and is the substrate
for FA-06/FA-09 collaboration and competition; P2 because the single-faction loop (US1–US3) is
playable without it.

**Independent Test**: Headless with multiple factions: verify World UL advances from aggregate
activity + baseline; verify publish raises World UL and emits a prestige-eligible event while
hold/patent does not; verify a trailing faction researches known ground more cheaply than the
frontier; verify the licence/partner interfaces grant the documented TRL/IP credit without moving
money; verify determinism.

**Acceptance Scenarios**:

1. **Given** several factions investing in a domain, **Then** its World UL advances from their aggregate plus a baseline, and each faction's private UL equals World UL plus its lead/lag.
2. **Given** a publish action, **Then** World UL accelerates, exclusivity is lost, and a prestige-eligible publish event is emitted; **given** hold/patent, **Then** the lead is retained and the tide is slower.
3. **Given** a faction trailing a domain by N levels, **Then** its research there is cheaper than the frontier per the documented catch-up model.

---

### User Story 5 - People Make It Happen (Priority: P2)

Research is produced by **people**. The player manages a roster of scientists, engineers, programme
managers, mission controllers and diplomats/lobbyists — each with a discipline/skill, a small set of
**traits** (e.g. Visionary, Closer, Maverick, Safe Hands) that shift low-TRL vs qual performance,
breakthrough odds, overrun variance and reliability — plus recruitment, poaching (relations cost),
multi-year training, morale, retention and aging. RP/DE generation depends on staffing quality,
domain mismatch and facility bottlenecks (surfaced as efficiency multipliers). Losing key people can
**reduce effective UL** in a niche domain (tacit knowledge).

**Why this priority**: Personnel is half the slice's name and the lever that makes funding produce
research; P2 because the domain/TRL machinery (US1–US3) can be exercised with a default staffing
before the full roster model lands.

**Independent Test**: Headless: build a roster; verify traits shift the documented outcomes (Visionary
helps low-TRL/basic, Closer helps 6→9 qual, Maverick raises breakthrough odds and overrun variance);
verify hiring/poaching/training/aging transitions; verify RP/DE efficiency multipliers respond to
over/under-staffing, domain mismatch and facility bottlenecks; verify disbanding a team reduces
effective UL in its niche domain; verify determinism.

**Acceptance Scenarios**:

1. **Given** staff with traits assigned to a program, **Then** their traits shift that program's low-TRL/qual progress, breakthrough odds, overrun variance and reliability per the documented model.
2. **Given** a hire/poach/train/retire action, **Then** the roster transitions deterministically; poaching incurs a relations-cost signal and training takes the documented years.
3. **Given** under-staffing, domain mismatch or a facility bottleneck, **Then** the program's RP/DE efficiency multiplier reflects it.
4. **Given** the loss of key personnel in a niche domain, **Then** the faction's effective UL there decreases (tacit-knowledge loss).

---

### User Story 6 - The Astronaut Pipeline (Priority: P2)

Astronauts are managed assets with a career pipeline: **select → train** (multi-year, requiring
facilities/analog missions) **→ assign-ready → age out**. This slice owns the roster and career state
(selection, training progress, readiness, accumulated career dose and health as a running budget,
morale, aging) up to the mission boundary, and exposes astronaut readiness and career state. In-mission
life-support closure and the acute dose/psychological/health dynamics during a flight are FA-08's, which
feeds dose/health changes back through this slice's declared interface.

**Why this priority**: Crewed play depends on a trained astronaut corps; P2 because robotic research and
the whole tech tree are exercisable before crew.

**Independent Test**: Headless: run a candidate through select→train→ready; verify training requires the
documented facility/time and produces a ready astronaut; verify career dose/health budgets accumulate
deterministically and that exceeding documented limits removes an astronaut from the ready pool; verify
the FA-08-facing interface accepts dose/health deltas and reflects them in career state; verify
determinism.

**Acceptance Scenarios**:

1. **Given** a candidate and a training facility, **Then** training advances over the documented years and yields a ready astronaut; without the facility it cannot complete.
2. **Given** career dose/health budgets, **When** an FA-08 in-mission update applies dose/health deltas, **Then** career state updates and an astronaut over a documented limit leaves the ready pool.
3. **Given** the same seed and commands, **Then** the astronaut roster and career state are bit-identical on a double run.

---

### User Story 7 - Maturity, Heritage & Understanding On Tap (Priority: P3)

Other slices act on what research has produced. A read-only query surface answers: a technology's
**maturity** (current TRL, reliability, flyability), its **flight heritage** (units, reliability
ceiling, derivative discounts), a faction's **domain understanding** (private and world UL, gating
status), **program status** (TRL, P50/P80 vs actual, risk/dead-end index), and **personnel/roster**
summaries (counts, traits, astronaut readiness). FA-04 reads maturity/heritage for capability and
reliability; FA-06 prices programs and licensing; FA-09 turns publish/firsts into prestige.

**Why this priority**: The contract other slices consume; P3 because its consumers (FA-04/06/09/10)
arrive after the engine exists, but the surface must be defined now.

**Independent Test**: Headless: query maturity, heritage, understanding, program status and personnel
across factions; verify queries are pure (no mutation, identical fingerprint before/after) and
faction-scoped; verify a flyability query refuses sub-TRL-6 technologies and reports reliability.

**Acceptance Scenarios**:

1. **Given** a matured technology, **When** maturity is queried, **Then** TRL, reliability and flyability are returned for the asking faction, never another faction's private state.
2. **Given** a program, **When** its status is queried, **Then** TRL, P50/P80-vs-actual, risk index and any dead-end hint are returned.
3. **Given** the query surface, **Then** calling any query between ticks leaves the state fingerprint unchanged (pure, FA-02 planning-query pattern).

---

### Edge Cases

- **Schedule compression past the floor**: funding cannot buy a TRL step below its minimum duration; over-funding converts to rising overrun risk, never sub-floor time.
- **Every-path-dead-end is impossible**: seeding MUST guarantee ≥1 viable path per capability category in every seed; a validation check enforces it across many seeds.
- **Breakthrough on applied-only investment**: rushing applied engineering without basic-science investment almost never triggers a breakthrough (insight pressure is basic-science-weighted).
- **Heritage before TRL 8**: a technology with no operational use has zero heritage; reliability rests on TRL + UL only until the first successful use.
- **Tacit-knowledge underflow**: losing personnel reduces *effective* UL for queries/gates but never corrupts the stored world/private UL ground state; the reduction is recomputed from roster state.
- **Mission injection for an unknown domain**: an injection naming a domain not in the catalogue is deterministically rejected, never silently dropped.
- **Catch-up below frontier**: a trailing faction's cheaper research can approach but not exceed the World UL without its own frontier investment.
- **Astronaut over career-dose limit mid-training**: career limits are checked on every dose update; an astronaut crossing a limit leaves the ready pool deterministically.
- **Data version vs saved research state**: saved programs/ULs reference tech-tree and domain ids; loading against a different research-data version fails actionably (FA-01 pinning + FA-02 catalogue-hash pattern extended to research data).
- **Parallel programs to one technology**: two programs targeting the same technology are allowed (the parallel-approach de-risk); heritage and TRL track per program, and the first to mature defines the usable technology.

## Requirements *(mandatory)*

IDs are FR-RESP-###. Umbrella traceability (FR-RES-###, FR-CRW-###) inline. All numeric content
(UL thresholds, TRL costs/min-times/facility needs, reliability curves, overrun and breakthrough
rates, trait modifiers, dead-end/seed parameters, tide rates) lives in schema-validated data files
with `source` provenance (Principles I, V); the engine reads it — no magic numbers in code (Principle II).

### Two-track model & knowledge domains (FR-RES-001/002/004)

- **FR-RESP-101**: The system MUST model **Knowledge Domains** (the full A1–A17 set) each as a continuous **Understanding Level (0–100)** with documented **diminishing returns** at high UL and **synergy bonuses** across coupled domains, all from sourced data. *(FR-RES-001/002)*
- **FR-RESP-102**: Domain UL MUST **gate** engineering-program availability and set each program's **risk floor**; the gating domain(s) and threshold(s) MUST be reportable. *(FR-RES-002)*
- **FR-RESP-103**: **Research Points (RP)** MUST be generated from funded scientists, facilities and instruments and spent by Research Programs to raise domain UL; a portfolio allocation command MUST split RP across active programs with efficiency multipliers for staffing quality, domain mismatch and facility bottlenecks. *(FR-RES-003)*
- **FR-RESP-104**: **Missions** MUST inject UL directly into named domains via a declared injection interface (flight/surface data advancing geoscience/astrobiology/flight-science domains beyond labs); injections are journaled commands with class/quality payloads. *(FR-RES-004)*

### TRL, programs & test campaigns (FR-RES-005/006/007)

- **FR-RESP-201**: **Engineering Programs** MUST advance a **Technology** through **TRL 1–9**; each TRL step MUST carry a sourced **cost**, a **minimum duration** that cannot be compressed below a floor (schedule-compression penalty), and a **facility requirement**; progress MUST be S-curved and gated by the relevant domains' UL. *(FR-RES-005)*
- **FR-RESP-202**: A technology MUST be **flyable only at TRL ≥ 6** with steep reliability penalties easing through 7–9; **reliability** MUST be exposed as a **scalar per-use success probability ∈ [0,1]**, computed from a documented function of TRL, accumulated flight-units and relevant domain UL, and reported alongside its raw inputs so consumers may layer duration/phase effects. The maturity→reliability model lives in this slice. *(FR-RES-005; clarified 2026-06-13)*
- **FR-RESP-203**: Program **cost and schedule MUST be estimated with uncertainty (P50/P80)** and realised with variance influenced by TRL-jump size, domain-UL margin, staffing, facility adequacy and political interference (the last as an opaque caller-supplied modifier this slice reads). *(FR-RES-006)*
- **FR-RESP-204**: **Test campaigns** MUST be simulated: a failure costs money/schedule but **injects UL**; the model MUST distinguish failure-that-teaches from a dead-end signal (repeated failure without UL growth); spectacular flight-test failures MUST emit an event carrying a political/PR-eligible payload (consumed by FA-09). *(FR-RES-007)*

### Dead ends, breakthroughs, leapfrogging (FR-RES-008/009/010)

- **FR-RESP-301**: The per-game **seed** MUST designate some engineering approaches as **dead ends within TRL bands**, surfaced by advance hints (rising risk index, stalled error bars, repeated failures without UL growth) before confirmation; **parallel-approach** pursuit MUST be supported as de-risking; every **capability category** MUST retain ≥1 viable path in **every** seed — guaranteed **by construction** (the seeding algorithm never closes a category's last viable path), not by post-hoc rejection. *(FR-RES-008; clarified 2026-06-13)*
- **FR-RESP-302**: **Breakthroughs** MUST accrue from sustained **basic-science** investment via hidden insight pressure with seeded thresholds, delivering one of: a tech-cluster discount, an early branch unlock, or a hidden-path reveal past a presumed dead end; cadence MUST be rare (order of once per ~8–15 years for a heavily-invested domain), basic-science-weighted (applied-only rarely triggers), and announced with a sourced Sojournal reference. *(FR-RES-009)*
- **FR-RESP-303**: **Leapfrogging** MUST be possible by over-investing basic science to satisfy a higher tier's prerequisites via UL rather than intermediate products, at higher cost/risk and without inherited heritage. *(FR-RES-010)*

### Global science tide (FR-RES-011)

- **FR-RESP-401**: Each domain MUST have a **World UL** advancing from all factions' aggregate activity plus an exogenous baseline; a faction's private UL MUST equal World UL plus its lead/lag. *(FR-RES-011)*
- **FR-RESP-402**: Factions MUST be able to **publish** (raise World UL + prestige-eligible event, lose exclusivity) or **hold/patent** (retain lead, slower tide); trailing factions MUST research known ground more cheaply than the frontier per a documented catch-up model. *(FR-RES-011)*
- **FR-RESP-403**: The slice MUST expose declared **licence / partner / buy-in interfaces** that grant the documented TRL and IP credit and share TRL progress; the **monetary settlement** of these is FA-06's (this slice moves knowledge/heritage credit, not money). *(FR-RES-011; clarified 2026-06-13)*

### Personnel (FR-RES-012)

- **FR-RESP-501**: Personnel MUST be modelled as managed assets — scientists, engineers, programme managers, mission controllers, diplomats/lobbyists (and astronauts, FR-RESP-601) — each with a discipline/skill rating and sourced **traits** (at minimum Visionary, Closer, Maverick, Safe Hands) affecting low-TRL vs qual performance, breakthrough odds, overrun variance and reliability. *(FR-RES-012)*
- **FR-RESP-502**: **Recruitment, poaching (with a relations-cost signal), multi-year training, morale, retention and aging** MUST be modelled as deterministic roster transitions; staffing quality, domain mismatch and facility bottlenecks MUST drive the RP/DE efficiency multipliers of FR-RESP-103. *(FR-RES-012)*
- **FR-RESP-503**: Losing key personnel MUST reduce a faction's **effective UL** in their niche domain (**tacit-knowledge loss**), recomputed from roster state without corrupting the stored ground UL. *(FR-RES-012)*

### Astronaut career pipeline (FR-CRW-007, FA-05 half)

- **FR-RESP-601**: The astronaut pipeline MUST cover **selection → training (multi-year, facility/analog-gated) → assignment readiness → aging**, holding running **career dose** and **health** budgets and morale; readiness and career state MUST be queryable. *(FR-CRW-007)*
- **FR-RESP-602**: The slice MUST declare the **FA-08-facing interface** by which in-mission dose/health/psychological deltas update an astronaut's career state; crossing a documented career limit MUST deterministically remove an astronaut from the ready pool. (In-mission ECLSS and acute physiology are FA-08; clarified 2026-06-13.) *(FR-CRW-007)*

### Tech-tree data & heritage (FR-RES + design §2)

- **FR-RESP-701**: The tech tree MUST ship as **web-shaped, schema-validated data**: the full Knowledge-Domain set (A1–A17) with synergy links, and engineering Technology nodes each carrying start-TRL, domain-UL floors, tech prerequisites (incl. cross-branch gates), capability category, and a **mandatory `source`**; CI MUST reject any node without a source and MUST verify ≥1 viable path per capability category. *(Principle I; design §2 L244–L253)*
- **FR-RESP-702**: A **representative sourced subset** of engineering nodes MUST ship that exercises every mechanic (cross-branch gates, ≥2 paths per category, seeded dead ends, breakthroughs, leapfrog, heritage discounts); the full node population is a documented data expansion behind the same schema. *(clarified 2026-06-13)*
- **FR-RESP-703**: **Flight Heritage** MUST accrue from operational-use events (driven by FA-04+), raising a technology's reliability asymptotically toward a per-tech ceiling and **discounting derivative programs** (an uprated proven technology starts partway up the TRL ladder). *(design §2 Heritage)*

### Queries & integration

- **FR-RESP-801**: A read-only query surface (kernel `with_slice` + pure functions, the FA-02 pattern) MUST answer, faction-scoped: technology **maturity** (TRL, scalar per-use reliability ∈ [0,1] + its raw inputs, flyability ≥ TRL 6), **flight heritage** (units, ceiling, derivative discounts), **domain understanding** (private + world UL, gating status), **program status** (TRL, P50/P80-vs-actual, risk/dead-end index), and **personnel/roster** summaries (counts, traits, astronaut readiness). No query may return another faction's private state. *(FR-RES integration; FA-04/06/09/10 seam)*
- **FR-RESP-802**: The module MUST conform to the kernel module contract (owned research/personnel slice; declared streams for dead-end/breakthrough seeding, overrun and test-outcome variance; journaled commands for allocate / start-program / set-publish-policy / hire / poach / train / inject-UL / register-heritage / publish / licence-or-partner — **TRL advancement is step-driven from allocated DE, not a command**; events for breakthrough, dead-end-confirmed, test-failure, program-milestone, TRL-advance, publish) and integrate via ModulePayload with no kernel code change anticipated.

### Determinism, validation & performance

- **FR-RESP-901**: All research data MUST pass schema + source-presence validation in CI (`validate-data` extended): domain set, synergy links, node prerequisites and cross-branch gates resolve, and reliability/overrun/breakthrough parameters are present and sourced. A CI **reachability sweep** over a sampled seed set MUST verify the FR-RESP-301 constructive guarantee (every capability category keeps ≥1 viable path), catching any algorithm regression.
- **FR-RESP-902**: The entire research/personnel evolution MUST be **seed-deterministic** (kernel double-run, save/round-trip and replay gates pass with the module installed); saved state MUST pin and verify the research-data version (FA-01 pinning + FA-02 hash pattern).
- **FR-RESP-903**: With a full roster, the documented domain set and the shipped node subset under a representative allocation, the kernel performance envelope MUST hold (umbrella SC-003: ≥1 sim-year/min on the reference machine) and the read-only research queries MUST return within 50 ms.

### Key Entities

- **Knowledge Domain**: id, name, UL (private per faction + world), diminishing-returns + synergy parameters, sourced.
- **Understanding Level**: continuous 0–100 value; private (per faction) over a world baseline.
- **Technology**: id, capability category, current TRL, reliability inputs, heritage, source; the unit FA-04 consumes.
- **Engineering Program**: target technology, owning faction, TRL progress, P50/P80 cost/schedule, assigned lead/staff, facility needs, risk/dead-end index, parallel-approach tag.
- **Test Campaign**: per-TRL-step outcome process (success / failure-that-teaches), cost/schedule realisation, UL injection.
- **Dead-End Seeding**: per-game, per-(approach, TRL band) viability fixed by the seed; capability-category reachability invariant.
- **Breakthrough**: seeded threshold + earned insight pressure → cluster discount | early unlock | hidden-path reveal; rare; sourced announcement.
- **World Science Tide**: per-domain World UL, baseline + aggregate; publish/hold; catch-up discount.
- **Personnel**: scientists/engineers/PMs/controllers/diplomats with skill, traits, morale, age; recruit/poach/train/retire.
- **Astronaut**: personnel sub-type with career pipeline state (training, readiness, career dose/health budgets); FA-08 interface.
- **Flight Heritage**: per-technology operational-use record → reliability ceiling + derivative discount.
- **Research Query Surface**: pure read-only functions over a snapshot (maturity, heritage, understanding, program status, personnel), faction-scoped.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001 (Science gates engineering)**: 100% of engineering programs are unavailable below their domain-UL floor and available above it; UL rises with diminishing returns and synergy from sourced parameters; mission injections raise named domains beyond lab-only growth — all verified headlessly and bit-identically per seed.
- **SC-002 (TRL is the spine)**: every TRL step respects its cost, minimum-duration floor and facility/UL gate; no technology is flyable below TRL 6; reliability tracks TRL + flight-units + UL; heritage raises reliability and discounts a derivative — demonstrated in the test suite.
- **SC-003 (The tree is alive and fair)**: across ≥100 seeds, some approaches are dead ends within TRL bands (hinted before confirmation) **and** every capability category retains ≥1 viable path in every seed; breakthroughs occur only with sustained basic-science investment at the documented rare cadence; leapfrogging reaches a higher tier via UL.
- **SC-004 (The tide bounds leads)**: World UL advances from aggregate + baseline; publish accelerates it and emits a prestige-eligible event; trailing factions research cheaper than the frontier; catch-up cannot exceed World UL without frontier investment.
- **SC-005 (People matter)**: traits shift the documented program outcomes; hire/poach/train/age transitions are deterministic; efficiency multipliers respond to staffing/mismatch/facility bottlenecks; disbanding a team reduces effective niche UL — all verified.
- **SC-006 (Astronaut pipeline)**: candidates advance select→train→ready under the documented facility/time gate; career dose/health budgets accumulate deterministically and limit the ready pool; the FA-08 interface round-trips dose/health deltas.
- **SC-007 (Integration & determinism)**: the maturity/heritage/understanding/program/personnel query surface is pure and faction-scoped; the module passes conformance and the kernel double-run / round-trip / replay gates; saves pin and verify the research-data version.
- **SC-008 (Plausibility & performance)**: 100% of tech-tree nodes and research parameters carry sources and pass CI validation; the full roster + node subset holds the ≥1 sim-year/min envelope; research queries return < 50 ms on the reference machine.

## Assumptions

- **Funding & facilities are opaque caller inputs**: this slice takes funded-staff counts, facility-capability descriptors and an RP/DE budget as caller-supplied inputs (a default in scenarios), exposing efficiency multipliers and facility-requirement gates; FA-06 binds real facilities, money and markets later without data migration — the same honest-seam pattern as FA-02's research-gate stand-in and FA-03's opaque faction ids.
- **Faction identity is opaque**: per-faction UL, programs, personnel and heritage key on caller-supplied faction ids; FA-09 binds real factions later.
- **Political interference is an input**: the directed-program/over-constraint modifier (FR-RESP-203) is read as an opaque caller value now; FA-09 supplies it later.
- **Heritage is event-fed**: operational-use/heritage events are emitted by FA-04+ flights; this slice defines the interface and the reliability/derivative effects, and scenarios drive synthetic heritage events for testing.
- **Mission injection is event-fed**: UL injections arrive as journaled commands from mission slices; scenarios drive synthetic injections for testing.
- **Numbers are data**: UL thresholds, TRL costs/min-times, reliability/overrun/breakthrough/tide parameters, trait modifiers and dead-end seeds live in sourced `data/research/*` and `data/tech/*` files; tuning is data, not code.
- **Public-data sourcing**: tech-tree nodes cite real programs/literature (NASA/ESA/JAXA/Roscosmos programs, peer-reviewed concepts) per Principle I; speculative items live behind Breakthroughs as clearly-flagged if-discovered entries (design L254–L256).
- **Reference hardware & envelope** inherit FA-01/FA-02 definitions.

## Out of Scope (this slice)

- Vehicle/propulsion performance models and the vehicle designer (FA-04) — this slice exposes maturity/heritage; FA-04 turns them into thrust/Isp/mass/reliability.
- Funding, six-currency economy, markets, contracts, facilities-as-assets, and the monetary settlement of licensing/partnerships (FA-06).
- In-mission life-support closure and acute crew physiology/radiation/psychology during flight (FA-08) — this slice owns the career roster up to the mission boundary and the FA-08 feedback interface.
- Missions, instruments, launches and the flights that inject UL and accrue heritage (FA-04+) — this slice defines the interfaces they drive.
- Politics, prestige, public mood, firsts and AI-faction behaviour (FA-09) — this slice emits the events they consume.
- All UI (FA-10), including the R&D screens, tech-graph view, portfolio sliders and personnel boards.
