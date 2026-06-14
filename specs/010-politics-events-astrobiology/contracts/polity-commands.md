# Contract — Commands & emitted events

Commands route via `Command::ModulePayload { module: "polity", kind, payload }` (postcard), encoded by
`polity_payload(&PolityCommand) -> Command`. Each returns `CommandOutcome::Applied` or `Rejected(reason)`.
Composed values are **captured at command time** (the FA-06 `OperateIsru` / FA-08 `OccupyAsset` pattern)
so the daily seeded step can evolve stored state.

## Commands (`PolityCommand`)

- **InitWorld** `{ factions: Vec<FactionInit>, candidates: Vec<CandidatePrior>, protections:
  Vec<SiteProtection>, difficulty: Difficulty }` — captures the roster + FA-03 priors/PP; **draws the
  seeded ground truth** (stream `polity/ground-truth`, body-id order). Once per game.
- **UpdateWorld** `{ science_tide: ScienceTide, difficulty: Difficulty }` — refresh composed tide/harshness.
- **RecordAchievement** `{ faction, milestone, facts: AchievementFacts }` — evaluate award conditions;
  award **world-first** (full weight) or **faction-first** (lesser fraction); emit `milestone-claimed`.
- **RecordOutcome** `{ faction, crew_facts: CrewFacts, economy_facts: EconomyFacts }` — feed the mood
  model (success/routine-failure/loss-of-crew/world-first/economic-cycle) and refresh budget bases.
- **CollectEvidence** `{ faction, candidate, stage, quality }` — append evidence; update that faction's
  posterior (consensus recomputed on the step).
- **SetPolicy** `{ id, level }` / **Lobby** `{ faction, id, direction }` — set/nudge a lever (bounded;
  lobby on stream `polity/lobby`).
- **RequestApproval** `{ faction, program }` — resolve grant/delay/deny from mood + policy.
- **EvaluateContamination** `{ mission: MissionFacts }` — forward (graded by overage × crash/soft) and/or
  back contamination; record penalty + degrade pristine value; emit `contamination`.
- **SelectGrandGoal** `{ faction, kind }` / **ChangeGrandGoal** `{ faction, kind }` — select/change (the
  latter applies the sourced penalty).

## Emitted events (data-driven classes added to `data/kernel/event-classes.ron`)

| Class | Kind | When |
|---|---|---|
| `milestone-claimed` | Interrupt | a world-/faction-first is awarded |
| `rival-milestone` | LogOnly | an AI faction claims a first the player was racing |
| `mood-shift` | LogOnly | a material mood change (e.g. post-loss-of-crew) |
| `approval-frozen` | Interrupt | crewed-flight approval freezes (loss-of-crew) |
| `anomaly` / `launch-failure` / `solar-storm` | Interrupt | state-driven event hazards fire |
| `funding-crisis` / `funding-boom` | Interrupt | economic-cycle events |
| `political-shakeup` / `supply-shock` / `personnel-event` | Interrupt | seeded world events |
| `discovery` | Interrupt | a science/exploration discovery (non-astrobiology) |
| `astrobiology-evidence` | LogOnly | new evidence shifts a posterior |
| `astrobiology-conclusive` | Interrupt | a candidate's question is conclusively resolved (top-tier) |
| `contamination` | Interrupt | forward/back contamination recorded |
| `grand-goal-met` | Interrupt | a Grand Goal's pass condition is reached |
| `soft-fail` | Interrupt | agency gutting / bankruptcy / loss-of-crew spiral |

## Rules

- Unknown faction/candidate/policy/milestone → `Rejected`.
- `InitWorld` twice → `Rejected` (the world is initialised once).
- A world-first already held → later achiever gets **faction-first** only; same-tick tie → **highest
  prestige, then lowest faction id**.
- Lobbying/drift clamp to `[min, max]`; mood saturates within bounds.
- No command can expose or set the ground truth; no command can grant an AI faction tech beyond the
  plausibility envelope.
- Determinism: every stochastic effect draws from a named stream; replaying the command journal
  reproduces identical state.
