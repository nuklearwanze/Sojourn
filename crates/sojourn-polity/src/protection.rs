//! COSPAR planetary protection (R7, FR-PEA-601…605): **graded** forward contamination
//! (degradation proportional to bioburden overage × crash/soft factor) + a
//! back-contamination chain. Pure functions over composed mission facts +
//! `ContaminationParams`; `module.rs` applies them to the stored pristine value.

use crate::params::ContaminationParams;

/// Forward-contamination severity ∈ [0,1] for a lander reaching a Special Region.
/// A compliant lander (bioburden ≤ limit) degrades nothing; otherwise degradation is
/// **graded by the overage ratio** (monotone) and **scaled by crash vs soft landing**.
pub fn forward_severity(
    lander_bioburden: f64,
    limit: f64,
    crash: bool,
    p: &ContaminationParams,
) -> f64 {
    if lander_bioburden <= limit || limit <= 0.0 {
        return 0.0;
    }
    let overage = lander_bioburden / limit - 1.0; // > 0
    let landing = if crash { p.crash_factor } else { p.soft_factor };
    let sev = p.overage_scale * libm::pow(overage, p.overage_exponent) * landing;
    sev.clamp(0.0, 1.0)
}

/// Apply a degradation severity to a pristine value ∈ [0,1] (monotone non-increasing).
pub fn degrade(pristine: f64, severity: f64) -> f64 {
    (pristine * (1.0 - severity)).clamp(0.0, 1.0)
}

/// The back-contamination penalty for a sample return: zero with a compliant
/// containment chain, the sourced penalty without one.
pub fn back_penalty(containment_chain: bool, p: &ContaminationParams) -> f64 {
    if containment_chain {
        0.0
    } else {
        p.backcontam_penalty
    }
}
