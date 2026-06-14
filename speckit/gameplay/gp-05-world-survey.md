# GP-05 — World Survey & Belief-State · `/speckit` set (FA-16)

**Branch:** `017-world-survey` · **Design:** `gameplay/06-WORLD-SURVEY.md` · **Depends:** GP-04

## /speckit.specify

```
/speckit.specify Give the player a reason to fly: let missions observe sites and prospect fields, refining the belief-state (what ESA knows versus ground truth) so resource grades and science potential resolve from unknown to measured-with-uncertainty. Add the map's resource/science layers and a site browser. Authoritative design: gameplay/06-WORLD-SURVEY.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles VIII, V) — read them.

WHY: the belief-state is the input to choosing where to mine, settle and search; without it there is no informed decision behind a mission.

Let the player task a craft/mission on station to OBSERVE specific site properties (with an observation class/quality derived from the craft's instruments and geometry) or PROSPECT a statistical field; fold the returns into the belief-state via the world slice's Gaussian belief update so uncertainty shrinks and grade estimates move toward (but never reveal) ground truth; browse sites with their surveyed properties (resource type/grade, illumination, slope, comms, planetary-protection category) each shown as a belief estimate ± uncertainty with observation provenance; shortlist and compare targets by grade estimate, uncertainty and Δv-cost; and populate the System Map's resources, science and PP-zones layers with sites rendered with an uncertainty halo. CRITICAL: the UI must never receive or display ground truth — only belief views — and uncertainty must monotonically decrease with good observations.

A mission gains a survey/science goal (attached via the mission module). Tasking is Direct once the craft is on station — the gated decision was flying there (GP-04). Intent expansion lives in the orchestration crate; the belief update stays in the world slice.

Acceptance: observations/prospecting refine the belief-state with core-computed uncertainty reduction; the map layers and site inspector show belief + uncertainty + provenance with no ground-truth leak; targets are comparable by grade/uncertainty/Δv; missions carry survey goals; numbers sourced; renderer holds no belief logic. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- How observation class/quality is derived from the craft's instruments (composed from the GP-03 design) and geometry.
- The view-model's honesty boundary: which belief fields are exposed and the guarantee that no ground-truth field is ever populated.
- Grade "resolution" semantics: when an estimate is considered resolved vs still uncertain.
- How a survey goal attaches to a mission and drives on-station tasking availability.

## /speckit.plan — guidance

- `sojourn-game` intents `Survey` (`WorldCommand::Observe`) and `Prospect` (`WorldCommand::Prospect`), Direct once on station; attach survey goal via the mission module.
- View-model: add `WorldSurveyVM`/`SiteDetailVM` carrying belief estimate + uncertainty + provenance only; extend the map view-model with the resource/science/PP layers and uncertainty halos. Renderer: S1 layers + site inspector; S5 "Targets" tab subscreens (Site detail, Prospecting fields, Target shortlist/compare, Tasking).
- Tests: harness `survey_play.ron` (fly a probe to a lunar polar site → Observe → uncertainty decreases, estimate moves toward but never equals seeded truth; prospect a field → yield estimate updates; assert the view never receives ground truth); determinism + round-trip; a view-model honesty unit test (no ground-truth field populated; uncertainty monotonic).

## /speckit.tasks & /speckit.analyze — notes

Separate intents/goal-attachment, view-model (with honesty guard), S1 layers, S5 targets, tests. `/speckit.analyze` must confirm: belief-vs-truth honesty enforced and tested (Principle VIII), sourced priors/noise params (Principle V), thin renderer (Principle IV), core audit green.
