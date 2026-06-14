# Phase 1 Data Model: Life Support & Crew (FA-08)

Entities, fields, relationships, validation rules and state transitions for `sojourn-crew`. Types are
illustrative (Rust-shaped); all quantities **SI** (dose Sv, mass kg, time days/seconds, volume m³,
fractions ∈ [0,1]). Ordered `BTreeMap`/`BTreeSet` throughout (determinism). "DATA" = sourced file under
`data/crew/`; "SLICE" = owned **persistent dynamic** state (this slice's defining trait, R3); "DERIVED" =
pure query-time computation. Cross-slice physics enters as **composed values**
(`contracts/integration-seams.md`).

---

## 1. Identity (`ids.rs`)

- `FactionId(u32)` — owner.
- `AssetId(u32)` — a crewed asset (vehicle in transit or occupied base).
- `AstronautId(String)` — a crew member (transparent; references the FA-05 astronaut roster).
- `MissionId(u32)` — a mission/occupancy context (for viability + duration).

---

## 2. Composed-value inputs (`inputs.rs`, R2)

- `AssetSizing { closure_capability: f64, shield_attenuation: f64, population_capacity: u32, spin_gravity: bool, habitat_volume_m3_per_crew: f64, consumables_capacity_kg: f64, crewed: bool }` — FA-04/FA-07.
- `EnvFacts { gcr_rate_sv_yr: f64, body: String, comms_lag_s: f64, abort_reach: bool }` — FA-03/FA-06.
- `Sex` enum `{ Male, Female }`; `AstronautFacts { age_years: f64, sex: Sex, traits: Vec<String>, training: f64 }`.
- `CrewRoster { members: BTreeMap<AstronautId, AstronautFacts> }` — FA-05.
- `TechMaturity { trl: u8, reliability: f64, flight_units: u32 }` — ECLSS tech, FA-05.
- `OpsLoad { oversubscription: f64 }` — FA-06 (≥ 0; 0 = within capacity).
- `CrewInputs { sizing: BTreeMap<AssetId, AssetSizing>, env: BTreeMap<AssetId, EnvFacts>, roster: CrewRoster, eclss_maturity: TechMaturity, ops: BTreeMap<FactionId, OpsLoad>, edl_suitability: BTreeMap<AssetId, EdlSuitability> }`.

`EdlSuitability { can_land: bool, has_heat_shield: bool, landing_tw: f64 }` mirrors FA-04 (plain value here).

---

## 3. Sourced parameters (`params.rs`, DATA)

- `consumables.ron` → `ConsumablesParams { o2_kg_per_crew_day, water_kg_per_crew_day, food_kg_per_crew_day, n2_kg_per_crew_day, source }`.
- `radiation.ron` → `RadiationParams { gcr_by_env: BTreeMap<String, f64>, spe_daily_prob, spe_magnitude_sv, shelter_attenuation, reid: ReidCurve, source }`;
  `ReidCurve { sv_per_pct_base, age_coeff, sex_coeff_female, threshold_pct: 3.0, source }` (dose→REID with age/sex).
- `physiology.ron` → `PhysiologyParams { bone_rate, muscle_rate, cardio_rate, vision_rate, exercise_eff, pharma_eff, artificial_g_eff, recovery_rate, capability_curve, source }`.
- `psychology.ron` → `PsychParams { base_rate, duration_sens, confinement_sens, comms_lag_sens, anomaly_base, capability_curve, source }`.
- `eclss.ron` → `EclssParams { failure_base_rate, maturity_mult, maintenance_mult, degradation_rate, spares_per_day, crew_hr_per_day, source }`.
- `edl.ron` → `EdlParams { body_difficulty: BTreeMap<String, f64>, suitability_mult, no_heatshield_mult, base_rate, source }` (Mars difficulty ≫ airless).
- `params.ron` → `HazardParams { anomaly_ops_mult, viability_thresholds: { reid_pct, capability_min, consumables_days }, source }`.

**Validation**: non-empty `source` on every entry; rates/multipliers ≥ 0; REID threshold = 3.0;
`edl.body_difficulty["mars"]` strictly greater than any airless body; no combat parameter (Principle IX).

---

## 4. Crew asset & members (`asset.rs`, SLICE — the stored dynamic state, R3)

- `CrewMember { id: AstronautId, asset: AssetId, career_dose_sv: f64, decon: Deconditioning, psych_load: f64, status: CrewStatus }`.
- `Deconditioning { bone: f64, muscle: f64, cardio: f64, vision: f64 }` (each ∈ [0,1], 0 = baseline).
- `CrewStatus` enum: **`Active → Grounded` (REID ≥ 3%) | `Lost` (loss-of-crew)**.
- `CrewedAsset { id: AssetId, faction: FactionId, mission: Option<MissionId>, consumables_kg: f64, eclss: EclssState, occupied_since_tick: u64, crew: BTreeSet<AstronautId> }`.
- `EclssState { reliability: f64, degradation: f64, maintenance_deficit: f64, failed: bool }`.
- **Transitions**: `CrewMember.status`: Active→Grounded when derived REID ≥ 3% (R6); Active/Grounded→Lost
  on a loss-of-crew event (consumables exhaustion / critical ECLSS failure beyond abort / EDL failure).
  `CrewedAsset`: a critical ECLSS failure beyond abort reach, or consumables exhaustion, ⇒ loss-of-crew.

---

## 5. Mission viability (`consumables.rs`, DERIVED)

- `Viability { consumables_ok: bool, dose_ok: bool, eclss_ok: bool, capability_ok: bool, viable: bool, makeup_rate_kg_day: f64 }`.
- **Make-up (R5, A1)**: ECLSS closure recycles **air/water only** (O₂/water/N₂/CO₂); **food is
  open-loop**. `makeup_rate = air_water_gross_per_day × (1 − closure_capability) + food_gross_per_day`
  (an on-base greenhouse food supply, if composed in, reduces the food term). A robotic asset
  (`crewed=false`) has zero consumption and is always consumables-viable.

---

## 6. Radiation & REID (`radiation.rs`, SLICE accrual + DERIVED REID)

- Per-step accrual (SLICE): `career_dose += gcr_rate × shield_attenuation × dt + spe_dose` (SPE seeded,
  shelter-attenuated).
- DERIVED `Reid { pct: f64, grounded: bool }` = `reid_curve(career_dose, age, sex)`; `grounded = pct ≥ 3`.

---

## 7. Physiology, psychology & capability (`physiology.rs`, `psychology.rs`, `hazard.rs`, SLICE + DERIVED)

- Per-step (SLICE): `decon.* += rate × (1 − countermeasure_eff) × dt` (artificial-g strongest);
  `psych_load += base × duration_sens × confinement(volume) × comms_lag(lag)`.
- DERIVED capability (R11): `capability = decon_factor(decon) × psych_factor(psych_load) ×
  health_factor(reid)` — each a sourced [0,1] curve; the **multiplicative product** (Q3).

---

## 8. ECLSS reliability & failure (`eclss.rs`, SLICE + DERIVED, Q1)

- Per-step (SLICE): `degradation += degradation_rate × dt`; `maintenance_deficit` rises without
  crew-time + spares; a daily **failure roll** on `crew/eclss-failure` uses the multiplicative hazard.
- DERIVED `EclssRisk { failure_prob: f64, critical: bool }` =
  `clamp(failure_base_rate × maturity_mult(trl,flight_units) × maintenance_mult(deficit) × degradation, 0, 1)`;
  `critical = failed && !abort_reach`.

---

## 9. EDL crew risk (`edl.rs`, DERIVED + command, Q1)

- DERIVED `EdlRisk { crew_loss_prob: f64 }` =
  `clamp(base_rate × suitability_factor(EdlSuitability) × body_difficulty(body) × crew_state_factor(capability), 0, 1)`.
  Mars `body_difficulty` ≫ airless (the modelled gap). An `EvaluateEdl` command rolls `crew/edl-risk`
  against it; a failure marks loss-of-vehicle / loss-of-crew.
- Note (L1): `EdlSuitability`/`body` appear both in the **`EvaluateEdl` command payload** (the
  plan→preview→commit value rolled against, like FA-06's `DispatchShipment` carrying `edge_price`) and in
  **`CrewInputs`** (for the read-only `edl_risk` query). This is intentional command-time vs query-time
  composition, not duplication.

---

## 10. The hazard primitive (`hazard.rs`, DERIVED)

- `hazard(base: f64, factors: &[f64]) -> f64 = (base × ∏ factors).clamp(0.0, 1.0)` — the shared
  multiplicative-hazard composition backing ECLSS failure, anomaly and EDL (FR-LSC-808).
- `capability(factors: &[f64]) -> f64 = (∏ factors).clamp(0.0, 1.0)` — the capability product (R11).

---

## 11. Module state & wiring (`module.rs`, SLICE)

- `CrewSlice { assets: BTreeMap<AssetId, CrewedAsset>, members: BTreeMap<AstronautId, CrewMember>,
  next_asset: u32, lost: BTreeSet<AstronautId>, last_tick: u64, data_hash: [u8; 32] }`.
- `CrewModule { params }` (loaded DATA) + `load(dir)`.
- Manifest: `id = "crew"`, owns the crew slice, **streams** `["crew/spe-storm","crew/eclss-failure",
  "crew/edl-risk","crew/anomaly"]`, emits the R14 events, `cadence_ticks = 86_400`, daily seeded `step`
  (consumption/dose/decon/psych accrual + SPE/ECLSS rolls + viability/loss-of-crew checks),
  `save_slice`/`load_slice` (verify `data_hash`).
- DERIVED (`query.rs`): `CrewSnapshot` composes the slice + `CrewInputs`; pure queries per R13.

---

## 12. Commands & events (`module.rs`)

- Commands (R4): `OccupyAsset{faction, asset, crew, mission, sizing-ref}`, `AssignCrew`, `Maintain{asset,
  crew_hr, spares}`, `Shelter{asset, sheltering}` (SPE), `Resupply{asset, kg}`, `EvaluateEdl{asset, years…}`,
  `VacateAsset{asset}`.
- Events (R14): `spe-storm` (Interrupt), `eclss-failure` (Interrupt), `crew-anomaly` (LogOnly),
  `astronaut-grounded` (Interrupt), `loss-of-crew` (Interrupt — consumed by FA-09).

---

## 13. Traceability (`trace.rs`, DERIVED)

- `TraceTree` (reused shape from FA-04/06/07): `Leaf{name,value,source} | Node{op,value,inputs}`.
- Every derived figure (REID, capability, a hazard probability, make-up rate) resolves to **sourced**
  leaves (`all_leaves_sourced` CI-checkable) — the FR-LSC-801/VIII honesty contract.

---

## 14. Entity relationship summary

```text
Faction ──1:N── CrewedAsset ──hosts── CrewMember (per-astronaut dynamic health, keyed to FA-05 id)
CrewedAsset ──sized by── AssetSizing (FA-04 vehicle / FA-07 base)   ──in── EnvFacts (FA-03/FA-06)
CrewMember ──REID from── career_dose × AstronautFacts.age/sex (FA-05 roster, sourced dose→risk curve)
EclssState ──reliability from── TechMaturity (FA-05) ──failure roll── multiplicative hazard (seeded)
capability = decon_factor × psych_factor × health_factor (multiplicative product) ──gates── EDL + viability
loss-of-crew (consumables / ECLSS-beyond-abort / EDL / dose) ── physical loss + event ──> FA-09 politics
CrewSnapshot ──exposes── per-member + per-asset state (read-only) to FA-09/10
```

All cross-slice arrows are **composed values**, not crate dependencies (R1/R2).
