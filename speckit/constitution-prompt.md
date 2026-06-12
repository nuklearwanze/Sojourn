# `/speckit.constitution` prompt

> A full constitution is already provided at `.specify/memory/constitution.md`. Use this only if
> you want the agent to (re)generate it. Run in Claude Code as:
> `/speckit.constitution <paste the block below>`

```
Create the governing constitution for "Sojourn", a hard-science, no-combat 4X strategy game about
exploring, industrialising and settling the Solar System from 2026 to 2126 (inspirations: Aurora
4x, The Expanse). Make these principles non-negotiable and enforceable in /speckit.analyze and
code review:

1. Scientific plausibility is law: every technology, propulsion method, resource process,
   trajectory, body and number must be grounded in flown hardware, a funded program, or peer-
   reviewed literature, and every data entry must carry a `source` field. No reactionless drives,
   no FTL, no "top speed", no balance-only invented scarcity. Speculative-but-plausible tech is
   allowed only behind the in-game Breakthrough mechanic and must be flagged.
2. Physics is the authoritative simulation: n-body orbital mechanics with relevant perturbations,
   the rocket equation, power/thermal budgets, and life-support closure are simulated as real
   state. A numerical propagator is the source of truth; planning aids are approximations
   reconciled against it. The physics engine holds no per-tech magic numbers — it reads sourced
   data files.
3. Deterministic, reproducible core: fixed-timestep, seeded PRNG, no wall-clock/unseeded
   randomness in the core; replayable from an event log. Non-determinism is release-blocking.
4. Simulation core fully decoupled from presentation and runnable headless; UI holds no game
   logic; every system is headlessly testable.
5. Data-driven content, code-driven mechanics: content (bodies, sites, tech nodes, constants,
   milestones, events) lives in schema-validated, sourced data files; rebalancing must not
   require recompiling logic; data validated in CI.
6. Research is a modelled process, not a purchase: two-track Science-Understanding → Engineering-
   TRL model with test campaigns, cost/schedule uncertainty, dead ends, failures-that-teach, rare
   breakthroughs, leapfrogging, and a global science tide.
7. The tyranny of mass and delta-v: mass/delta-v are the dominant constraints surfaced in
   gameplay and UI; money is a proxy for mass-to-orbit and every cost traces to a physical basis;
   crewed missions are materially harder than robotic; ISRU/reuse/depots matter because they
   relax mass/delta-v, not via arbitrary bonuses.
8. Educational honesty: an in-game encyclopedia explains the real science with references and
   stays consistent with the simulation; no misinformation presented as fact.
9. Scope discipline: v1.0 has no weapons, combat, sabotage, or alien civilisations. Microbial/
   chemical life may be modelled as a seeded scientific question but is never an actor. Reserved
   features must not have logic built in v1.0.

Engineering constraints: SI units only; explicit seeded determinism; data provenance enforced in
CI; defined performance budgets for a 3000+ body simulation and virtualised large tables;
accessibility (colour-blind-safe, scalable text, keyboard nav, 1280x720→4K); deterministic
versioned round-trip-tested saves; tech stack is free as long as it supports a headless
deterministic data-driven core and a 2D data-dense UI.

Quality gates: spec-driven workflow with mandatory clarification for physics/research/economy/
world features; required automated tests including physics validation against analytic cases
(Hohmann delta-v, two-body periods, flybys), a determinism double-run test, data schema+source
validation, and save/load round-trip; a realism-review gate citing sources for any change to a
plausibility-bearing value; /speckit.analyze must check source presence and the no-combat/no-
aliens scope. Include a semantic-versioning governance section and a Sync Impact Report comment.
```
