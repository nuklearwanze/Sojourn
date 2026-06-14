# Quickstart — User Interface & Presentation Layer (FA-10)

Verifies the FA-10 slice. The **view-model** (the testable UI logic) runs headlessly in `cargo test`; the
**renderer** is the desktop binary. The core stays on its own headless determinism path, untouched.

## Headless view-model tests (the constitutional seam)

```pwsh
cargo test -p sojourn-ui
```

Covers (one test file per screen + the shared primitives), all over **stub core snapshots, no renderer**:
- **units.rs** — SI formatting is correct and contains no imperial units (SC-007).
- **trace.rs** — a core `TraceTree` flattens to a renderable derivation; a missing source is flagged; a
  value with no tree renders "derivation unavailable" (SC-002).
- **ppc.rs** — irreversible actions are gated through a core preview + confirm; reversible actions are not;
  a rejected commit surfaces its reason (SC-003).
- **map / planner / research / vehicle / operations / economy / bases / personnel / politics / astrobiology
  / sojournal / alerts** — each screen's `build()` maps stub snapshots to the expected view-model
  (table derivation, widget data shapes, traced values).
- **astrobiology.rs** — the **honesty guard**: never a conclusive-positive the snapshot did not set; no
  ground-truth input exists (SC-009).
- **table.rs** — virtualisation/derive math (filter/sort/group/visible-range) is correct on a thousands-row
  stub (SC-006 logic).

## Run the desktop app

```pwsh
cargo run -p sojourn-ui-desktop
```

Launches the persistent shell + System Map over a fresh game (all nine modules hosted in-process). Verify
interactively: zoom/multi-focus, layer toggles, inspector pinning + trace expansion, time-warp/pause, screen
nav, plan→preview→commit on a burn/launch, virtualised fleet/ledger tables, the porkchop planner, the base
schematic, the astrobiology evidence meter, keyboard navigation, and text scaling (1280×720→4K).

## Whole-workspace gates

```pwsh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings   # builds sojourn-ui + sojourn-ui-desktop too
cargo test --workspace                                  # FA-01…09 stay green; sojourn-ui view-model tests run headlessly
```

## Architecture / determinism audit (unchanged, must stay green)

```pwsh
cargo tree -p sojourn-core --prefix none   # MUST contain no egui/eframe/winit/wgpu (UI is off the core tree)
cargo run -q -p sojourn-harness -- verify scenarios/smoke_decade.ron   # determinism path: UI not involved
```

## Success-criteria map

| SC | Verified by |
|---|---|
| SC-001 inspect a body fast, every value traced | map.rs + trace.rs view-model tests; manual map |
| SC-002 100% derived numbers traceable or flagged | trace.rs + per-screen view-model tests |
| SC-003 100% irreversible actions preview+confirm | ppc.rs view-model tests |
| SC-004 core headless + UI view-model tested no-renderer | `cargo test -p sojourn-ui`; harness determinism unchanged |
| SC-005 plan a transfer vs available Δv, queue burns | planner.rs tests; manual planner |
| SC-006 thousands-row/body interactive at high warp | table.rs tests; manual virtualised tables/map |
| SC-007 zero non-SI units; no colour-only meaning | units.rs tests; theme review |
| SC-008 keyboard path for every primary action | shell hotkey map; manual keyboard nav |
| SC-009 astrobiology honesty (no false positive, no ground truth) | astrobiology.rs honesty-guard test |
| SC-010 no game logic / no authoritative state in the view | architecture audit: `sojourn-ui` deps are read-only surfaces; core tree presentation-free |
| SC-011 usable while paused; never blocks stepping | host.rs read-sync tests; manual |
| SC-012 newcomer completes the loop via disclosure + Sojournal | progressive-disclosure + Sojournal view-model tests; manual |
