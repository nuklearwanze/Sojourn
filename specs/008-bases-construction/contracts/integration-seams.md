# Contract: Integration Seams — Composed-Value Inputs (FA-07)

`sojourn-base` depends **only on `sojourn-core`** (R1). Every cross-slice value enters as a **plain,
serializable input** the host composes from the upstream query surfaces — the FA-04/FA-06 decoupling.
This file defines the four value shapes and who fills them. Tests pass stubs; the harness/Tauri host
fills them from the real upstream modules.

## `BaseInputs` (bundle passed to commands at decision time and to the snapshot at query time)

```text
BaseInputs {
    sites:      BTreeMap<SiteId, SiteFacts>,                       // from FA-03 world belief-state
    maturities: BTreeMap<(FactionId, String /*tech*/), TechMaturity>, // from FA-05 research
    delivery:   BTreeMap<(BaseId, ModuleId), DeliveryStatus>,      // from FA-06 delivery accounting
    isru:       BTreeMap<(BaseId, String /*commodity*/), IsruOutput>, // from FA-06 ISRU plants
}
```

## The four value shapes

| Shape | Fields | Source query (host-side) | Honesty note |
|---|---|---|---|
| `SiteFacts` | `pp_category, illumination, thermal_k, slope_deg, comms_visible, hazard_level, radiation_env_sv_yr, resource_grade, solar_distance_m` | FA-03 `believed_site` / `sites_on` / `site_target` | the **surveyed belief-state**, never ground truth |
| `TechMaturity` | `trl, understanding, flyable` | FA-05 `maturity` / `understanding` | gates module composition (D3/D4/D6/F/G/I10/I11) |
| `DeliveryStatus` | `delivered_mass_kg, crew_time_hr` | FA-06 `Project` / delivery accounting | the **delivered** mass — never an instant placement |
| `IsruOutput` | `rate_kg_per_day` | FA-06 ISRU plant output at the base's location | local material feeding construction/sustainment |

## Why composed values, not crate deps

- **Testability**: the base is unit-tested with stub inputs — no world/research/economy crate needed to
  test compose/derive, construction, siting, self-sufficiency (mirrors FA-06's stub inputs).
- **Honesty**: the base structurally acts on the **surveyed** site facts and the **delivered** mass — it
  *cannot* reach hidden truth or instant-place, because it never holds the upstream state.
- **Acyclic graph, no new edges**: `base → core` only; the arrow to FA-08/09 points the other way. The
  host composes inputs at the IPC boundary.

## Host composition (harness & Tauri)

The harness `base` scenario flag installs `BaseModule` and fills `BaseInputs` from stub or real upstream
queries before issuing `DeliverToBase`/`EvaluateEmbargo` and reading the `BaseSnapshot`. No upstream
module is modified. The host bridges FA-06 deliveries (a `Project` reaching a delivery milestone) into
`DeliverToBase` commands, closing the construction loop.
