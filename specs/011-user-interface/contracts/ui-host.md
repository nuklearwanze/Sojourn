# Contract — The in-process UI host (`sojourn-ui::UiHost`)

The bridge between the renderer and the headless core. `UiHost` owns a `SimCore` (all nine modules
installed — the harness module set) and is the **only** place the UI touches the core. It exposes reads,
the event feed, time control and typed command submission; it holds **no game logic** and **never blocks or
alters** the core's deterministic stepping.

## Construction

- `UiHost::new_game(config) -> UiHost` / `UiHost::load(save) -> UiHost` — create/load a `SimCore` with the
  full module set (`modules.rs` loads each slice's `data/*` dir + module, exactly as the harness does).
- The module-set builder is the **composition root**; it is the one component that links every slice.

## Clock & time-warp

- `status() -> Status { tick, date, lifecycle, funds_by_faction, political_capital }` — the clock that
  **every** pulled snapshot matches.
- `set_time_warp(rate)` — sets the stepping rate (1 s/s … high warp); `pause()` / `resume()`.
- `tick()` — advances the core by the warp-appropriate `StepRequest`; auto-pauses on configured interrupt
  classes. MUST return promptly and never deadlock the renderer.

## Reads (typed, in-process — the boundary)

- Typed snapshot pulls of the slices' **existing read-only surfaces**: `world()`, `crew()`, `polity()`
  snapshots; astro/research/vehicle/economy/base query accessors. Each returns a **consistent** snapshot
  matching `status().tick`.
- `events(filter) -> Vec<EventView>` — the FA-01 event feed (paged/filterable).
- `pending_interrupts() -> Vec<InterruptView>`; `acknowledge(id)` — the interrupt-and-pause loop.
- Reads MUST be cheap enough to pull **on event / on navigation / on a throttled tick** at high warp.

## Commands (typed, journalled)

- `preview(draft: DraftPlan) -> Preview` — the **core-computed** consequences of an irreversible action
  (via a core read/dry-run); the UI never computes them.
- `submit(Command) -> CommandOutcome` — submits the **same** typed `Command`/`ModulePayload` envelope the
  headless harness uses; returns `Applied` or `Rejected(reason)`.
- `set_pause_policy(class, interrupt: bool)` — submits the **FA-01 journalled** pause-policy command (pause
  config is core state, not UI state).

## Guarantees

- **No authoritative state in the host beyond the owned `SimCore`** — the host is a thin façade over the
  core's public API.
- **Consistent reads** — a snapshot + `status()` always describe one coherent core state.
- **Off the determinism path** — the same `SimCore` runs headless in the harness; `UiHost` adds no
  nondeterminism and is not part of the determinism gates.
- **One-way dependency** — `sojourn-ui` depends on the slices; the slices never depend on `sojourn-ui`.
