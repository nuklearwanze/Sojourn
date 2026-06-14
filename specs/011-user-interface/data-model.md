# Phase 1 Data Model — User Interface & Presentation Layer (FA-10)

The UI owns **no authoritative data**. This "model" is the **view-model** — the pure display structs the
`sojourn-ui` library derives from core snapshots — plus the UI-only session state and the command/preview
envelope. Everything authoritative lives in the core and is **read**, never held. All view-model derivation
is pure and **headlessly tested** over stub snapshots.

## 1. The host (`host.rs`) — the in-process bridge to the core

- **UiHost**: owns a `SimCore` with all nine modules installed (the harness module set). Exposes:
  - `status() -> { tick, date, lifecycle, funds_by_faction, political_capital }` (clock matches every snapshot).
  - `set_time_warp(rate)` / `pause()` / `step()` — drives `StepRequest`; **never blocks** the core elsewhere.
  - `events(filter) -> Vec<EventView>` and `pending_interrupts() -> Vec<…>` (the FA-01 feed).
  - `acknowledge(interrupt_id)`.
  - `submit(Command) -> CommandOutcome` (typed; `Applied` / `Rejected(reason)`).
  - typed **snapshot pulls**: `world()`/`crew()`/`polity()` snapshots + astro/research/vehicle/economy/base
    query accessors — the slices' existing read-only surfaces, in-process.
- **ReadSync** (`read.rs`): tracks the last pulled snapshot generation; re-pulls **on event, on navigation,
  on a throttled tick**; guarantees a **consistent snapshot + matching clock** (never a half-updated mix).

## 2. UI-only session state (ephemeral/local — never in the deterministic save)

- **UiState**: `{ active_screen: ScreenId, map_focus, map_zoom, layers_on: set, frame: Inertial|Rotating,
  pinned_inspectors: Vec<InspectorRef>, draft_plans: Vec<DraftPlan>, theme, pixels_per_point, hotkeys }`.
- **ScreenId**: `Map | Planner | Research | Vehicle | Operations | Economy | Bases | Personnel | Politics |
  Astrobiology | Sojournal | Alerts`.
- Persists (optionally) to a **local UI-profile** only; **NOT** part of the core's deterministic save
  (FR-UI-1505). The pause-policy is **NOT** here — it is journalled core state (FA-01 `SetPausePolicy`).

## 3. Command, draft plan & preview (`command.rs`)

- **DraftPlan**: a UI-only draft of an irreversible action (burn/launch/build/cancel/manoeuvre/scrap) — its
  parameters, not yet submitted.
- **Preview**: the **core-computed** consequences of a draft `{ deltas: Vec<TracedValue>, irreversible: bool,
  warnings: Vec<String>, becomes_unrecoverable: Vec<String> }` — obtained from a core read/dry-run, never
  computed in the UI.
- **CommitOutcome**: `Applied | Rejected(reason)` surfaced from `CommandOutcome`; on stale state the UI
  **re-previews** before committing (FR-UI-304).

## 4. Shared view-model primitives

- **TracedValue**: `{ label, formatted: String /*SI*/, raw: f64, tree: Option<TraceRender> }` — a display
  value that can expand into its sourced derivation. `TraceRender` mirrors the core `TraceTree` (operation
  nodes + **sourced leaves**; a missing source is **flagged**).
- **Disclosure**: `Summary | Full` — every screen view-model renders at both levels without changing values.
- **Units** (`units.rs`): pure SI formatters (velocity/mass/power/energy/temp/dose/pressure/currency/Δv/
  TOF/date) — the single source of formatting; a unit audit verifies no imperial.
- **TableModel<Row>**: `{ rows, columns, sort, filter, group, visible_range }` — the virtualisation/derive
  math (which rows are visible, sorted/filtered/grouped order) computed in the view-model; the renderer
  paints only `visible_range`.

## 5. Per-screen view-models (`viewmodel/*`) — pure `snapshot(s) → display struct`

- **ShellVM**: top bar (date/warp/funds/political-capital/alert-summary), nav items, pinned inspector(s),
  event ticker.
- **MapVM**: bodies (pos/size/label, LOD-culled), orbits (zoom-resolution), craft + trajectories, SOI/
  Lagrange regions, transport-graph nodes, active layers, frame; selection → inspector; context verbs.
- **PlannerVM**: porkchop field (Δv/TOF/C3 from FA-02), selected transfer, manoeuvre nodes, low-thrust arc,
  flyby legs, **required Δv vs the selected vehicle's available Δv** (feasible?), queued burns.
- **ResearchVM**: Domain UL bars (+ world-tide ghost + insight-pressure), active programs, RP/DE allocation,
  TRL ladders (+ test-campaign + risk), P50/P80-vs-actual, dead-end warnings, tech-tree nodes (+ source tags).
- **VehicleVM**: component picker (researched-only), live mass/Δv/power/thermal/reliability/cost as
  `TracedValue`s, realism red-flags (from the core), saved classes, side-by-side compare.
- **OperationsVM**: `TableModel<CraftRow>` (status/location/fuel/health/task/ops/comms), mission timeline,
  launch manifest/cadence, crew health/dose (FA-08).
- **EconomyVM**: P&L (company) / appropriation (agency) with mood modifiers (FA-09, traced), resource-by-
  location ledger (`TableModel`, Δv-addressed), market prices/trends, logistics graph (nodes/edges Δv/TOF),
  RFP board, facilities, learning curve.
- **BasesVM**: site browser (FA-03 properties + PP category), base schematic with emergent-property gauges
  (power/closure/population/sustainability from FA-07), construction stages, ISRU status.
- **PersonnelVM**: pools (scientists/engineers/PMs/astronauts/controllers/diplomats), skills/traits/
  assignments/morale/recruitment, crew health careers (FA-05/FA-08).
- **PoliticsVM**: milestone race board (world-/faction-first + unclaimed, FA-09), relationships/prestige/
  mood/policy (traced), rival activity feed.
- **AstrobiologyVM**: per-candidate **evidence meter** (consensus/posteriors/band/conclusive/disagreement —
  the **honesty guard** forbids a conclusive-positive the core has not set and exposes no ground truth),
  belief-state per body (Geoscience UL), incoming data, discoveries log.
- **SojournalVM**: search results, cross-links, source-cited entries (belief-state-aware), term→entry resolver.
- **AlertsVM**: event feed (`TableModel`, filter-by-class, link-to-screen), pause-policy config (which classes
  interrupt — edits submit the FA-01 command), pending interrupts.

## 6. Bespoke-widget data shapes (`widgets_data.rs`)

- **PorkchopField**: a Δv/TOF/C3 grid + the pickable optimum (from FA-02).
- **DvLadder**: stage/segment Δv vs the vehicle's available Δv (live).
- **TrlLadder**: TRL rungs + current + test-campaign + risk overlay.
- **UnderstandingBars**: per-Domain UL + world-tide ghost + insight-pressure level.
- **ResourceLedger**: inventory rows grouped by Δv-location.
- **LogisticsGraph**: nodes (dynamical locations) + edges (Δv/TOF prices).
- **BaseSchematic**: module layout + emergent-property gauge values.
- **EvidenceMeter**: stages, per-faction posteriors, consensus, band, conclusive flag, disagreement flag.

## 7. Invariants (tested in the view-model)

- **No computation of authoritative values**: every `TracedValue.raw` originates from a core query; the
  view-model only **formats/derives presentation** (sort/filter/group/visible-range), never physics.
- **Traceability**: a `TracedValue` with a core tree renders every operation node + **sourced leaf**; a
  missing source is **flagged**, never hidden; a value with no tree shows "derivation unavailable".
- **Plan→preview→commit**: an irreversible `DraftPlan` cannot be submitted without a `Preview`; a reversible
  action carries none; a rejected commit surfaces its reason.
- **SI only**: every formatted value passes through `units.rs`; an audit finds no imperial units.
- **Astrobiology honesty**: the `AstrobiologyVM` never marks a candidate conclusive-positive unless the core
  did, and has **no** access to ground truth (FA-09 exposes none).
- **UI owns no save state**: `UiState` is ephemeral/local; the deterministic save is exactly the core's.
- **Consistent snapshot**: a screen view-model is built from **one** coherent snapshot + its matching clock.
