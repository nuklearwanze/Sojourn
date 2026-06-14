# Contract — The twelve screens (read map · widgets · commands)

Each screen is a `viewmodel/<screen>.rs` (pure, tested) rendered by a `sojourn-ui-desktop/screens/<screen>.rs`
(thin egui). Every screen reads **only** the listed slice surfaces, paints the listed bespoke widgets, and
submits the listed commands through **plan→preview→commit** where irreversible.

| # | Screen | Reads (slice surfaces) | Bespoke widgets | Commands (→ core) |
|---|--------|------------------------|-----------------|-------------------|
| S1 | **System Map** (hero) | astro (`state_at`/elements), world (catalogue/sites/locations), ops/craft, polity (PP zones) | — (custom map painter; layers, LOD) | navigation only (reversible); right-click verbs route to other screens |
| S2 | **Trajectory Planner** | astro (transfer/Lambert/porkchop field), vehicle (available Δv) | **porkchop plot**, **Δv ladder** | queue burns / manoeuvre nodes (PPC, auto-pause) |
| S3 | **R&D** | research (UL/TRL/P50-P80/insight/dead-ends/tech-tree) | **TRL ladder**, **Understanding bars** | RP/DE allocation, lead assignment, start/cancel program (PPC for cancel) |
| S4 | **Vehicle Designer** | vehicle (derived figures + red-flags), research (researched components) | **Δv ladder** | save/iterate class (reversible); scrap (PPC) |
| S5 | **Operations / Fleet** | vehicle/economy/base (assets), crew (health/dose), ops (tasks/comms) | resource/timeline tables | launch manifest, task orders (PPC for launch) |
| S6 | **Economy & Contracts** | economy (budgets/ledger/market/contracts/facilities), polity (mood modifiers) | **resource-by-location ledger**, **logistics-graph view** | post/bid contract, build facility (PPC) |
| S7 | **Bases & Construction** | base (emergent properties/construction/ISRU), world (sites/PP) | **base schematic** | found base, add module, deliver/build (PPC) |
| S8 | **Personnel** | research (personnel), crew (health careers) | — (tables) | recruit/train/assign (PPC for irreversible) |
| S9 | **World / Politics** | polity (milestones/relationships/prestige/mood/policy) | milestone race board | lobby, select/change Grand Goal (PPC for change) |
| S10 | **Science Returns & Astrobiology** | polity (astrobiology), world (Geoscience UL/belief) | **astrobiology evidence meter** | — (read-heavy; evidence accrues via missions elsewhere) |
| S11 | **Sojournal** | world (Sojournal entries + citations), belief-state | — (search/cross-link) | — (reference only) |
| S12 | **Alerts / Event Log** | core event/interrupt feed | event table | set pause-policy (FA-01 journalled), acknowledge interrupt |

## Shell (all screens)

- Top bar: date/clock, time-warp controls, funds/political-capital, alert summary (from `UiHost::status`).
- Left nav (keyboard-switchable) to all twelve screens; central work area; right **inspector** (pin +
  expand to `TraceRender`); bottom **event ticker**.
- **Global**: time-warp + screen-switch + common-action hotkeys; `pixels_per_point` scaling; colour-blind-safe
  theme. Focus/zoom/layer/screen state is **UI-ephemeral**.

## Cross-cutting obligations (every screen)

- **SI units** via `units` (FR-UI-1401); **traceability** on every derived number (FR-UI-201); **progressive
  disclosure** Summary/Full (FR-UI-203); any term → **Sojournal** (FR-UI-204/1302); **virtualised** tables/
  map (FR-UI-1405); **keyboard path** for every primary action (FR-UI-1404); **plan→preview→commit** for
  every irreversible action (FR-UI-301).
