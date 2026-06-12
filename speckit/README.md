# Sojourn × Spec Kit — How to Drive Development in Claude Code

This folder turns the design docs in `../design/` into a Spec-Driven-Development workflow using
[GitHub Spec Kit](https://github.com/github/spec-kit) inside Claude Code.

## 0. One-time setup

```bash
# In an empty directory (or your repo root)
uvx --from git+https://github.com/github/spec-kit.git specify init .
# choose "Claude Code" as the agent when prompted
```

Copy `../design/` and this `speckit/` folder into the project, then commit. Spec Kit installs the
`/speckit.*` slash commands into Claude Code.

> Recommended full workflow (use the quality gates):
> `/speckit.constitution → /speckit.specify → /speckit.clarify → /speckit.checklist →
>  /speckit.plan → /speckit.tasks → /speckit.analyze → /speckit.implement`

## 1. Establish the constitution

A complete constitution already exists at `.specify/memory/constitution.md`. To (re)generate or
update it from intent, run `/speckit.constitution` with the prompt in
[`constitution-prompt.md`](./constitution-prompt.md). Otherwise, keep the provided file — it is
authoritative.

## 2. Specify the game (the big one)

Run **`/speckit.specify`** with the prompt in [`specify-prompt.md`](./specify-prompt.md) (also
reproduced at the bottom of this file). It focuses on *what* and *why*, not tech stack, and points
the agent at the design docs for detail.

## 3. Clarify, then plan in slices

Sojourn is too large for one plan. After `/speckit.specify`, run `/speckit.clarify`, then plan
**feature by feature** using the slice prompts in [`feature-prompts.md`](./feature-prompts.md).
Recommended build order (each is its own specify→plan→tasks→implement cycle on its own branch):

1. **Sim core & time** — deterministic fixed-timestep engine, seeded PRNG, headless harness,
   save/load, event-driven time-warp with interrupts.
2. **Astrodynamics** — n-body propagator + patched-conic planner, manoeuvre nodes, porkchop,
   low-thrust arcs, flybys; validated against analytic cases.
3. **World data model** — bodies/sites/ephemeris from real data, belief-state layer, Sojournal.
4. **Vehicle designer & propulsion model** — component composition, mass/Δv/power/thermal/
   reliability, realism guards.
5. **Research system** — two-track Science/Engineering, TRL gates, dead ends, breakthroughs,
   leapfrogging, global tide, personnel.
6. **Economy & logistics** — six currencies, budgets/cash, markets, contracts, ISRU economics,
   delta-v-addressed resource ledger, facilities.
7. **Bases & construction** — sites, modules, emergent properties, ISRU plants.
8. **Life support & crew** — closure model, radiation/health/psych, crew pipeline, EDL.
9. **Politics, events & milestones** — relationships, mood, policy, planetary protection,
   astrobiology evidence system, milestone race, AI world.
10. **UI/UX** — the data-dense 2D shell and all screens, reading from the headless core.

Keep each slice honest to the constitution; run `/speckit.analyze` before `/speckit.implement`.

## 4. The starter prompt (copy-paste)

The exact one-liner to begin development is in [`specify-prompt.md`](./specify-prompt.md).
