# Contract: Astronaut Career ↔ FA-08 Crew-Feedback Interface (FR-RESP-601/602)

The clarified FA-05 ↔ FA-08 seam (Q1:A). FA-05 owns the astronaut **roster and career pipeline** up to
the mission boundary; FA-08 later owns **in-mission** ECLSS and acute physiology and feeds career deltas
back through the single command declared here.

## What FA-05 owns (this slice)

- The astronaut **roster** (a Person sub-type) and pipeline stage: `Candidate → Training → Ready →
  Retired`.
- **Training**: multi-year, facility/analog-gated; advances per step; yields a `Ready` astronaut.
- Running **career budgets**: `career_dose`, `health`, `psych`, plus morale and age.
- **Readiness** exposed via `personnel(faction).astronaut_readiness` (FR-RESP-801).

## The FA-08-facing command

```text
CrewFeedback { faction, astronaut, dose_delta, health_delta, psych_delta }
```

- Applied as a journaled command (deterministic). FA-08 emits it during/after a flight; scenarios drive
  synthetic deltas for testing.
- Effect: the astronaut's `career_dose` / `health` / `psych` budgets update; crossing a **documented
  career limit** (data) deterministically moves the astronaut out of the `Ready` pool (and may retire
  them). Career state is queryable.

## What FA-08 owns (later, NOT here)

- In-mission life-support **closure** (consumables, water/O₂/CO₂ loops).
- **Acute** dose/psychological/physiological dynamics *during* a flight (this slice only accumulates the
  resulting career deltas FA-08 reports).

## Guarantee

The interface is the **only** coupling between in-mission crew dynamics and the career roster: FA-05
never reaches into mission state, FA-08 never mutates the roster except through `CrewFeedback`. This
keeps each slice's determinism and slice ownership intact.
