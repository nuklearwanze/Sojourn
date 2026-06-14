# Contract — The view-model (`sojourn-ui::viewmodel`)

The pure, **headlessly-testable** mapping from core snapshots to display structs. Every screen has one
view-model module: `fn build(snapshot(s), ui_state) -> ScreenVM`. The view-model performs **no physics and
no authoritative mutation** — only presentation derivation (format, sort/filter/group, visible-range,
trace-tree flattening, gating). The renderer paints the result.

## Shared primitives

- `TracedValue { label, formatted: String, raw: f64, tree: Option<TraceRender> }` — a display value
  expandable into its **sourced** derivation (`TraceRender` mirrors the core `TraceTree`). A leaf missing a
  `source` is **flagged**; a value with no tree renders "derivation unavailable". The UI never fabricates a
  derivation.
- `units::*` — the single SI formatter set (velocity/mass/power/energy/temp/dose/pressure/currency/Δv/TOF/
  date). **No imperial** anywhere; tested.
- `TableModel<Row> { rows, columns, sort, filter, group, visible_range }` — virtualisation/derive math; the
  renderer paints only `visible_range`. Sort/filter/group are deterministic pure functions.
- `Disclosure::{Summary, Full}` — both levels render from the same values (progressive disclosure).

## Per-screen `build` functions (one pure module each)

Each returns its `…VM` (see data-model §5). Contract obligations common to all:

1. **Source-of-truth**: every value originates from a passed-in snapshot; the function is **pure** in its
   inputs (same snapshot ⇒ same VM) — directly unit-testable over **stub snapshots**.
2. **Traceability**: any derived figure the core exposes with a tree is surfaced as a `TracedValue`.
3. **SI units**: all quantities formatted through `units`.
4. **No invention**: unknown/unsurveyed/unavailable reads render as such, never a fabricated value.

## Plan→preview→commit gating (`viewmodel::ppc`)

- `is_irreversible(action) -> bool` — classifies an action; irreversible actions MUST route through
  `host.preview(draft)` and an explicit confirm before `host.submit`.
- `gate(action, draft, ui_state) -> Gated(Preview) | Direct` — the tested gate; reversible actions return
  `Direct` (no confirm), irreversible return `Gated` with the core preview.

## Astrobiology honesty guard (`viewmodel::astrobiology`)

- `meter(candidate, polity_snapshot) -> EvidenceMeter` — renders consensus/posteriors/band/disagreement;
  **asserts** it never emits `conclusive = Positive` unless the snapshot's `conclusive(candidate)` is
  `Positive`, and it has **no** ground-truth input (FA-09 exposes none). Tested as an invariant.

## Testing

- `sojourn-ui/tests/*` build **stub snapshots** (hand-constructed slice snapshot values) and assert the VM
  output: table derivation, trace flattening, SI formatting, ppc gating, the honesty guard, widget data
  shapes. **No renderer is instantiated** (Principle IV; FR-UI-1506).
