//! Reconciliation (FR-ASTRO-402): predicted-vs-propagated divergence, computed
//! honestly, with Lagrange-region named locations (FR-ASTRO-203).

use super::Regime;
use crate::bodies::{BodyId, Catalog, rails::BodyStates};
use crate::maneuver::AimPoint;
use crate::math::{StateVec, Vec3, universal_propagate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A reconciliation report for a planned craft.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationReport {
    /// Conic-predicted miss distance from the aim body at the aim epoch (m).
    pub miss_m: f64,
    /// Target closest-approach radius (m).
    pub target_radius_m: f64,
    /// Divergence beyond the target (m).
    pub divergence_m: f64,
    /// Threshold (m).
    pub threshold_m: f64,
    /// Exceeded?
    pub exceeded: bool,
    /// Honesty tag (heliocentric conic prediction across SOIs is patched).
    pub regime: Regime,
}

/// Compute the report for a craft's absolute state versus an aim point.
pub fn reconcile(
    catalog: &Catalog,
    motions: &BTreeMap<BodyId, crate::bodies::BodyMotion>,
    abs_state: &StateVec,
    now_tick: u64,
    aim: &AimPoint,
    threshold_m: f64,
) -> Option<ReconciliationReport> {
    if aim.epoch_tick <= now_tick {
        return None;
    }
    let mu = catalog.body(catalog.root()).expect("root").mu;
    let coast = (aim.epoch_tick - now_tick) as f64;
    let pred = universal_propagate(mu, abs_state, coast)?;
    let aim_ns = aim.epoch_tick as i64 * 1_000_000_000;
    let mut states = BodyStates::new(catalog, motions, aim_ns);
    let body_pos = states.absolute(aim.body).r;
    let miss = (pred.r - body_pos).norm();
    let divergence = (miss - aim.target_radius_m).max(0.0);
    Some(ReconciliationReport {
        miss_m: miss,
        target_radius_m: aim.target_radius_m,
        divergence_m: divergence,
        threshold_m,
        exceeded: divergence > threshold_m,
        regime: Regime::LowConfidence,
    })
}

/// A named Lagrange-region location of a (primary, secondary) pair
/// (FR-ASTRO-203): collinear L1/L2 from the classical quintic, solved by
/// bounded bisection on the collinear axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LagrangePoint {
    /// "L1" or "L2".
    pub name: String,
    /// Distance from the secondary along the primary–secondary line (m;
    /// negative = between the bodies).
    pub offset_from_secondary_m: f64,
}

/// Solve L1/L2 offsets for a pair (circular-orbit approximation of the rails).
/// Classical CR3BP collinear equilibria, found by bisection on the axial
/// rotating-frame acceleration balance (positive direction = toward/through the
/// secondary; barycentric centrifugal term included).
pub fn lagrange_points(
    catalog: &Catalog,
    primary: BodyId,
    secondary: BodyId,
) -> Vec<LagrangePoint> {
    let (Some(p), Some(s)) = (catalog.body(primary), catalog.body(secondary)) else {
        return Vec::new();
    };
    let Some(el) = &s.elements else {
        return Vec::new();
    };
    let d = el.sma; // separation (circular idealisation)
    let mu_frac = s.mu / (p.mu + s.mu); // barycentre offset fraction
    let omega2 = (p.mu + s.mu) / (d * d * d);

    // L1: between the bodies, x = distance from the secondary toward the primary.
    // Balance: mu_s/x² − mu_p/(d−x)² + ω²·(d(1−mu_frac) − x) = 0.
    let f1 = |x: f64| -> f64 {
        s.mu / (x * x) - p.mu / ((d - x) * (d - x)) + omega2 * (d * (1.0 - mu_frac) - x)
    };
    // L2: beyond the secondary, x = distance from the secondary away from the primary.
    // Balance: ω²·(d(1−mu_frac) + x) − mu_p/(d+x)² − mu_s/x² = 0.
    let f2 = |x: f64| -> f64 {
        omega2 * (d * (1.0 - mu_frac) + x) - p.mu / ((d + x) * (d + x)) - s.mu / (x * x)
    };

    let bisect = |f: &dyn Fn(f64) -> f64, mut lo: f64, mut hi: f64| -> Option<f64> {
        let (flo, fhi) = (f(lo), f(hi));
        if (flo > 0.0) == (fhi > 0.0) {
            return None;
        }
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if hi - lo < 1.0 {
                return Some(mid);
            }
            if (f(mid) > 0.0) == (flo > 0.0) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(0.5 * (lo + hi))
    };

    let mut out = Vec::new();
    if let Some(x) = bisect(&f1, 1.0e3, d * 0.5) {
        out.push(LagrangePoint {
            name: "L1".into(),
            offset_from_secondary_m: -x,
        });
    }
    if let Some(x) = bisect(&f2, 1.0e3, d * 0.5) {
        out.push(LagrangePoint {
            name: "L2".into(),
            offset_from_secondary_m: x,
        });
    }
    out
}

/// Absolute position of a named Lagrange point at a time.
pub fn lagrange_position(
    catalog: &Catalog,
    motions: &BTreeMap<BodyId, crate::bodies::BodyMotion>,
    primary: BodyId,
    secondary: BodyId,
    point: &LagrangePoint,
    t_ns: i64,
) -> Vec3 {
    let mut states = BodyStates::new(catalog, motions, t_ns);
    let p = states.absolute(primary);
    let s = states.absolute(secondary);
    let axis = (s.r - p.r).normalized();
    s.r + axis * point.offset_from_secondary_m
}
