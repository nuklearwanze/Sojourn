//! Radiation shielding (R7, FR-BC-105): a **mass-attenuation exponential** model
//! that composes per material in the exponent — attenuation = `exp(−Σᵢ ρxᵢ/λᵢ)` —
//! so mixed materials compose as the product of their per-material attenuations.

use crate::base::Base;
use crate::catalogue::{Catalogue, ModuleParams};
use crate::inputs::SiteFacts;
use crate::trace::TraceTree;
use serde::{Deserialize, Serialize};

/// A base's shielding state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Shielding {
    /// Total shielding areal density (kg/m²).
    pub areal_density_kg_m2: f64,
    /// Dose-attenuation factor ∈ (0,1].
    pub attenuation_factor: f64,
    /// Transmitted dose to crew (Sv/yr).
    pub transmitted_dose_sv_yr: f64,
    /// Transmitted dose exceeds the crew limit?
    pub shortfall: bool,
}

/// Σᵢ ρxᵢ/λᵢ — the dimensionless attenuation exponent (summed per material).
fn exponent(base: &Base, cat: &Catalogue) -> f64 {
    let mut sum = 0.0;
    for m in base.modules.values().filter(|m| m.commissioned) {
        if let Some(t) = cat.module(&m.type_id)
            && let ModuleParams::Shielding {
                material,
                areal_density_kg_m2,
            } = &t.params
            && let Some(lambda) = cat.params.attenuation_length(material)
        {
            sum += areal_density_kg_m2 / lambda;
        }
    }
    sum
}

/// Total shielding areal density (kg/m²).
fn areal_density(base: &Base, cat: &Catalogue) -> f64 {
    base.modules
        .values()
        .filter(|m| m.commissioned)
        .filter_map(|m| cat.module(&m.type_id))
        .filter_map(|t| match &t.params {
            ModuleParams::Shielding {
                areal_density_kg_m2,
                ..
            } => Some(*areal_density_kg_m2),
            _ => None,
        })
        .sum()
}

/// Compute the shielding state for a base at its site.
pub fn compute(base: &Base, cat: &Catalogue, site: Option<&SiteFacts>) -> Shielding {
    let factor = libm::exp(-exponent(base, cat));
    let env = site.map(|s| s.radiation_env_sv_yr).unwrap_or(0.0);
    let transmitted = env * factor;
    Shielding {
        areal_density_kg_m2: areal_density(base, cat),
        attenuation_factor: factor,
        transmitted_dose_sv_yr: transmitted,
        shortfall: transmitted > cat.params.crew_dose_limit_sv_yr,
    }
}

/// Traceability tree for the attenuation factor.
pub fn trace(base: &Base, cat: &Catalogue) -> TraceTree {
    let mut inputs = Vec::new();
    for m in base.modules.values().filter(|m| m.commissioned) {
        if let Some(t) = cat.module(&m.type_id)
            && let ModuleParams::Shielding {
                material,
                areal_density_kg_m2,
            } = &t.params
            && let Some(lambda) = cat.params.attenuation_length(material)
        {
            inputs.push(TraceTree::leaf(
                format!("{material}: ρx/λ = {areal_density_kg_m2}/{lambda}"),
                areal_density_kg_m2 / lambda,
                t.source.clone(),
            ));
        }
    }
    TraceTree::node(
        "attenuation = exp(−Σ ρxᵢ/λᵢ)",
        libm::exp(-exponent(base, cat)),
        inputs,
    )
}
