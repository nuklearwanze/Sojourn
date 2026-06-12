<!--
Sync Impact Report
- Version: (none) → 1.0.0
- Ratified: 2026-06-12
- Modified principles: initial adoption (all new)
- Added sections: Core Principles (I–IX), Engineering Constraints, Development Workflow & Quality Gates, Governance
- Removed sections: none
- Templates requiring updates: ✅ plan/spec/tasks consume this file at runtime (no template edits needed at adoption)
- Follow-up TODOs: none
-->

# Sojourn Constitution

Sojourn is a hard-science, no-combat 4X strategy game about exploring, industrialising and
settling the Solar System, 2026–2126. This constitution defines the non-negotiable principles
that govern every specification, plan, task and line of code. When any other document conflicts
with this one, this document wins.

## Core Principles

### I. Scientific Plausibility Is Law (NON-NEGOTIABLE)
Every technology, propulsion method, resource process, trajectory, body, and number in the game
MUST be grounded in flown hardware, a funded program, or peer-reviewed literature. No reactionless
drives, no faster-than-light, no "top speed," no artificial scarcity invented purely for balance.
Every quantitative entry in the game's data files MUST carry a `source` field citing its basis.
Code review and `/speckit.analyze` MUST reject any data node lacking a source. "Future" tech is
permitted ONLY when it is a real funded program or a published plausible concept; speculative
items with no citable basis MUST be gated behind the in-game Breakthrough system and clearly
flagged as such.
Rationale: plausibility is the product. It is the reason this game exists and the promise to the
player; violating it is a defect, not a balance choice.

### II. Physics Is the Authoritative Simulation
Orbital mechanics (n-body with relevant perturbations), the rocket equation, power and thermal
budgets, and life-support closure MUST be simulated as the real game state, never approximated as
flavour. The numerical propagator is the source of truth; planning aids (patched-conics,
porkchop plots) are approximations that MUST be reconciled against it. The physics engine MUST
contain NO per-technology magic numbers — it reads all constants from sourced data files so
realism is auditable and tunable without code changes.
Rationale: the simulation's correctness is the foundation everything else stands on.

### III. Deterministic, Reproducible Core
The simulation core MUST be deterministic: a given seed plus a given sequence of player decisions
MUST always produce identical state. Fixed-timestep integration; no wall-clock or unseeded
randomness in the core; all stochastic outcomes (dead ends, breakthroughs, anomalies,
astrobiology ground truth) MUST derive from the per-game seed. The core MUST be replayable from
its event log. Any non-determinism in the simulation core is a release-blocking bug.
Rationale: determinism enables testing, reproducible bug reports, fair competition, and trust.

### IV. Simulation Core Is Decoupled From Presentation
The deterministic simulation core MUST run headless with no dependency on the UI, rendering, or
input layers. The UI reads state and issues commands through a defined boundary; it contains NO
game logic. Every gameplay system MUST be testable headlessly without a renderer.
Rationale: separation enables automated testing, determinism, and a clean data-dense UI that can
evolve independently.

### V. Data-Driven Content, Code-Driven Mechanics
Game content — bodies, sites, tech-tree nodes, propulsion/vehicle/economy constants, milestones,
events — MUST live in versioned, schema-validated data files (with sources per Principle I), not
hard-coded. Mechanics live in code; content lives in data. Changing a number to rebalance or
correct realism MUST NOT require recompiling logic. Data files MUST be validated against schemas
in CI.
Rationale: auditability, moddability, and the ability to track the game's realism against reality
as it changes.

### VI. Research Is a Modelled Process, Not a Purchase
The research systems MUST implement the two-track model (Science Understanding Levels →
Engineering programs advanced through TRL gates) including test campaigns, cost/schedule
uncertainty, dead ends, failures-that-teach, rare breakthroughs, leapfrogging, and a global
science tide. Research MUST NOT be reducible to "spend points, wait, unlock." These are core
mechanics, not optional flavour.
Rationale: the realistic research process is a defining pillar of the game's identity.

### VII. The Tyranny of Mass and Delta-v
Mass and delta-v MUST be the dominant constraints surfaced throughout gameplay and UI. Money is a
proxy for mass-to-orbit; the player MUST always be able to trace any cost down to its physical
basis. Crewed missions MUST be materially harder than robotic ones (life support, radiation,
health, mass, abort). ISRU, reuse, depots and refuelling MUST be mechanically meaningful because
they relax the mass/delta-v constraint, not because of arbitrary bonuses.
Rationale: this is the central, felt tension that makes the game's decisions matter.

### VIII. Educational Honesty
The game MUST be able to teach the player real spaceflight. The in-game encyclopedia
("Sojournal") MUST explain the actual science behind each mechanic with references and MUST stay
consistent with the simulation. The game MUST NOT present misinformation as fact, even for
convenience or drama.
Rationale: honesty toward the player and the subject is a stated purpose of the product.

### IX. Scope Discipline — No Combat, No Aliens (v1.0)
v1.0 MUST NOT contain weapons, combat, sabotage, or alien civilisations. Microbial/chemical life
elsewhere in the Solar System MAY be modelled as a seeded scientific question, but discovered
life is a science object, never an actor or antagonist. Competitive pressure comes from the
milestone race, economics and politics. Reserved features (combat, aliens, interstellar) MUST NOT
have logic built in v1.0; they may be left as clearly-marked extension points only.
Rationale: the player defined this scope deliberately; scope creep here changes the game's genre.

## Engineering Constraints

- **Units:** SI units everywhere in simulation, data, and player-facing UI. No imperial units.
- **Determinism tooling:** fixed-timestep core; seeded PRNG threaded explicitly; no hidden global
  randomness or time-of-day dependence in the core.
- **Data provenance:** every data entry carries `source`; CI fails on missing/empty sources for
  plausibility-bearing fields.
- **Performance targets:** the core MUST sustain the full simulation (3,000+ catalogued bodies,
  large fleets, economy/logistics graph) at high time-warp on commodity hardware; UI tables with
  thousands of rows MUST virtualise. Define and track explicit frame-time and tick-time budgets.
- **Accessibility:** colour-blind-safe palettes, scalable text, full keyboard navigation, from
  1280×720 to 4K.
- **Saves:** deterministic, versioned, forward-migratable save format; a save MUST reproduce
  identical state on load (round-trip tested).
- **Tech-stack freedom:** the constitution does not mandate a language/engine; whatever is chosen
  MUST satisfy headless-core, determinism, data-driven content, and 2D data-dense UI requirements.

## Development Workflow & Quality Gates

- **Spec-driven:** features flow constitution → specify → clarify → plan → tasks → analyze →
  implement. Ambiguity MUST be resolved via `/speckit.clarify` before planning for any feature
  touching physics, research, economy, or world data.
- **Testing (required):**
  - Deterministic simulation systems MUST have automated tests, including **physics validation
    against known analytic cases** (Hohmann delta-v, two-body periods, simple flybys, rocket-
    equation identities) within defined tolerances.
  - Determinism MUST be enforced by a test that runs a seed+decision-script twice and asserts
    bit-identical state.
  - Data files MUST pass schema + source-presence validation in CI.
  - Save/load round-trip MUST be tested for state identity.
  - Headless integration tests MUST cover each major system (research, economy, spaceflight,
    world/events) independent of the UI.
- **Realism review gate:** any change adding or altering a plausibility-bearing value (propulsion
  performance, ISRU yield, body data, life-support closure, costs) requires a realism review
  citing sources; `/speckit.analyze` MUST check constitutional compliance including source
  presence and the no-combat/no-aliens scope.
- **Reviews:** every PR MUST verify compliance with the Core Principles relevant to its scope and
  state which principles it touched.

## Governance

This constitution supersedes other practices and documents for the Sojourn project. Amendments
require: a written rationale, an explicit version bump per the policy below, an update to the Sync
Impact Report at the top of this file, and migration notes when principles change in a breaking
way.

Versioning policy (semantic):
- **MAJOR:** removal or backward-incompatible redefinition of a principle or governance rule.
- **MINOR:** a new principle/section or materially expanded guidance.
- **PATCH:** clarifications and wording that do not change requirements.

Compliance: `/speckit.analyze` and code review are the enforcement points. Any feature that
cannot be implemented without violating a principle MUST be redesigned or escalated as a proposed
amendment — it MUST NOT be shipped in violation. Complexity that bends a principle MUST be
justified in writing or removed.

**Version:** 1.0.0 | **Ratified:** 2026-06-12 | **Last Amended:** 2026-06-12
