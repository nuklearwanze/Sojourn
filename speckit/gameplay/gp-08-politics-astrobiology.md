# GP-08 — Politics, Events, Astrobiology & Scoring · `/speckit` set (FA-19)

**Branch:** `020-politics-astrobiology` · **Design:** `gameplay/09-POLITICS-ASTROBIO-SCORING.md` · **Depends:** GP-00…GP-07

## /speckit.specify

```
/speckit.specify Turn the simulation into a game you can win or lose: the milestone race, the politics of money and approval, the honest staged unveiling of whether life exists elsewhere, AI faction competitors, and a chosen Grand Goal scored against the 100-year horizon. Make the World/Politics (S9) and Astrobiology (S10) screens interactive and add a Grand-Goal/Score surface. Authoritative design: gameplay/09-POLITICS-ASTROBIO-SCORING.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles VIII, IX, V) — read them.

WHY: with no combat, tension comes from racing for historic firsts, the politics of approval, the search for life, and a scored Grand Goal. This increment closes the prestige→budget loop and adds the win/lose condition.

Let the player see the milestone race (world-first vs faction-first, holder, status) and claim firsts as ESA achieves them (a qualifying achievement reported by any slice records to the milestone ledger, moving prestige); watch public/political mood drive appropriations and approvals (the appropriation political modifier from GP-01 becomes live here); set policy and treaties and lobby (launch licensing, nuclear-launch approval enabling NTP flight, PP stringency, export controls) with real consequences, gating and drift; obey planetary protection with graded contamination outcomes; pursue the staged astrobiology evidence loop on candidate worlds (habitability → biosignature → corroboration → consensus, with abiotic competitors, prestige-weighted consensus, the ≥0.9/≤0.1 confidence band and the sample-return gate) — NEVER a binary "life found", and the UI's astrobiology view keeps its honesty guard (never a conclusive positive the core has not set, no ground-truth input); watch AI factions research/build/fly/contract and race milestones via the same intent surface; and SELECT A GRAND GOAL (Pathfinder/Homestead/Prospector/Seeker) and see the composite score accrue against the horizon, with a change penalty, reaching a scored end.

All scoring/consensus/mood math stays in the polity slice — the UI displays and previews, never recomputes. Intent expansion and achievement reporting live in the orchestration crate.

Acceptance: milestones claim deterministically and move prestige→budget; mood/policy/lobby/PP/contamination behave with consequences and gating; astrobiology evidence is staged, honesty-guarded and sample-return-gated with no ground-truth leak; AI factions race via the same intent surface; a Grand Goal is selectable and the composite score accrues to a scored end against the horizon (change penalty applies); numbers sourced; renderer holds no scoring logic. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- How each slice's qualifying achievements are detected and reported (`RecordAchievement`) without coupling the slices — the orchestration crate observes outcomes and reports.
- Which policies hard-gate which capabilities (e.g. nuclear-launch licence ↔ NTP flight from GP-03) and the lobby/drift model.
- The astrobiology evidence pipeline: how GP-05 observations and sample-return (GP-04/06/07) flow as `CollectEvidence`, and the exact honesty-guard contract in the view-model.
- AI-faction cadence/visibility and difficulty scaling; Grand-Goal thresholds, change penalty, composite-score weights and the scored-end trigger.

## /speckit.plan — guidance

- `sojourn-game` reports achievements/outcomes (`PolityCommand::RecordAchievement`/`RecordOutcome`), and adds intents `SetPolicy`/`Lobby`, `CollectEvidence`, `EvaluateContamination`, `SelectGrandGoal`/`ChangeGrandGoal` (gated; penalty preview), `RecordHomestead` expanding to the real `PolityCommand` variants. Make the GP-01 appropriation political modifier live. Drive AI factions through the same intent surface.
- View-model: extend `WorldView` into milestone-race/mood/policy/AI builders; extend `AstrobiologyVM` into staged candidate detail keeping the honesty guard; add `GrandGoalVM`/`ScoreVM`. Renderer: S9 subscreens (Milestone race, Mood & approval, Policy & treaties, AI standings, Planetary protection), S10 subscreens (Candidate worlds, Candidate detail, Incoming data, Discoveries log), a Grand-Goal/Score surface; wire policy/goal/contamination gates.
- Tests: harness `politics_play.ron` (claim a first → ledger claims world-first, prestige↑, next appropriation modifier↑; loss-of-crew → mood↓, budget↓; nuclear policy gates NTP flight; staged evidence moves the posterior but never reports conclusive-positive without the core flag + sample-return gate; AI factions claim some firsts; select a Grand Goal, advance to horizon → composite score computed, scored end; changing the goal applies the penalty); determinism + round-trip; view-model honesty + score tests.

## /speckit.tasks & /speckit.analyze — notes

Separate achievement reporting + intents, view-model builders (incl. honesty guard + score card), S9/S10/score renderer, tests. `/speckit.analyze` must confirm: astrobiology honesty guard enforced and tested + no ground-truth input (Principle VIII), scope discipline — life is a science object, no combat/aliens (Principle IX), sourced polity params (Principle V), scoring displayed not recomputed (Principle IV), core audit green.
