//! Thermal/radiator balance (FR-VD-401/006, R7): waste-heat load vs radiator
//! rejection capacity. Radiators are explicit Thermal components (their mass is
//! already counted in dry mass); an undersized radiator set is a red-flag.

use crate::catalogue::{Catalogue, ComponentParams};
use crate::design::VehicleDesign;
use crate::propulsion;
use serde::{Deserialize, Serialize};

/// A thermal balance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ThermalBalance {
    /// Waste-heat load (W).
    pub heat_w: f64,
    /// Radiator rejection capacity (W).
    pub reject_w: f64,
    /// Margin (W) — must be ≥ 0.
    pub margin_w: f64,
}

/// Thermal balance: heat from engines/payloads vs radiator capacity.
pub fn balance(design: &VehicleDesign, cat: &Catalogue) -> ThermalBalance {
    let mut heat = 0.0;
    let mut reject = 0.0;
    for id in design.all_components() {
        let Some(c) = cat.component(id) else { continue };
        match &c.params {
            ComponentParams::Thermal { reject_w_per_kg } => {
                reject += c.dry_mass_kg * reject_w_per_kg
            }
            ComponentParams::Payload { waste_heat_w, .. } => heat += waste_heat_w,
            ComponentParams::Engine { .. } => heat += propulsion::engine_waste_heat(c),
            _ => {}
        }
    }
    ThermalBalance {
        heat_w: heat,
        reject_w: reject,
        margin_w: reject - heat,
    }
}
