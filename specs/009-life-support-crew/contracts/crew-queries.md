# Contract: Crew Query Surface (FA-08)

The read-only, between-ticks surface FA-09 (politics) and FA-10 (UI) consume. Pure functions over a
`CrewSnapshot` composing the crew slice (stored dynamic state) with the composed-value inputs
(`integration-seams.md`). No mutation, no hidden truth, IPC-serializable. Implements FR-LSC-701/703 and
the R13 design-query decision.

## Snapshot construction

```text
CrewSnapshot::from_core(&core, &crew_module, inputs: &CrewInputs) -> Result<CrewSnapshot, CoreError>
CrewSnapshot::from_parts(&crew_module, slice, inputs)   // tests: build directly from parts
```

- `from_core` reads the `"crew"` slice via kernel `with_slice` (read-only, between ticks).
- `CrewInputs` bundles the composed values the host assembles per `integration-seams.md` (asset sizing,
  env facts, crew roster + age/sex, ECLSS maturity, ops load, EDL suitability). Tests pass stubs.

## Queries (pure, faction-scoped)

| Query | Returns | Notes |
|---|---|---|
| `member(astronaut)` | `CrewMemberView` | career dose, REID %, deconditioning indices, psych load, capability, status (FR-LSC-701) |
| `reid(astronaut)` | `Reid { pct, grounded }` | dose→REID via sourced curve + age/sex; grounded ≥ 3% (FR-LSC-203) |
| `capability(astronaut)` | `f64` | multiplicative product of decon × psych × health factors (FR-LSC-303) |
| `consumables(asset)` | `Consumables { stock_kg, gross_per_day, makeup_rate_kg_day }` | make-up = gross × (1 − closure) (FR-LSC-102) |
| `eclss_risk(asset)` | `EclssRisk { failure_prob, critical }` | multiplicative hazard; critical = failed && !abort_reach (FR-LSC-501/503) |
| `edl_risk(asset)` | `EdlRisk { crew_loss_prob }` | suitability × body × crew-state; Mars gap (FR-LSC-601/602) |
| `viability(asset, mission_days)` | `Viability { consumables_ok, dose_ok, eclss_ok, capability_ok, viable }` | composite (FR-LSC-703) |
| `loss_of_crew(astronaut)` | `bool` | physically lost? (FR-LSC-702) |
| `roster_state(faction)` | `Vec<CrewMemberView>` | all crew + status (for politics/UI) |
| `trace(astronaut\|asset, figure)` | `TraceTree` | any derived figure → sourced leaves (FR-LSC-801) |

## Guarantees

- **Read-only / deterministic**: no query mutates the slice; identical snapshot + inputs ⇒ identical
  results (no wall-clock, libm-only).
- **No hidden truth**: REID/viability/EDL queries act on the **composed** sizing, roster age/sex and
  maturity supplied as inputs — never the upstream authoritative state.
- **Stored-vs-derived**: the dynamic health *state* (career dose, deconditioning, psych load, ECLSS
  degradation) is **stored** and evolved on the step; REID, capability, hazards and viability are
  **derived** here (R3).
- **Traceability**: every derived figure is drillable to sourced leaves (`all_leaves_sourced`
  CI-checkable).
- **Faction scoping**: crew + assets are per faction.
