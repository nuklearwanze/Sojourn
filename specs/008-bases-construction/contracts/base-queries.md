# Contract: Base Query Surface (FA-07)

The read-only, between-ticks surface FA-08 (life support), FA-09 (politics) and FA-10 (UI) consume.
Pure functions over a `BaseSnapshot` that composes the base slice with the composed-value inputs
(`integration-seams.md`). No mutation, no hidden truth, IPC-serializable. Implements FR-BC-106,
FR-BC-601/602/603, and the R13 design-query decision.

## Snapshot construction

```text
BaseSnapshot::from_core(&core, &base_module, inputs: &BaseInputs) -> Result<BaseSnapshot, CoreError>
BaseSnapshot::from_parts(&base_module, slice, inputs)   // tests: build directly from parts
```

- `from_core` reads the `"base"` slice via kernel `with_slice` (read-only, between ticks).
- `BaseInputs` bundles the composed values the host assembles per `integration-seams.md` (site facts,
  tech maturity, delivery status, ISRU output). Tests pass stubs.

## Queries (pure, faction-scoped where site belief is involved)

| Query | Returns | Notes |
|---|---|---|
| `emergent(base)` | `EmergentProperties` | power margin, shielding/dose, closure, population, self-sufficiency, hazard (FR-BC-102) |
| `power(base)` | `PowerBalance` | Σgen(solar-scaled) − Σdemand over commissioned modules (FR-BC-103) |
| `shielding(base)` | `Shielding` | mass-attenuation exp; transmitted dose + shortfall (FR-BC-105) |
| `life_support(base)` | `LifeSupport` | closure fraction + population capacity (gated) (FR-BC-104) |
| `self_sufficiency(base)` | `SelfSufficiency` | limiting-factor index + binding loop (FR-BC-501) |
| `embargo(base, years)` | `EmbargoResult` | analytic rate+buffer survival (FR-BC-502) |
| `construction(base)` | `ConstructionProgress` | delivered vs remaining, % complete, commissioned count (FR-BC-205) |
| `siting_flags(base)` | `Vec<RedFlag>` | PP / suitability / shielding / power red-flags (FR-BC-301/302) |
| `production_consumption(base)` | `ProductionConsumption` | inputs consumed + outputs produced at the base's location (FR-BC-603) |
| `local_production(base)` | `LocalProduction` | regolith-construction / manufacturing rates + import mass avoided (FR-BC-404) |
| `milestones(base)` | `Vec<String>` | settlement milestones (e.g., embargo-survivor) for politics/scoring (FR-BC-601) |
| `trace(base, property)` | `TraceTree` | any emergent number → sourced module/site leaves (FR-BC-106) |
| `compare(a, b)` | `(EmergentProperties, EmergentProperties)` | diff two bases (FR-BC-602) |

## Guarantees

- **Read-only / deterministic**: no query mutates the slice; identical snapshot + inputs ⇒ identical
  results (no wall-clock, libm-only).
- **No hidden truth**: siting/shielding queries act on the **surveyed** `SiteFacts` supplied as inputs —
  never world ground truth.
- **Partial base**: only **commissioned** modules contribute to emergent properties (FR-BC-203).
- **Traceability**: every emergent number is drillable to sourced leaves (`all_leaves_sourced`
  CI-checkable).
- **Faction scoping**: a base and its site belief are per faction; the module catalogue is global.
