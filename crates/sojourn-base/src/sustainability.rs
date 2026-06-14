//! Self-sufficiency & embargo (R11/R12, FR-BC-501/502/503). The self-sufficiency
//! index is the **limiting factor (minimum)** over per-loop closure ratios; the
//! embargo stress test is an **analytic rate-plus-buffer** check (no time-stepping
//! — the dynamic sim is Slice 8).

use crate::base::Base;
use crate::catalogue::{Catalogue, ModuleParams};
use crate::lifesupport::LifeSupport;
use crate::power::PowerBalance;
use crate::production::LocalProduction;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One closure loop's supply/demand/buffer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopState {
    /// Loop id ("air-water" | "food" | "power" | "materials" | "spares").
    pub id: String,
    /// Local supply rate (kg/day or W for power).
    pub supply_rate: f64,
    /// Demand rate.
    pub demand_rate: f64,
    /// Stored buffer (kg; 0 for power).
    pub buffer: f64,
}

impl LoopState {
    /// Closure ratio = supply/demand, capped at 1 (1 when there is no demand).
    pub fn ratio(&self) -> f64 {
        if self.demand_rate <= 1e-12 {
            1.0
        } else {
            (self.supply_rate / self.demand_rate).min(1.0)
        }
    }

    /// Survives an embargo of `days` iff supply ≥ demand or the buffer bridges the deficit.
    pub fn survives(&self, days: f64) -> bool {
        if self.supply_rate + 1e-9 >= self.demand_rate {
            return true;
        }
        // Power has no multi-year buffer (buffer = 0) → fails if under-supplied.
        self.buffer + 1e-9 >= (self.demand_rate - self.supply_rate) * days
    }
}

/// The self-sufficiency index + binding loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelfSufficiency {
    /// Limiting-factor index ∈ [0,1].
    pub index: f64,
    /// The binding (weakest) loop.
    pub binding_loop: String,
    /// Per-loop ratios.
    pub ratios: Vec<(String, f64)>,
}

/// The embargo stress-test result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbargoResult {
    /// Survives the full span?
    pub survives: bool,
    /// The embargo span (years).
    pub embargo_years: f64,
    /// Loops that fail (if any).
    pub failing_loops: Vec<String>,
}

/// Sum commissioned storage buffers by commodity.
fn buffers(base: &Base, cat: &Catalogue) -> BTreeMap<String, f64> {
    let mut b: BTreeMap<String, f64> = BTreeMap::new();
    for m in base.modules.values().filter(|m| m.commissioned) {
        if let Some(t) = cat.module(&m.type_id)
            && let ModuleParams::Storage {
                commodity,
                buffer_capacity_kg,
            } = &t.params
        {
            *b.entry(commodity.clone()).or_insert(0.0) += buffer_capacity_kg;
        }
    }
    b
}

/// Build the five closure loops from the base's derived state.
pub fn loops(
    base: &Base,
    cat: &Catalogue,
    ls: &LifeSupport,
    pb: &PowerBalance,
    lp: &LocalProduction,
) -> Vec<LoopState> {
    let pop = f64::from(ls.population_capacity);
    let d = &cat.params.loop_demands;
    let buf = buffers(base, cat);

    // Greenhouse food supply.
    let food_supply: f64 = base
        .modules
        .values()
        .filter(|m| m.commissioned)
        .filter_map(|m| cat.module(&m.type_id))
        .filter_map(|t| match &t.params {
            ModuleParams::Greenhouse {
                food_kg_per_day, ..
            } => Some(*food_kg_per_day),
            _ => None,
        })
        .sum();

    let materials_supply = lp
        .manufacturing_rates
        .get("materials")
        .copied()
        .unwrap_or(0.0)
        + lp.manufacturing_rates.get("metals").copied().unwrap_or(0.0);
    let spares_supply = lp.manufacturing_rates.get("spares").copied().unwrap_or(0.0);

    // air-water: gross demand = consumables make-up / (1 − closure); ratio = closure.
    let gross_water = if (1.0 - ls.closure_fraction) > 1e-9 {
        ls.consumables_per_day_kg / (1.0 - ls.closure_fraction)
    } else {
        ls.consumables_per_day_kg
    };
    vec![
        LoopState {
            id: "air-water".into(),
            supply_rate: ls.closure_fraction * gross_water,
            demand_rate: gross_water,
            buffer: buf.get("water").copied().unwrap_or(0.0)
                + buf.get("consumables").copied().unwrap_or(0.0),
        },
        LoopState {
            id: "food".into(),
            supply_rate: food_supply,
            demand_rate: d.food_kg_per_crew_day * pop,
            buffer: buf.get("food").copied().unwrap_or(0.0),
        },
        LoopState {
            id: "power".into(),
            supply_rate: pb.generation_w,
            demand_rate: pb.demand_w,
            buffer: 0.0,
        },
        LoopState {
            id: "materials".into(),
            supply_rate: materials_supply,
            demand_rate: d.materials_kg_per_day,
            buffer: buf.get("materials").copied().unwrap_or(0.0),
        },
        LoopState {
            id: "spares".into(),
            supply_rate: spares_supply,
            demand_rate: d.spares_kg_per_day,
            buffer: buf.get("spares").copied().unwrap_or(0.0),
        },
    ]
}

/// The limiting-factor self-sufficiency index (R11).
pub fn self_sufficiency(loops: &[LoopState]) -> SelfSufficiency {
    let ratios: Vec<(String, f64)> = loops.iter().map(|l| (l.id.clone(), l.ratio())).collect();
    let (binding_loop, index) = ratios
        .iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(id, r)| (id.clone(), *r))
        .unwrap_or_else(|| ("none".into(), 1.0));
    SelfSufficiency {
        index,
        binding_loop,
        ratios,
    }
}

/// The analytic embargo stress test (R12).
pub fn embargo(loops: &[LoopState], years: f64) -> EmbargoResult {
    let days = years * 365.25;
    let failing_loops: Vec<String> = loops
        .iter()
        .filter(|l| !l.survives(days))
        .map(|l| l.id.clone())
        .collect();
    EmbargoResult {
        survives: failing_loops.is_empty(),
        embargo_years: years,
        failing_loops,
    }
}
