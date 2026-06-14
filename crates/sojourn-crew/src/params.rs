//! The sourced parameter set (DATA, immutable). Consumables/radiation/physiology/
//! psychology/ECLSS/EDL + hazard thresholds, each carrying `source`; no
//! per-asset/per-crew magic numbers in code (Principle II). The dose→REID curve
//! (age/sex) and the per-body EDL difficulty (the Mars gap) live here.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

fn normalize(t: &str) -> String {
    t.replace("\r\n", "\n")
}

/// Per-crew-day consumables rates (A1: air/water recycled by closure, food open-loop).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumablesParams {
    /// O₂ per crew-day (kg) — air/water loop.
    pub o2_kg_per_crew_day: f64,
    /// Water per crew-day (kg) — air/water loop.
    pub water_kg_per_crew_day: f64,
    /// N₂ buffer per crew-day (kg) — air/water loop.
    pub n2_kg_per_crew_day: f64,
    /// Food per crew-day (kg) — **open-loop** (not recycled by ECLSS closure).
    pub food_kg_per_crew_day: f64,
    /// Provenance.
    pub source: String,
}

impl ConsumablesParams {
    /// Air/water gross per crew-day (kg) — recycled by ECLSS closure.
    pub fn air_water_per_crew_day(&self) -> f64 {
        self.o2_kg_per_crew_day + self.water_kg_per_crew_day + self.n2_kg_per_crew_day
    }
}

/// The dose→REID curve (Q2): REID% from accumulated dose + age/sex.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReidCurve {
    /// Baseline REID percent per sievert (at the baseline age, male).
    pub pct_per_sv: f64,
    /// Per-year age coefficient (younger ⇒ higher lifetime risk).
    pub age_coeff: f64,
    /// Age baseline (years).
    pub age_baseline: f64,
    /// Female multiplier (> 1: higher risk per dose).
    pub sex_mult_female: f64,
    /// Grounding threshold (percent) — 3.0 (NASA).
    pub threshold_pct: f64,
    /// Provenance.
    pub source: String,
}

/// Radiation params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadiationParams {
    /// GCR rate (Sv/yr) per environment id (fallback "default").
    pub gcr_by_env: BTreeMap<String, f64>,
    /// Daily SPE-storm probability.
    pub spe_daily_prob: f64,
    /// SPE acute dose (Sv) when a storm hits unsheltered.
    pub spe_magnitude_sv: f64,
    /// Shelter attenuation factor applied to the SPE dose when sheltering.
    pub shelter_attenuation: f64,
    /// The dose→REID curve.
    pub reid: ReidCurve,
    /// Provenance.
    pub source: String,
}

/// Physiology / deconditioning params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysiologyParams {
    /// Per-day micro-g accrual for each index (fraction/day).
    pub bone_rate: f64,
    /// Muscle accrual.
    pub muscle_rate: f64,
    /// Cardiovascular accrual.
    pub cardio_rate: f64,
    /// Vision accrual.
    pub vision_rate: f64,
    /// Exercise countermeasure effectiveness ∈ [0,1].
    pub exercise_eff: f64,
    /// Pharmacological countermeasure effectiveness ∈ [0,1].
    pub pharma_eff: f64,
    /// Artificial-gravity (spin-hab) effectiveness ∈ [0,1] — the strongest.
    pub artificial_g_eff: f64,
    /// Post-mission recovery rate (fraction/day).
    pub recovery_rate: f64,
    /// Provenance.
    pub source: String,
}

/// Psychology params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PsychParams {
    /// Base psych-load accrual per day.
    pub base_rate: f64,
    /// Duration sensitivity (load scales with elapsed days).
    pub duration_sens: f64,
    /// Reference habitat volume per crew (m³): less volume ⇒ more confinement load.
    pub confinement_ref_m3: f64,
    /// Comms-lag sensitivity (load scales with light-time).
    pub comms_lag_sens: f64,
    /// Anomaly base probability per day.
    pub anomaly_base: f64,
    /// Ops-oversubscription anomaly multiplier per unit overage.
    pub anomaly_ops_mult: f64,
    /// Provenance.
    pub source: String,
}

/// ECLSS reliability/failure params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EclssParams {
    /// Base daily failure probability (at full maturity, no deficit).
    pub failure_base_rate: f64,
    /// Reference TRL (≥ this ⇒ maturity multiplier ~1).
    pub trl_ref: u8,
    /// Per-TRL-below-ref failure multiplier.
    pub low_trl_mult: f64,
    /// Maintenance-deficit multiplier per unit deficit.
    pub maintenance_mult: f64,
    /// Degradation accrual per day (raises the hazard).
    pub degradation_rate: f64,
    /// Spares consumed per day to fully maintain.
    pub spares_per_day: f64,
    /// Crew-hours per day to fully maintain.
    pub crew_hr_per_day: f64,
    /// Provenance.
    pub source: String,
}

/// EDL crew-risk params (the Mars gap is `body_difficulty`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdlParams {
    /// Per-body EDL difficulty multiplier (Mars ≫ airless).
    pub body_difficulty: BTreeMap<String, f64>,
    /// Base EDL crew-loss probability.
    pub base_rate: f64,
    /// Multiplier when no heat shield is carried (atmospheric bodies).
    pub no_heatshield_mult: f64,
    /// Multiplier when the vehicle cannot land (T/W < 1).
    pub cant_land_mult: f64,
    /// Provenance.
    pub source: String,
}

/// Hazard thresholds (DATA, `params.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HazardParams {
    /// REID grounding threshold (percent).
    pub viability_reid_pct: f64,
    /// Minimum crew capability for viability.
    pub viability_capability_min: f64,
    /// Minimum consumables coverage (days) for viability.
    pub viability_consumables_days: f64,
    /// Provenance.
    pub source: String,
}

/// The full loaded, validated parameter set.
#[derive(Debug, Clone)]
pub struct Params {
    /// Consumables.
    pub consumables: ConsumablesParams,
    /// Radiation.
    pub radiation: RadiationParams,
    /// Physiology.
    pub physiology: PhysiologyParams,
    /// Psychology.
    pub psychology: PsychParams,
    /// ECLSS.
    pub eclss: EclssParams,
    /// EDL.
    pub edl: EdlParams,
    /// Hazard thresholds.
    pub hazard: HazardParams,
    /// Content hash (saves pin it).
    pub content_hash: [u8; 32],
}

impl Params {
    /// Load and validate from a directory (`data/crew`).
    pub fn load_dir(dir: &Path) -> Result<Params, String> {
        let read =
            |n: &str| std::fs::read_to_string(dir.join(n)).map_err(|e| format!("reading {n}: {e}"));
        let texts = [
            ("consumables.ron", read("consumables.ron")?),
            ("radiation.ron", read("radiation.ron")?),
            ("physiology.ron", read("physiology.ron")?),
            ("psychology.ron", read("psychology.ron")?),
            ("eclss.ron", read("eclss.ron")?),
            ("edl.ron", read("edl.ron")?),
            ("params.ron", read("params.ron")?),
        ];
        let consumables: ConsumablesParams =
            ron::from_str(&texts[0].1).map_err(|e| format!("consumables.ron: {e}"))?;
        let radiation: RadiationParams =
            ron::from_str(&texts[1].1).map_err(|e| format!("radiation.ron: {e}"))?;
        let physiology: PhysiologyParams =
            ron::from_str(&texts[2].1).map_err(|e| format!("physiology.ron: {e}"))?;
        let psychology: PsychParams =
            ron::from_str(&texts[3].1).map_err(|e| format!("psychology.ron: {e}"))?;
        let eclss: EclssParams =
            ron::from_str(&texts[4].1).map_err(|e| format!("eclss.ron: {e}"))?;
        let edl: EdlParams = ron::from_str(&texts[5].1).map_err(|e| format!("edl.ron: {e}"))?;
        let hazard: HazardParams =
            ron::from_str(&texts[6].1).map_err(|e| format!("params.ron: {e}"))?;

        // Source presence.
        for (n, s) in [
            ("consumables", &consumables.source),
            ("radiation", &radiation.source),
            ("radiation.reid", &radiation.reid.source),
            ("physiology", &physiology.source),
            ("psychology", &psychology.source),
            ("eclss", &eclss.source),
            ("edl", &edl.source),
            ("params", &hazard.source),
        ] {
            if s.trim().is_empty() {
                return Err(format!("{n}: empty source"));
            }
        }
        // REID threshold = 3% (NASA).
        if (radiation.reid.threshold_pct - 3.0).abs() > 1e-9 {
            return Err("radiation.reid.threshold_pct must be 3.0 (NASA REID)".into());
        }
        // Mars EDL gap: Mars strictly harder than an airless body (the Moon).
        let mars = *edl
            .body_difficulty
            .get("mars")
            .ok_or("edl: missing 'mars' difficulty")?;
        let moon = *edl
            .body_difficulty
            .get("moon")
            .ok_or("edl: missing 'moon' difficulty")?;
        if mars <= moon {
            return Err(format!(
                "edl: Mars difficulty ({mars}) must exceed an airless body (moon {moon})"
            ));
        }

        let canonical = texts
            .iter()
            .map(|(_, t)| normalize(t))
            .collect::<Vec<_>>()
            .join("\0");
        let content_hash = *blake3::hash(canonical.as_bytes()).as_bytes();
        Ok(Params {
            consumables,
            radiation,
            physiology,
            psychology,
            eclss,
            edl,
            hazard,
            content_hash,
        })
    }
}
