# Contract: Integration Seams — Composed-Value Inputs (FA-08)

`sojourn-crew` depends **only on `sojourn-core`** (R1). Every cross-slice value enters as a **plain,
serializable input** the host composes from the upstream query surfaces — the FA-04/06/07 decoupling.
This file defines the value shapes and who fills them. Tests pass stubs; the harness/Tauri host fills
them from the real upstream modules.

## `CrewInputs` (bundle passed to commands at decision time and to the snapshot at query time)

```text
CrewInputs {
    sizing:         BTreeMap<AssetId, AssetSizing>,      // from FA-04 vehicle sizing / FA-07 base state
    env:            BTreeMap<AssetId, EnvFacts>,         // from FA-03 environment / FA-06 light-time + abort reach
    roster:         CrewRoster,                          // from FA-05 (astronaut id → age/sex/traits/training)
    eclss_maturity: TechMaturity,                        // from FA-05 (ECLSS tech maturity + heritage)
    ops:            BTreeMap<FactionId, OpsLoad>,        // from FA-06 ops pool
    edl_suitability:BTreeMap<AssetId, EdlSuitability>,   // from FA-04 designer EDL check
}
```

## The value shapes

| Shape | Fields | Source query (host-side) | Honesty note |
|---|---|---|---|
| `AssetSizing` | `closure_capability, shield_attenuation, population_capacity, spin_gravity, habitat_volume_m3_per_crew, consumables_capacity_kg, crewed` | FA-04 `DesignSnapshot` / FA-07 `BaseSnapshot` | the **static sizing**; FA-08 evolves the dynamic state from it |
| `EnvFacts` | `gcr_rate_sv_yr, body, comms_lag_s, abort_reach` | FA-03 site/body env + FA-06 light-time + logistics abort reach | the surveyed/known environment |
| `AstronautFacts` | `age_years, sex, traits, training` | FA-05 astronaut roster | **age/sex feed the REID model** (Q2) |
| `TechMaturity` | `trl, reliability, flight_units` | FA-05 `maturity()` | ECLSS reliability/heritage |
| `OpsLoad` | `oversubscription` | FA-06 `ops_utilisation()` | raises the anomaly hazard (FA-06 consistency) |
| `EdlSuitability` | `can_land, has_heat_shield, landing_tw` | FA-04 `red_flags`/EDL check | the vehicle's static EDL suitability |

## Why composed values, not crate deps

- **Testability**: the crew model is unit-tested with stub inputs — no vehicle/research/economy/base
  crate needed to test consumables/dose/decon/psych/ECLSS/EDL (mirrors FA-06/07's stub inputs).
- **Honesty**: the model acts on the **composed** sizing/roster/maturity — it never holds the upstream
  authoritative state, so it cannot reach hidden truth.
- **Acyclic graph, no new edges**: `crew → core` only; the arrow to FA-09 points the other way. The host
  composes inputs at the IPC boundary, including bridging FA-05's astronaut pipeline (age/sex) → the
  roster and FA-08's career dose/REID back to the FA-05 pipeline record.

## Host composition (harness & Tauri)

The harness `crew` scenario flag installs `CrewModule` and fills `CrewInputs` from stub or real upstream
queries before issuing `OccupyAsset`/`Maintain`/`EvaluateEdl` and reading the `CrewSnapshot`. No upstream
module is modified.
