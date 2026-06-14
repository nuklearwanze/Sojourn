//! The composition root (R1, FR-UI-1502). The UI is the one place that links every
//! slice to assemble the full nine-module `SimCore` — the same module set the harness
//! builds. The slices never depend on the UI, so this one-way coupling keeps the core
//! headless and the audit clean.

use sojourn_core::SimModule;
use std::path::Path;

/// Build the full gameplay module set, loading each slice's data directory under
/// `data_root` (e.g. the repo root). Returns a human error if any slice's data fails
/// to load.
pub fn build_modules(data_root: &Path) -> Result<Vec<Box<dyn SimModule>>, String> {
    let mut mods: Vec<Box<dyn SimModule>> = Vec::new();
    let d = |p: &str| data_root.join(p);

    mods.push(Box::new(
        sojourn_astro::AstroModule::load(&d("data/world")).map_err(|e| format!("astro: {e}"))?,
    ));
    mods.push(Box::new(
        sojourn_world::WorldModule::load(&d("data/world")).map_err(|e| format!("world: {e}"))?,
    ));
    mods.push(Box::new(
        sojourn_research::ResearchModule::load(&d("data")).map_err(|e| format!("research: {e}"))?,
    ));
    mods.push(Box::new(
        sojourn_vehicle::VehicleModule::load(&d("data/vehicle"))
            .map_err(|e| format!("vehicle: {e}"))?,
    ));
    mods.push(Box::new(
        sojourn_economy::EconomyModule::load(&d("data/econ"))
            .map_err(|e| format!("economy: {e}"))?,
    ));
    mods.push(Box::new(
        sojourn_base::BaseModule::load(&d("data/base")).map_err(|e| format!("base: {e}"))?,
    ));
    mods.push(Box::new(
        sojourn_crew::CrewModule::load(&d("data/crew")).map_err(|e| format!("crew: {e}"))?,
    ));
    mods.push(Box::new(
        sojourn_polity::PolityModule::load(&d("data/polity"))
            .map_err(|e| format!("polity: {e}"))?,
    ));
    Ok(mods)
}
