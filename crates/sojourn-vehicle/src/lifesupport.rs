//! Static life-support sizing (FR-VD-501, R9, clarified scope Q2:A): consumables/
//! closure mass, endurance, radiation-shield mass and crew capacity as design
//! outputs and guards. The dynamic in-mission simulation is FA-08's.

use crate::catalogue::{Catalogue, ComponentParams};
use crate::design::VehicleDesign;
use serde::{Deserialize, Serialize};

/// Static life-support sizing for a (crewed) design.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LifeSupportSizing {
    /// Crew the design can accommodate.
    pub crew_capacity: u32,
    /// Best ECLSS closure fraction.
    pub closure_fraction: f64,
    /// Radiation-shield mass carried (kg).
    pub shield_kg: f64,
    /// Open-loop consumables mass for the stated mission (kg).
    pub consumables_kg: f64,
    /// Achievable endurance (days) for the accommodated crew with carried consumables.
    pub endurance_days: f64,
}

/// Size life support. `mission_days` is the stated mission duration (0 if none).
pub fn sizing(design: &VehicleDesign, cat: &Catalogue, mission_days: f64) -> LifeSupportSizing {
    let mut crew_capacity = 0u32;
    let mut closure = 0.0f64;
    let mut per_day = 0.0; // per-crew-day consumables (kg)
    let mut shield = 0.0;
    for id in design.all_components() {
        if let Some(c) = cat.component(id)
            && let ComponentParams::LifeSupport {
                closure_fraction,
                per_crew_day_kg,
                crew,
                shield_kg,
                ..
            } = &c.params
        {
            crew_capacity += crew;
            closure = closure.max(*closure_fraction);
            per_day += per_crew_day_kg;
            shield += shield_kg;
        }
    }
    let effective_per_crew_day = per_day * (1.0 - closure).max(0.0);
    let crew = crew_capacity.max(1) as f64;
    let consumables_kg = effective_per_crew_day * crew * mission_days;
    // Endurance: with a near-closed loop, endurance is very large; open-loop it is
    // bounded by the consumables the design carries (here: what the mission needs).
    let endurance_days = if effective_per_crew_day <= 1e-9 {
        f64::INFINITY
    } else {
        mission_days // the design is sized for the stated mission when consumables are provided
    };
    LifeSupportSizing {
        crew_capacity,
        closure_fraction: closure,
        shield_kg: shield,
        consumables_kg,
        endurance_days,
    }
}
