//! Reliability composition (FR-VD-501, R8, clarified Q2:A): a reliability-block-
//! diagram over component maturities — series chains multiply; declared redundancy
//! blocks evaluate as parallel `1 − ∏(1 − rᵢ)`. Per-component reliability comes from
//! the maturity provider (FA-05 in production, a stub in tests).

use crate::catalogue::Catalogue;
use crate::design::VehicleDesign;
use crate::ids::ComponentId;
use crate::query::ComponentMaturity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Composed reliability + the per-component breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReliabilityResult {
    /// Composed per-use success probability ∈ [0,1].
    pub composed: f64,
    /// Per-component reliability (id, r).
    pub per_component: Vec<(ComponentId, f64)>,
    /// Components below TRL 6 (red-flagged).
    pub sub_trl6: Vec<ComponentId>,
}

/// Compose vehicle reliability. `maturity_of` returns a component's maturity given
/// its researching technology key.
pub fn compose(
    design: &VehicleDesign,
    cat: &Catalogue,
    maturity_of: &impl Fn(&str) -> ComponentMaturity,
) -> ReliabilityResult {
    // Map each redundant component to its block index.
    let mut block_of: BTreeMap<ComponentId, usize> = BTreeMap::new();
    for (i, blk) in design.redundancy.iter().enumerate() {
        for id in &blk.components {
            block_of.insert(id.clone(), i);
        }
    }

    let mut per_component = Vec::new();
    let mut sub_trl6 = Vec::new();
    let mut series = 1.0;
    let mut block_fail: BTreeMap<usize, f64> = BTreeMap::new(); // ∏(1 − rᵢ) per block

    for id in design.all_components() {
        let Some(c) = cat.component(id) else { continue };
        let m = maturity_of(&c.tech);
        per_component.push((id.clone(), m.reliability));
        if m.trl < 6 {
            sub_trl6.push(id.clone());
        }
        if let Some(&blk) = block_of.get(id) {
            *block_fail.entry(blk).or_insert(1.0) *= 1.0 - m.reliability;
        } else {
            series *= m.reliability;
        }
    }
    // Each redundancy block contributes (1 − ∏(1 − rᵢ)).
    for prod in block_fail.values() {
        series *= 1.0 - prod;
    }

    ReliabilityResult {
        composed: series.clamp(0.0, 1.0),
        per_component,
        sub_trl6,
    }
}
