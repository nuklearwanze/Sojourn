# GP-09 — Sojournal, Onboarding & UX Polish (FA-20)

**Spec dir:** `specs/021-journal-onboarding` · **Depends on:** GP-00…GP-08 (cross-cuts everything) · **Speckit:** `speckit/gameplay/gp-09-journal-onboarding.md`

Makes the now-complete game **legible and learnable**. The source-cited Sojournal becomes the deep-linked "why this number" layer behind every figure; the event log gains filtering and configurable pause policies; onboarding layers ease a newcomer in; and the New/Continue entry is finished. No new simulation — this is the educational-honesty and usability pass that turns depth into something a player can actually navigate.

## Goal & player-facing capability

Click any derived number or entity and jump to its source-cited Sojournal entry (the rocket equation behind a Δv, the dose→REID curve behind a risk, the source behind a body's mass); search and browse the encyclopedia; filter the event log by class and configure which event classes pause the game; define personal watch conditions; follow optional onboarding layers ("first launch," "first transfer," "first base") that point at the right screen without scripting the player's choices; and start a New ESA game or Continue the latest save from a finished entry screen.

## Orchestration intents → command fan-out

Mostly UI/host, little new simulation:
- Pause policy: `Intent::SetPausePolicy { class, policy }` → the kernel's existing `Command::SetPausePolicy` (journalled core state).
- Watches: `Intent::RegisterWatch/ModifyWatch/RemoveWatch { spec }` → the kernel's `Command::RegisterWatch` etc.
- New/Continue → `Session::start_new_esa()` / `Session::load()` (GP-00).
- Deep-links and onboarding are pure presentation reading existing data (`data/world` Sojournal entries + the `TraceRender` trees the slices already expose) — **no new authoritative data** beyond the existing UI theme/hotkey config.

## Cross-system causality & state touched

Deep-linking reads the traceability and Sojournal surfaces that every prior increment already exposes; pause policy and watches are existing journalled kernel state. No new module, no new sim. Onboarding state is UI-local (out of the save).

## ESA data

No new authoritative numbers. Uses existing `data/world` Sojournal entries (source-cited) and the UI theme/palette/hotkey config (no plausibility-bearing fields). If a number lacks a Sojournal target, that is a content gap to fill in `data/world` (with sources), not new mechanics.

## UI/UX — S11 Sojournal + S12 Alerts/Event Log + cross-cutting polish

S11 Sojournal:
- **Browser / search** — full-text search across entries; categories (bodies, propulsion, tech, biology, economics, policy).
- **Entry** — the source-cited article; "Sources" block with provenance; related entries.
- **Cross-links** — every entity and **every traced number** in the app gets a "Sojournal ⓘ" affordance that opens the relevant entry; the trace panel (Δv = …) links its terms to entries.

S12 Alerts / Event Log:
- **Full feed** — filter by class (NAV/SCI/ECO/POL/OPS…), search, jump-to-event.
- **Interrupt review** — the GP-00 modal, refined: per-event decisions, acknowledge.
- **Pause-policy config** — per event class: pause / notify / ignore (writes `SetPausePolicy`).
- **Watch conditions** — player-defined watches (register/modify/remove), e.g. "pause when funds < X" or "pause when a Mars window opens."

Cross-cutting polish: consistent trace affordance everywhere; calm **empty/first-run states** with onboarding pointers; **onboarding layers** as dismissible coach-marks tied to "first time you open screen X / have no fleet / etc." (UI-local, never blocking); the **New / Continue** entry screen finished; keyboard map surfaced (a hotkey cheat-sheet); accessibility pass (colour-blind palette, scaling 1280×720→4K, full keyboard nav).

Plan→preview→commit: pause-policy and watch changes are `Direct` (reversible config). No gated sim verbs here.

View-model: a `SojournalVM` (search/entry/cross-link resolution), an `EventLogVM` (filter/config), and a small `OnboardingVM` (which coach-marks are due, UI-local). Unit-test the cross-link resolution (every traced term resolves to an entry or is flagged a content gap) and the filter logic. Renderer wires search, deep-links, pause config, watches, onboarding.

## Testability

Harness/host: assert that for a representative set of derived figures (a Δv, a dose risk, an ISRU break-even, a body mass) the trace resolves to a Sojournal entry (or reports a content gap), and that `SetPausePolicy`/`RegisterWatch` change interrupt behaviour deterministically and round-trip. View-model tests for search and cross-link resolution. Human: hover a Δv → open the rocket-equation entry; set "anomalies don't pause"; add a "funds < €1bn" watch; start a New ESA game from the entry screen.

## Acceptance criteria

Every traced number/entity deep-links to a source-cited Sojournal entry (gaps flagged, not faked); the event log filters and pause policies are configurable and journalled; watches work; onboarding layers guide without blocking; New/Continue works; accessibility floor met; no new authoritative data beyond theme/hotkeys; renderer holds no logic.

## Out of scope

A full guided tutorial campaign (the onboarding here is non-blocking coach-marks). A new-game configurator (still ESA-default by design). Any new simulation mechanics.
