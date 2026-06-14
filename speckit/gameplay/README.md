# Sojourn — Gameplay Programme `/speckit` Driver (FA-11 … FA-20)

This folder holds the `/speckit.*` command sets that turn the implemented simulation (FA-01…FA-10) into a *playable game as ESA*, one testable increment at a time. Read [`../../gameplay/00-CORE-LOOP.md`](../../gameplay/00-CORE-LOOP.md) and [`../../gameplay/UI-UX-CONVENTIONS.md`](../../gameplay/UI-UX-CONVENTIONS.md) first; each increment also has a design doc under `gameplay/`.

## Numbering & branches

The repo's spec series continues unbroken (sim-core = FA-01 = `specs/002-…`; UI = FA-10 = `specs/011-…`). The gameplay increments are:

| Increment | Feature area | Spec dir / branch | Design doc | Speckit file |
|---|---|---|---|---|
| GP-00 Session & ESA bootstrap | FA-11 | `012-session-bootstrap` | `gameplay/01-SESSION-AND-BOOTSTRAP.md` | `gp-00-session-bootstrap.md` |
| GP-01 Economy & budget | FA-12 | `013-economy-budget` | `gameplay/02-ECONOMY-BUDGET.md` | `gp-01-economy-budget.md` |
| GP-02 Research & personnel | FA-13 | `014-research-personnel` | `gameplay/03-RESEARCH-PERSONNEL.md` | `gp-02-research-personnel.md` |
| GP-03 Vehicle designer | FA-14 | `015-vehicle-designer` | `gameplay/04-VEHICLE-DESIGNER.md` | `gp-03-vehicle-designer.md` |
| GP-04 Flight & fleet | FA-15 | `016-flight-fleet` | `gameplay/05-FLIGHT-AND-FLEET.md` | `gp-04-flight-fleet.md` |
| GP-05 World survey | FA-16 | `017-world-survey` | `gameplay/06-WORLD-SURVEY.md` | `gp-05-world-survey.md` |
| GP-06 Bases & construction | FA-17 | `018-bases-construction` | `gameplay/07-BASES-CONSTRUCTION.md` | `gp-06-bases-construction.md` |
| GP-07 Crew & life support | FA-18 | `019-crew-life-support` | `gameplay/08-CREW-LIFE-SUPPORT.md` | `gp-07-crew-life-support.md` |
| GP-08 Politics, astrobiology & scoring | FA-19 | `020-politics-astrobiology` | `gameplay/09-POLITICS-ASTROBIO-SCORING.md` | `gp-08-politics-astrobiology.md` |
| GP-09 Sojournal, onboarding & polish | FA-20 | `021-journal-onboarding` | `gameplay/10-JOURNAL-ONBOARDING-POLISH.md` | `gp-09-journal-onboarding.md` |

## Workflow per increment

Run the full gated cycle on its own branch, exactly as FA-01…FA-10 were built:

```
/speckit.specify   ← the block in the increment's speckit file
/speckit.clarify   ← resolve that increment's ambiguities (the file lists focus points)
/speckit.checklist ← optional quality gate
/speckit.plan      ← architecture (the file gives guidance: where orchestration lives, decoupling, determinism, egui)
/speckit.tasks     ← decompose
/speckit.analyze   ← constitution + consistency check (run before EVERY implement)
/speckit.implement ← build; land only when its harness scenario + determinism/round-trip + view-model tests are green AND a human can perform the verb in sojourn-ui-desktop
```

Use the repo's git extension (`/speckit.git.feature`, `/speckit.git.commit`) as you have been.

## Build order (strict)

GP-00 → GP-01 → GP-02 → GP-03 → GP-04 → GP-05 → GP-06 → GP-07 → GP-08 → GP-09.

A minimal end-to-end playable game exists after **GP-04** (fund → research → design → launch → fly). GP-05–07 add the reasons to fly and the difficulty of crewing. GP-08 adds the win condition. GP-09 makes it legible.

## Standing constraints (every increment)

- **New crates this programme introduces:** `sojourn-game` (stateless orchestration/intent layer + ESA bootstrap, headless-testable; created in GP-00) and `sojourn-mission` (durable, journalled `SimModule` for mission threads/standing orders; created in GP-04). No other new crates without justification.
- **Decoupling preserved:** slices and `sojourn-core` stay as they are; cross-system causality lives only in `sojourn-game` (composition) or `sojourn-mission` (journalled state). The `sojourn-core` dependency audit must still pass.
- **Determinism:** durable state goes through the journal; UI/session ephemera stay out of the save. Every increment ships a determinism double-run + save round-trip test.
- **Plausibility & traceability:** every new number is sourced `data/*`; every UI figure traces to inputs; previews are core-computed (`sojourn-game` composes, never invents).
- **Renderer is thin:** `sojourn-ui-desktop` gains layout + wiring only; new view-model builders and widget shapes are unit-tested headlessly in `sojourn-ui` first.
- **Scope discipline (Principle IX):** no combat, no aliens; discovered life is a science object; the astrobiology honesty guard holds.
- **ESA-only:** no new-game configurator; the default path is ESA via the GP-00 bootstrap.

The constitution at `.specify/memory/constitution.md` remains authoritative.
