# Phase 0 Research — User Interface & Presentation Layer (FA-10)

All NEEDS-CLARIFICATION items are resolved. The four `/speckit-clarify` decisions (events + pulled
snapshots; in-process typed queries; view-model tests on stub snapshots; config journalled / view
ephemeral) and the three `/speckit-specify` decisions (desktop-native; all twelve screens fully; no
onboarding) are settled and recorded against the relevant items.

---

## R1 — UI tech stack (the one deferred decision)

- **Decision**: **`eframe`/`egui`** (immediate-mode Rust desktop GUI) + **`egui_extras`** (virtualised
  `TableBuilder`). The renderer is a thin binary (`sojourn-ui-desktop`) over the headless `sojourn-ui`
  view-model library.
- **Rationale**: (a) immediate-mode "render from a pulled snapshot each frame" matches the events+pulled-
  snapshots read model exactly; (b) first-class **custom painting** (`egui::Painter`) for the eight bespoke
  widgets and the log-zoom System Map; (c) `egui_extras::TableBuilder` gives **row virtualisation**
  (`show_rows`) for thousands-row catalogues/fleets/ledgers; (d) built-in **keyboard navigation** and
  **`pixels_per_point`** text scaling for accessibility; (e) native cross-platform desktop (winit + wgpu/
  glow) for 1280×720→4K; (f) pure-Rust, in-process with the core — no IPC/serialization layer needed.
- **Alternatives considered**: `iced` (Elm-retained — clean but heavier for live data-viz dashboards and
  custom plots); `tauri`+web (a web frontend — rejected per the desktop-native clarify and to avoid a
  serialization boundary); raw `wgpu`/`winit` (too low-level — we are building data viz, not an engine).
- **Audit safety**: `eframe`/`egui`/`winit`/`wgpu` live only in `sojourn-ui-desktop`'s tree; `sojourn-core`
  has **no** dependency on the UI, so `cargo tree -p sojourn-core` stays presentation-free (FR-CORE-702).

## R2 — The headless view-model / thin-renderer split (Principle IV + clarify Q3)

- **Decision**: split the slice into **`sojourn-ui`** (lib: the **view-model + host**, no GUI dep) and
  **`sojourn-ui-desktop`** (bin: the egui renderer). The view-model is **pure functions** `snapshot(s) →
  display structs`; the renderer paints those structs.
- **Rationale**: it puts *all* testable UI logic (traceability flattening, plan→preview→commit gating, SI
  formatting, table derivation, widget data shapes, the astrobiology honesty guard) behind a **renderer-free,
  headlessly-testable** boundary — directly satisfying FR-UI-1506 and Principle IV ("every gameplay system…
  testable headlessly without a renderer", applied to the view layer). The renderer stays thin and is not
  pixel-tested (out of automated scope per clarify).
- **Alternatives**: a single GUI crate with logic inside widgets (rejected — couples logic to egui, breaks
  headless testability and the audit cleanliness).

## R3 — The in-process host & read-sync model (clarify Q1, Q2)

- **Decision**: `UiHost` (in `sojourn-ui`) **owns a `SimCore`** with all nine modules installed (the same
  module set the harness assembles), drives **time-warp stepping** (`StepRequest::Ticks`), and exposes:
  (a) a **subscribe** to the FA-01 event/interrupt feed; (b) **on-demand snapshot pulls** of the typed slice
  surfaces (`WorldSnapshot::from_core`, `CrewSnapshot::from_core`, astro/world/research/vehicle/economy/base
  queries); (c) **typed command submission** (the same `Command`/`ModulePayload` envelopes). The renderer
  pulls **on event, on navigation, and on a throttled tick** — never a full recompute every frame.
- **Rationale**: events tell the UI *what changed*; pulled snapshots give a **consistent core state**
  (coherent snapshot + matching clock, FR-UI-1503) cheaply at high warp. In-process typed access (FR-UI-1502)
  needs no serialization. The host never blocks the core's stepping.
- **Alternatives**: poll-every-frame (rejected per clarify — wasteful at warp); a serialized view-DTO
  envelope (rejected per clarify — unnecessary for a single desktop binary; the public query API is the seam).

## R4 — Command path, plan→preview→commit & rejection (US3)

- **Decision**: a `command.rs` envelope: a **draft plan** is UI-only state; a **preview** is the **core-
  computed** consequence (obtained by a read/dry-run query the relevant slice exposes, or by a guarded
  no-op evaluation), shown before an explicit **commit** submits exactly the previewed `Command`. A command
  the core **rejects** (`CommandOutcome::Rejected`) surfaces its reason; the UI never shows a rejected
  command as applied. Re-preview on stale state (FR-UI-304).
- **Rationale**: the safety contract (FR-UI-301…305). The consequence is always the **core's**, never the
  UI's. Reversible actions (layer toggles, sort) bypass the gate.
- **Alternatives**: optimistic UI (rejected — risks showing an outcome the core would reject; determinism +
  honesty favour submit→apply→re-read).

## R5 — Traceability rendering (US2, Principle VIII)

- **Decision**: a `trace.rs` that renders the slices' existing **`TraceTree`** (sourced `Leaf`/operation
  `Node`, `all_leaves_sourced`) into an expandable inspector — operation nodes and **sourced leaves shown
  exactly as the core provides**. A value with no tree shows the value + "derivation unavailable"; a leaf
  missing a `source` is **visibly flagged** (a core data defect, surfaced not hidden).
- **Rationale**: traceability is the trust contract; the UI **renders** the derivation, never reconstructs
  it (FR-UI-201/202). The slices already publish `trace.rs` trees (FA-04/06/07/08/09).
- **Alternatives**: UI-side recomputation of breakdowns (rejected — violates Principle IV/II).

## R6 — SI-unit formatting (US14, FR-UI-1401)

- **Decision**: a pure `units.rs` with tested formatters — velocity (km/s, m/s), mass (kg/t), power (W/kW/
  MW), energy, temperature (K), dose (Sv/mSv), pressure, currency, Δv, TOF/dates — with **consistent
  formatting** and explicit units, **no imperial**. SI formatting is part of the testable view-model.
- **Rationale**: a unit audit (SC-007) must find zero non-SI units; centralising formatting makes that
  auditable and testable.

## R7 — Accessibility: colour, scaling, keyboard (US14, FR-UI-1402…1404)

- **Decision**: a `theme.rs` + `data/ui/theme.ron` with **colour-blind-safe palettes** where colour is
  **never the sole carrier** (always paired with shape/label/value); **`pixels_per_point`** text scaling for
  1280×720→4K; **full keyboard navigation** + hotkeys (time-warp, screen switching, common actions) via
  egui's focus/keyboard model. No twitch input — the interrupt-and-pause loop gives reading time.
- **Rationale**: constitutional accessibility constraints; the design's "colour-blind-safe, scalable fonts,
  full keyboard nav".

## R8 — Virtualised tables & map level-of-detail (US14, FR-UI-1405)

- **Decision**: `egui_extras::TableBuilder` with **`show_rows`** virtualisation for the fleet/catalogue/
  ledger tables (render only visible rows); the **System Map** uses **level-of-detail** (cull/aggregate
  off-screen and sub-pixel bodies, draw orbits at zoom-appropriate resolution) so thousands of bodies stay
  interactive at high warp. Filter/sort/group are computed in the view-model over the pulled snapshot.
- **Rationale**: the performance success criterion (SC-006) at the highest time-warp.

## R9 — The System Map (US1, FR-UI-102/103)

- **Decision**: a custom-painted 2D map (`egui::Painter`) with **logarithmic zoom** and **multi-focus**
  (heliocentric ↔ planetocentric ↔ local-ops); bodies/orbits from the FA-02/03 astro+catalogue queries
  (`state_at`, elements), craft/trajectories from the ops surfaces, SOI/Lagrange/transport-graph nodes;
  **toggleable layers** (resources, comms, traffic, PP zones, science) and an **inertial/rotating frame**
  selector. Click → inspector; right-click → context verbs. Focus/zoom/layer state is **UI-ephemeral**.
- **Rationale**: the hero screen; everything spatial reads from the astro/world surfaces, nothing computed
  in the UI.

## R10 — The Trajectory Planner & porkchop (US4)

- **Decision**: the **porkchop** widget renders the **Δv/TOF/C3 contour field the core provides** (FA-02
  Lambert/transfer queries) as a custom-painted contour plot with a pickable point; the manoeuvre-node
  editor / low-thrust arc / flyby designer update the **core-previewed** trajectory + Δv; the **required-vs-
  available Δv** check reads the selected vehicle (FA-04). Queue burns via plan→preview→commit with
  auto-pause at nodes.
- **Rationale**: the most distinctive bespoke screen; the contours/Δv are the core's, the UI plots them.
- **Note**: if FA-02 does not yet expose a porkchop-field query, it is added as a **headless, tested** read
  on `sojourn-astro` (no logic in the UI) — the Assumption's "extend the slice's query surface" path.

## R11 — The bespoke widgets (US4–US12, FR §4)

- **Decision**: eight widgets as **(data shape in `sojourn-ui::widgets_data`, tested) + (egui painter in
  `sojourn-ui-desktop::widgets`, thin)**: porkchop plot, Δv ladder, TRL ladder, Understanding bars (with
  world-tide ghost + insight-pressure shimmer), resource-by-location ledger, logistics-graph view (nodes/
  edges priced in Δv/TOF), base schematic (live emergent-property gauges), astrobiology evidence meter.
- **Rationale**: each is a renderer of a specific **core data shape**; the data shape is testable, the paint
  is thin.

## R12 — The astrobiology evidence meter & honesty guard (US12, FR-UI-1202, Principle VIII)

- **Decision**: the meter renders FA-09's per-candidate **community consensus + per-faction posteriors +
  confidence band + conclusive status + public disagreement** — **never a binary popup**. A **view-model
  honesty guard** (tested) ensures the UI **never** shows a conclusive-positive the core has not set and
  **never** reads/has-access-to the hidden ground truth (FA-09 exposes no ground-truth accessor, so the UI
  structurally cannot).
- **Rationale**: educational honesty is non-negotiable; the guard is a tested invariant (SC-009).

## R13 — UI-only state vs the deterministic save (clarify Q4, FR-UI-1505)

- **Decision**: **state-changing config is journalled core state** — the pause-policy is set via the FA-01
  `SetPausePolicy` command (so it round-trips with the save). **UI-only state** (active screen, zoom/focus,
  pinned inspectors, **pre-commit draft plans**, layout, theme/scale) is **ephemeral/local** and **not** part
  of the deterministic save; it MAY persist in a separate local UI-profile (a convenience, never affecting
  the sim). The deterministic save remains exactly the core's (FA-01).
- **Rationale**: keeps the save deterministic and renderer-independent; the UI can be closed/reopened/swapped
  without changing game state.

## R14 — Testing strategy & CI (clarify Q3, FR-UI-1506)

- **Decision**: **view-model tests** in `sojourn-ui/tests/` over **stub core snapshots** (hand-built display
  inputs) cover traceability flattening, plan→preview→commit gating, SI formatting, table derivation
  (filter/sort/group/virtualise math), widget data shapes, and the astrobiology honesty guard — **no
  renderer**. They run in the normal `cargo test --workspace`. The **determinism/headless/audit jobs are
  unchanged** (the UI is off that path). CI builds `sojourn-ui-desktop` (egui) in the existing lint/test
  jobs; if a runner lacks the windowing dev libs, the build step installs them — the **determinism job
  (which builds only `sojourn-harness`) and the core audit are never touched**.
- **Rationale**: satisfies the headless-testability mandate while keeping the deterministic gates clean.
- **Alternatives**: full UI automation (rejected per clarify — heavy/flaky); manual-only (rejected — no
  regression safety).

## R15 — Closing core-read gaps without UI logic (Assumption)

- **Decision**: if a screen needs a value the core does not yet expose, the gap is closed by a **small,
  headless, tested addition to the relevant slice's query surface** (e.g. a porkchop-field read on
  `sojourn-astro`, an aggregate roster read), **never** by computing it in the UI. Such additions keep
  Principle IV intact and are covered by that slice's own tests.
- **Rationale**: the UI must remain a pure consumer (FR-UI-1501/1504); the honest fix for a missing read is
  to expose it from the core, headlessly.
