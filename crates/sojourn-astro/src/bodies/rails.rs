//! Railed ephemerides (research R3): body states as pure functions of time,
//! derived from catalogue elements (or diversion overrides), computed on demand
//! and memoised per query context. Bodies are never stepped.

use super::{BodyId, BodyMotion, Catalog};
use crate::math::{StateVec, Vec3, elements_to_state};
use std::collections::BTreeMap;

/// Per-evaluation memo of absolute (root-centred inertial) body states.
pub struct BodyStates<'a> {
    catalog: &'a Catalog,
    motions: &'a BTreeMap<BodyId, BodyMotion>,
    t_ns: i64,
    memo: BTreeMap<BodyId, StateVec>,
}

impl<'a> BodyStates<'a> {
    /// New evaluation context at a fixed time.
    pub fn new(
        catalog: &'a Catalog,
        motions: &'a BTreeMap<BodyId, BodyMotion>,
        t_ns: i64,
    ) -> BodyStates<'a> {
        BodyStates {
            catalog,
            motions,
            t_ns,
            memo: BTreeMap::new(),
        }
    }

    /// The evaluation time.
    pub fn t_ns(&self) -> i64 {
        self.t_ns
    }

    /// Absolute (root-centred inertial) state of a body at this context's time.
    pub fn absolute(&mut self, id: BodyId) -> StateVec {
        if let Some(s) = self.memo.get(&id) {
            return *s;
        }
        let s = self.compute(id);
        self.memo.insert(id, s);
        s
    }

    fn compute(&mut self, id: BodyId) -> StateVec {
        let body = self.catalog.body(id).expect("known body");
        match self.motions.get(&id) {
            Some(BodyMotion::Diverted { dominant, state }) => {
                let dom = self.absolute(*dominant);
                StateVec {
                    r: dom.r + state.r,
                    v: dom.v + state.v,
                }
            }
            Some(BodyMotion::ReRailed { elements }) => {
                let parent = body.parent.expect("re-railed bodies have parents");
                let parent_mu = self.catalog.body(parent).expect("parent").mu;
                let rel = elements_to_state(parent_mu, elements, self.t_ns);
                let p = self.absolute(parent);
                StateVec {
                    r: p.r + rel.r,
                    v: p.v + rel.v,
                }
            }
            None => match (&body.parent, &body.elements) {
                (Some(parent), Some(el)) => {
                    let parent_mu = self.catalog.body(*parent).expect("parent").mu;
                    let rel = elements_to_state(parent_mu, el, self.t_ns);
                    let p = self.absolute(*parent);
                    StateVec {
                        r: p.r + rel.r,
                        v: p.v + rel.v,
                    }
                }
                _ => StateVec {
                    r: Vec3::ZERO,
                    v: Vec3::ZERO,
                }, // root
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Elements;

    fn catalog() -> Catalog {
        // Minimal inline two-body catalogue for rail tests.
        let src = r#"(
            bodies: [
                (id: 0, name_id: "star", mu: 1.0e20, radius: 1.0e8, j2: None,
                 rotation_period_s: None, atmosphere: None, gravitating: true,
                 divertible: false, parent: None, elements: None, source: "test"),
                (id: 1, name_id: "p1", mu: 1.0e14, radius: 1.0e6, j2: None,
                 rotation_period_s: None, atmosphere: None, gravitating: true,
                 divertible: false, parent: Some(0),
                 elements: Some((sma: 1.0e11, ecc: 0.0, inc: 0.0, raan: 0.0,
                                 argp: 0.0, mean_anomaly_at_epoch: 0.0, epoch_ns: 0)),
                 source: "test"),
            ],
        )"#;
        Catalog::from_source(src).unwrap()
    }

    #[test]
    fn circular_rail_returns_after_one_period() {
        let cat = catalog();
        let motions = BTreeMap::new();
        let period = 2.0 * core::f64::consts::PI * libm::sqrt(1.0e33 / 1.0e20);
        let t1 = (period * 1e9) as i64;
        let s0 = BodyStates::new(&cat, &motions, 0).absolute(BodyId(1));
        let s1 = BodyStates::new(&cat, &motions, t1).absolute(BodyId(1));
        assert!(
            (s1.r - s0.r).norm() / s0.r.norm() < 1e-6,
            "rail closes after a period"
        );
    }

    #[test]
    fn quarter_period_is_perpendicular() {
        let cat = catalog();
        let motions = BTreeMap::new();
        let period = 2.0 * core::f64::consts::PI * libm::sqrt(1.0e33 / 1.0e20);
        let tq = (period / 4.0 * 1e9) as i64;
        let s0 = BodyStates::new(&cat, &motions, 0).absolute(BodyId(1));
        let sq = BodyStates::new(&cat, &motions, tq).absolute(BodyId(1));
        let cosang = s0.r.dot(sq.r) / (s0.r.norm() * sq.r.norm());
        assert!(
            cosang.abs() < 1e-6,
            "quarter period sweeps 90 degrees on a circular rail"
        );
    }

    #[test]
    fn elements_epoch_offset_respected() {
        let cat = catalog();
        let motions = BTreeMap::new();
        let mut ctx = BodyStates::new(&cat, &motions, 0);
        let s = ctx.absolute(BodyId(1));
        // M0 = 0 at epoch 0 ⇒ at periapsis ⇒ along +x for our rotation convention.
        assert!((s.r.normalized().x - 1.0).abs() < 1e-9);
        let _ = Elements {
            sma: 1.0,
            ecc: 0.0,
            inc: 0.0,
            raan: 0.0,
            argp: 0.0,
            mean_anomaly_at_epoch: 0.0,
            epoch_ns: 0,
        };
    }
}
