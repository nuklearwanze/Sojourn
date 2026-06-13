//! Mass model (FR-VD-201): dry/wet mass and mass fractions, with traceability.
//! Reactor + shielding mass on nuclear engines is first-class dry mass; radiators
//! are explicit Thermal components (their mass is already counted).

use crate::catalogue::{Catalogue, Component, ComponentParams};
use crate::design::VehicleDesign;
use crate::trace::TraceTree;

/// Dry mass of one component, incl. derived reactor/shielding mass on engines.
pub fn component_dry_mass(c: &Component) -> f64 {
    let mut m = c.dry_mass_kg;
    if let ComponentParams::Engine {
        reactor_mass_kg, ..
    } = &c.params
    {
        m += reactor_mass_kg;
    }
    m
}

/// Total dry mass of a design.
pub fn dry_mass(design: &VehicleDesign, cat: &Catalogue) -> f64 {
    design
        .all_components()
        .filter_map(|id| cat.component(id))
        .map(component_dry_mass)
        .sum()
}

/// Total propellant (kg).
pub fn propellant(design: &VehicleDesign) -> f64 {
    design.stages.iter().map(|s| s.propellant_kg).sum()
}

/// Wet (total) mass.
pub fn wet_mass(design: &VehicleDesign, cat: &Catalogue) -> f64 {
    dry_mass(design, cat) + propellant(design)
}

/// Traceability tree for dry mass (each component a sourced leaf).
pub fn trace_dry_mass(design: &VehicleDesign, cat: &Catalogue) -> TraceTree {
    let mut leaves = Vec::new();
    for id in design.all_components() {
        if let Some(c) = cat.component(id) {
            leaves.push(TraceTree::leaf(
                format!("{}.dry_mass_kg", c.id.0),
                component_dry_mass(c),
                c.source.clone(),
            ));
        }
    }
    let total = leaves.iter().map(TraceTree::value).sum();
    TraceTree::node("sum(component dry mass)", total, leaves)
}
