# GP-07 — Crew & Life Support · `/speckit` set (FA-18)

**Branch:** `019-crew-life-support` · **Design:** `gameplay/08-CREW-LIFE-SUPPORT.md` · **Depends:** GP-04, GP-06, GP-02

## /speckit.specify

```
/speckit.specify Put humans in space: crewing a craft or base becomes a life-support problem — consumables vs ECLSS closure, radiation dose, deconditioning, psychology, maintenance, EDL risk and the real possibility of loss of crew. Add the crew layer across Operations (S5) and Personnel (S8) plus storm and EDL interrupts. Authoritative design: gameplay/08-CREW-LIFE-SUPPORT.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles I, VII, VIII, V) — read them.

WHY: crewed missions must be materially harder and costlier than robotic ones (the tyranny of mass and Δv applied to keeping people alive).

Let the player assign astronauts to a craft or base, size its ECLSS and load consumables (a crewing intent gated by a core-computed preview of consumables + crew-time draw, projected dose rate, ECLSS closure and viability — the irreversible part being exposing crew to the mission's hazards); watch dose accrue against career limits and life-support state evolve with the slice's dose→REID curve, deconditioning/psychology accrual and ECLSS consumption; respond to a solar-particle-event interrupt by ordering crew to the storm shelter (with a dose-averted preview); perform maintenance (crew-time + spares) and resupply (mass); evaluate EDL suitability before a crewed descent with the per-body difficulty including the Mars gap (gated); and face graded consequences up to loss of crew, recorded for the political system. Astronaut careers (cumulative dose, health, eligibility) persist across missions; over-limit astronauts are grounded.

The dose/physiology/psych/hazard/EDL derivations stay in the crew slice — the UI displays and previews, never recomputes. SPE storms arrive as events surfaced here as interrupts. Intent expansion lives in the orchestration crate.

Acceptance: crewing draws consumables + crew-time and is gated with a core-computed dose/viability preview; dose/physiology/psych/ECLSS evolve and display; SPE storms interrupt and shelter averts dose; EDL eval reflects per-body difficulty incl. the Mars gap; loss-of-crew is a graded outcome fed to the political system; astronaut careers persist; numbers sourced; renderer holds no crew derivation. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- Asset sizing inputs (`AssetSizing`, `EnvFacts`, `eclss_maturity`, `ops_oversub`, `consumables_kg`) and how they come from the GP-03/06 design + GP-02 maturity.
- How SPE-storm events (GP-08 event system) are surfaced here as shelter interrupts ahead of GP-08, or stubbed until then.
- EDL gate semantics per body and the Mars-gap representation.
- Loss-of-crew outcome handoff to polity (`RecordOutcome`) and astronaut grounding rules.

## /speckit.plan — guidance

- `sojourn-game` intents `CrewAsset` (`OccupyAsset`; gated), `AssignCrew`, `Maintain`/`Resupply` (Direct, show the draw), `Shelter` (Direct, from interrupt), `EvaluateEdl` (gated), `UpdateEnv`/`VacateAsset` expanding to the real `CrewCommand` variants.
- View-model: add `CrewVM`/`LifeSupportVM` (dose/closure/consumables/psych gauges + previews + EDL suitability). Renderer: crew rows + life-support panel on S5; new subscreens (Asset life-support detail, Storm response interrupt, EDL planner); S8 Astronaut careers; wire crewing, shelter, maintenance, EDL.
- Tests: harness `crew_play.ron` (crew a LEO station → consumables + crew-time drawn, dose previewed; advance → dose accrues toward limit, ECLSS consumes per closure; fire an SPE → shelter interrupt → shelter averts dose; attempt a crewed Mars descent → EDL gate reflects the Mars gap; force a hazard → graded consequence up to loss-of-crew recorded for polity); determinism + round-trip; view-model tests.

## /speckit.tasks & /speckit.analyze — notes

Separate intents/previews, view-model builders, S5/S8 renderer + interrupts, tests. `/speckit.analyze` must confirm: crew hazard/dose math displayed not recomputed (Principle II/IV), the cost/difficulty of crew is real (Principle VII), honest risk display (Principle VIII), sourced crew params (Principle I/V), thin renderer (Principle IV), core audit green.
