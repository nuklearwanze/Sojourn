# Contract: World-Query Surface (FR-WORLD-701)

The read-only seam other slices and the future UI (FA-10) consume: *"what is here, what do we
believe is here, how certain are we, what changed."* Same shape as FA-02's planning queries — pure
functions over an immutable snapshot taken via the kernel's `with_slice`, callable only **between
ticks** (query-at-tick-boundary rule). **Structurally truth-free** (R4): the snapshot type carries
belief + public catalogue data and *no* ground-truth fields, so no query can leak truth.

## Snapshot

```text
WorldSnapshot::from_core(&core) -> WorldSnapshot
    // projects the world slice's belief + public catalogue/site/location/sojournal data.
    // Composes with astro AstroSnapshot for positions. Contains NO GroundTruthStore.
```

## Query functions (pure; no mutation, no streams, unjournaled)

| Function | Returns | Notes |
|---|---|---|
| `catalogue(filter)` | `Vec<BodyView>` | indexed by type / region / parent / flag; base ∪ generated; never full-scan |
| `body(id)` | `Option<BodyView>` | public catalogue fields only |
| `sites_on(body)` | `Vec<SiteView>` | existence + PP category always; other props via belief |
| `locations()` / `resolve_location(id, t)` | list / `Point\|Region` | L-points + frames via FA-02 |
| `believed(faction, target, property)` | `Estimate{value, uncertainty}` | belief only — **never** truth; wide honest default if no prior |
| `certainty(faction, target, property)` | scalar | derived from belief variance/entropy |
| `belief_delta_since(faction, tick)` | `Vec<Change>` | from the per-faction tick-stamped change log; for UI refresh |
| `sojournal(id)` / `sojournal_for(ref)` | `Option<Entry>` | cited encyclopedia data; truth-free |

## Guarantees

- **No truth path** (SC-002): a standing audit test enumerates this entire surface and asserts no
  unsurveyed seeded truth is reachable; adding a new query later re-runs the audit.
- **Per-faction isolation**: `believed`/`certainty`/`belief_delta_since` reflect only the named
  faction; one faction's survey never changes another's returns.
- **Determinism**: queries are pure functions of committed slice state; identical between
  double-run executions; safe to call repeatedly between ticks.
- **Latency** (SC-006): indexed queries < 50 ms on the reference machine at full catalogue scale.
- **IPC-ready**: views are serde DTOs (the Tauri-host / harness seam), like FA-02's query results.
