# GP-07 — Crew & Life Support (FA-18)

**Spec dir:** `specs/019-crew-life-support` · **Depends on:** GP-04 (craft to crew), GP-06 (bases to crew), GP-02 (astronaut pool) · **Speckit:** `speckit/gameplay/gp-07-crew-life-support.md`

The hard part: humans in space. Crewing a craft or base turns it from a robotic asset into a life-support problem — consumables vs ECLSS closure, radiation dose, physiological deconditioning, psychology under isolation and comms-lag, maintenance and spares, EDL risk, and the real possibility of loss of crew. Crewed missions become materially harder and costlier than robotic ones, exactly as intended (Principle VII).

## Goal & player-facing capability

Assign astronauts to a craft or base; size its ECLSS and load consumables; watch dose accrue against career limits and life-support state evolve; respond to a solar-particle-event interrupt by ordering crew to the storm shelter; perform maintenance (crew-time + spares) and resupply; evaluate EDL suitability before a crewed landing (the Mars EDL gap bites); and face graded consequences up to loss of crew. Astronaut careers (dose, health, eligibility) track across missions.

## Orchestration intents → command fan-out

In `sojourn-game`:
- `Intent::CrewAsset { faction, asset, sizing, env, eclss_maturity, ops_oversub, consumables }` → `CrewCommand::OccupyAsset` — **gated** (preview: consumables mass + crew-time draw, projected dose rate, ECLSS closure, viability; what becomes unrecoverable = exposing crew to the mission's hazards).
- `Intent::AssignCrew { asset, astronaut }` → `CrewCommand::AssignCrew` (with the astronaut's career facts from the personnel pool, GP-02).
- `Intent::Maintain { asset, crew_hr, spares }` / `Intent::Resupply { asset, kg }` → matching `CrewCommand` (draw crew-time/mass).
- `Intent::Shelter { asset, sheltering }` → `CrewCommand::Shelter` (the SPE-storm response; usually issued from an interrupt).
- `Intent::EvaluateEdl { asset, suitability }` → `CrewCommand::EvaluateEdl` — **gated** before a crewed descent (preview: per-body difficulty incl. the Mars gap, success odds).
- `Intent::UpdateEnv` / `Intent::VacateAsset` as the mission environment changes.

The dose→REID curve, deconditioning/psych accrual, the multiplicative-hazard failure/anomaly/EDL model and the capability product are the crew slice's derivations — displayed and previewed, never recomputed.

## Cross-system causality & state touched

Crewing draws consumables (mass) + crew-time (economy/ops, GP-01) and ECLSS maturity (research, GP-02); a craft's/base's life-support sizing comes from the vehicle/base design (GP-03/06); SPE storms arrive as events (GP-08 event system, surfaced here as interrupts); loss of crew is an outcome fed to polity (GP-08) with heavy prestige/mood consequences. State: crew slice (journalled) + astronaut career state (personnel/crew).

## ESA data

Reuses `data/crew/*` (consumables rates + closure tiers; GCR/SPE radiation + shelter attenuation + dose→REID; deconditioning + countermeasure/artificial-gravity effectiveness + capability curves; ECLSS failure rates + maturity/maintenance/heritage multipliers; per-body EDL incl. Mars gap; hazard rates + viability thresholds + the 3% REID threshold; validation cases). Confirm sources.

## UI/UX — crew layer across S5 Operations + S8 Personnel (+ interrupts)

S5 Operations: crewed assets gain crew rows with **dose** and **health** gauges, ECLSS closure %, consumables-remaining, and a maintenance/resupply control; the craft inspector shows the full life-support panel (consumables vs closure, dose accrual rate, deconditioning, psych load, spares).

New subscreens:
- **Asset life-support detail** — consumables timeline vs ECLSS closure; dose accrual vs career/limit; deconditioning & countermeasure status (incl. artificial-gravity); psychology load + comms-lag; spares/maintenance schedule.
- **Storm response** — surfaced as an interrupt when an SPE fires: "shelter crew?" with the dose-averted preview; the Shelter verb.
- **EDL planner** — per-body EDL suitability with the difficulty (Mars gap explicit); gate before a crewed landing.

S8 Personnel: **Astronaut careers** — per-astronaut cumulative dose, health, eligibility (grounded if over limits), mission history; assignment from here.

Plan→preview→commit verbs: Crew asset (Build/Launch kind — irreversible exposure), Evaluate/commit EDL (Burn/Build), Maintain/Resupply (Direct, but show the draw), Shelter (Direct, from interrupt). Empty state: no crewed assets → "All missions robotic — crew an asset to begin human spaceflight (harder, costlier)."

View-model: a `CrewVM`/`LifeSupportVM` carrying the gauges + previews + EDL suitability; unit-test the dose/closure display, the storm-response preview pass-through, and the EDL gate. Renderer wires crewing, shelter (from interrupt), maintenance, EDL.

## Testability

Harness `crew_play.ron`: boot ESA → crew a LEO station (assert consumables + crew-time drawn, dose rate previewed) → advance → assert dose accrues toward limit, ECLSS consumes consumables per closure → fire an SPE event → assert a shelter interrupt; shelter → assert dose averted → attempt a crewed Mars descent → assert EDL gate reflects the Mars gap; force a hazard → assert graded consequence up to loss-of-crew recorded for polity. Determinism + round-trip. View-model tests. Human: crew a station, watch dose climb, get a storm interrupt, order shelter, try (and probably fail to cheaply) a Mars EDL.

## Acceptance criteria

Crewing draws consumables + crew-time and is gated with a core-computed dose/viability preview; dose/physiology/psych/ECLSS evolve and display; SPE storms interrupt and shelter averts dose; EDL eval reflects per-body difficulty incl. the Mars gap; loss-of-crew is a graded outcome fed to polity; astronaut careers persist; numbers sourced; renderer holds no crew derivation.

## Out of scope

The prestige/mood impact of loss-of-crew (recorded here, scored in GP-08). Bioregenerative-ECLSS research (GP-02 tech). The Grand-Goal crew-legacy scoring (GP-08).
