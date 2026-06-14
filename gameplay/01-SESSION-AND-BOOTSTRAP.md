# GP-00 — Session & ESA Bootstrap (FA-11)

**Spec dir:** `specs/012-session-bootstrap` · **Depends on:** FA-01…FA-10 (all implemented) · **Speckit:** `speckit/gameplay/gp-00-session-bootstrap.md`

The foundation. It makes the game *boot into ESA, January 2026* with coherent, plausible numbers; stands up the **`sojourn-game`** orchestration layer; and makes the persistent shell **live** (clock, currencies, run-state, event queue, interrupt review, save/load). Nothing deep is playable yet — but the world exists, time runs, and the loop's skeleton (observe → advance → consequence) works.

## Goal & player-facing capability

Launch `sojourn-ui-desktop`, land in ESA 2026, and: see real opening balances on the six currencies; read a seeded event queue; advance time at chosen warp; have the core **interrupt-and-pause** on the first event (e.g. the upcoming budget vote); review and acknowledge it; save, quit, reload, and continue bit-identically.

## Where it sits in the loop

Steps 1 (observe), 4 (advance) and 5 (consequence) of the session beat. Decisions (2/3) come online from GP-01.

## Orchestration — the `sojourn-game` layer

Create the crate (§3 of `00-CORE-LOOP.md`). For GP-00 it provides:

- **`EsaBootstrap`** — builds the deterministic tick-0 command script from `data/scenario/esa_2026.ron`, emitting (in order): `PolityCommand::InitWorld { factions (ESA=0, AI agencies + private archetypes), candidate priors bridged from data/world/astrobiology.ron, site PP categories, difficulty }`; `EconomyCommand::RegisterFunding` + `ApplyAppropriation` for ESA; opening `SetBalance` for each currency; baseline `ResearchCommand::InjectUnderstanding` for 2026 ULs and starting roster `Hire`; `AstroCommand::SpawnCraft` for ISS/sats at real states; initial `WorldCommand::Observe` for well-known bodies. All values come from sourced data.
- **`Session`** — a thin owner around `UiHost` exposing: `start_new_esa()`, `advance(warp)`, `pending_interrupts()`, `acknowledge()/acknowledge_all()`, `save()/load()`. It holds **no** authoritative state (the core does); UI-local ephemera (active screen, warp selection) stay outside the save.

`UiHost::new_game()` gains `new_game_esa(data_root, seed)` that creates the core *then* runs `EsaBootstrap` before returning, so the first frame already shows a populated ESA.

## Cross-system causality & state touched

Touches every slice once, at boot, through the bootstrap script. After tick 0 the only live loop is the kernel's: appropriations re-arm on the fiscal calendar (`RegisterFunding` period end), and seeded events feed interrupts. Durable state: all in the core (journalled). No `sojourn-mission` yet.

## ESA starting data (`data/scenario/esa_2026.ron`, all `source`-tagged)

Factions (ESA player + competitors), ESA annual appropriation and directed share, opening six-currency balances, fiscal-year vote date, 2026 Understanding Levels per domain (chemical high, electric mid, NTP/NEP low, closed-ecology low, astrodynamics high), starting personnel (a handful per role), starting assets (ISS/Columbus, 1–2 sats, **no off-Earth bases**), seeded milestone ledger (historical firsts pre-claimed), no Grand Goal yet.

## UI/UX

**Shell goes live** (see `UI-UX-CONVENTIONS.md` §1–2). Specifically:
- Top bar: real date + `T+NNNd` + **run-state** indicator that flips PAUSED/RUNNING with the warp control and the Space hotkey; six currencies show real balances and each is clickable (drills to its ledger from GP-01); alerts bell shows pending-interrupt count.
- **Interrupt review modal** (S12 subscreen): on pause, list the firing event(s) with class, message and ⏸ reason; "Acknowledge" / "Acknowledge all"; returns to PAUSED.
- **Event ticker** (bottom) renders the live feed with class colours and ⏸ markers.
- **New / Continue** entry point: a minimal launch screen with "New game (ESA)" (runs the bootstrap) and "Continue" (load latest save). No configurator.
- Empty states on every other screen read "ESA, 2026 — nothing here yet" with a one-line pointer to the screen that starts that activity.

View-model: extend `ShellVM` + add a `BootstrapSummary` view (opening balances, fiscal calendar, asset count) unit-tested headlessly. Renderer: wire the warp control to `Session::advance`, the bell/modal to `pending_interrupts`/`acknowledge`.

## Testability (the playable thread)

Harness scenario `esa_bootstrap.ron`: boot ESA → assert non-zero plausible balances, seeded factions, non-empty roster, ISS present; advance 1 year → next appropriation applied, balance changes by the sourced amount; first interrupt fires and is acknowledgeable. Plus: determinism double-run (same seed ⇒ identical state hash + event log) and save→load→continue identity. View-model unit tests for `BootstrapSummary` and the interrupt list. Human check: the desktop app opens into a populated ESA and time advances to an interrupt.

## Acceptance criteria

Boots into ESA with coherent sourced numbers on all six currencies and a seeded world; time-warp + interrupt-and-pause + acknowledge work from the shell; save/load round-trips bit-identically; `sojourn-game` exists and is headless-testable; no game logic in the renderer; all start-state numbers are sourced data.

## Out of scope (deferred)

All deep verbs (allocate, design, buy, fly, found, assign) — they arrive in GP-01+. The Grand Goal selection (GP-08). The Sojournal deep-linking and pause-policy editor (GP-09).
