# Contract — Composed-value integration seams (the FA-04 C1 decoupling)

`sojourn-polity` depends **only on `sojourn-core`**. Every cross-slice fact enters as a plain,
serializable **composed value** the host assembles from the upstream query surfaces and **captures into
the slice at command time** (`InitWorld`/`UpdateWorld`/`RecordAchievement`/`RecordOutcome`/
`CollectEvidence`/`EvaluateContamination`). No upstream gameplay crate is a dependency; the dependency
graph gains **no new edge**.

## Inputs the host composes IN

| Input shape | Bridged from | Feeds |
|---|---|---|
| `CandidatePrior { candidate, presence_prob, tier }` | **FA-03** `data/world/astrobiology.ron` | the seeded ground-truth draw (US3) |
| `SiteProtection { body, category, special_region, bioburden_limit }` | **FA-03** `data/world/sites.ron` + active stringency | the PP regime (US6) |
| `ScienceTide { global_level, per_faction_level }` | **FA-05** | AI progress, event hazards, breakthrough-gated firsts (US4/US7) |
| `AchievementFacts { ... }` (propellant_sold_kg, body, crewed, reusable, isru_origin, …) | **FA-04/06/07/08** mission/economy outputs | milestone award conditions (US1) |
| `EconomyFacts { base_appropriation, base_valuation, off_earth_tonnage_profit }` | **FA-06** | mood modifiers (over, never generating, value) + Prospector (US2/US8) |
| `CrewFacts { loss_of_crew, routine_failure, success, world_first }` | **FA-08** (loss-of-crew) + mission outcomes | mood/politics (US2) |
| `HomesteadFacts { embargo_survival_index }` | **FA-07/08** | Homestead goal (US8) |
| `MissionFacts { body, special_region, lander_bioburden, crash, sample_return, containment_chain }` | **FA-04/07/08** | contamination (US6) |
| `Difficulty { ai_funding_mult, ai_competence_mult, event_rate_mult }` | host setting | AI tuning + event rates (US7) — never physics |

## Outputs the host composes OUT (via `WorldSnapshot`)

- Per-faction **prestige**, **mood** and the derived **appropriation/valuation/approval** modifiers (the
  host applies them to the FA-06 economy).
- **Policy lever levels** + **gate verdicts** (the host gates/penalises launches/partnerships).
- **PP category/Special-Region status**, **pristine value** and **contamination records**.
- Per-candidate **per-faction posteriors + weighted consensus + conclusive status + public-disagreement +
  achievable evidence value** (never ground truth).
- The **event feed** (interrupts already routed through the FA-01 loop).
- Grand-Goal **progress + pass/fail verdict** and the **final composite score**.

## Why composed values (not crate deps)

- **Testability**: each sub-system is unit-tested with stub priors/sites/economy/crew facts — no upstream
  crate spin-up.
- **No new graph edge**: the dependency + scope audit stays green; `sojourn-core` remains presentation-
  and gameplay-edge-free.
- **Honest seam**: the ground truth lives only inside the slice; the host can never read it, and upstream
  crates never learn the political/astrobiology state — a one-way composition.
