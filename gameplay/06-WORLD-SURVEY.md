# GP-05 — World Survey & Belief-State (FA-16)

**Spec dir:** `specs/017-world-survey` · **Depends on:** GP-04 (fly probes to targets) · **Speckit:** `speckit/gameplay/gp-05-world-survey.md`

Gives the player a reason to fly. Missions now *observe* sites and *prospect* fields, refining the belief-state — what ESA knows versus the ground truth — so resource grades and science potential resolve from "unknown" to "measured with uncertainty." This is the input to choosing where to mine, settle and search.

## Goal & player-facing capability

Task a craft/mission at a target to **observe** site properties or **prospect** a field; watch the belief-state uncertainty shrink and grades resolve as data returns; browse sites with their surveyed properties (resource type/grade, illumination, slope, comms, PP category) and the modelled uncertainty; shortlist and compare targets by Δv-cost and expected value; see the map's resource/science layers populate. Nothing leaks ground truth — you act on belief.

## Orchestration intents → command fan-out

In `sojourn-game`:
- `Intent::Survey { mission/craft, site, property, class, quality }` → `WorldCommand::Observe`. The observation class/quality derive from the craft's instruments (composed in from the design) and geometry; `Direct` once the craft is on station (it's a tasking, the *flying there* was the gated decision in GP-04).
- `Intent::Prospect { mission/craft, field, effort }` → `WorldCommand::Prospect`.

A mission gains a **science/survey goal** (attached via `sojourn-mission`); arriving on station lets the player issue observations that the world slice folds into the belief-state (its Gaussian belief update on the kernel's seeded streams). The returned deltas (uncertainty reduction, grade estimate) are core-computed.

## Cross-system causality & state touched

Belief-state grades gate base siting (GP-06 only lets you found at sufficiently-surveyed sites) and feed expected-value comparisons for the economy (GP-01 ISRU break-even). Astrobiology candidates (GP-08) accrue evidence partly from these observations. State: world slice belief-state (journalled) + mission goal (mission module).

## ESA data

Reuses `data/world/*` (catalogue by class, sites, locations, prospecting fields, priors/floors/noise params, astrobiology distributions, resource taxonomy, Sojournal entries, reference ephemerides). Ground truth stays owned by the world slice and is never sent to the UI; only belief views are exposed.

## UI/UX — S1 Map layers + S5 tasking + site browser

S1 System Map: enable the **resources**, **science** and **PP-zones** layers; sites render with a glyph keyed to known resource type and an **uncertainty halo** sized by belief variance. The reticle inspector for a selected site shows surveyed properties with explicit ± uncertainty and "last observed" provenance.

New subscreens (on S1 inspector / a S5 "Targets" tab):
- **Site detail** — surveyed properties, each with belief estimate ± uncertainty, the observation history that produced it, PP category, and Δv-from-LEO.
- **Prospecting fields** — statistical fields for uncatalogued populations; prospect effort vs expected yield.
- **Target shortlist / compare** — pin candidate sites; compare on grade estimate, uncertainty, Δv-cost, illumination, PP category; an expected-value column once economy break-even is wired.
- **Tasking** — for a mission on station: choose property + observation class; issue Survey/Prospect.

Verbs: Survey, Prospect (Direct once on station; the gated decision was getting there). Empty state: unsurveyed target → "Unknown — send an instrument to observe."

View-model: a `WorldSurveyVM`/`SiteDetailVM` carrying belief estimates + uncertainty + provenance (with the honesty rule: belief only). Unit-test that no ground-truth field is ever populated and that uncertainty monotonically decreases with good observations. Renderer wires tasking.

## Testability

Harness `survey_play.ron`: boot ESA → fly a probe to a lunar polar site (GP-04) → Observe with a given instrument quality → assert belief uncertainty decreases and the grade estimate moves toward (but never equals, pre-resolution) the seeded truth; prospect a field → assert yield estimate updates; confirm the UI view never receives ground truth. Determinism + round-trip. View-model honesty test. Human: send a probe, observe, watch the uncertainty halo shrink and a grade resolve.

## Acceptance criteria

Observations/prospecting refine the belief-state with core-computed uncertainty reduction; the map layers and site inspector show belief + uncertainty + provenance with no ground-truth leak; targets are comparable by grade/uncertainty/Δv; missions carry survey goals; numbers sourced; renderer holds no belief logic.

## Out of scope

Founding a base at a surveyed site (GP-06). The astrobiology evidence staging (GP-08, though observations feed it). ISRU break-even economics display (GP-06/GP-08 economy depth).
