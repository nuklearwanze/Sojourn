# Phase 1 Data Model: Bases & Construction (FA-07)

Entities, fields, relationships, validation rules and state transitions for `sojourn-base`. Types are
illustrative (Rust-shaped); all quantities **SI** (kg, m, m², W, kg/m², closure ∈ [0,1], crew count,
seconds). Ordered `BTreeMap`/`BTreeSet` throughout (determinism). "DATA" = sourced file under
`data/base/`; "SLICE" = owned persistent state; "DERIVED" = pure query-time computation (never stored).
Cross-slice physics enters as **composed values** (see `contracts/integration-seams.md`).

---

## 1. Identity (`ids.rs`)

- `FactionId(u32)` — owner (aligns with world/research/economy faction id).
- `BaseId(u32)` — a base/station instance.
- `ModuleId(u32)` — a module instance within a base.
- `ModuleTypeId(String)` — a catalogue module type (transparent).
- `SiteId(String)` — a world Site or dynamical-location id (transparent; references FA-03).
- `ProjectId(u32)` — a construction project.

---

## 2. Module catalogue (`catalogue.rs`, DATA — `modules.ron`)

- `ModuleParams` enum (class is the variant):
  - `Habitat { crew_accommodation: u32, pressurized_volume_m3: f64 }`
  - `Power { gen_w: f64, solar: bool }`
  - `Eclss { closure_fraction: f64, per_crew_day_kg: f64, crew_support: u32 }`
  - `Greenhouse { food_kg_per_day: f64, crew_fed: u32, power_demand_w: f64 }` — bioregenerative food
    production (F4/F5); the local-supply source for the **food** self-sufficiency loop (R11).
  - `IsruHost { process_ref: String }` — hosts an FA-06 ISRU process (output composed in).
  - `Science { data_rate: f64 }`
  - `Storage { commodity: String, buffer_capacity_kg: f64 }`
  - `Manufacturing { output_commodity: String, rate_kg_per_day: f64 }`
  - `Shielding { material: String, areal_density_kg_m2: f64 }`
- `ModuleType { id: ModuleTypeId, tech: String, dry_mass_kg: f64, power_demand_w: f64, params: ModuleParams, source: String }`.
- **Validation**: non-empty `source`; `dry_mass_kg`/`power_demand_w` ≥ 0; `closure_fraction` ∈ [0,1];
  `Shielding.material` resolves against `params.ron` attenuation lengths; **no combat module** (name +
  kind screen, Principle IX); unique ids.

---

## 3. Tuning params (`catalogue.rs`/`shielding.rs`/`sustainability.rs`, DATA — `params.ron`)

- `ShieldMaterial { id, attenuation_length_kg_m2: f64, source }` — λ for `exp(−ρx/λ)` (R7).
- `DoseLimit { crew_annual_sv: f64, source }` — the shielding-shortfall threshold.
- `ClosureLoop { id: "air-water" | "food" | "materials" | "power" | "spares", source }` — the loops the
  self-sufficiency index minimises over (R11).
- `ConstructionParams { crew_hr_per_kg: f64, regolith_to_shield_ratio: f64, regolith_to_structure_ratio: f64, source }`
  — build labour and local-material conversion (R8/R10).
- `Params { solar_ref_m: f64, … , source }` — PV reference distance (solar scaling), etc.

---

## 4. Base & modules (`base.rs`, SLICE)

- `ModuleInstance { id: ModuleId, type_id: ModuleTypeId, commissioned: bool, delivered_mass_kg: f64, crew_time_hr: f64, built_local: bool }`.
- `Base { id: BaseId, faction: FactionId, site: SiteId, class: String, modules: BTreeMap<ModuleId, ModuleInstance>, next_module: u32, project: Option<ProjectId> }`.
- **State transition (module)**: `planned (commissioned=false) → operational (commissioned=true)` when
  `delivered_mass ≥ required_mass` **and** `crew_time ≥ required_crew_time` (R8); `built_local` marks
  modules whose mass was satisfied by on-site production (R10) rather than import.
- **Validation**: `site`/`class`/`type_id` resolve; a module's `tech` must be flyable per the composed
  `TechMaturity` (gated at query/command time, the honest seam).

---

## 5. Construction project (`construction.rs`, SLICE)

> **FA-06 vs FA-07 layering (C1)**: FA-06's `Project` (FR-EC-808) is the **delivery** accounting — what
> mass/crew-time has *landed at a location* via the logistics graph. FA-07's `ConstructionProject` is
> the **assembly** accounting — which *modules commission* from those deliveries. They are distinct,
> complementary layers; the host bridges an FA-06 delivery milestone into a `DeliverToBase` command
> (`integration-seams.md`). No duplication.

- `ConstructionProject { id: ProjectId, base: BaseId, demands: BTreeMap<ModuleId, ModuleDemand>, started_tick: u64 }`.
- `ModuleDemand { required_mass_kg: f64, required_crew_time_hr: f64 }` — derived from the module type's
  `dry_mass_kg` + `ConstructionParams.crew_hr_per_kg`.
- DERIVED progress: delivered vs remaining mass/crew-time, commissioned-module count, % complete.
- **Transitions**: `Open → InProgress (some delivered) → Complete (all modules commissioned)`.

---

## 6. Composed-value inputs (`inputs.rs`, R2)

- `SiteFacts { pp_category: PpCategory, illumination: f64, thermal_k: f64, slope_deg: f64, comms_visible: bool, hazard_level: f64, radiation_env_sv_yr: f64, resource_grade: f64, solar_distance_m: f64 }`.
- `TechMaturity { trl: u8, understanding: f64, flyable: bool }`.
- `DeliveryStatus { delivered_mass_kg: f64, crew_time_hr: f64 }` keyed by `(BaseId, ModuleId)`.
- `IsruOutput { rate_kg_per_day: f64 }` keyed by `(BaseId, commodity)`.
- `BaseInputs { sites: BTreeMap<SiteId, SiteFacts>, maturities: BTreeMap<(FactionId, ModuleTypeId-tech), TechMaturity>, isru: BTreeMap<(BaseId, String), IsruOutput> }`.

`PpCategory` enum (I…V + SpecialRegion) mirrors the FA-03 world value (a plain enum here, not the world
crate's type — the core-only decoupling).

---

## 7. Emergent properties (`power.rs`/`shielding.rs`/`lifesupport.rs`, DERIVED)

- `PowerBalance { generation_w, demand_w, margin_w }` — Σ over **commissioned** modules; PV scaled by
  `(solar_ref_m / SiteFacts.solar_distance_m)²` (R5).
- `Shielding { areal_density_kg_m2, attenuation_factor, transmitted_dose_sv_yr, shortfall: bool }` —
  `exp(−Σᵢ ρxᵢ/λᵢ)` summed **per material in the exponent** across shielding modules (mixed materials
  compose as the product of per-material attenuations); shortfall vs `DoseLimit` (R7).
- `LifeSupport { closure_fraction, population_capacity, consumables_per_day_kg }` — best-module closure;
  **population capacity = min(Σ habitat accommodation, Σ ECLSS crew_support)**. Power is a **separate
  viability flag**: a negative power margin red-flags the base (FR-BC-103) rather than fractionally
  reducing population — there is no per-crew power figure (U1) (R6).
- `EmergentProperties { power: PowerBalance, shielding: Shielding, life_support: LifeSupport,
  self_sufficiency: f64, hazard_exposure: f64 }` — the full derived state.

---

## 8. On-site production (`production.rs`, DERIVED + SLICE accounting)

- DERIVED `LocalProduction { regolith_shield_rate_kg_day, manufacturing_rates: BTreeMap<commodity, f64>,
  import_mass_avoided_kg }` — from `IsruOutput` + manufacturing modules + `ConstructionParams`.
- A `BuildLocal` command marks a module `built_local` (its required mass satisfied by on-site
  regolith construction), reducing the project's imported-mass demand (R10, FR-BC-402/404).

---

## 9. Sustainability & embargo (`sustainability.rs`, DERIVED)

- `SelfSufficiency { index: f64, binding_loop: String, ratios: BTreeMap<String, f64> }` —
  `index = min over loops of (local_supply / demand).min(1)` (R11).
- `EmbargoResult { survives: bool, embargo_years: f64, failing_loops: Vec<String> }` — per loop,
  survive iff `production ≥ demand` or `buffer ≥ (demand − production) × span`; survives iff all do (R12).

---

## 10. Siting & guards (`siting.rs`, DERIVED)

- `Severity` enum `Hard | Soft`; `RedFlag { constraint: String, severity: Severity, detail: String }`.
- `siting_flags(base, site_facts, derived)` → flags: PP Special-Region without containment (Hard),
  negative power margin (Hard), shielding shortfall (Hard), permanent-shadow solar base (Hard),
  unbuildable slope (Soft/Hard), no comms visibility (Soft), sub-maturity module (Soft) (R9, FR-BC-301…303).
- `ForwardContamination { pp_value_loss: f64 }` — representable consequence on violation.

---

## 11. Module state & wiring (`module.rs`, SLICE)

- `BaseSlice { bases: BTreeMap<BaseId, Base>, projects: BTreeMap<ProjectId, ConstructionProject>,
  next_base: u32, next_project: u32, milestones: BTreeSet<String>, data_hash: [u8; 32] }`.
- `BaseModule { catalogue, params, classes }` (loaded DATA) + `load(dir)`.
- Manifest: `id = "base"`, owns the base slice, **zero streams** (deterministic derivations), emits the
  R14 events, `cadence_ticks = 86_400`, daily `step` (time-based construction bookkeeping; commissioning
  lands via commands), `save_slice`/`load_slice` (verify `data_hash`).
- DERIVED (`query.rs`): `BaseSnapshot` composes the slice + `BaseInputs`; pure queries per R13.

---

## 12. Traceability (`trace.rs`, DERIVED)

- `TraceTree` (reused shape from FA-04/06): `Leaf { name, value, source } | Node { op, value, inputs }`.
- Every emergent property resolves to a tree whose leaves are **sourced** module/site values
  (`all_leaves_sourced` is CI-checkable) — the FR-BC-106 / Principle VIII honesty contract.

---

## 13. Entity relationship summary

```text
Faction ──1:N── Base ──at── Site (SiteFacts composed from FA-03 belief-state)
Base ──N── ModuleInstance ──typed by── ModuleType (catalogue, gated by TechMaturity from FA-05)
Base ──1── ConstructionProject ──demands── delivered-mass + crew-time (DeliveryStatus from FA-06)
Base ──hosts── IsruOutput (FA-06) ──feeds── regolith construction → import-mass substitution
EmergentProperties ──DERIVED from── commissioned modules + SiteFacts (power, shielding, closure, …)
SelfSufficiency = min(loop ratios)   EmbargoResult = analytic rate+buffer over loops + storage buffers
BaseSnapshot ──exposes── properties/production-consumption/habitat-state/milestones (to FA-06/08/09)
```

All cross-slice arrows are **composed values**, not crate dependencies (R1/R2).
