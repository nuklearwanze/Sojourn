# Quickstart: Bases & Construction (FA-07)

A headless walk through the slice that proves the spec's success criteria. Everything runs under the
deterministic harness; no UI. Commands are `Command::ModulePayload { module: "base", … }`; queries are
pure functions over a `BaseSnapshot`. Cross-slice physics is fed as composed values
(`contracts/integration-seams.md`).

## 0. Build & validate data

```text
cargo test -p sojourn-base
cargo run -p sojourn-harness -- validate-data data/base       # schema + sources + analytic gates
cargo run -p sojourn-harness -- conformance --module base     # manifest, double-run, serde round-trip, cadence
```

## 1. Compose a base; emergent properties (US1 → SC-001)

1. `FoundBase` at a sourced Site; `AddModule` power + habitat + ECLSS + shielding modules.
2. Query `power(base)` → margin = Σ generation (solar-scaled) − Σ demand; add another power module →
   margin rises by its sourced generation; remove it → falls.
3. Query `trace(base, "power-margin")` → resolves to sourced module/site leaves.

## 2. Construction routed through logistics (US2 → SC-003)

1. `OpenConstruction(base)` → derives per-module delivered-mass + crew-time demands.
2. `DeliverToBase` one module's mass + crew-time → it commissions; query `emergent` → only that module
   contributes (partial base ⇒ partial properties).
3. Complete all deliveries → `base-operational`; the base reaches its full composed properties.

## 3. Siting respects planetary protection & suitability (US3 → SC-002)

1. Found a base on a Special-Region Site without containment → `siting_flags` reports a **Hard** PP
   violation citing the rule; `pp-violation` event fires.
2. Found a solar-only base on a permanently-shadowed Site → negative-power / illumination flag.
3. A base lacking shielding for its site's dose → shielding-shortfall flag.

## 4. On-site production reduces imports (US4 → SC-004)

1. Host an FA-06 ISRU output + a regolith-construction capability; `BuildLocal` a shielding module from
   local regolith → its mass is satisfied **without** imported (launched) mass.
2. Query `local_production(base)` → import mass avoided > 0; the construction project's remaining
   imported-mass demand falls.

## 5. Self-sufficiency & embargo (US5 → SC-005/006)

1. Query `self_sufficiency(base)` → limiting-factor index = the minimum loop ratio; improving the
   binding loop raises it (monotonic).
2. `EvaluateEmbargo(base, 5)` on a base above threshold → **survives**; on one below → **fails** (a
   loop short of both production and buffer), deterministically.

## 6. Base state exposed (US6 → SC-009)

1. Query `production_consumption(base)` → inputs consumed + outputs produced at the base's location
   (for the economy's stocks).
2. Query `life_support(base)` → habitat capacity, closure, shielding, population (for Slice 8).
3. `compare(a, b)` → diffs two bases' emergent properties.

## 7. Determinism, save/load, perf (cross-cutting → SC-007/008/010)

```text
cargo run -p sojourn-harness -- verify    scenarios/base_construction.ron
cargo run -p sojourn-harness -- roundtrip scenarios/base_construction.ron --save-at-ticks <t1>,<t2>
cargo run -p sojourn-harness -- mutate --all
```

- SC-008: every `data/base/*` entry has a `source`; CI `validate-data base` enforces it.
- SC-010: dozens of bases × hundreds of modules sustain ≥1 sim-year/min at high warp.

## Success-criteria coverage map

| SC | Proven by |
|---|---|
| SC-001 | §1 (emergent power margin additive + traceable) |
| SC-002 | §3 (PP / suitability / shielding red-flags) |
| SC-003 | §2 (delivery-driven commissioning; partial base) |
| SC-004 | §4 (on-site regolith construction reduces imported mass) |
| SC-005 | §5 (embargo survive/fail) |
| SC-006 | §5 (self-sufficiency index monotonic) |
| SC-007 | §7 `verify` + `roundtrip` |
| SC-008 | §0 `validate-data base` source presence |
| SC-009 | §6 (production/consumption, habitat state, compare) |
| SC-010 | §7 perf envelope |
