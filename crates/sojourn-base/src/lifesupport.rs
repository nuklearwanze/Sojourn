//! Static life-support derivation (R6, FR-BC-104). ECLSS closure = best-module;
//! population capacity = `min(Σ habitat accommodation, Σ ECLSS crew_support)`.
//! Power is a **separate viability flag** (not a per-crew divisor; U1). The
//! dynamic in-mission simulation is Slice 8.

use crate::base::Base;
use crate::catalogue::{Catalogue, ModuleParams};
use serde::{Deserialize, Serialize};

/// A base's static life-support sizing.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LifeSupport {
    /// Best ECLSS closure fraction ∈ [0,1].
    pub closure_fraction: f64,
    /// Population capacity (crew).
    pub population_capacity: u32,
    /// Open-loop consumables demand for the accommodated crew (kg/day).
    pub consumables_per_day_kg: f64,
}

/// Compute static life-support for a base (commissioned modules only).
pub fn compute(base: &Base, cat: &Catalogue) -> LifeSupport {
    let mut accommodation = 0u32;
    let mut eclss_support = 0u32;
    let mut best_closure = 0.0f64;
    let mut per_crew_day = 0.0f64;
    for m in base.modules.values().filter(|m| m.commissioned) {
        let Some(t) = cat.module(&m.type_id) else {
            continue;
        };
        match &t.params {
            ModuleParams::Habitat {
                crew_accommodation, ..
            } => accommodation += crew_accommodation,
            ModuleParams::Eclss {
                closure_fraction,
                per_crew_day_kg,
                crew_support,
            } => {
                eclss_support += crew_support;
                if *closure_fraction > best_closure {
                    best_closure = *closure_fraction;
                    per_crew_day = *per_crew_day_kg;
                }
            }
            _ => {}
        }
    }
    let population_capacity = accommodation.min(eclss_support);
    let consumables_per_day_kg =
        per_crew_day * f64::from(population_capacity) * (1.0 - best_closure).max(0.0);
    LifeSupport {
        closure_fraction: best_closure,
        population_capacity,
        consumables_per_day_kg,
    }
}
