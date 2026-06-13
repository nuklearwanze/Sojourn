//! Divertible small bodies (FR-ASTRO-107, clarified 2026-06-13): the
//! Railed → Diverted → ReRailed lifecycle. Transitions are continuity-exact at
//! the transition tick, journaled (they are command consequences), and bounded
//! by the configured diversion budget.

use super::{BodyId, BodyMotion, Catalog, rails::BodyStates};
use crate::math::{StateVec, Vec3, state_to_elements};
use crate::soi::dominant_of;
use std::collections::BTreeMap;

/// Take a divertible body off its rail with a velocity impulse. Returns the new
/// motion entry, or a rejection reason. Continuity: the diverted state equals
/// the railed state at this exact time, plus the impulse on velocity only.
pub fn divert(
    catalog: &Catalog,
    motions: &BTreeMap<BodyId, BodyMotion>,
    budget: u32,
    id: BodyId,
    impulse: Vec3,
    t_ns: i64,
) -> Result<BodyMotion, String> {
    let body = catalog.body(id).ok_or("unknown body")?;
    if !body.divertible {
        return Err(format!("body '{}' is not divertible", body.name_id));
    }
    if matches!(motions.get(&id), Some(BodyMotion::Diverted { .. })) {
        return Err(format!("body '{}' is already diverted", body.name_id));
    }
    let diverted_now = motions
        .values()
        .filter(|m| matches!(m, BodyMotion::Diverted { .. }))
        .count() as u32;
    if diverted_now >= budget {
        return Err(format!("diversion budget ({budget}) exhausted"));
    }
    if impulse.non_finite() {
        return Err("impulse components must be finite".into());
    }
    let mut states = BodyStates::new(catalog, motions, t_ns);
    let abs = states.absolute(id);
    let pushed = StateVec {
        r: abs.r,
        v: abs.v + impulse,
    };
    // Dominant for the new path (exclude self-dominance for tiny bodies: the
    // body cannot be its own SOI owner).
    let mut dom = dominant_of(catalog, &mut states, pushed.r);
    if dom == id {
        dom = body.parent.unwrap_or(catalog.root());
    }
    let dom_abs = states.absolute(dom);
    Ok(BodyMotion::Diverted {
        dominant: dom,
        state: StateVec {
            r: pushed.r - dom_abs.r,
            v: pushed.v - dom_abs.v,
        },
    })
}

/// Re-rail a diverted body: convert its current path to elements about its
/// original parent. Continuity: the railed state at this time equals the
/// diverted state exactly (same conversion both ways).
pub fn re_rail(
    catalog: &Catalog,
    motions: &BTreeMap<BodyId, BodyMotion>,
    id: BodyId,
    t_ns: i64,
) -> Result<BodyMotion, String> {
    let body = catalog.body(id).ok_or("unknown body")?;
    if !matches!(motions.get(&id), Some(BodyMotion::Diverted { .. })) {
        return Err(format!("body '{}' is not diverted", body.name_id));
    }
    let parent = body.parent.ok_or("root cannot re-rail")?;
    let parent_mu = catalog.body(parent).ok_or("parent")?.mu;
    let mut states = BodyStates::new(catalog, motions, t_ns);
    let abs = states.absolute(id);
    let p_abs = states.absolute(parent);
    let rel = StateVec {
        r: abs.r - p_abs.r,
        v: abs.v - p_abs.v,
    };
    let el = state_to_elements(parent_mu, &rel, t_ns);
    if el.ecc >= 1.0 || !el.sma.is_finite() || el.sma <= 0.0 {
        return Err(format!(
            "body '{}' is not on a bound elliptic orbit about its parent (e={:.3}); \
             cannot re-rail",
            body.name_id, el.ecc
        ));
    }
    Ok(BodyMotion::ReRailed { elements: el })
}
