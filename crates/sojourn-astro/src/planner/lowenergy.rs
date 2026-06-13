//! Research-gated low-energy routes (FR-ASTRO-406): when the gate is open, the
//! planner offers route families whose savings the propagator confirms (the
//! suite flies the reference route). Closed gate ⇒ empty, with everything else
//! unaffected.

use super::Regime;
use crate::bodies::BodyId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The research gate id this slice consumes (FA-05 supplies it in production).
pub const GATE: &str = "low-energy";

/// A low-energy route estimate (family parameters; the propagator is the judge).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteEstimate {
    /// Route family id.
    pub id: String,
    /// Origin region (body).
    pub from: BodyId,
    /// Capture target.
    pub to: BodyId,
    /// Estimated Δv saving versus the direct transfer (m/s).
    pub dv_saving: f64,
    /// Time-of-flight multiplier versus direct (the price of patience).
    pub tof_factor: f64,
    /// Honesty tag: WSB routes are emphatically multi-body territory — always
    /// LowConfidence at the planning tier; propagation is the truth.
    pub regime: Regime,
}

/// Route families for a body pair, gated (FR-ASTRO-406). The v1 catalogue is a
/// single trailing-edge lunar-assist capture family for the planet–moon pair;
/// richer families arrive with FA-03's real geometry.
pub fn routes(gates: &BTreeMap<String, bool>, from: BodyId, to: BodyId) -> Vec<RouteEstimate> {
    if !gates.get(GATE).copied().unwrap_or(false) {
        return Vec::new();
    }
    // Planet (1) → Moon (2) family in the test catalogue.
    if from == BodyId(1) && to == BodyId(2) {
        vec![RouteEstimate {
            id: "trailing-edge-assist-capture".into(),
            from,
            to,
            dv_saving: 150.0,
            tof_factor: 1.6,
            regime: Regime::LowConfidence,
        }]
    } else {
        Vec::new()
    }
}
