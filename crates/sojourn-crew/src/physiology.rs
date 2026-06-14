//! Physiological deconditioning + countermeasures (R7, FR-LSC-301…303). Micro-g
//! deconditioning accrues; countermeasures (artificial gravity strongest) slow it;
//! the deconditioning capability factor falls with the indices.

use crate::asset::{CrewedAsset, Deconditioning};
use crate::params::PhysiologyParams;

/// Countermeasure effectiveness ∈ [0,1]: artificial gravity (spin-hab) is strongest,
/// else exercise (always applied).
pub fn countermeasure_eff(asset: &CrewedAsset, p: &PhysiologyParams) -> f64 {
    if asset.sizing.spin_gravity {
        p.artificial_g_eff
    } else {
        p.exercise_eff
    }
    .clamp(0.0, 1.0)
}

/// Accrue one day of deconditioning into a member's indices.
pub fn accrue(decon: &mut Deconditioning, eff: f64, p: &PhysiologyParams) {
    let m = (1.0 - eff).max(0.0);
    decon.bone = (decon.bone + p.bone_rate * m).clamp(0.0, 1.0);
    decon.muscle = (decon.muscle + p.muscle_rate * m).clamp(0.0, 1.0);
    decon.cardio = (decon.cardio + p.cardio_rate * m).clamp(0.0, 1.0);
    decon.vision = (decon.vision + p.vision_rate * m).clamp(0.0, 1.0);
}

/// The deconditioning capability factor ∈ [0,1] (1 = baseline).
pub fn capability_factor(decon: &Deconditioning) -> f64 {
    (1.0 - decon.mean()).clamp(0.0, 1.0)
}
