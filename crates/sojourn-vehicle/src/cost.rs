//! Physical cost & build-time estimate (FR-VD-701, R11, clarified scope Q3:A):
//! mass × maturity × learning-curve. A physical quantity — FA-06 prices it in the
//! six-currency economy.

use crate::catalogue::Params;
use serde::{Deserialize, Serialize};

/// A physical cost/build estimate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Unit cost (abstract, mass-driven).
    pub unit_cost: f64,
    /// Build time (days).
    pub build_days: f64,
    /// Learning-curve factor applied (1 at first unit, < 1 as count rises).
    pub learning_factor: f64,
}

/// Estimate cost/build from dry mass, average component maturity (mean TRL) and the
/// cumulative production count.
pub fn estimate(
    dry_mass: f64,
    params: &Params,
    avg_trl: f64,
    production_count: u32,
) -> CostEstimate {
    let immaturity = 1.0 + params.immaturity_cost_mult * ((9.0 - avg_trl).max(0.0) / 9.0);
    let learning = libm::pow((production_count as f64) + 1.0, -params.learning_exponent);
    CostEstimate {
        unit_cost: dry_mass * params.cost_per_kg * immaturity * learning,
        build_days: dry_mass * params.build_days_per_kg,
        learning_factor: learning,
    }
}
