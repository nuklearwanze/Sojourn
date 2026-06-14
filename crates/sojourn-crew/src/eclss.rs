//! ECLSS reliability, degradation, maintenance & seeded failure (R9, Q1,
//! FR-LSC-501…504). The failure probability is a multiplicative hazard over
//! maturity, maintenance deficit and degradation; a critical failure beyond abort
//! reach is a loss-of-crew risk.

use crate::asset::{CrewedAsset, EclssState};
use crate::hazard;
use crate::inputs::TechMaturity;
use crate::params::EclssParams;

/// Initial reliability captured from technology maturity + flight heritage.
pub fn reliability_from_maturity(m: &TechMaturity, p: &EclssParams) -> f64 {
    let below = (i32::from(p.trl_ref) - i32::from(m.trl)).max(0) as f64;
    let heritage = 1.0 - 1.0 / (f64::from(m.flight_units) + 2.0); // → 1 with heritage
    (m.reliability * heritage / (1.0 + p.low_trl_mult * below)).clamp(0.0, 1.0)
}

fn maturity_mult(m: &TechMaturity, p: &EclssParams) -> f64 {
    let below = (i32::from(p.trl_ref) - i32::from(m.trl)).max(0) as f64;
    1.0 + p.low_trl_mult * below
}

/// The daily ECLSS failure probability (multiplicative hazard, R12/Q1).
pub fn failure_prob(asset: &CrewedAsset, p: &EclssParams) -> f64 {
    hazard::hazard(
        p.failure_base_rate,
        &[
            maturity_mult(&asset.eclss_maturity, p),
            1.0 + p.maintenance_mult * asset.eclss.maintenance_deficit.max(0.0),
            1.0 + asset.eclss.degradation.max(0.0),
        ],
    )
}

/// Advance ECLSS degradation + maintenance for one day (maintenance reduces the deficit).
pub fn accrue(eclss: &mut EclssState, maint_crew_hr: f64, maint_spares_kg: f64, p: &EclssParams) {
    eclss.degradation += p.degradation_rate;
    // Maintenance coverage ∈ [0,1]: enough crew-time AND spares ⇒ deficit clears.
    let crew_cov = if p.crew_hr_per_day > 0.0 {
        (maint_crew_hr / p.crew_hr_per_day).min(1.0)
    } else {
        1.0
    };
    let spares_cov = if p.spares_per_day > 0.0 {
        (maint_spares_kg / p.spares_per_day).min(1.0)
    } else {
        1.0
    };
    let coverage = crew_cov.min(spares_cov);
    // Deficit rises by (1 − coverage); maintenance also pays down degradation.
    eclss.maintenance_deficit = (eclss.maintenance_deficit + (1.0 - coverage) * 0.1).max(0.0);
    if coverage > 0.0 {
        eclss.degradation = (eclss.degradation - coverage * p.degradation_rate).max(0.0);
        eclss.maintenance_deficit = (eclss.maintenance_deficit - coverage * 0.1).max(0.0);
    }
}

/// A critical failure beyond abort reach is a loss-of-crew risk (FR-LSC-503).
pub fn critical(asset: &CrewedAsset) -> bool {
    asset.eclss.failed && !asset.env.abort_reach
}
