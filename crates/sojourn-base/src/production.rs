//! On-site production & import reduction (R10, FR-BC-401…404). FA-06 owns
//! resource-extraction ISRU (output composed in as `IsruOutput`); FA-07 owns the
//! **construction use** of local materials — regolith construction turns local
//! feedstock into shielding/structure that is **not launched**, and manufacturing
//! modules produce spares/materials locally. `built_local` modules' mass is
//! satisfied without import (Principle VII).

use crate::base::Base;
use crate::catalogue::{Catalogue, ModuleParams};
use crate::ids::BaseId;
use crate::inputs::BaseInputs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A base's on-site production state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalProduction {
    /// Regolith-construction shielding rate (kg/day), gated by ISRU regolith feedstock.
    pub regolith_shield_rate_kg_day: f64,
    /// Manufacturing rates by output commodity (kg/day).
    pub manufacturing_rates: BTreeMap<String, f64>,
    /// Cumulative imported mass avoided by on-site builds (kg).
    pub import_mass_avoided_kg: f64,
}

/// Compute on-site production for a base, composing FA-06 ISRU output.
pub fn compute(base: &Base, cat: &Catalogue, inputs: &BaseInputs, id: BaseId) -> LocalProduction {
    // Regolith construction: local regolith/metals feedstock → shielding mass.
    let regolith_rate = inputs.isru_rate(id, "regolith") + inputs.isru_rate(id, "metals");
    let regolith_shield = regolith_rate * cat.params.regolith_to_shield_ratio;

    let mut manufacturing_rates: BTreeMap<String, f64> = BTreeMap::new();
    for m in base.modules.values().filter(|m| m.commissioned) {
        if let Some(t) = cat.module(&m.type_id)
            && let ModuleParams::Manufacturing {
                output_commodity,
                rate_kg_per_day,
            } = &t.params
        {
            *manufacturing_rates
                .entry(output_commodity.clone())
                .or_insert(0.0) += rate_kg_per_day;
        }
    }

    // Imported mass avoided: the dry mass of every module built from local material.
    let import_mass_avoided = base
        .modules
        .values()
        .filter(|m| m.built_local)
        .filter_map(|m| cat.module(&m.type_id))
        .map(|t| t.dry_mass_kg)
        .sum();

    LocalProduction {
        regolith_shield_rate_kg_day: regolith_shield,
        manufacturing_rates,
        import_mass_avoided_kg: import_mass_avoided,
    }
}
