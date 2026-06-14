//! Consumables vs ECLSS closure (R5, FR-LSC-101…105, A1). ECLSS closure recycles
//! the **air/water loop only**; **food is open-loop**. A robotic asset consumes
//! nothing.

use crate::asset::CrewedAsset;
use crate::params::ConsumablesParams;

/// Gross daily consumption (kg) for a crew of `n`: (air/water, food).
pub fn gross_per_day(n: u32, p: &ConsumablesParams) -> (f64, f64) {
    let crew = f64::from(n);
    (
        p.air_water_per_crew_day() * crew,
        p.food_kg_per_crew_day * crew,
    )
}

/// Total daily consumption (kg) drawn from the stock (a robotic asset draws 0).
pub fn daily_consumption(asset: &CrewedAsset, p: &ConsumablesParams) -> f64 {
    if !asset.sizing.crewed {
        return 0.0;
    }
    let (air_water, food) = gross_per_day(asset.crew.len() as u32, p);
    air_water + food
}

/// The resupply make-up rate (kg/day): air/water recycled by closure, food open-loop (A1).
pub fn makeup_rate(asset: &CrewedAsset, p: &ConsumablesParams) -> f64 {
    if !asset.sizing.crewed {
        return 0.0;
    }
    let (air_water, food) = gross_per_day(asset.crew.len() as u32, p);
    air_water * (1.0 - asset.sizing.closure_capability).max(0.0) + food
}

/// Days of consumables coverage at the current stock (∞ for a robotic asset).
pub fn coverage_days(asset: &CrewedAsset, p: &ConsumablesParams) -> f64 {
    let consumption = daily_consumption(asset, p);
    if consumption <= 1e-12 {
        f64::INFINITY
    } else {
        asset.consumables_kg / consumption
    }
}
