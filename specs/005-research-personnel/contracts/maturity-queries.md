# Contract: Maturity / Heritage / Understanding Query Surface (FR-RESP-801)

The read-only seam FA-04 (vehicle/propulsion), FA-06 (economy), FA-09 (politics) and FA-10 (UI)
consume. Same shape as FA-02's planning queries and FA-03's world queries: pure functions over an
immutable snapshot taken via the kernel's `with_slice`, callable only **between ticks**. **Faction-
scoped**: no query returns another faction's private state (the tide's World UL is the only shared
channel).

## Snapshot

```text
ResearchSnapshot::from_core(&core, &module) -> ResearchSnapshot
    // projects the research slice (per-faction UL, programs, heritage, personnel, world tide)
    // + the immutable tree/domain/params data. Pure; no truth-style hidden fields beyond the
    // per-faction privacy already enforced by the faction-scoped query signatures.
```

## Query functions (pure; no mutation, no streams, unjournalled)

| Function | Returns | Notes |
|---|---|---|
| `maturity(faction, tech)` | `{ trl, reliability: f64∈[0,1], inputs:{trl,flight_units,ul_margin}, flyable: trl≥6 }` | the FA-04 capability/reliability contract (R10); flyability false below TRL 6 |
| `heritage(faction, tech)` | `{ flight_units, ceiling, derivative_discount }` | drives FA-04 reliability ceiling + derivative starts |
| `understanding(faction, domain)` | `{ private_ul, world_ul, effective_ul, gates:[{program/tech, met}] }` | private = world + lead/lag; effective = tacit-knowledge-adjusted |
| `program_status(faction, program)` | `{ trl, step_progress, p50, p80, actual, risk_index, dead_end_hint }` | the R&D-screen contract |
| `available_programs(faction)` | `[{ tech, gating_domains, leapfroggable }]` | UL-gated availability (R3/R8) |
| `personnel(faction)` | `{ counts_by_role, traits_summary, astronaut_readiness }` | roster summary |
| `tide(domain)` | `{ world_ul, baseline, catch_up_discount(faction) }` | the shared knowledge channel |

## Guarantees

- **Faction privacy**: every faction-scoped query reflects only the asking faction; private leads never
  leak (only `tide`/`world_ul` is shared).
- **Reliability contract** (R10): `maturity().reliability` is a single per-use success probability with
  its raw inputs exposed; consumers may layer duration/phase effects without re-deriving maturity.
- **Determinism / purity**: queries are pure functions of committed slice state; identical across a
  double run; calling any query between ticks leaves the state fingerprint unchanged (FA-02 pattern).
- **Latency** (SC-008): < 50 ms on the reference machine at full multi-faction scale.
- **IPC-ready**: results are serde DTOs (the Tauri-host / FA-04 seam).
