//! Reliability & Flight Heritage (FR-RESP-202/703, R10, clarified Q1:A). Reliability
//! is a **scalar per-use success probability ∈ [0,1]** computed from TRL +
//! accumulated flight-units + domain-UL margin; heritage raises it toward a per-tech
//! ceiling and discounts derivatives.

use crate::ids::{FactionId, TechId};
use crate::tree::ReliabilityCurve;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Operational-use record for a technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Heritage {
    /// Successful operational uses.
    pub flight_units: u32,
}

/// Per-`(faction, tech)` heritage.
pub type HeritageMap = BTreeMap<(FactionId, TechId), Heritage>;

/// Scalar per-use reliability (R10). Returns 0 below TRL 6 (unflyable). The raw
/// inputs are exposed by the query layer alongside this number.
pub fn reliability(
    curve: &ReliabilityCurve,
    trl: u8,
    flight_units: u32,
    ul_margin: f64,
    trait_bonus: f64,
) -> f64 {
    if trl < 6 {
        return 0.0;
    }
    let trl_term = ((trl as f64 - 6.0) / 3.0).clamp(0.0, 1.0); // 0 @6 .. 1 @9
    let heritage_term = 1.0 - libm::exp(-(flight_units as f64) * curve.heritage_weight);
    let ul_term = (ul_margin / 100.0).clamp(0.0, 1.0) * curve.ul_weight;
    let base = curve.trl6_floor
        + (curve.ceiling - curve.trl6_floor) * (0.5 * trl_term + 0.5 * heritage_term)
        + ul_term
        + trait_bonus;
    base.clamp(0.0, curve.ceiling)
}

/// Is a technology flyable (TRL ≥ 6)?
pub fn flyable(trl: u8) -> bool {
    trl >= 6
}

/// Derivative discount: starting TRL for a program derived from a proven tech (it
/// starts partway up the ladder per accumulated heritage).
pub fn derivative_start_trl(base_start_trl: u8, parent_heritage: u32) -> u8 {
    let bump = if parent_heritage >= 3 {
        2
    } else if parent_heritage >= 1 {
        1
    } else {
        0
    };
    (base_start_trl + bump).min(7)
}
