//! Low-thrust arc estimates (FR-ASTRO-405): Edelbaum-class circular-to-circular
//! spiral Δv ≈ |v1 − v2| with duration/propellant from the constant-thrust
//! rocket equation (Edelbaum 1961). Reconciled against flown arcs by the suite.

use super::Regime;
use serde::{Deserialize, Serialize};

/// Spiral estimate between circular radii.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArcEstimate {
    /// Total Δv (m/s).
    pub dv: f64,
    /// Burn duration (s).
    pub duration_s: f64,
    /// Propellant (kg).
    pub propellant_kg: f64,
    /// Honesty tag (Edelbaum assumes slow spiral: low accel vs gravity).
    pub regime: Regime,
}

/// Estimate a tangential-thrust spiral from `r1` to `r2` (circular, coplanar).
pub fn spiral_estimate(
    mu: f64,
    r1: f64,
    r2: f64,
    thrust_n: f64,
    exhaust_velocity: f64,
    mass0_kg: f64,
) -> Option<ArcEstimate> {
    if r1 <= 0.0 || r2 <= 0.0 || thrust_n <= 0.0 || mass0_kg <= 0.0 {
        return None;
    }
    let v1 = libm::sqrt(mu / r1);
    let v2 = libm::sqrt(mu / r2);
    let dv = libm::fabs(v1 - v2);
    // Constant-thrust rocket equation: m1 = m0·e^(−Δv/ve); t = (m0−m1)·ve/T.
    let m1 = mass0_kg * libm::exp(-dv / exhaust_velocity);
    let propellant = mass0_kg - m1;
    let duration = propellant * exhaust_velocity / thrust_n;
    // Edelbaum regime check: thrust accel ≪ local gravity.
    let accel = thrust_n / mass0_kg;
    let gravity = mu / (r1 * r1);
    let regime = if accel < 0.01 * gravity {
        Regime::WellConditioned
    } else {
        Regime::LowConfidence
    };
    Some(ArcEstimate {
        dv,
        duration_s: duration,
        propellant_kg: propellant,
        regime,
    })
}
