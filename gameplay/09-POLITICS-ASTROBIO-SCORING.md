# GP-08 — Politics, Events, Astrobiology & Scoring (FA-19)

**Spec dir:** `specs/020-politics-astrobiology` · **Depends on:** GP-00…GP-07 (consumes all of them as facts) · **Speckit:** `speckit/gameplay/gp-08-politics-astrobiology.md`

The meta-game and the **win condition**. Without combat, tension comes from the race for historic firsts, the politics of money and approval, the honest staged unveiling of whether life exists elsewhere, and a chosen Grand Goal scored against a horizon. AI factions race alongside ESA. This increment turns the simulation into a game you can win or lose.

## Goal & player-facing capability

See the **milestone race** (world-first vs faction-first) and claim firsts as ESA achieves them; watch **public/political mood** drive appropriations and approvals (closing the GP-01 budget loop with a real modifier); set **policy & treaties** and **lobby** (launch licensing, nuclear-launch approval, PP stringency, export controls) with real consequences and drift; obey **planetary protection** with graded **contamination** outcomes; pursue the staged **astrobiology** evidence loop on candidate worlds (habitability → biosignature → corroboration → consensus, abiotic competitors, sample-return gate) — never a binary "life found"; watch **AI factions** research/build/fly/contract and race milestones; **select a Grand Goal** (Pathfinder/Homestead/Prospector/Seeker) and see the **composite score** accrue against the 100-year horizon (with a change penalty). Reaching/missing the goal ends scored play.

## Orchestration intents → command fan-out

In `sojourn-game`:
- Achievements: when a slice reports a qualifying first (an arrival, a settlement, a sample return), `sojourn-game` emits `PolityCommand::RecordAchievement(...)`; the milestone ledger claims world-first/faction-first deterministically.
- `Intent::SetPolicy { id, level }` → `PolityCommand::SetPolicy` (gated where it has hard consequences, e.g. nuclear-launch licence enabling a propulsion class). `Intent::Lobby { id, direction }` → `PolityCommand::Lobby` (reversible nudge; drifts over time).
- Outcomes: crew/economy facts flow as `PolityCommand::RecordOutcome` (loss-of-crew, bankruptcy) → mood/prestige shifts → `ApplyAppropriation`'s political modifier (now live) feeds GP-01.
- Astrobiology: observations/instrument returns (GP-05) flow as `PolityCommand::CollectEvidence(...)`; the slice runs the per-faction Bayesian update + prestige-weighted consensus behind the **honesty band** (≥0.9/≤0.1) and the sample-return gate. The UI's astrobiology view keeps its honesty guard (never a conclusive positive the core hasn't set).
- PP: `Intent::EvaluateContamination` → `PolityCommand::EvaluateContamination(MissionFacts)` graded outcome.
- Goal: `Intent::SelectGrandGoal { kind }` → `PolityCommand::SelectGrandGoal`; `Intent::ChangeGrandGoal` → `ChangeGrandGoal` (gated, takes the penalty). `RecordHomestead` for settlement progress.
- AI factions: driven by the polity slice's abstracted heuristics + difficulty; `sojourn-game` advances them through the same intent surface so they race fairly.

All scoring/consensus/mood math is the polity slice's — displayed and previewed, never recomputed.

## Cross-system causality & state touched

This increment **closes the loops**: prestige/mood ← achievements/outcomes from every system → budget/valuation/approval → back into what ESA can afford. Astrobiology evidence ← survey/instruments (GP-05) and sample-return (GP-04/06/07). Policy gates which tech can fly (e.g. nuclear-launch licence ↔ NTP from GP-03). State: polity slice (journalled) — milestone ledger, mood, per-faction belief, policies, goals, score.

## ESA data

Reuses `data/polity/*` (the ~120-first milestone catalogue; mood coefficients/decay/loss-of-crew severity + mood→budget/valuation/approval curves; event-class catalogue + base rates + interrupt classification; policy/treaty levers + bounds + drift/lobby + gating; COSPAR I–V + Special Regions + bioburden + contamination grading; astrobiology evidence-stage likelihoods + abiotic competitors + consensus weighting + confidence band + sample-return gate; AI tuning + difficulty; Grand-Goal thresholds + change penalty + composite-score weights). Astrobiology priors stay owned by `data/world/astrobiology.ron`. Confirm sources.

## UI/UX — S9 World/Politics + S10 Astrobiology + a Grand-Goal/Score surface

S9 subscreens:
- **Milestone race** — the firsts ledger as a race ladder (world-first vs faction-first, holder, status open/contested/claimed), filterable by era; ESA's progress highlighted.
- **Mood & approval** — mood gauge, the mood→budget/valuation curve, recent drivers (achievements, outcomes); the live political modifier on the next appropriation.
- **Policy & treaties** — levers with bounds, current level, drift, gating notes (e.g. "nuclear-launch licence: enables NTP flight"); Set (gated) + Lobby (nudge).
- **AI faction standings** — competitors' visible prestige/milestones/known intentions (belief, not truth).
- **Planetary protection** — site categories, Special Regions, contamination grade per mission with consequences.

S10 subscreens:
- **Candidate worlds** — the staged **evidence meter** per candidate (Mars subsurface, Europa/Enceladus, Titan, Ceres, Venus clouds): Habitability → Biosignature → Corroboration → Consensus, each a probability with the honesty guard; abiotic-competitor weight shown.
- **Candidate detail** — evidence history, instruments contributing (GP-05), the sample-return gate, consensus over time; "P(life)" shown only as the core's belief, never a conclusion.
- **Incoming data** — instruments → UL/evidence deltas. **Discoveries log**.

Grand-Goal / Score surface (top-bar/S9): select a Grand Goal (with its win condition + composite-score weights), the live score, horizon countdown, change-penalty warning, and the **end-of-game** scored summary.

Plan→preview→commit verbs: Set policy (Reversible/Build by consequence), Select/Change Grand Goal (ChangeGoal kind — penalty preview), Evaluate contamination. Lobby is reversible.

View-model: `WorldView` extended into milestone-race/mood/policy/AI builders; `AstrobiologyVM` extended into staged candidate detail (keeping the honesty guard); a `GrandGoalVM`/`ScoreVM`. Unit-test the honesty guard (no conclusive positive without the core flag; no ground-truth input), the milestone-claim shaping, and the score card. Renderer wires policy/goal/contamination gates.

## Testability

Harness `politics_play.ron`: boot ESA → claim a first (assert ledger claims world-first, prestige↑, next appropriation modifier↑) → record a loss-of-crew (assert mood↓, budget↓) → set the nuclear-launch policy (assert it gates NTP flight) → collect astrobiology evidence across stages (assert the per-faction posterior moves but never reports conclusive-positive without the core flag; assert sample-return gate) → run AI factions (assert they claim some firsts) → select a Grand Goal, advance to the horizon (assert composite score computed, scored end reached); changing the goal applies the penalty. Determinism + round-trip. View-model honesty + score tests. Human: claim a first and watch the budget rise; chase a biosignature without ever getting a fake "life found"; pick a Grand Goal and see the score.

## Acceptance criteria

Milestones claim deterministically and move prestige→budget; mood/policy/lobby/PP/contamination behave with consequences and gating; astrobiology evidence is staged, honesty-guarded and sample-return-gated with no ground-truth leak; AI factions race via the same intent surface; a Grand Goal is selectable and the composite score accrues to a scored end against the horizon (change penalty applies); numbers sourced; renderer holds no scoring logic.

## Out of scope

The Sojournal entries explaining each milestone/policy/candidate (GP-09 deep-links existing data). Onboarding for the meta-game (GP-09).
