//! Conic planning predictions (FR-ASTRO-401): orbit summaries and node-chain
//! outcome prediction via universal-variable propagation, SOI-aware regime tags.

use super::Regime;
use crate::math::{Elements, StateVec, Vec3, state_to_elements, universal_propagate};
use serde::{Deserialize, Serialize};

/// Orbit summary around a body of parameter `mu`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrbitSummary {
    /// Classical elements at the summary epoch.
    pub elements: Elements,
    /// Period (s) for bound orbits.
    pub period_s: Option<f64>,
    /// Periapsis radius (m).
    pub periapsis_m: f64,
    /// Apoapsis radius (m; None when unbound).
    pub apoapsis_m: Option<f64>,
    /// Honesty tag.
    pub regime: Regime,
}

/// Summarise an orbit from a body-centred state.
pub fn orbit_summary(mu: f64, s: &StateVec, t_ns: i64, soi_radius: f64) -> OrbitSummary {
    let el = state_to_elements(mu, s, t_ns);
    let bound = el.ecc < 1.0 && el.sma.is_finite() && el.sma > 0.0;
    let period_s =
        bound.then(|| 2.0 * core::f64::consts::PI * libm::sqrt(el.sma * el.sma * el.sma / mu));
    let periapsis_m = el.sma * (1.0 - el.ecc);
    let apoapsis_m = if bound {
        Some(el.sma * (1.0 + el.ecc))
    } else {
        None
    };
    // Leaving (or living near) the SOI is patched-conic territory.
    let regime = match apoapsis_m {
        Some(apo) if apo > 0.8 * soi_radius => Regime::LowConfidence,
        None => Regime::LowConfidence,
        _ => Regime::WellConditioned,
    };
    OrbitSummary {
        elements: el,
        period_s,
        periapsis_m,
        apoapsis_m,
        regime,
    }
}

/// Predict the state after applying a chain of impulsive burns and coasts in a
/// single dominant-body conic regime. Burns are (coast_s, dv_world) pairs applied
/// in order. Returns the final state plus a regime tag.
pub fn predict_chain(
    mu: f64,
    start: &StateVec,
    legs: &[(f64, Vec3)],
    soi_radius: f64,
) -> Option<(StateVec, Regime)> {
    let mut s = *start;
    let mut regime = Regime::WellConditioned;
    for (coast_s, dv) in legs {
        if *coast_s > 0.0 {
            s = universal_propagate(mu, &s, *coast_s)?;
        }
        s.v = s.v + *dv;
        if s.r.norm() > soi_radius {
            regime = Regime::LowConfidence;
        }
    }
    Some((s, regime))
}
