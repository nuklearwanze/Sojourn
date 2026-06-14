//! Power balance (R5, FR-BC-103): Σ generation (PV solar-distance-scaled) − Σ
//! demand over **commissioned** modules. A negative margin is a red-flag.

use crate::base::Base;
use crate::catalogue::{Catalogue, ModuleParams};
use crate::inputs::SiteFacts;
use crate::trace::TraceTree;
use serde::{Deserialize, Serialize};

/// A base's power balance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PowerBalance {
    /// Total generation (W).
    pub generation_w: f64,
    /// Total demand (W).
    pub demand_w: f64,
    /// Margin (W).
    pub margin_w: f64,
}

/// The PV solar-distance scaling factor `(ref / distance)²`.
fn pv_scale(cat: &Catalogue, site: Option<&SiteFacts>) -> f64 {
    let d = site
        .map(|s| s.solar_distance_m)
        .unwrap_or(cat.params.solar_ref_m);
    (cat.params.solar_ref_m / d.max(1.0)).powi(2)
}

/// Compute the power balance for a base at its site.
pub fn balance(base: &Base, cat: &Catalogue, site: Option<&SiteFacts>) -> PowerBalance {
    let scale = pv_scale(cat, site);
    let mut generation_w = 0.0;
    let mut demand_w = 0.0;
    for m in base.modules.values().filter(|m| m.commissioned) {
        let Some(t) = cat.module(&m.type_id) else {
            continue;
        };
        demand_w += t.power_demand_w;
        if let ModuleParams::Power { gen_w, solar } = &t.params {
            generation_w += if *solar { gen_w * scale } else { *gen_w };
        }
    }
    PowerBalance {
        generation_w,
        demand_w,
        margin_w: generation_w - demand_w,
    }
}

/// Traceability tree for the power margin.
pub fn trace(base: &Base, cat: &Catalogue, site: Option<&SiteFacts>) -> TraceTree {
    let b = balance(base, cat, site);
    let mut inputs = Vec::new();
    let scale = pv_scale(cat, site);
    for m in base.modules.values().filter(|m| m.commissioned) {
        if let Some(t) = cat.module(&m.type_id) {
            if let ModuleParams::Power { gen_w, solar } = &t.params {
                let g = if *solar { gen_w * scale } else { *gen_w };
                inputs.push(TraceTree::leaf(
                    format!("gen:{}", t.id.0),
                    g,
                    t.source.clone(),
                ));
            }
            if t.power_demand_w > 0.0 {
                inputs.push(TraceTree::leaf(
                    format!("demand:{}", t.id.0),
                    -t.power_demand_w,
                    t.source.clone(),
                ));
            }
        }
    }
    TraceTree::node("power-margin = Σ generation − Σ demand", b.margin_w, inputs)
}
