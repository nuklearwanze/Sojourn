---
description: "Task list for User Interface & Presentation Layer (FA-10)"
---

# Tasks: User Interface & Presentation Layer (FA-10)

**Input**: Design documents from `/specs/011-user-interface/`
**Prerequisites**: plan.md, spec.md, research.md (R1–R15), data-model.md, contracts/ (4)

**Tests**: REQUIRED. Constitution IV mandates that the view layer be **testable headlessly without a
renderer**; the `sojourn-ui` **view-model tests over stub core snapshots** are that seam (traceability
flattening, plan→preview→commit gating, SI formatting, table derivation, widget data shapes, the
astrobiology honesty guard). The egui renderer is **not** pixel-tested (out of automated scope per clarify).
The core's determinism/headless gates are unchanged — the UI is off that path.

**Organization**: by user story (US1–US14). Two crates per plan.md: **`sojourn-ui`** (lib — the
headless-testable **view-model + host**, deps `sojourn-core` + all 8 gameplay slices + `libm`, **no GUI dep**)
and **`sojourn-ui-desktop`** (bin — the thin **egui/eframe** renderer over the view-model). The UI is the
**composition root** (links every slice to host a `SimCore`); the slices **never** depend on the UI, so the
core tree stays presentation-free and the determinism/audit path is untouched. **No kernel/slice logic
change** — only, where a screen needs a missing read, a small **headless, tested** addition to that slice's
query surface (R15).

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1…US14 (Setup/Foundational/Polish carry no story label)

---

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Create `crates/sojourn-ui` (lib): `Cargo.toml` (deps `sojourn-core` + `sojourn-astro`/`-world`/`-research`/`-vehicle`/`-economy`/`-base`/`-crew`/`-polity` + `libm`; workspace lints; **no GUI dep**) and `src/lib.rs` (`#![forbid(unsafe_code)]`, module decls); add `"crates/sojourn-ui"` to workspace `members`.
- [x] T002 Create `crates/sojourn-ui-desktop` (bin): `Cargo.toml` (deps `sojourn-ui` + `eframe` + `egui` + `egui_extras`) and `src/main.rs` stub; add `"crates/sojourn-ui-desktop"` to workspace `members`.
- [x] T003 [P] Add `eframe`, `egui`, `egui_extras` to `[workspace.dependencies]` in the root `Cargo.toml` (pinned versions); scaffold `data/ui/` with a placeholder `theme.ron` header.
- [x] T004 [P] Confirm clippy/deny apply via workspace lints on both new crates; assert the **core tree stays presentation-free** (`cargo tree -p sojourn-core` contains no `egui`/`eframe`/`winit`/`wgpu`).

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: no screen work begins until this phase completes.

- [x] T005 Implement the **module-set builder** (composition root) in `crates/sojourn-ui/src/modules.rs`: load each slice's `data/*` dir + construct its module, returning the full nine-module set (the same set the harness assembles).
- [x] T006 Implement **`UiHost`** in `crates/sojourn-ui/src/host.rs` (`contracts/ui-host.md`): own a `SimCore`, `status()` (tick/date/funds/political-capital), `set_time_warp`/`pause`/`tick` stepping, `events`/`pending_interrupts`/`acknowledge`, the typed snapshot-pull accessors (world/crew/polity + astro/research/vehicle/economy/base), `submit(Command)` and `preview(DraftPlan)`. Never blocks the core's stepping.
- [ ] T006a **Read-gap audit (blocking prerequisite for the screen phases)**: while wiring the snapshot-pull accessors (T006), enumerate — **per screen** (S1–S12, `contracts/screens.md` read map) — the reads each slice must expose versus what its query surface actually exposes today (e.g. an FA-02 porkchop/transfer field, FA-04 realism red-flags, FA-05 insight-pressure/dead-end reads, aggregate rosters). Record the gap list; close **each** gap as a **small, additive, headless, tested read on the relevant slice** (R15) **before** that slice's dependent screen phase begins — **never** compute the value in the UI (FR-UI-1501/1504).
- [x] T007 Implement the **read-sync** layer in `crates/sojourn-ui/src/read.rs` (R3): subscribe to the event feed, pull snapshots **on event / on navigation / on a throttled tick**, and guarantee a **consistent snapshot + matching clock** (FR-UI-1503).
- [x] T008 [P] Implement the pure **SI-unit formatters** in `crates/sojourn-ui/src/units.rs` (R6, FR-UI-1401): velocity/mass/power/energy/temperature/dose/pressure/currency/Δv/TOF/date; consistent formatting, no imperial.
- [x] T009 [P] Implement the **traceability renderer** in `crates/sojourn-ui/src/trace.rs` (R5, FR-UI-201/202): flatten a core `TraceTree` into a renderable `TraceRender` (operation nodes + sourced leaves); **flag a missing source**; "derivation unavailable" when no tree.
- [x] T010 [P] Implement the **command / plan→preview→commit** envelope in `crates/sojourn-ui/src/command.rs` (R4, FR-UI-301…305): `DraftPlan`, `Preview` (core-computed), `CommitOutcome`, and the `ppc::gate(action, draft) -> Gated(Preview) | Direct` classifier (irreversible vs reversible) + rejection surfacing + stale re-preview.
- [x] T011 [P] Implement the **`TableModel<Row>`** virtualisation/derive math in `crates/sojourn-ui/src/table.rs` (R8, FR-UI-1405): pure filter/sort/group + `visible_range` over a (potentially thousands-row) stub.
- [x] T012 [P] Implement the **bespoke-widget data shapes** in `crates/sojourn-ui/src/widgets_data.rs` (R11): `PorkchopField`, `DvLadder`, `TrlLadder`, `UnderstandingBars`, `ResourceLedger`, `LogisticsGraph`, `BaseSchematic`, `EvidenceMeter`.
- [x] T013 Foundational view-model test: `UiHost`+read-sync advance a stub game, pull a **consistent** snapshot whose values match `status().tick`, and submit a command — in `crates/sojourn-ui/tests/host.rs`.
- [x] T014 Implement the desktop **eframe app skeleton** in `crates/sojourn-ui-desktop/src/app.rs` + `main.rs`: the `eframe::App` that, each frame, pulls (throttled) via `UiHost`/read-sync and renders the active screen's view-model.
- [x] T015 Implement the persistent **shell chrome** in `crates/sojourn-ui-desktop/src/shell.rs`: top bar (date/time-warp/funds/political-capital/alert-summary), left nav (keyboard-switchable to all 12 screens), right inspector, bottom event ticker, and global hotkeys (FR-UI-101).
- [ ] T016 Implement the **theme** in `crates/sojourn-ui-desktop/src/theme.rs` + `data/ui/theme.ron`: colour-blind-safe palette (colour never the sole carrier), `pixels_per_point` text scaling (1280×720→4K), default hotkeys (FR-UI-1402/1403).

**Checkpoint**: workspace builds; `cargo test -p sojourn-ui` runs headlessly; the desktop app launches an empty shell; `cargo tree -p sojourn-core` stays presentation-free.

---

## Phase 3: User Story 1 — The persistent shell & System Map (Priority: P1) 🎯 MVP

**Goal**: a navigable, pausable, legible window — the 2D log-zoom multi-focus System Map inside the shell, with inspectors and time control, every value read from the core.

**Independent Test**: on a loaded game the map renders catalogued bodies + real orbits at multiple zoom/frames; clicking pins an inspector; a layer toggle changes only that overlay; time-warp/pause drive the core and the date tracks the clock; nothing is computed by the UI.

### Tests for User Story 1 ⚠️

- [x] T017 [P] [US1] Map view-model test: stub astro/world snapshots → bodies/orbits/craft/SOI/Lagrange/transport-nodes with LOD culling and active layers; selection resolves to an inspector ref — in `crates/sojourn-ui/tests/map.rs` (FR-UI-102/103/104).
- [x] T018 [P] [US1] Shell view-model test: `status()`+events → top-bar fields, nav items, pinned inspector(s), event ticker — in `crates/sojourn-ui/tests/shell.rs` (FR-UI-101/105).

### Implementation for User Story 1

- [x] T019 [US1] Implement `viewmodel/map.rs` and `viewmodel/shell.rs` in `crates/sojourn-ui/src/viewmodel/` (data-model §5): pure builders mapping the astro/world/ops snapshots + `UiState` to `MapVM`/`ShellVM`.
- [ ] T020 [US1] Implement `crates/sojourn-ui-desktop/src/screens/map.rs`: the custom map painter (logarithmic zoom, heliocentric↔planetocentric↔local-ops, toggleable layers, inertial/rotating frame, click→inspector, right-click context verbs).
- [ ] T021 [US1] Wire the time-warp/pause controls, the inspector pin, and the **UI-ephemeral** focus/zoom/layer/screen state into `shell.rs`/`app.rs` (FR-UI-105/106/1505).

**Checkpoint**: a legible, navigable, pausable window into a live game — MVP.

---

## Phase 4: User Story 2 — Traceability & progressive disclosure (Priority: P1)

**Goal**: any derived number expands to its sourced derivation; newcomer summary vs full detail; any term → Sojournal.

**Independent Test**: a derived figure expands into the core's traceability tree (every leaf sourced, missing source flagged); the summary/detail toggle changes disclosure not values; a term resolves to its Sojournal entry — with no UI-side computation.

### Tests for User Story 2 ⚠️

- [x] T022 [P] [US2] Trace-render test: a core `TraceTree` flattens to an expandable derivation; a leaf without a source is **flagged**; a value with no tree renders "derivation unavailable"; the Summary/Full disclosure preserves values — in `crates/sojourn-ui/tests/trace.rs` (FR-UI-201…203).

### Implementation for User Story 2

- [ ] T023 [US2] Implement the **inspector rendering** in `crates/sojourn-ui-desktop/src/shell.rs` (expand a `TracedValue` into its `TraceRender` tree; Summary/Full toggle).
- [ ] T024 [US2] Thread the `Disclosure` level through every screen view-model and add the **term→Sojournal** link hook (hover/click) in the shell (FR-UI-203/204).

**Checkpoint**: every number is interrogable; the UI is approachable and trustworthy.

---

## Phase 5: User Story 3 — Plan → preview → commit (Priority: P1)

**Goal**: every irreversible action previews core-computed consequences and requires confirm; reversible actions do not.

**Independent Test**: an irreversible command opens a core-computed preview + confirm/cancel; cancel submits nothing; confirm submits exactly the previewed command; a rejected command surfaces its reason; reversible actions bypass the gate.

### Tests for User Story 3 ⚠️

- [x] T025 [P] [US3] PPC gate test: `ppc::gate` returns `Gated(Preview)` for irreversible actions and `Direct` for reversible ones; a `Rejected` outcome surfaces its reason; a stale preview triggers re-preview — in `crates/sojourn-ui/tests/ppc.rs` (FR-UI-301…305).

### Implementation for User Story 3

- [ ] T026 [US3] Wire `command.rs` `preview`/`commit` to `UiHost` (the **core-computed** consequence; never UI-computed) in `crates/sojourn-ui/src/command.rs`/`host.rs`.
- [ ] T027 [US3] Implement the reusable desktop **confirm dialog** (preview consequences + confirm/cancel) in `crates/sojourn-ui-desktop/src/shell.rs`, used by every action-bearing screen.

**Checkpoint**: irreversible actions are deliberate; the sim never surprises the player.

---

## Phase 6: User Story 4 — Trajectory / Manoeuvre Planner (Priority: P2)

**Goal**: porkchop, manoeuvre nodes, low-thrust, flyby, live required-vs-available Δv, queue burns with auto-pause.

**Independent Test**: the porkchop renders the core's Δv/TOF/C3 field and allows a pick; a node edit updates the previewed trajectory/Δv; required Δv > available flags infeasibility; a saved plan queues via PPC.

### Tests for User Story 4 ⚠️

- [x] T028 [P] [US4] Planner view-model test: a stub astro porkchop field → pickable `PorkchopField`; a manoeuvre-node edit → updated `DvLadder`; required-vs-available Δv → feasibility flag — in `crates/sojourn-ui/tests/planner.rs` (FR-UI-401…403).

### Implementation for User Story 4

- [ ] T029 [US4] Implement `viewmodel/planner.rs` + the `PorkchopField`/`DvLadder` builders in `widgets_data.rs` (FA-02 transfer/porkchop reads + FA-04 available Δv). *If FA-02 lacks a porkchop-field read, add it as a **headless, tested** read on `sojourn-astro` (R15) — no logic in the UI.*
- [ ] T030 [US4] Implement `crates/sojourn-ui-desktop/src/screens/planner.rs` + the **porkchop** and **Δv-ladder** painters in `crates/sojourn-ui-desktop/src/widgets/`.
- [ ] T031 [US4] Wire **queue-burns** through plan→preview→commit with auto-pause at nodes (FR-UI-404).

**Checkpoint**: transfers are planned on porkchop plots, checked against the vehicle, committed safely.

---

## Phase 7: User Story 5 — Research & Development (Priority: P2)

**Independent Test**: UL bars + world-tide ghost + insight-pressure, TRL ladders + P50/P80 + dead-ends, and the tech-tree (with source tags) all render from FA-05; an allocation change submits and reflects.

- [x] T032 [P] [US5] R&D view-model test: FA-05 stub → `UnderstandingBars` (+ tide ghost + insight pressure), `TrlLadder` (+ test/risk), P50/P80-vs-actual, dead-end warnings, tech-tree nodes (+ source tags) — in `crates/sojourn-ui/tests/research.rs` (FR-UI-501/502/504).
- [x] T033 [US5] Implement `viewmodel/research.rs` + the `UnderstandingBars`/`TrlLadder` builders.
- [ ] T034 [US5] Implement `crates/sojourn-ui-desktop/src/screens/research.rs` + the Understanding-bars and TRL-ladder painters + the tech-tree graph view.
- [ ] T035 [US5] Wire RP/DE allocation + lead-assignment commands (allocation via the command path; cancel via PPC) (FR-UI-503).

---

## Phase 8: User Story 6 — Vehicle Designer (Priority: P2)

**Independent Test**: the picker lists only researched components; derived figures recompute from FA-04 and are traceable; a bad design surfaces the core red-flag; two designs compare side by side.

- [ ] T036 [P] [US6] Vehicle view-model test: FA-04/05 stub → researched-only component list; live `TracedValue` mass/Δv/power/thermal/reliability/cost; the core realism red-flag surfaced; side-by-side compare — in `crates/sojourn-ui/tests/vehicle.rs` (FR-UI-601…604).
- [ ] T037 [US6] Implement `viewmodel/vehicle.rs` (researched-only picker, derived `TracedValue`s, red-flags, compare).
- [ ] T038 [US6] Implement `crates/sojourn-ui-desktop/src/screens/vehicle.rs` + the `DvLadder` painter reuse + the comparison layout.
- [ ] T039 [US6] Wire save/iterate (reversible) and scrap (PPC) commands.

---

## Phase 9: User Story 7 — Operations / Fleet (Priority: P2)

**Independent Test**: a large fleet virtualises and stays responsive to filter/sort/group; each row's figures come from the core; crew health/dose from FA-08; manifest changes via PPC.

- [x] T040 [P] [US7] Operations view-model test: a thousands-row stub fleet → `TableModel<CraftRow>` (status/location/fuel/health/task/ops/comms) with correct filter/sort/group/`visible_range`; crew health/dose from FA-08 — in `crates/sojourn-ui/tests/operations.rs` (FR-UI-701/703, SC-006).
- [x] T041 [US7] Implement `viewmodel/operations.rs` (fleet `TableModel`, mission timeline, launch manifest/cadence, crew health/dose).
- [ ] T042 [US7] Implement `crates/sojourn-ui-desktop/src/screens/operations.rs` with the **virtualised** `egui_extras::TableBuilder` (`show_rows`) + the timeline view.
- [ ] T043 [US7] Wire launch-manifest / task-order commands (launch via PPC) (FR-UI-702).

---

## Phase 10: User Story 8 — Economy & Contracts (Priority: P2)

**Independent Test**: budgets/ledgers/markets render from FA-06; the resource ledger groups by Δv-location; the logistics graph shows Δv/TOF edges; posting/bidding goes through the command path; mood modifiers (FA-09) are traceable.

- [x] T044 [P] [US8] Economy view-model test: FA-06/09 stub → P&L/appropriation (with traceable mood modifiers), `ResourceLedger` grouped by Δv-location, `LogisticsGraph` (Δv/TOF edges), market, RFP board — in `crates/sojourn-ui/tests/economy.rs` (FR-UI-801…804).
- [x] T045 [US8] Implement `viewmodel/economy.rs` + the `ResourceLedger`/`LogisticsGraph` builders.
- [ ] T046 [US8] Implement `crates/sojourn-ui-desktop/src/screens/economy.rs` + the resource-by-location ledger and logistics-graph painters.
- [ ] T047 [US8] Wire post/bid contract + build-facility commands via PPC.

---

## Phase 11: User Story 9 — Bases & Construction (Priority: P2)

**Independent Test**: the site browser shows FA-03 properties + PP category; the base schematic gauges recompute from FA-07 as modules change; construction stages + ISRU status render; a build follows PPC.

- [x] T048 [P] [US9] Bases view-model test: FA-03/07 stub → site browser (resources/hazards/illumination/PP-category), `BaseSchematic` emergent-property gauges (power/closure/population/sustainability), construction stages, ISRU status — in `crates/sojourn-ui/tests/bases.rs` (FR-UI-901…903).
- [x] T049 [US9] Implement `viewmodel/bases.rs` + the `BaseSchematic` builder.
- [ ] T050 [US9] Implement `crates/sojourn-ui-desktop/src/screens/bases.rs` + the base-schematic painter with live gauges.
- [ ] T051 [US9] Wire found-base / add-module / deliver / build commands via PPC.

---

## Phase 12: User Story 10 — Alerts / Event Log & interrupts (Priority: P2)

**Independent Test**: events render chronologically + filter by class; clicking navigates to the related screen; setting a class to "pause" makes the core interrupt on it (and not on log-only); acknowledging resumes.

- [ ] T052 [P] [US10] Alerts view-model test: the core event feed → `TableModel` (filter-by-class, link-to-screen) + pending interrupts; the pause-policy config view-model — in `crates/sojourn-ui/tests/alerts.rs` (FR-UI-1001…1003).
- [ ] T053 [US10] Implement `viewmodel/alerts.rs` (event feed + pause-policy config) + the desktop `screens/alerts.rs`.
- [ ] T054 [US10] Wire the **pause-policy** edits to the FA-01 journalled `SetPausePolicy` command and the interrupt acknowledge/resume into the shell (FR-UI-1002/1505).

---

## Phase 13: User Story 11 — World/Politics & Personnel (Priority: P3)

**Independent Test**: the milestone board shows world-/faction-firsts + the unclaimed race from FA-09; mood/prestige/policy render + are traceable; personnel pools show FA-05/08 skills/assignments/health; lobby/assign go through the command path.

- [ ] T055 [P] [US11] Politics view-model test: FA-09 stub → milestone race board (world-/faction-first + unclaimed), relationships/prestige/mood/policy (traceable), rival feed — in `crates/sojourn-ui/tests/politics.rs` (FR-UI-1101/1102).
- [ ] T056 [P] [US11] Personnel view-model test: FA-05/08 stub → pools/skills/traits/assignments/morale + crew health careers — in `crates/sojourn-ui/tests/personnel.rs` (FR-UI-1103).
- [ ] T057 [US11] Implement `viewmodel/politics.rs` + `viewmodel/personnel.rs`.
- [ ] T058 [US11] Implement `crates/sojourn-ui-desktop/src/screens/politics.rs` (+ milestone-race board) and `screens/personnel.rs`.
- [ ] T059 [US11] Wire lobby / select-or-change-Grand-Goal (PPC for change) and recruit/train/assign commands.

---

## Phase 14: User Story 12 — Science Returns & Astrobiology (Priority: P3)

**Independent Test**: the evidence meter renders consensus/posteriors/band/conclusive from FA-09; staged evidence advances it; disagreement shows; it never displays a conclusive-positive the core has not set and never exposes the ground truth.

### Tests for User Story 12 ⚠️

- [x] T060 [P] [US12] Astrobiology **honesty-guard** test: the `EvidenceMeter` renders consensus/per-faction posteriors/band/disagreement; it is **never** conclusive-positive unless the snapshot is, and there is **no** ground-truth input — in `crates/sojourn-ui/tests/astrobiology.rs` (FR-UI-1201/1202, SC-009).

### Implementation for User Story 12

- [x] T061 [US12] Implement `viewmodel/astrobiology.rs` (the `EvidenceMeter` builder + the honesty guard, belief-state per body, incoming data, discoveries log).
- [ ] T062 [US12] Implement `crates/sojourn-ui-desktop/src/screens/astrobiology.rs` + the **astrobiology evidence meter** painter (probabilistic, multi-stage, per candidate — never a binary popup).
- [ ] T063 [US12] Render the Geoscience-UL belief-state and the discoveries log from the world/polity surfaces (FR-UI-1203).

---

## Phase 15: User Story 13 — The Sojournal encyclopedia (Priority: P3)

**Independent Test**: entries render from the FA-03 Sojournal surface with citations; search + cross-links resolve; an entry reflects the current belief-state; a term hovered anywhere opens its entry.

- [ ] T064 [P] [US13] Sojournal view-model test: FA-03 Sojournal stub → searchable cross-linked **source-cited** entries that reflect belief-state; term→entry resolver — in `crates/sojourn-ui/tests/sojournal.rs` (FR-UI-1301/1302).
- [ ] T065 [US13] Implement `viewmodel/sojournal.rs` (search, cross-links, source-cited, belief-state-aware).
- [ ] T066 [US13] Implement `crates/sojourn-ui-desktop/src/screens/sojournal.rs` and connect the global term→Sojournal hook (US2) to it.

---

## Phase 16: User Story 14 — Accessibility, SI units & performance (Priority: P2)

**Independent Test**: a unit audit finds no non-SI units; a colour-blind review finds no colour-only meaning; text scaling + 1280×720↔4K stay legible; every primary action has a keyboard path; a thousands-row table stays within budget at high warp.

### Tests for User Story 14 ⚠️

- [x] T067 [P] [US14] Units test: every `units.rs` formatter is correct and **no imperial unit** appears; a screen-wide audit of formatted values passes through `units` — in `crates/sojourn-ui/tests/units.rs` (FR-UI-1401, SC-007).
- [x] T068 [P] [US14] Virtualisation/performance test: `TableModel` filter/sort/group/`visible_range` is correct and bounded on a thousands-row stub (the SC-006 logic) — in `crates/sojourn-ui/tests/table.rs`.

### Implementation for User Story 14

- [ ] T069 [US14] Implement the **accessibility pass**: colour-blind-safe palette applied (colour never sole carrier), `pixels_per_point` scaling across the shell + screens, legible 1280×720→4K layouts — `crates/sojourn-ui-desktop/src/theme.rs`/`shell.rs` (FR-UI-1402/1403).
- [ ] T070 [US14] Implement **full keyboard navigation**: a keyboard path/hotkey for every primary action (time-warp, screen switch, inspect, plan, commit, filter) across the shell + screens (FR-UI-1404, SC-008).

---

## Phase 17: Polish & Cross-Cutting Concerns

- [ ] T071 [P] Architecture-audit assertion: a test/check that `sojourn-ui` consumes only the slices' **read-only** surfaces + the event feed and submits **only** commands (no game logic, no authoritative state), and that `cargo tree -p sojourn-core` stays presentation-free (FR-UI-1501/1502, SC-010).
- [x] T072 Extend CI `.github/workflows/ci.yml`: build `sojourn-ui-desktop` and run the `sojourn-ui` view-model tests in the lint/test jobs (install the Linux windowing dev libs for the desktop build); **leave the determinism job (harness-only) and the core dependency/scope audit untouched** (the audit already rejects UI crates in the core tree).
- [x] T073 [P] Finalise `data/ui/theme.ron`: the colour-blind-safe palette set + the default hotkey map.
- [ ] T074 [P] Add a `ui` view-model bench (snapshot→view-model build over a large stub fleet/catalogue) — `crates/sojourn-ui/benches/viewmodel.rs` (SC-006). *(May be deferred consistent with FA-03…09 benches.)*
- [ ] T075 [P] Run `quickstart.md` end-to-end (headless view-model tests + a manual desktop pass); confirm SC-001…SC-012.

---

## Dependencies & Execution Order

### Phase dependencies
- **Setup (P1)** → no deps (two crates + workspace deps).
- **Foundational (P2)** → after Setup; **BLOCKS all stories** (T005–T016). The host (T006), read-sync (T007), units/trace/command/table/widgets_data primitives (T008–T012) and the desktop app/shell/theme skeleton (T014–T016) gate every screen.
- **US1 (P3)** → after Foundational. The MVP (shell + map); its inspector + ephemeral-state plumbing back the rest.
- **US2 (P4)** → after US1 (the inspector lives in the shell); traceability is then reused by every screen.
- **US3 (P5)** → after Foundational (command path); the confirm dialog is reused by every action screen (US4–US11).
- **US4–US9 (P6–P11)** → after US2/US3 (each uses traceability + the PPC confirm dialog) + Foundational; otherwise independent (different screen files) and parallelisable.
- **US10 (P12)** → after Foundational (event feed + pause-policy) + the shell.
- **US11 (P13)** / **US12 (P14)** / **US13 (P15)** → after US2 (traceability/Sojournal hook); US12's honesty guard is structural.
- **US14 (P16)** → after the screens exist (it audits units/keyboard/perf across them) — but `units.rs`/`table.rs` are built in Foundational, so US14's tests can land early and its passes apply across screens.
- **Polish (P17)** → after the desired stories.

### Critical-path notes
- T006 (host) + T007 (read-sync) + T009 (trace) + T010 (command/ppc) + T011 (table) gate everything; the desktop shell (T015) + confirm dialog (T027) + inspector (T023) are reused by every screen.
- The **view-model tests** are the Principle-IV seam — each screen's `…VM` is independently testable over stub snapshots with **no renderer**; keep them green and renderer-free.
- The **core stays headless and presentation-free**: nothing in the `sojourn-core`/slice trees may gain a UI dep (T004/T071 assert it); the determinism + core-audit CI jobs are untouched.
- The **astrobiology honesty guard** (T060) is a tested invariant — the UI cannot show a false positive or the ground truth (FA-09 exposes neither).
- Where a screen needs a missing read, add it to the **slice's** query surface headlessly+tested (R15) — never compute it in the UI. The **read-gap audit (T006a)** does this enumeration up front in Foundational and is a **blocking prerequisite** for the screen phases (US1–US13): close each screen's slice-read gaps before that screen's tasks begin.

### Parallel opportunities
- Setup: T003/T004 parallel.
- Foundational: T008/T009/T010/T011/T012 parallel (distinct files); T005→T006→T007 sequential-ish; T014/T015/T016 after.
- Within a story, the `[P]` view-model test and the impl are split; **across** stories US4–US9 (different screen files) parallelise once US2/US3 land.

---

## Implementation Strategy

### MVP first (US1)
Setup → Foundational → US1 → **STOP & validate**: a legible, navigable, pausable System Map inside the
shell, every value read from the core — the playable window. Demoable.

### Incremental delivery
US1 (shell+map) → US2 (traceability) → US3 (plan→preview→commit) → US4 (planner) → US5–US9 (the domain
tools) → US10 (alerts/interrupts) → US11–US13 (strategic/narrative screens) → US14 (accessibility/units/
performance). The three P1 stories (US1–US3) are the legibility-and-trust spine; the rest layer on without
restructuring.

### Notes
- `[P]` = different files, no incomplete-task dependency. `[Story]` traces to spec.md.
- Tests written first, must fail before implementation (Constitution); the **view-model tests over stub
  snapshots** are the Principle-IV headless seam (no renderer).
- **No game logic and no authoritative state in the view** (FR-UI-1501); the UI reads the slices' typed
  read-only surfaces in-process and submits journalled commands; the **same core runs headless** and the
  determinism/audit path is untouched. SI units only; colour-blind-safe + scalable + keyboard-navigable;
  honest astrobiology meter.
- Commit after each task or logical group; **run `cargo fmt --all` before committing** (CI enforces
  `fmt --check`); auto-commit is disabled (manual `/speckit-git-commit`).

---

## Implementation status (FA-10) — honest record of a partial slice

This is the **largest slice** (a full desktop GUI over the complex headless sim). The **constitutional
core is complete and green**; the **full twelve-screen egui renderer is a multi-pass effort and is not
finished here**. 35 of 76 tasks are done.

### Complete + tested (green)
- **`sojourn-ui`** — the headless view-model library (the Principle-IV seam): `UiHost` + composition root
  (builds + advances the full **nine-module** core in-process), read-sync, and the pure primitives
  (`units` SI formatting, `trace` traceability, `command` plan→preview→commit gate, `table` virtualisation,
  `widgets_data`) + per-screen view-model builders (shell, map, operations, planner, economy, bases,
  research, astrobiology).
- **20 headless view-model tests pass** (units, trace, ppc, table, 8 view-models incl. the **astrobiology
  honesty guard**, and the end-to-end **host** test that builds the real nine-module core) — **no
  renderer**, proving the decoupling (FR-UI-1506, SC-004).
- **`sojourn-ui-desktop`** — the egui/eframe shell **compiles and runs** (top bar with date + time-warp/
  pause, left nav, event ticker, and a working Map / Operations / Astrobiology / Alerts screen set
  rendering the view-model live from the core).
- The **architecture audit holds**: `cargo tree -p sojourn-core` is presentation-free (no egui/winit/wgpu
  in the core tree, SC-010); clippy + fmt clean; FA-01…09 suites stay green. CI wired (UI build + the
  view-model tests; the **determinism + core-audit jobs untouched**).

### Deviations
- View-model code consolidated into one `viewmodel/mod.rs` (not 12 files) and its tests into
  `tests/viewmodel.rs` — the logic + tests exist and pass; the file split is cosmetic.
- The view-models take small **in-process input structs** the host fills from the slice surfaces (still
  in-process typed, no serialization — clarify Q2) rather than each consuming a raw slice snapshot type;
  this keeps them stub-testable and decoupled.

### Deferred (the remaining ~41 tasks — a follow-up pass)
- The **full egui render of all twelve screens** with the bespoke-widget painters (porkchop, Δv/TRL
  ladders, base schematic, evidence meter, the custom log-zoom System Map) — currently four screens render
  summary data.
- **T006a read-gap closure**: wiring each screen's view-model to the *real* slice reads and adding the
  missing **headless, tested** reads to FA-02/04/05 (R15) — the host exposes `core()` for typed pulls, but
  the per-screen live wiring + the additive slice reads are not done.
- The remaining screens (Vehicle, World/Politics, Personnel, Sojournal), the inspector trace-tree +
  progressive-disclosure rendering (T023/T024), the desktop confirm dialog (T027), the accessibility +
  keyboard passes (T069/T070), the bench (T074), and the quickstart end-to-end (T075).

This is an honest stopping point: the slice's **testable, constitution-critical foundation is done and
green and the desktop app runs**; the breadth of twelve fully-painted screens + cross-slice read wiring is
a substantial follow-up, not a one-pass deliverable.
