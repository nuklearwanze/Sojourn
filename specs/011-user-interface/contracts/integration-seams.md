# Contract — Integration seams (the in-process typed boundary)

`sojourn-ui` is the **composition root**: it links `sojourn-core` + every gameplay slice, hosts a `SimCore`,
and consumes the slices' **read-only typed query/snapshot/traceability surfaces in-process** (clarify Q2 —
no serialized DTO layer). The slices **never** depend on `sojourn-ui`; the dependency is strictly one-way,
so the core stays headless and the audit (which inspects only the `sojourn-core` tree) is unaffected.

## Read map (slice surface → consumed by)

| Slice | Read surface the UI consumes (existing) | Screens |
|---|---|---|
| **FA-01 core** | `status()`/clock, the event/interrupt feed, `submit`/`acknowledge`, save/load | shell, alerts, all |
| **FA-02 astro** | `state_at`, elements, transfer/Lambert + porkchop-field read | map, planner, Sojournal |
| **FA-03 world** | catalogue/sites/locations, belief-state, **Sojournal entries + citations**, PP categories | map, bases, astrobiology, Sojournal |
| **FA-05 research** | UL/TRL/P50-P80/insight-pressure/dead-ends/tech-tree, personnel | R&D, personnel |
| **FA-04 vehicle** | derived mass/Δv/power/thermal/reliability/cost + realism red-flags + traces | vehicle, planner |
| **FA-06 economy** | budgets/ledger (by location)/market/contracts/facilities/learning + traces | economy, operations |
| **FA-07 base** | emergent properties/construction/ISRU + traces | bases, operations |
| **FA-08 crew** | `CrewSnapshot` (health/dose/capability/viability) + traces | operations, personnel |
| **FA-09 polity** | `WorldSnapshot` (milestones/prestige/mood/policy/PP/astrobiology consensus) + traces | politics, astrobiology, economy |

## Command map (UI → core, typed journalled)

- The UI submits the **same** `Command`/`ModulePayload` envelopes the harness uses: astro/world/research/
  vehicle/economy/base/crew/polity command payloads + kernel commands (incl. `SetPausePolicy`).
- Irreversible commands route through **plan→preview→commit** (`UiHost::preview` → confirm → `submit`).

## Closing read gaps (the honest path)

- If a screen needs a value not yet exposed (e.g. a porkchop-field read on `sojourn-astro`, an aggregate
  roster read), the gap is closed by a **small, headless, tested addition to that slice's query surface** —
  **never** by computing it in the UI (FR-UI-1501/1504, R15). Such additions are covered by the slice's own
  headless tests and keep the determinism path clean.

## What the UI never does

- Never holds authoritative state, never recomputes physics, never fabricates a value, never exposes the
  astrobiology ground truth, never blocks the core's stepping, and never introduces a dependency **into**
  the core or any slice.
