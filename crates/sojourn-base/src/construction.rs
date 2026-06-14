//! Construction project: per-module delivered-mass + crew-time demands, and the
//! delivery-driven commissioning bookkeeping (R8, data-model §5). FA-06's `Project`
//! is the *delivery* accounting; this is the *assembly* accounting (C1).

use crate::base::Base;
use crate::catalogue::{Catalogue, ModuleType};
use crate::ids::{BaseId, ModuleId, ProjectId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The delivered-mass + crew-time a module needs to commission.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModuleDemand {
    /// Required delivered mass (kg) = the module's dry mass.
    pub required_mass_kg: f64,
    /// Required crew-time (hours) = mass × `crew_hr_per_kg`.
    pub required_crew_time_hr: f64,
}

/// A construction project (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionProject {
    /// Identity.
    pub id: ProjectId,
    /// The base being built.
    pub base: BaseId,
    /// Per-module demands.
    pub demands: BTreeMap<ModuleId, ModuleDemand>,
    /// Tick the project opened.
    pub started_tick: u64,
}

/// Derived construction progress.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConstructionProgress {
    /// Total modules.
    pub module_count: usize,
    /// Commissioned modules.
    pub commissioned_count: usize,
    /// Remaining imported mass (kg).
    pub remaining_mass_kg: f64,
    /// Remaining crew-time (hours).
    pub remaining_crew_hr: f64,
    /// Fraction complete ∈ [0,1].
    pub pct_complete: f64,
}

/// The demand for a module type (mass = dry mass; crew-time = mass × crew_hr_per_kg).
pub fn demand_for(t: &ModuleType, cat: &Catalogue) -> ModuleDemand {
    ModuleDemand {
        required_mass_kg: t.dry_mass_kg,
        required_crew_time_hr: t.dry_mass_kg * cat.params.crew_hr_per_kg,
    }
}

/// Compute progress for a base's project.
pub fn progress(base: &Base, project: &ConstructionProject) -> ConstructionProgress {
    let module_count = base.modules.len();
    let commissioned_count = base.modules.values().filter(|m| m.commissioned).count();
    let mut remaining_mass = 0.0;
    let mut remaining_crew = 0.0;
    for (id, demand) in &project.demands {
        if let Some(m) = base.modules.get(id)
            && !m.commissioned
        {
            // On-site-built mass does not count against the imported-mass remainder.
            if !m.built_local {
                remaining_mass += (demand.required_mass_kg - m.delivered_mass_kg).max(0.0);
            }
            remaining_crew += (demand.required_crew_time_hr - m.crew_time_hr).max(0.0);
        }
    }
    let pct = if module_count == 0 {
        0.0
    } else {
        commissioned_count as f64 / module_count as f64
    };
    ConstructionProgress {
        module_count,
        commissioned_count,
        remaining_mass_kg: remaining_mass,
        remaining_crew_hr: remaining_crew,
        pct_complete: pct,
    }
}
