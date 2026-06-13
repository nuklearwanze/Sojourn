//! ISRU economics (data-model §7, R10, FR-EC-501…505). A plant converts a local
//! resource into a useful commodity at a yield governed by sourced process
//! params, the **surveyed belief-state grade** (seeded uncertainty), power and
//! plant mass. Break-even = launch-cost-saved − plant cost: ISRU pays off only
//! when the physics and economics close — **never free fuel**. Scale-up: a
//! pilot→production ramp so first-of-a-kind plants under-perform.

use crate::ids::{CommodityId, FactionId, PlantId};
use crate::inputs::GradeBelief;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One step of the scale-up ramp.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleStep {
    /// Yield multiplier at this level (≤ 1 early, → 1 at production).
    pub yield_mult: f64,
    /// Reliability at this level ∈ [0,1].
    pub reliability: f64,
}

/// An ISRU process (DATA, `isru.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IsruProcess {
    /// Stable id.
    pub id: String,
    /// Input raw commodity.
    pub input: CommodityId,
    /// Output processed commodity.
    pub output: CommodityId,
    /// Nameplate yield (kg/day at grade 1, production scale).
    pub base_yield_per_day: f64,
    /// Electrical power demand (W).
    pub power_demand_w: f64,
    /// Plant mass to deliver (kg).
    pub plant_mass_kg: f64,
    /// Operating cost (Funds/day).
    pub operate_cost_per_day: f64,
    /// Days spent at each scale level before advancing.
    pub ramp_days_per_level: u64,
    /// The pilot→production ramp.
    pub scaleup_curve: Vec<ScaleStep>,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct IsruFile {
    processes: Vec<IsruProcess>,
}

/// The loaded, validated ISRU processes (module data).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IsruDefs {
    /// Processes by id.
    pub by_id: BTreeMap<String, IsruProcess>,
    /// Canonical source text (for the econ content hash).
    pub canonical: String,
}

impl IsruDefs {
    /// Parse + validate `isru.ron`.
    pub fn from_source(text: &str) -> Result<IsruDefs, String> {
        let f: IsruFile = ron::from_str(text).map_err(|e| format!("isru.ron: {e}"))?;
        let mut by_id = BTreeMap::new();
        for p in f.processes {
            if p.source.trim().is_empty() {
                return Err(format!("isru process '{}' has empty source", p.id));
            }
            if p.base_yield_per_day < 0.0 || p.plant_mass_kg < 0.0 || p.power_demand_w < 0.0 {
                return Err(format!("isru process '{}': negative param", p.id));
            }
            if p.scaleup_curve.is_empty() {
                return Err(format!("isru process '{}': empty scaleup curve", p.id));
            }
            if by_id.insert(p.id.clone(), p).is_some() {
                return Err("duplicate isru process".to_string());
            }
        }
        Ok(IsruDefs {
            by_id,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// A process by id.
    pub fn get(&self, id: &str) -> Option<&IsruProcess> {
        self.by_id.get(id)
    }
}

/// An ISRU plant (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsruPlant {
    /// Identity.
    pub id: PlantId,
    /// Owner.
    pub faction: FactionId,
    /// The node it sits at.
    pub node: String,
    /// The process id it runs.
    pub process: String,
    /// Current scale level (0 = pilot).
    pub scale_level: usize,
    /// Days operated at the current level.
    pub days_at_level: u64,
    /// Tick it reached production scale (0 if not yet).
    pub online_tick: u64,
    /// Cumulative output (kg).
    pub cumulative_output: f64,
}

/// The grade factor from a belief and a seeded uniform draw `u ∈ [0,1)`: the
/// believed grade perturbed within ±(1−certainty) (FR-EC-505).
pub fn grade_factor(grade: &GradeBelief, u: f64) -> f64 {
    let spread = (1.0 - grade.certainty.clamp(0.0, 1.0)) * (2.0 * u.clamp(0.0, 1.0) - 1.0);
    (grade.believed_grade * (1.0 + spread)).max(0.0)
}

/// Daily yield for a plant given its scale level, the grade factor and a power
/// factor ∈ [0,1] (available/demanded power).
pub fn daily_yield(
    process: &IsruProcess,
    scale_level: usize,
    grade_factor: f64,
    power_factor: f64,
) -> f64 {
    let step = process
        .scaleup_curve
        .get(scale_level.min(process.scaleup_curve.len() - 1))
        .copied()
        .unwrap_or(ScaleStep {
            yield_mult: 1.0,
            reliability: 1.0,
        });
    process.base_yield_per_day
        * step.yield_mult
        * step.reliability
        * grade_factor
        * power_factor.clamp(0.0, 1.0)
}

/// The break-even outcome over an operating horizon.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BreakEven {
    /// Net Funds (saved − cost) over the horizon.
    pub net: f64,
    /// Launch cost saved by producing locally.
    pub saved: f64,
    /// Total plant cost (delivery + operate over the horizon).
    pub plant_cost: f64,
    /// Whether the plant pays off.
    pub positive: bool,
}

/// Compute break-even over `days` of operation: `net = days·(yield·price_saved −
/// operate) − plant_mass·delivery_price`. Negative below break-even, positive
/// above — never free fuel (FR-EC-502).
pub fn break_even(
    process: &IsruProcess,
    yield_per_day: f64,
    price_per_kg_saved: f64,
    delivery_price_per_kg: f64,
    days: f64,
) -> BreakEven {
    let saved = yield_per_day * price_per_kg_saved * days;
    let operate = process.operate_cost_per_day * days;
    let delivery = process.plant_mass_kg * delivery_price_per_kg;
    let plant_cost = operate + delivery;
    let net = saved - plant_cost;
    BreakEven {
        net,
        saved,
        plant_cost,
        positive: net > 0.0,
    }
}
