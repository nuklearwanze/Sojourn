//! Breakthroughs (FR-RESP-302, R7): hidden **insight pressure** accrues per
//! `(faction, domain)`, weighted toward **basic-science** investment; crossing a
//! seeded threshold triggers a rare breakthrough (announced with a sourced
//! reference). Leapfrogging (FR-RESP-303) is handled at program-start by treating
//! `UlSatisfiable` prereqs as satisfiable via UL (see `module`).

use crate::ids::{DomainId, FactionId};
use crate::seeding::Thresholds;
use crate::tree::{Params, ResearchData};
use std::collections::BTreeMap;

/// Per-`(faction, domain)` accrued insight pressure.
pub type InsightMap = BTreeMap<(FactionId, DomainId), f64>;

/// Accrue insight for a faction's domain from `rp` this step. Applied domains
/// accrue little — breakthroughs are basic-science-weighted (R7).
pub fn accrue(
    insight: &mut InsightMap,
    data: &ResearchData,
    params: &Params,
    f: FactionId,
    dom: &DomainId,
    rp: f64,
    breakthrough_mult: f64,
) {
    let Some(d) = data.domains.get(dom) else { return };
    let weight = if d.basic_science { 1.0 } else { 0.1 };
    let cur = insight.get(&(f, dom.clone())).copied().unwrap_or(0.0);
    insight.insert(
        (f, dom.clone()),
        cur + rp * params.insight_per_basic_rp * weight * breakthrough_mult,
    );
}

/// If the faction's domain insight has crossed its seeded threshold, consume it and
/// signal a breakthrough.
pub fn check_trigger(
    insight: &mut InsightMap,
    thresholds: &Thresholds,
    f: FactionId,
    dom: &DomainId,
) -> bool {
    let cur = insight.get(&(f, dom.clone())).copied().unwrap_or(0.0);
    let thr = thresholds.get(dom).copied().unwrap_or(f64::INFINITY);
    if cur >= thr {
        insight.insert((f, dom.clone()), 0.0); // consumed
        true
    } else {
        false
    }
}
