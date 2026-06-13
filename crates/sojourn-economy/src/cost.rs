//! Cost model (data-model §6, R9, FR-EC-401…404). P50/P80 uncertainty bands over
//! the FA-04 `VehicleCost` basis + maturity, a learning curve over cumulative
//! production (Wright's law, exponent from data), and a seeded overrun
//! realisation. Every figure traces to its mass/maturity basis.

use crate::inputs::{TechMaturity, VehicleCost};
use crate::trace::TraceTree;
use serde::{Deserialize, Serialize};

/// Cost-uncertainty parameters (DATA, `cost.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostParams {
    /// P80 sits this fraction above P50 (e.g. 0.4 ⇒ P80 = 1.4·P50).
    pub p80_spread: f64,
    /// Maximum fractional overrun above P50 the realisation can reach.
    pub overrun_max: f64,
    /// Wright's-law learning exponent (unit cost ∝ (units+1)^-exponent).
    pub learning_exponent: f64,
    /// Immaturity cost penalty per unit of (9 − TRL)/9.
    pub immaturity_penalty: f64,
    /// Provenance.
    pub source: String,
}

/// A cost estimate with uncertainty bands and a traceable basis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostEstimate {
    /// P50 (median) cost.
    pub p50: f64,
    /// P80 cost (the 80th-percentile band).
    pub p80: f64,
    /// The learning factor applied (≤ 1).
    pub learning_factor: f64,
}

/// The learning factor for a cumulative production count.
pub fn learning_factor(cumulative_units: u32, params: &CostParams) -> f64 {
    libm::pow(f64::from(cumulative_units) + 1.0, -params.learning_exponent)
}

fn immaturity_mult(trl: u8, params: &CostParams) -> f64 {
    let immaturity = ((9i32 - i32::from(trl)).max(0) as f64) / 9.0;
    1.0 + params.immaturity_penalty * immaturity
}

/// Estimate P50/P80 over the vehicle cost basis + maturity + cumulative production.
pub fn estimate(
    basis: &VehicleCost,
    maturity: &TechMaturity,
    cumulative_units: u32,
    params: &CostParams,
) -> CostEstimate {
    let lf = learning_factor(cumulative_units, params);
    let p50 = basis.unit_cost_basis * lf * immaturity_mult(maturity.trl, params);
    let p80 = p50 * (1.0 + params.p80_spread);
    CostEstimate {
        p50,
        p80,
        learning_factor: lf,
    }
}

/// Realise a cost from the estimate on a seeded uniform draw `u ∈ [0,1)`. The
/// realisation lies in `[p50, p50·(1+overrun_max)]` — always ≥ P50 (overruns),
/// reproducible per seed (FR-EC-402).
pub fn realise(est: &CostEstimate, params: &CostParams, u: f64) -> f64 {
    est.p50 * (1.0 + params.overrun_max * u.clamp(0.0, 1.0))
}

/// Build the traceability tree for an estimate (money → mass/maturity basis).
pub fn trace_estimate(
    basis: &VehicleCost,
    maturity: &TechMaturity,
    cumulative_units: u32,
    params: &CostParams,
) -> TraceTree {
    let lf = learning_factor(cumulative_units, params);
    let imm = immaturity_mult(maturity.trl, params);
    let p50 = basis.unit_cost_basis * lf * imm;
    TraceTree::node(
        "p50 = unit-cost-basis × learning-factor × immaturity-mult",
        p50,
        vec![
            TraceTree::leaf(
                "unit-cost-basis",
                basis.unit_cost_basis,
                "FA-04 vehicle cost basis (mass × cost-per-kg, learning applied)",
            ),
            TraceTree::leaf(
                "learning-factor",
                lf,
                "cost.ron learning_exponent (Wright's law)",
            ),
            TraceTree::leaf(
                "immaturity-mult",
                imm,
                "cost.ron immaturity_penalty × (9−TRL)/9",
            ),
        ],
    )
}
