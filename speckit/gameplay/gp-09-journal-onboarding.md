# GP-09 — Sojournal, Onboarding & UX Polish · `/speckit` set (FA-20)

**Branch:** `021-journal-onboarding` · **Design:** `gameplay/10-JOURNAL-ONBOARDING-POLISH.md` · **Depends:** GP-00…GP-08

## /speckit.specify

```
/speckit.specify Make the complete game legible and learnable: deep-link the source-cited Sojournal behind every derived number, add event-log filtering and configurable pause policies and player watches, layer in non-blocking onboarding, and finish the New/Continue entry. Make the Sojournal (S11) and Alerts/Event Log (S12) screens interactive. Authoritative design: gameplay/10-JOURNAL-ONBOARDING-POLISH.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles VIII, IV) — read them.

WHY: the depth is now built; this is the educational-honesty and usability pass that turns it into something a player can navigate. No new simulation.

Let the player click any derived number or entity and jump to its source-cited Sojournal entry (the rocket equation behind a Δv, the dose→REID curve behind a risk, the source behind a body's mass), reading the trace tree the slices already expose with its terms linked to entries; search and browse the encyclopedia; filter the event log by class and configure which event classes pause the game; define personal watch conditions (e.g. pause when funds < X, or when a Mars window opens); follow optional non-blocking onboarding coach-marks ("first launch", "first transfer", "first base") that point at the right screen without scripting the player's choices; and start a New ESA game or Continue the latest save from a finished entry screen. CRITICAL: introduce NO new authoritative data beyond the existing UI theme/hotkey config — every figure must resolve to an existing source-cited entry, and any figure that cannot is a content gap to FLAG (and fill in data/world with sources), never to fake.

Pause-policy and watch changes use the kernel's existing journalled commands. Deep-links and onboarding are pure presentation reading existing data and traceability surfaces. Onboarding state is UI-local and out of the save.

Acceptance: every traced number/entity deep-links to a source-cited Sojournal entry (gaps flagged, not faked); the event log filters and pause policies are configurable and journalled; watches work; onboarding layers guide without blocking; New/Continue works; the accessibility floor (colour-blind palette, 1280×720→4K scaling, full keyboard nav) is met; no new authoritative data beyond theme/hotkeys; renderer holds no logic. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- The cross-link resolution contract: how a traced term maps to a Sojournal entry id, and how a missing target is reported as a content gap.
- Pause-policy granularity (pause/notify/ignore per event class) and the watch-condition expression surface.
- Onboarding trigger model (first-time-on-screen / no-fleet / etc.) and that it never blocks input.
- The accessibility checklist for the egui renderer.

## /speckit.plan — guidance

- `sojourn-game`/host intents wrap the kernel's existing `Command::{SetPausePolicy, RegisterWatch, ModifyWatch, RemoveWatch}` and `Session::start_new_esa()`/`load()`. No new sim.
- View-model: add `SojournalVM` (search/entry/cross-link resolution against `data/world` Sojournal entries + `TraceRender`), `EventLogVM` (filter/config), small UI-local `OnboardingVM`. Renderer: S11 (Browser/search, Entry, Cross-links), S12 (Full feed, Interrupt review, Pause-policy config, Watch conditions), the global trace "Sojournal ⓘ" affordance, coach-marks, the finished New/Continue entry, a hotkey cheat-sheet, the accessibility pass.
- Tests: assert that a representative set of derived figures (a Δv, a dose risk, an ISRU break-even, a body mass) resolve to a Sojournal entry or report a content gap; assert `SetPausePolicy`/`RegisterWatch` change interrupt behaviour deterministically and round-trip; view-model tests for search + cross-link resolution + filter logic.

## /speckit.tasks & /speckit.analyze — notes

Separate cross-link resolution + view-models, S11/S12 renderer + global trace affordance, onboarding + New/Continue + accessibility, tests. `/speckit.analyze` must confirm: educational honesty — every figure traces to a sourced entry or is flagged a gap, no faked sources (Principle VIII), no new authoritative data (Principle V), pause/watch state journalled + round-trips (Principle III), renderer holds no logic (Principle IV), core audit green.
