//! Radiation dose accrual + the REID dose-limit model (R6, Q2, FR-LSC-201…205).
//! Continuous GCR (shielding-attenuated) + seeded SPE storms (storm-shelter
//! mitigated); career dose → REID via a sourced dose→risk curve (age/sex);
//! grounded at 3%.

use crate::asset::CrewedAsset;
use crate::inputs::{AstronautFacts, Sex};
use crate::params::RadiationParams;

const DAYS_PER_YEAR: f64 = 365.25;

/// The daily GCR dose (Sv) for an asset's crew, shielding-attenuated.
pub fn daily_gcr(asset: &CrewedAsset, p: &RadiationParams) -> f64 {
    let env_rate = p
        .gcr_by_env
        .get(&asset.env.body)
        .copied()
        .or_else(|| p.gcr_by_env.get("default").copied())
        .unwrap_or(asset.env.gcr_rate_sv_yr);
    // Use the asset's composed GCR rate if it exceeds the catalogue default.
    let rate = env_rate.max(asset.env.gcr_rate_sv_yr);
    rate * asset.sizing.shield_attenuation / DAYS_PER_YEAR
}

/// The acute SPE dose (Sv) when a storm hits, shielding- and shelter-attenuated.
pub fn spe_dose(asset: &CrewedAsset, p: &RadiationParams) -> f64 {
    let shelter = if asset.sheltering {
        p.shelter_attenuation
    } else {
        1.0
    };
    p.spe_magnitude_sv * asset.sizing.shield_attenuation * shelter
}

/// Risk of Exposure-Induced Death (percent) for a career dose + age/sex (Q2).
pub fn reid_pct(career_dose_sv: f64, facts: &AstronautFacts, p: &RadiationParams) -> f64 {
    let c = &p.reid;
    // Younger crew accrue more lifetime risk per dose; females higher.
    let age_factor = 1.0 + c.age_coeff * (c.age_baseline - facts.age_years);
    let sex_factor = match facts.sex {
        Sex::Female => c.sex_mult_female,
        Sex::Male => 1.0,
    };
    (career_dose_sv * c.pct_per_sv * age_factor.max(0.0) * sex_factor).max(0.0)
}

/// True iff the REID reaches the grounding threshold.
pub fn grounded(career_dose_sv: f64, facts: &AstronautFacts, p: &RadiationParams) -> bool {
    reid_pct(career_dose_sv, facts, p) >= p.reid.threshold_pct
}
