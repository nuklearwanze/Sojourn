//! Track A — Understanding Levels (FR-RESP-101/102, R3): continuous per-`(faction,
//! domain)` UL with diminishing returns + cross-domain synergy, the catch-up
//! discount, and UL gating read through an `effective_ul()` indirection (identity
//! until personnel tacit-knowledge supplies a penalty — U1 remediation).

use crate::ids::{DomainId, FactionId};
use crate::tree::{DomainDef, Params, ResearchData, TechNode};
use std::collections::BTreeMap;

/// Per-`(faction, domain)` ground Understanding Level.
pub type UlMap = BTreeMap<(FactionId, DomainId), f64>;
/// Per-domain World Understanding Level (the tide).
pub type WorldUlMap = BTreeMap<DomainId, f64>;

/// Ground UL (0 if unseen).
pub fn ground_ul(ul: &UlMap, f: FactionId, d: &DomainId) -> f64 {
    ul.get(&(f, d.clone())).copied().unwrap_or(0.0)
}

/// World UL for a domain (0 if unseen).
pub fn world_ul(w: &WorldUlMap, d: &DomainId) -> f64 {
    w.get(d).copied().unwrap_or(0.0)
}

/// Effective UL used by gates and queries = ground UL minus a tacit-knowledge
/// penalty (0 until personnel modelling supplies one). Never negative.
pub fn effective_ul(ul: &UlMap, f: FactionId, d: &DomainId, tacit_penalty: f64) -> f64 {
    (ground_ul(ul, f, d) - tacit_penalty).max(0.0)
}

/// Diminishing-returns factor: 1 below the knee, falling toward 0 toward UL 100.
pub fn dr_factor(ul: f64, knee: f64, steepness: f64) -> f64 {
    if ul <= knee {
        1.0
    } else {
        let over = ((ul - knee) / (100.0 - knee).max(1.0)).clamp(0.0, 1.0);
        libm::pow((1.0 - over).max(0.0), steepness)
    }
}

/// Synergy multiplier from coupled domains' ULs (≥ 1).
pub fn synergy_bonus(ul: &UlMap, f: FactionId, d: &DomainDef) -> f64 {
    let mut b = 1.0;
    for (s, w) in &d.synergy {
        b += w * ground_ul(ul, f, &s.clone()) / 100.0;
    }
    b
}

/// Grow a faction's UL in a domain from `rp` research points over the step.
/// Diminishing returns × synergy × catch-up (trailing the World UL is cheaper).
pub fn grow_domain(
    ul: &mut UlMap,
    data: &ResearchData,
    params: &Params,
    f: FactionId,
    dom: &DomainId,
    rp: f64,
    world: f64,
) {
    let Some(d) = data.domains.get(dom) else { return };
    let cur = ground_ul(ul, f, dom);
    let dr = dr_factor(cur, d.dr_knee, d.dr_steepness);
    let syn = synergy_bonus(ul, f, d);
    let catchup = 1.0 + params.catchup_per_level * (world - cur).max(0.0);
    let delta = rp * params.ul_per_rp * dr * syn * catchup;
    // Catch-up cannot push a faction past the World UL by itself (edge case).
    let cap = if cur < world { (cur + delta).min(world.max(cur)) } else { cur + delta };
    ul.insert((f, dom.clone()), cap.min(100.0));
}

/// Inject UL directly (a mission, FR-RESP-104) — bypasses the cost curve.
pub fn inject(ul: &mut UlMap, f: FactionId, dom: &DomainId, amount: f64) {
    let cur = ground_ul(ul, f, dom);
    ul.insert((f, dom.clone()), (cur + amount).min(100.0));
}

/// The first unmet UL floor of a node for a faction (gating, FR-RESP-102), using
/// effective UL. `None` ⇒ all floors met.
pub fn first_unmet_floor(
    ul: &UlMap,
    f: FactionId,
    node: &TechNode,
    tacit: &impl Fn(FactionId, &DomainId) -> f64,
) -> Option<(DomainId, f64)> {
    for (dom, floor) in &node.ul_floors {
        if effective_ul(ul, f, dom, tacit(f, dom)) < *floor {
            return Some((dom.clone(), *floor));
        }
    }
    None
}
