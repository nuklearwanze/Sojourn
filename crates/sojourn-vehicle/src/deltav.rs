//! Δv and thrust/T-W (FR-VD-201, R6): the rocket equation per stage/mode (never
//! blended), thrust and T/W against supplied gravity. With traceability.

use crate::catalogue::{Catalogue, ComponentParams};
use crate::design::VehicleDesign;
use crate::mass;
use crate::propulsion;
use crate::trace::TraceTree;
use sojourn_astro::propulsion::G0;

/// Per-stage Δv (m/s). Staging: stage `i`'s initial mass is the dry mass of stages
/// `i..` plus the propellant of stages `i..`; final mass drops this stage's
/// propellant. Modes are never blended into one fictitious Isp (edge case).
pub fn stage_dvs(design: &VehicleDesign, cat: &Catalogue, available_power_w: f64) -> Vec<f64> {
    let n = design.stages.len();
    // Dry mass per stage.
    let stage_dry: Vec<f64> = design
        .stages
        .iter()
        .map(|s| {
            s.components
                .iter()
                .filter_map(|id| cat.component(id))
                .map(mass::component_dry_mass)
                .sum::<f64>()
        })
        .collect();
    let mut dvs = Vec::with_capacity(n);
    for i in 0..n {
        let above_dry: f64 = stage_dry[i..].iter().sum();
        let above_prop: f64 = design.stages[i..].iter().map(|s| s.propellant_kg).sum();
        let m0 = above_dry + above_prop;
        let m1 = m0 - design.stages[i].propellant_kg;
        let ve = design.stages[i]
            .engine
            .as_ref()
            .and_then(|id| cat.component(id))
            .and_then(|c| match &c.params {
                ComponentParams::Engine { isp_s, .. } => Some(isp_s * G0),
                _ => None,
            })
            .unwrap_or(0.0);
        let dv = if ve > 0.0 && m1 > 0.0 && m0 > m1 {
            ve * libm::log(m0 / m1)
        } else {
            0.0
        };
        let _ = available_power_w;
        dvs.push(dv);
    }
    dvs
}

/// Total Δv (sum of stage Δvs — they fire in sequence).
pub fn total_dv(design: &VehicleDesign, cat: &Catalogue, available_power_w: f64) -> f64 {
    stage_dvs(design, cat, available_power_w).iter().sum()
}

/// Thrust (N) of the first active stage's engine at full throttle (power-limited
/// for EP), for T/W.
pub fn first_stage_thrust(design: &VehicleDesign, cat: &Catalogue, available_power_w: f64) -> f64 {
    design
        .stages
        .iter()
        .find_map(|s| s.engine.as_ref())
        .and_then(|id| cat.component(id))
        .map(|c| propulsion::delivered_max_thrust(c, available_power_w))
        .unwrap_or(0.0)
}

/// Thrust-to-weight against a supplied surface gravity `g` (m/s²).
pub fn thrust_to_weight(
    design: &VehicleDesign,
    cat: &Catalogue,
    available_power_w: f64,
    g: f64,
) -> f64 {
    let w = mass::wet_mass(design, cat) * g;
    if w <= 0.0 {
        0.0
    } else {
        first_stage_thrust(design, cat, available_power_w) / w
    }
}

/// Traceability for a stage's Δv.
pub fn trace_stage_dv(design: &VehicleDesign, cat: &Catalogue, stage: usize) -> TraceTree {
    let dvs = stage_dvs(design, cat, f64::INFINITY);
    let value = dvs.get(stage).copied().unwrap_or(0.0);
    let mut inputs = Vec::new();
    if let Some(c) = design
        .stages
        .get(stage)
        .and_then(|s| s.engine.as_ref())
        .and_then(|id| cat.component(id))
        && let ComponentParams::Engine { isp_s, .. } = &c.params
    {
        inputs.push(TraceTree::leaf(
            format!("{}.isp_s", c.id.0),
            *isp_s,
            c.source.clone(),
        ));
        inputs.push(TraceTree::leaf("g0", G0, "CODATA standard gravity"));
    }
    inputs.push(TraceTree::leaf(
        format!("stage{stage}.propellant_kg"),
        design
            .stages
            .get(stage)
            .map(|s| s.propellant_kg)
            .unwrap_or(0.0),
        "design composition",
    ));
    TraceTree::node("rocket-equation (v_e·ln(m0/m1))", value, inputs)
}
