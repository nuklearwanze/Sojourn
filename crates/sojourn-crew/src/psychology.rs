//! Psychology (R8, FR-LSC-401…403). Psychological load accrues with mission
//! duration, confinement (habitat volume per crew) and comms-lag; it lowers
//! capability and raises the seeded anomaly hazard (a high-load crew earns its
//! anomaly probability).

use crate::asset::CrewedAsset;
use crate::hazard;
use crate::params::PsychParams;

/// One day's psychological-load increment for an asset.
pub fn daily_load(asset: &CrewedAsset, elapsed_days: f64, p: &PsychParams) -> f64 {
    let confinement =
        (p.confinement_ref_m3 / asset.sizing.habitat_volume_m3_per_crew.max(1.0)).clamp(0.0, 10.0);
    let comms = p.comms_lag_sens * asset.env.comms_lag_s;
    p.base_rate * (1.0 + p.duration_sens * elapsed_days + confinement + comms)
}

/// The psychological capability factor ∈ [0,1] (1 = baseline).
pub fn capability_factor(psych_load: f64) -> f64 {
    (1.0 - psych_load).clamp(0.0, 1.0)
}

/// The seeded daily anomaly probability (multiplicative hazard, R12): rises with
/// psych load and ops oversubscription.
pub fn anomaly_prob(psych_load: f64, ops_oversub: f64, p: &PsychParams) -> f64 {
    hazard::hazard(
        p.anomaly_base,
        &[1.0 + psych_load, 1.0 + ops_oversub * p.anomaly_ops_mult],
    )
}
