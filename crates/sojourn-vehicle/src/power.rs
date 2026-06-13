//! Power balance (FR-VD-401, R7): generation (PV scaled by solar distance, RTG,
//! fission) vs demand (propulsion/avionics/payload/comms/life-support), per the
//! evaluation distance.

use crate::catalogue::{Catalogue, ComponentParams};
use crate::design::VehicleDesign;
use crate::propulsion;
use serde::{Deserialize, Serialize};

/// A power balance at a solar distance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PowerBalance {
    /// Generation (W).
    pub generation_w: f64,
    /// Demand (W).
    pub demand_w: f64,
    /// Margin (W) — must be ≥ 0 in some mode.
    pub margin_w: f64,
}

/// Power balance at `solar_dist_m` from the Sun (PV ∝ (ref/dist)²).
pub fn balance(design: &VehicleDesign, cat: &Catalogue, solar_dist_m: f64) -> PowerBalance {
    let ref_m = cat.params.solar_ref_m;
    let mut generation = 0.0;
    let mut demand = 0.0;
    for id in design.all_components() {
        let Some(c) = cat.component(id) else { continue };
        match &c.params {
            ComponentParams::Power { gen_w, solar } => {
                generation += if *solar {
                    gen_w * (ref_m / solar_dist_m.max(1.0)).powi(2)
                } else {
                    *gen_w
                };
            }
            ComponentParams::Avionics { power_demand_w }
            | ComponentParams::Comms { power_demand_w }
            | ComponentParams::Payload { power_demand_w, .. }
            | ComponentParams::LifeSupport { power_demand_w, .. } => demand += power_demand_w,
            ComponentParams::Engine { .. } => demand += propulsion::engine_power_demand(c),
            _ => {}
        }
    }
    PowerBalance {
        generation_w: generation,
        demand_w: demand,
        margin_w: generation - demand,
    }
}
