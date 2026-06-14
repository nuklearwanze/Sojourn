# Implementation Plan: User Interface & Presentation Layer (FA-10)

**Branch**: `011-user-interface` | **Date**: 2026-06-14 | **Spec**: `specs/011-user-interface/spec.md`
**Input**: Feature specification from `/specs/011-user-interface/spec.md`

## Summary

Build Sojourn's **desktop-class, data-dense, mostly-2D user interface** — the player's entire window into
the game and the project's final slice. It is the first slice that builds **on top of** the headless core
rather than as another `SimModule`: it **reads the slices' typed read-only query/snapshot/traceability
surfaces in-process and submits typed journalled commands**, holding **no game logic and no authoritative
state** (Constitution IV). A persistent shell (top bar, left nav, central work area, right inspector,
bottom ticker) hosts **all twelve screens fully** (System Map, Trajectory Planner, R&D, Vehicle Designer,
Operations, Economy, Bases, Personnel, World/Politics, Science-Returns & Astrobiology, Sojournal, Alerts)
plus the **bespoke widgets** (porkchop plot, Δv/TRL ladders, Understanding bars, resource-by-location
ledger, logistics-graph view, base schematic, astrobiology evidence meter), with **traceability on every
derived number, plan→preview→commit on every irreversible action, SI units, colour-blind-safe + scalable +
keyboard-navigable accessibility, and virtualised performance** for thousands of bodies and large fleets
at high time-warp.

Per the `/speckit-clarify` decisions, the architecture is: a **stay-in-sync-by-events-plus-pulled-snapshots**
read model over an **in-process typed boundary** (the UI links the slice crates and calls
`WorldSnapshot`/`CrewSnapshot`/… directly), with **state-changing config journalled to the core** (pause-policy
via the FA-01 command) and **all UI-only state ephemeral/local** (active screen, zoom, pinned inspectors,
pre-commit draft plans, layout — never in the deterministic save). The view-model — the pure mapping from
core snapshots to display data — is the **headless-testable seam** (view-model tests over stub snapshots,
no renderer); the rendering shell is thin. **No slice depends on the UI**; the core stays headless and the
determinism/audit path is untouched.

## Technical Context

**Language/Version**: Rust, workspace-pinned 1.96.0, edition 2024 (unchanged from FA-01…09).
**Primary Dependencies**: **Two new crates.** (1) `sojourn-ui` (lib, the **view-model + host**): depends on `sojourn-core` **and every gameplay slice** (`sojourn-astro`/`-world`/`-research`/`-vehicle`/`-economy`/`-base`/`-crew`/`-polity`) to pull their typed read-only snapshots, assemble the module set, and submit typed commands — **plus `libm` only** for SI formatting; **no GUI dependency** (so it builds + tests headlessly). (2) `sojourn-ui-desktop` (bin, the **renderer**): depends on `sojourn-ui` + **`eframe`/`egui`** (immediate-mode desktop GUI) + **`egui_extras`** (virtualised `TableBuilder`); it is the thin rendering shell over the view-model. The UI being the **composition root** that links all slices does NOT violate decoupling — the slices have no UI dependency, so the core's tree stays presentation-free (the audit greps `sojourn-core` only).
**Storage**: No new authoritative data. UI-only session state (active screen, zoom/focus, pinned inspectors, pre-commit draft plans, layout, theme/scale) is **ephemeral/local**, never part of the deterministic save (FR-UI-1505). The pause-policy is journalled **core** state (FA-01 `SetPausePolicy`). The UI reads the existing `data/*` (sourced) only through the slices' query surfaces; it adds no data files of its own except a small **UI theme/palette config** (colour-blind-safe palettes, default hotkeys) which carries no plausibility-bearing numbers.
**Testing**: `cargo test` (the **view-model tests** — pure functions mapping stub core snapshots → display structs: traceability-tree flattening, plan→preview→commit gating, SI-unit formatting, table row/sort/filter/group derivation, the bespoke-widget data shapes, the astrobiology-honesty guard). The view layer is tested **headlessly with no renderer**; egui rendering is not pixel-tested (out of automated scope per clarify). The core's determinism/headless suites are unchanged — the UI is **off** that path.
**Target Platform**: **Desktop-native** (Windows/Linux/macOS), one window legible from **1280×720 to 4K** (the Aurora 4x / EVE Online feel), via `eframe` (winit + wgpu/glow).
**Project Type**: A desktop application = a headless **view-model library** (`sojourn-ui`) + a thin **renderer binary** (`sojourn-ui-desktop`). No kernel/slice change beyond **small, headless, tested additions to a slice's query surface** if a screen needs a read the core does not yet expose (Assumption: no logic moves into the UI).
**Performance Goals**: The shell renders from a **pulled snapshot** (events + on-demand, throttled), not a per-frame recompute; dense tables and the System Map **virtualise / level-of-detail** so thousands of catalogued bodies and large fleets/ledgers stay interactive (scroll/filter/pan/zoom) at **the highest time-warp**; the UI never blocks the core's stepping.
**Constraints**: **Principle IV (the defining constraint)** — no game logic, no authoritative state in the view; reads only the slices' typed query/snapshot/traceability surfaces + the FA-01 event/interrupt feed; submits only typed journalled commands; the same core runs headless without the UI. **SI units only** (FR-UI-1401); **colour-blind-safe, scalable text, full keyboard nav** (FR-UI-1402…1404); **traceability** of every derived number to its sourced leaves (FR-UI-201); **plan→preview→commit** for every irreversible action (FR-UI-301); **honest astrobiology** (never a conclusive-positive the core has not set; never expose ground truth — FR-UI-1202).
**Scale/Scope**: All **twelve screens fully** (read + command + bespoke widgets), the persistent shell, eight bespoke widgets, and the cross-cutting accessibility/units/performance layer. The largest slice by surface; bounded by the uniform **view-model → widget** decomposition.

## Constitution Check

*GATE: evaluated against Constitution v1.0.0 before Phase 0; re-checked after Phase 1.*

| Principle | Status | How this plan complies |
|---|---|---|
| I. Plausibility / sourced data | PASS (consumed) | The UI introduces **no** plausibility-bearing numbers; it renders the slices' sourced data + traceability. The only UI data is a theme/palette/hotkey config (no physics). |
| II. Physics authoritative / no magic numbers | PASS (consumed) | The UI **never recomputes** physics; every derived figure comes from a core query and is rendered with its `TraceTree`. A red-flag/Δv/consensus shown is the core's, not the UI's. |
| III. Deterministic core | PASS (off-path) | The UI is **not** on the determinism/headless path; it drives time-warp + reads snapshots + submits journalled commands, never altering the core's stepping. The determinism suites run headless, unchanged. |
| IV. Headless / decoupled (**the headline**) | PASS | The core + every slice run **headless with no UI dependency**; the UI is a **pure consumer** of the typed query/snapshot/traceability surfaces + the event feed, contains **no game logic and no authoritative state** (FR-UI-1501…1506), and its display logic is **view-model-tested headlessly with no renderer**. The audit (`cargo tree -p sojourn-core`) stays presentation-free because nothing in the core tree depends on the UI. |
| V. Data-driven content | PASS | No mechanics added; the UI is code that renders data. Its theme/palette/hotkey config is data, schema-light, no sources needed (carries no realism). |
| VI. Research a modelled process | N/A (rendered) | R&D state (UL/TRL/P50-P80/insight-pressure/dead-ends) comes from FA-05 and is rendered, never reimplemented. |
| VII. Tyranny of mass / Δv | PASS (surfaced) | The UI is where mass/Δv become **felt and legible**: the Δv ladder vs available, the resource-by-location (Δv-addressed) ledger, the logistics graph priced in Δv/TOF, the vehicle designer's live mass/Δv. It surfaces the tyranny, never softens it. |
| VIII. Educational honesty | PASS | Traceability on every number (FR-UI-201/202), the source-cited Sojournal a click from any term (FR-UI-204/1301/1302), and an **honest astrobiology meter** that never shows a conclusive-positive the core has not set and never exposes the ground truth (FR-UI-1202). |
| IX. No combat/aliens | PASS | A presentation layer over a combat-free sim; discovered life is rendered as a science object (the evidence meter), never an actor. |
| Engineering constraints | PASS | **SI everywhere**, colour-blind-safe + scalable + full keyboard nav (1280×720→4K), virtualised thousands-row tables; the UI tech stack (`eframe`/`egui`) satisfies headless-core, data-dense-2D and accessibility requirements and keeps the determinism core renderer-free. |
| **Cross-slice coupling** | NOTED (one-way) | The UI **depends on** `sojourn-core` + all eight gameplay slices (it is the composition root that hosts the core and pulls every snapshot). This is the **correct** direction: the slices do **not** depend on the UI, so the core stays headless and the dependency/scope audit (which inspects the *core* tree) is unaffected. No new edge **into** the core. |

**Initial gate (pre-Phase-0)**: PASS. **Post-Phase-1 re-check (2026-06-14)**: design artifacts introduce no
new violations; no kernel/slice logic change (only, if needed, additive headless query reads); the UI holds
no authoritative state; the core tree stays presentation-free; the view-model is headless-testable. Gate
remains **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/011-user-interface/
├── plan.md              # This file
├── research.md          # Phase 0 output (R1–R15)
├── data-model.md        # Phase 1 output (the view-model + UI-state model)
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── ui-host.md               # The in-process host: snapshot pulls, event feed, time-warp stepping, command submit + rejection
│   ├── viewmodel.md             # The per-screen view-model shapes + shared trace-tree/units/widget data contracts
│   ├── screens.md               # The twelve screens → core read surfaces + bespoke widgets + commands
│   └── integration-seams.md     # The read map (slice query → screen) + the command map; the in-process typed boundary
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
├── sojourn-core/                # FA-01 — unchanged (no UI dependency; tree stays presentation-free)
├── sojourn-astro/ -world/ -research/ -vehicle/ -economy/ -base/ -crew/ -polity/
│                                # FA-02…09 — unchanged; their read-only query/snapshot/traceability surfaces are the UI's data
├── sojourn-ui/                  # THIS SLICE (lib) — the headless-testable VIEW-MODEL + HOST (no GUI dep)
│   ├── Cargo.toml               # deps: sojourn-core + all 8 gameplay slices + libm
│   └── src/
│       ├── lib.rs               # public surface: UiHost, the screen view-models, the units/trace helpers
│       ├── host.rs              # UiHost: owns a SimCore (all 9 modules installed), time-warp stepping, event feed, snapshot pulls, typed command submit + rejection
│       ├── modules.rs           # the module-set builder (load each slice's data dir + module) — the composition root
│       ├── read.rs              # the read-sync layer: subscribe to the FA-01 event feed; pull snapshots on event/nav/throttled tick; consistent-snapshot guarantee
│       ├── command.rs           # typed command submission + the plan→preview→commit envelope + rejection surfacing
│       ├── units.rs             # SI-unit formatting (pure, tested): km/s, kg, W, K, Sv, $, dates — consistent formatting
│       ├── trace.rs             # flatten/render-data for the core TraceTree (sourced-leaf rendering; no-tree fallback)
│       ├── viewmodel/           # ONE pure module per screen: snapshot(s) → display structs (tested headlessly)
│       │   ├── shell.rs         #   top bar (clock/warp/funds/alerts), nav, inspector, ticker view-model
│       │   ├── map.rs           #   System Map: bodies/orbits/craft/SOI/Lagrange/nodes/layers (LOD)
│       │   ├── planner.rs       #   porkchop field, manoeuvre nodes, low-thrust, flyby, Δv-vs-available
│       │   ├── research.rs      #   UL bars, TRL ladders, P50/P80, insight-pressure, dead-ends, tech-tree
│       │   ├── vehicle.rs       #   component picker (researched-only), live derived figures, red-flags, compare
│       │   ├── operations.rs    #   virtualised fleet table, timelines, manifest, crew health/dose
│       │   ├── economy.rs       #   P&L/appropriation, resource-by-location ledger, logistics graph, market, RFP
│       │   ├── bases.rs         #   site browser, base schematic gauges, construction stages, ISRU
│       │   ├── personnel.rs     #   pools/skills/traits/assignments/morale/health careers
│       │   ├── politics.rs      #   milestone race board, relationships/prestige/mood/policy, rival feed
│       │   ├── astrobiology.rs  #   the evidence meter (consensus/posteriors/band/disagreement; honesty guard), belief-state, discoveries
│       │   ├── sojournal.rs     #   search, cross-links, source-cited entries (belief-state-aware)
│       │   └── alerts.rs        #   event feed (filter/link), pause-policy config view-model
│       └── widgets_data.rs      # the data shapes for the 8 bespoke widgets (porkchop/Δv/TRL/UL/ledger/graph/schematic/meter)
│   └── tests/                   # viewmodel tests over STUB snapshots: map, planner, research, vehicle, operations,
│                                # economy, bases, personnel, politics, astrobiology, sojournal, alerts, units, trace, ppc (plan-preview-commit), common/mod.rs (stub builders)
├── sojourn-ui-desktop/          # THIS SLICE (bin) — the thin egui/eframe RENDERER over the view-model
│   ├── Cargo.toml               # deps: sojourn-ui + eframe + egui + egui_extras
│   └── src/
│       ├── main.rs              # eframe entry; wires UiHost + the shell + the active screen
│       ├── app.rs               # the eframe::App: per-frame pull(+throttle) → render the active screen's view-model
│       ├── shell.rs             # the persistent shell chrome (top bar, nav, inspector, ticker) + global hotkeys + scale/theme
│       ├── theme.rs             # colour-blind-safe palette, scalable text (pixels_per_point), formatting hookup
│       ├── widgets/             # egui rendering of the 8 bespoke widgets (custom Painter): porkchop, ladders, bars, ledger, graph, schematic, meter
│       └── screens/             # egui rendering of each screen's view-model (12 files; thin: layout + virtualised tables + widgets + inspectors)
data/
└── ui/
    └── theme.ron                # colour-blind-safe palettes + default hotkeys (no plausibility-bearing values)
```

**Structure Decision**: A **headless view-model library (`sojourn-ui`) + a thin renderer binary
(`sojourn-ui-desktop`)** on **`eframe`/`egui`** (immediate-mode Rust desktop GUI). The split is the heart of
the plan: the **view-model** — pure functions mapping core snapshots to display structs — carries *all* the
testable UI logic (traceability flattening, plan→preview→commit gating, SI formatting, table derivation, the
bespoke-widget data shapes, the astrobiology honesty guard) and is unit-tested **headlessly with no renderer**
(satisfying FR-UI-1506 and Principle IV's "testable without a renderer"); the **renderer** is a thin egui
layer that paints those structs. The UI is the **composition root** — `sojourn-ui` links every slice to host
a `SimCore` (the same module set the harness assembles) and pull typed snapshots in-process (FR-UI-1502) —
which is the correct one-way coupling: slices never depend on the UI, so the core stays headless and the
audit (which inspects only the `sojourn-core` tree) is unaffected. `egui` is chosen for immediate-mode
"render-from-snapshot" fit, first-class **custom painting** for the eight bespoke widgets and the log-zoom
System Map, `egui_extras::TableBuilder` **row virtualisation** for thousands-row catalogues/fleets/ledgers,
built-in **keyboard navigation** and **`pixels_per_point` text scaling**, and native cross-platform desktop
via `eframe`.

## Complexity Tracking

No constitution violations to justify — table intentionally empty. This is the **largest slice by surface**
(12 screens + a shell + 8 bespoke widgets), but it adds **no new architectural complexity**: a single,
uniform **view-model → widget** decomposition repeated per screen, one host, one read-sync layer, one
command path, over the slices' existing query surfaces. Breadth is contained by that uniformity and by the
headless view-model seam (each screen's logic is an independently-testable pure module). The UI being the
composition root that links all slices is the **intended** one-way dependency, not a violation — the core
and slices remain headless and renderer-free.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
