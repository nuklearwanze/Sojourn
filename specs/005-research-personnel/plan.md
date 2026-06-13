# Implementation Plan: Research & Personnel (FA-05)

**Branch**: `005-research-personnel` | **Date**: 2026-06-13 | **Spec**: `specs/005-research-personnel/spec.md`
**Input**: Feature specification from `/specs/005-research-personnel/spec.md`

## Summary

Build `sojourn-research`, the module that makes research a *modelled process, not a purchase*: the
two-track engine where **Track A Science** raises continuous **Understanding Levels** across the
A1–A17 knowledge domains (diminishing returns + cross-domain synergy) and **Track B Engineering**
advances **Technologies** through **TRL 1–9** via domain-UL-gated **test campaigns** with cost/
schedule uncertainty and overruns. On top of that spine it models the things that make the tree feel
earned: per-seed **dead ends** within TRL bands (with reachability guaranteed *by construction* —
no seed bricks a capability category), **failures-that-teach**, rare seeded-and-earned
**breakthroughs**, **leapfrogging**, and the global science **tide** (World UL, publish-vs-hold,
cheaper catch-up). It carries the **personnel roster** (scientists, engineers, programme managers,
controllers, diplomats, and astronauts as managed assets) with traits, recruitment/poaching/training,
morale/aging and tacit-knowledge loss, plus the **astronaut career pipeline** up to the mission
boundary. It exposes a pure, read-only **maturity / heritage / understanding** query surface — the
contract FA-04 (vehicle/propulsion) consumes for capability and a **scalar per-use reliability**, FA-06
prices, and FA-09 turns into prestige.

The slice is a sibling module on the FA-01 kernel, **depending only on `sojourn-core`** (research is
physics-independent — FA-04 will depend on *it*, not the reverse). It needs **no kernel change**
(commands route through FA-02's established `ModulePayload`/`on_command`; new event classes are
data-registry additions). All numbers — UL curves, TRL costs/min-times, overrun/breakthrough/tide
rates, reliability curves, trait modifiers, dead-end seeds — are sourced data the engine reads (no
magic numbers; Principle II/V/VI). Funding, facilities and the monetary side of licensing are opaque
caller inputs (FA-06 binds them later), the same honest-seam pattern as FA-02's research gate and
FA-03's opaque faction ids.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01/02/03).
**Primary Dependencies**: `sojourn-core` (kernel contracts), `serde`/`postcard` (slice + DTO serialization), `libm` (all transcendentals — FA-01 float policy), `ron` (data), `thiserror`, `rand_core` (the kernel stream trait). **No astro/world dependency** (research is independent of physics and world data). No new third-party math/stats deps: the UL curves, S-curve progress, P50/P80 draws, insight pressure and seeded outcomes are in-crate on the kernel's seeded streams.
**Storage**: Data files only — `data/research/` (domains + synergy, RP/DE generation, overrun/breakthrough/tide/reliability params, trait definitions, dead-end-seeding params) and `data/tech/` (the web-shaped tech-tree: A1–A17 domains + a representative sourced engineering-node subset + capability-category → candidate-path map), all carrying `source` provenance and validated in CI.
**Testing**: `cargo test` (unit + integration: UL gating, TRL/test campaigns, dead-end+reachability, breakthroughs, leapfrog, tide, personnel, astronaut pipeline, the query surface); kernel conformance (`conformance --module research`); harness determinism gates (verify/roundtrip/replay with the module installed); a CI **reachability sweep** over sampled seeds; `validate-data` extended to research/tech; criterion bench for the multi-faction tick budget + query latency.
**Target Platform**: Same as workspace — Windows/Linux/macOS desktop; per-platform determinism.
**Project Type**: Library crate (`crates/sojourn-research`) implementing the FA-01 `SimModule` contract + a public read-only maturity/heritage/understanding query API; harness/bench/data extensions.
**Performance Goals**: SC-008 — full roster + the documented domain set + the shipped node subset across multiple factions holds the kernel envelope (≥1 sim-year/wall-minute on the reference machine); read-only research queries < 50 ms.
**Constraints**: Full kernel determinism (ordered `BTreeMap` stores, libm-only, declared streams for dead-end seeding / breakthrough / overrun / test-outcome variance, no wall-clock); seed fixes dead ends + breakthrough thresholds; **capability-category reachability guaranteed by construction**; queries pure (between ticks, no mutation/streams); RP/DE/UL are dimensionless model quantities sourced as engineering defaults; every tech-tree node carries a `source`; no combat/weapons nodes (B5.5 Orion locked out per design).
**Scale/Scope**: A1–A17 domains; a representative sourced engineering-node subset (~30–60 nodes exercising every mechanic, full population a data expansion); multiple factions (faction-parametric; FR-POL-007 — all factions run the same engine, FA-09 drives the AI later); tens–low-hundreds of personnel per faction; century horizons.

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS | Every tech-tree node, domain, UL curve, TRL cost/time, reliability/overrun/breakthrough/tide parameter and trait modifier lives in `data/research/*` or `data/tech/*` with a `source`; `validate-data` (extended) fails CI on any missing source; speculative items live behind Breakthroughs as clearly-flagged if-discovered entries (design L254–256). |
| II. Physics authoritative / no magic numbers | PASS | No model constant is hard-coded — the engine reads UL/TRL/reliability/overrun/breakthrough numbers from data; the research process is mechanics in code, content in data. |
| III. Deterministic core | PASS | Module conforms to the kernel contract: named streams for dead-end seeding, breakthrough thresholds, overrun and test-outcome variance; ordered stores; libm-only; double-run/roundtrip/replay/conformance gates run with the module. |
| IV. Headless / decoupled | PASS | Pure library module + read-only query functions; zero UI deps; everything driven and audited via harness scenarios. |
| V. Data-driven content | PASS | Domains, tech nodes, all tuning parameters and trait definitions are schema-validated data; new event classes use the existing registry (no kernel code change). |
| VI. Research a modelled process (NON-NEGOTIABLE — this slice is the embodiment) | PASS | Implements the two-track model in full: UL-gated TRL programs, test campaigns, dead ends with parallel-approach mitigation, failures-that-teach, rare seeded+earned breakthroughs, leapfrogging, the global tide. Research is **not** reducible to spend-points-and-unlock (FR-RESP-101/201/301/302). |
| VII. Tyranny of mass / Δv | N/A (this slice) | Research feeds the constraint via maturity/heritage; FA-04 turns it into vehicle performance. |
| VIII. Educational honesty | PASS | Breakthroughs are announced as discoveries with a sourced Sojournal reference; every node cites real programs/literature; nothing presents misinformation as fact. |
| IX. No combat/aliens | PASS | No weapons/combat nodes; the design's Orion-type pulse propulsion (B5.5) is intentionally locked out and present only as a historical Sojournal entry. |
| Engineering constraints | PASS | Deterministic seeded streams; performance budgets tracked by bench; research-data version pinned in saves (extends FA-02's hash pattern); fully offline. |
| **Kernel contract** | NONE (no amendment) | Commands route through FA-02's `Command::ModulePayload` → `SimModule::on_command`; new event classes (`breakthrough`, `dead-end-confirmed`, `test-failure`, `program-milestone`, `trl-advance`, `publish`) are data-registry additions. No kernel code change. |

**Post-Phase-1 re-check (2026-06-13)**: design artifacts introduce no violations; no kernel
amendment; the slice embodies Principle VI. Gate remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/005-research-personnel/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R16)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── maturity-queries.md      # FR-RESP-801: the read-only maturity/heritage/understanding surface (FA-04/06/09/10 seam)
│   ├── research-commands.md     # FR-RESP-802: commands (via ModulePayload) + event classes + streams
│   ├── tech-tree-data.md        # FR-RESP-701/702/901: tech-tree + research-param data format, sourcing, reachability, validation
│   └── crew-interface.md        # FR-RESP-601/602: the FA-08-facing astronaut dose/health career-feedback interface
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged this slice (new event classes are data only)
├── sojourn-research/            # THIS SLICE — pure library, SimModule implementor (depends on sojourn-core only)
│   ├── Cargo.toml               # deps: sojourn-core, serde, postcard, libm, ron, thiserror, rand_core
│   └── src/
│       ├── lib.rs               # public surface: ResearchModule, ResearchCommand, queries
│       ├── ids.rs               # FactionId, DomainId, TechId, ProgramId, PersonId
│       ├── tree.rs              # tech-tree data load + validation: A1–A17 domains, engineering nodes, cross-branch gates, capability categories
│       ├── domains.rs           # Understanding Levels: diminishing-returns + synergy growth; UL gating
│       ├── rp_de.rs             # RP/DE generation from staff/facilities/instruments; portfolio allocation + efficiency multipliers
│       ├── programs.rs          # engineering programs: TRL steps, S-curve, P50/P80 cost/schedule, overruns, schedule-compression floor
│       ├── campaigns.rs         # test campaigns: seeded success/failure-that-teaches, UL injection, dead-end signal
│       ├── seeding.rs           # per-game dead-end seeding with CONSTRUCTIVE capability-reachability; breakthrough thresholds
│       ├── breakthrough.rs      # insight-pressure accumulators (basic-science-weighted), triggers, leapfrog
│       ├── tide.rs              # World UL, publish/hold, catch-up discount, licence/partner/buy-in interfaces (knowledge credit, no money)
│       ├── reliability.rs       # scalar per-use reliability from TRL+flight-units+UL; Flight Heritage + derivative discounts
│       ├── personnel.rs         # roster, traits, recruit/poach/train/age/morale, tacit-knowledge → effective-UL
│       ├── astronaut.rs         # career pipeline (select→train→ready→age), dose/health budgets, FA-08 feedback interface
│       ├── query.rs             # ResearchSnapshot + pure read-only query fns (FR-RESP-801)
│       └── module.rs            # SimModule impl: manifest, slice, step (time-stepped), on_command, publish, save/load_slice
│   └── tests/                   # domains.rs, programs.rs, campaigns.rs, seeding.rs (+reachability), breakthrough.rs,
│                                # tide.rs, personnel.rs, astronaut.rs, queries.rs, conformance.rs
├── sojourn-harness/             # + `research` scenario flag (install module), validate-data research, conformance --module research, reachability sweep, bench
data/
├── kernel/event-classes.ron     # + breakthrough, dead-end-confirmed, test-failure, program-milestone, trl-advance, publish (data-only)
├── research/
│   ├── domains.ron              # A1–A17 with synergy links + diminishing-returns params (sourced)
│   ├── params.ron               # RP/DE generation, overrun, breakthrough, tide, reliability, trait modifiers, dead-end-seeding params (sourced)
│   └── traits.ron               # personnel trait definitions (Visionary/Closer/Maverick/Safe Hands…) with modifiers + sources
└── tech/
    ├── tech-tree.ron            # representative sourced engineering nodes (start-TRL, UL floors, prereqs, cross-branch gates, capability category, source)
    └── capability-categories.ron# category → candidate-path map (the reachability invariant's domain)
scenarios/                       # + research_understanding.ron, research_program.ron, research_tide.ron
```

**Structure Decision**: `sojourn-research` is a third sibling module crate, parallel to
`sojourn-astro` and above only the kernel — it has **no** dependency on physics or world data, since
research is a self-contained process; the dependency arrow runs *toward* it (FA-04 will read its
maturity/heritage). Like FA-03 it needs no kernel change: commands ride `ModulePayload` and new event
classes are data-registry additions. Unlike FA-03's event-driven world module, the research module is
**time-stepped** at a fixed cadence (programs advance, ULs grow, insight accrues, the tide rises,
personnel age each step) — research has genuine per-tick dynamics but no state-driven step escalation,
so a single fixed cadence suffices. The maturity/heritage/understanding surface is plain public crate
API (pure functions over a snapshot), the same IPC-serializable seam FA-04/06/09 and the Tauri host
call between ticks.

## Complexity Tracking

No constitution violations to justify — table intentionally empty. The slice is large (it embodies a
whole pillar) but introduces no architectural deviation: one module crate, the established command/
event/query patterns, no kernel amendment.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
