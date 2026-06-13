# Contract: Design-Query & Traceability Surface (FR-VD-801)

The read-only seam FA-06 (economy), FA-09 (politics) and FA-10 (UI) consume. Same shape as the
FA-02/03/05 query surfaces — pure functions over an immutable snapshot taken via `with_slice`,
callable **between ticks**. The snapshot **composes** the design (FA-04 slice), the current FA-05
**maturity** (so reliability/cost track research) and caller-supplied **gravity** (for T/W).
Faction-scoped where research state is involved.

## Snapshot

```text
DesignSnapshot::from_core(&core, &vehicle, &research, gravity) -> DesignSnapshot
    // vehicle slice (designs) + research maturity (per component tech) + gravity map (body → g).
```

## Query functions (pure; no mutation, no streams, unjournalled)

| Function | Returns | Notes |
|---|---|---|
| `derive(faction, design)` | `DerivedOutputs { mass, dv_by_stage_mode, thrust, tw_by_body, power_balance, thermal_balance, reliability, life_support, edl_suitability, cost_estimate }` | the full physics derivation (R3) |
| `trace(faction, design, output)` | `TraceTree` | recursive derivation tree; leaves are sourced data values (FR-VEH-003/Principle VIII) |
| `red_flags(faction, design)` | `[RedFlag { constraint, value, severity }]` | realism guards (R13); `Hard` = refused, `Soft` = buildable gamble |
| `engine_defs(faction, design)` | `[EngineDef]` | the FA-02 endpoint params per engine (R2) — what spawn carries inline |
| `reliability(faction, design)` | `{ composed, per_component, sub_trl6: [..] }` | reliability-block-diagram (R8) |
| `cost(faction, design)` | `{ unit_cost, build_days, learning_factor }` | physical estimate (R11); FA-06 prices it |
| `availability(faction, component)` | `{ available, gating_tech, maturity }` | query-time gating (R4) |
| `compare(faction, a, b)` | field-by-field diff | side-by-side comparison (FR-VEH-007) |

## Guarantees

- **No magic numbers**: every leaf of every `trace()` is a sourced data value; a CI check asserts no
  derived output has a non-sourced constant (Principle II).
- **Reliability honesty**: `reliability().per_component` equals FA-05 `maturity().reliability`;
  composition follows the documented block-diagram; sub-TRL-6 parts are surfaced.
- **Determinism / purity**: derivations are pure functions of design + maturity + gravity; identical
  between double-run executions; calling a query between ticks leaves the fingerprint unchanged.
- **Faction privacy**: research-dependent results reflect only the asking faction.
- **Latency** (SC-008): < 50 ms; individual derivations are sub-millisecond.
- **IPC-ready**: results are serde DTOs (the Tauri-host / FA-06 / FA-10 seam).
