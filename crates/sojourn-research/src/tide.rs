//! The global science tide (FR-RESP-401/402, R9): a per-domain World UL advancing
//! from an exogenous baseline plus the aggregate of all factions' understanding.
//! Publishing accelerates it (and emits a prestige-eligible event); holding keeps a
//! lead. The catch-up discount lives in `domains::grow_domain`. **No money** here
//! (FA-06 settles licensing).

use crate::domains::{UlMap, WorldUlMap, ground_ul, world_ul};
use crate::ids::FactionId;
use crate::tree::{Params, ResearchData};

/// Advance the World UL for every domain over `dt_days`. `publishing(f, dom)` says
/// whether faction `f` publishes that domain (publishers add extra tide growth).
pub fn advance(
    world: &mut WorldUlMap,
    ul: &UlMap,
    data: &ResearchData,
    params: &Params,
    factions: &[FactionId],
    publishing: &impl Fn(FactionId, &crate::ids::DomainId) -> bool,
    dt_days: f64,
) {
    for dom in data.domains.keys() {
        let cur = world_ul(world, dom);
        // Aggregate: the leading faction's UL pulls the tide up (weighted), plus a
        // bonus from publishers.
        let mut lead = 0.0f64;
        let mut publish_bonus = 0.0;
        for &f in factions {
            let u = ground_ul(ul, f, dom);
            lead = lead.max(u);
            if publishing(f, dom) {
                publish_bonus += params.tide_baseline_per_day * dt_days; // publishers double-pump
            }
        }
        let baseline = params.tide_baseline_per_day * dt_days;
        let toward = (params.tide_aggregate_weight * lead - cur).max(0.0);
        let next = cur + baseline + publish_bonus + toward * 0.01 * dt_days.clamp(0.01, 1.0);
        world.insert(dom.clone(), next.min(100.0));
    }
}
