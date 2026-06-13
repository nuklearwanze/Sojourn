//! EDL/landing suitability checks (FR-VD-401, R10, clarified scope Q1:A): design-
//! time "can it land here?" checks (T/W vs local gravity, heat-shield presence,
//! ballistic coefficient) as red-flags. The EDL flight phase is FA-02/later.

use crate::catalogue::{Catalogue, ComponentParams};
use crate::deltav;
use crate::design::VehicleDesign;
use serde::{Deserialize, Serialize};

/// EDL/landing suitability for a target body.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdlSuitability {
    /// T/W at the target-body surface gravity.
    pub landing_tw: f64,
    /// Can it land propulsively (T/W ≥ 1)?
    pub can_land: bool,
    /// Carries a heat shield (for atmospheric entry).
    pub has_heat_shield: bool,
}

/// Suitability at a target-body surface gravity `target_g` (m/s²).
pub fn suitability(
    design: &VehicleDesign,
    cat: &Catalogue,
    target_g: f64,
    available_power_w: f64,
) -> EdlSuitability {
    let tw = deltav::thrust_to_weight(design, cat, available_power_w, target_g.max(1e-9));
    let has_heat_shield = design.all_components().any(|id| {
        cat.component(id)
            .is_some_and(|c| matches!(c.params, ComponentParams::EdlKit { .. }))
    });
    EdlSuitability {
        landing_tw: tw,
        can_land: tw >= 1.0,
        has_heat_shield,
    }
}
