# GP-00 — Session & ESA Bootstrap · `/speckit` set (FA-11)

**Branch:** `012-session-bootstrap` · **Design:** `gameplay/01-SESSION-AND-BOOTSTRAP.md` · **Depends:** FA-01…FA-10

## /speckit.specify

```
/speckit.specify Make Sojourn boot into a coherent, playable ESA starting position in January 2026 and stand up the orchestration layer the rest of the gameplay programme builds on. Authoritative design: gameplay/00-CORE-LOOP.md, gameplay/01-SESSION-AND-BOOTSTRAP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles III, IV, V, VIII) — read them.

WHY: today UiHost::new_game() builds a core with every module loaded but empty (all Understanding Levels 0, no factions seeded, no funding, no fleet, no bases). There is no coherent ESA start-state and nothing that composes the decoupled slices into a single coherent game. The whole programme depends on a populated world and a place for cross-system causality to live.

Build two things. (1) A new STATELESS orchestration crate that sits above the slices and below the renderer: it builds the ESA default start-state as a deterministic tick-0 command script from sourced data, and exposes a thin Session over the existing UI host (start new ESA game, advance at a chosen time-warp, list and acknowledge pending interrupts, save, load). It holds NO authoritative state — the core does — and is testable headlessly with no renderer. (2) The ESA bootstrap itself: seed the world (factions with ESA as the player faction plus the always-AI competitors, astrobiology candidate priors, site planetary-protection categories, difficulty); register ESA's funding and apply the first appropriation; set opening balances for the six currencies; inject plausible 2026 Understanding Levels and a small starting personnel roster; spawn the real January-2026 assets (ISS/Columbus and one or two satellites at their real states, NO off-Earth bases); initialise the belief-state (well-known bodies known, prospecting fields unknown); seed the milestone ledger with the firsts already achieved in reality and leave the rest open; no Grand Goal selected yet. EVERY starting number must be sourced data in a new data/scenario/esa_2026.ron with source fields, never hard-coded.

Make the persistent shell live: the top bar shows the real simulated date and a run-state that flips PAUSED/RUNNING with the time-warp control and a pause hotkey; the six currencies show real balances; the alerts bell shows the pending-interrupt count; advancing time uses the kernel's existing interrupt-and-pause so the run halts on the first event; an interrupt-review surface lists firing events and lets the player acknowledge; the bottom event ticker renders the live feed. Add a minimal New/Continue entry (New game = run the ESA bootstrap; Continue = load latest save) — NO new-game configurator. Every other screen shows a calm "ESA 2026 — nothing here yet" empty state.

Acceptance: booting ESA shows plausible sourced numbers on all six currencies and a seeded world; advancing one year applies the next appropriation; the first interrupt fires and is acknowledgeable from the shell; save→load→continue is bit-identical; the orchestration crate is headless-testable; the renderer holds no game logic. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace's stack is fixed (Rust, egui renderer); reuse it.
```

## /speckit.clarify — focus points

- Crate name and exact boundary of the orchestration layer (proposed `sojourn-game`): confirm it is stateless and that `UiHost` depends on it rather than wiring slices ad-hoc.
- Exact ESA opening numbers (appropriation, balances, 2026 ULs, roster size, which sats) and their sources — these are data decisions needing a sourced answer.
- Which competitor factions are seeded at start and at what visibility.
- Whether the New/Continue entry is a separate screen or a shell overlay.

## /speckit.plan — guidance

- Create `sojourn-game` (lib) depending on `sojourn-core` + all slices; add `EsaBootstrap` (data → ordered `Vec<Command>` using the real `PolityCommand::InitWorld`, `EconomyCommand::{RegisterFunding, ApplyAppropriation, SetBalance}`, `ResearchCommand::{InjectUnderstanding, Hire}`, `AstroCommand::SpawnCraft`, `WorldCommand::Observe`) and a `Session` wrapper over `UiHost`. Add `UiHost::new_game_esa(data_root, seed)` that creates the core then runs `EsaBootstrap` before returning.
- Keep all authoritative state in the core; Session/UI ephemera (active screen, warp selection, onboarding) stay out of the save (as FR-UI-1505 already requires).
- Reuse the kernel's time-warp + watches + `AcknowledgeInterrupt`; do not reimplement interrupt logic.
- Renderer: extend the shell in `sojourn-ui-desktop`; add the interrupt-review modal and New/Continue; wire warp → `Session::advance`. New view-model: extend `ShellVM`, add `BootstrapSummary`.
- Tests: harness scenario `esa_bootstrap.ron`; determinism double-run + save round-trip; `sojourn-ui` view-model unit tests for `BootstrapSummary` + interrupt list.

## /speckit.tasks & /speckit.analyze — notes

Tasks should separate (a) crate + bootstrap data, (b) bootstrap script, (c) Session/host, (d) shell wiring, (e) tests. `/speckit.analyze` must confirm: `sojourn-core` audit still green (no new deps into the core), all start-state numbers sourced (Principle I/V), determinism + round-trip tests present (Principle III), renderer holds no logic (Principle IV).
