//! Reference frames (FR-ASTRO-201): all dynamic state lives in body-centred
//! inertial frames (dominant-body convention, research R6); these helpers
//! convert between them and into rotating (co-orbiting) frames for
//! Lagrange-region work. Conversions are exact vector arithmetic.

use crate::bodies::{BodyId, rails::BodyStates};
use crate::math::{StateVec, Vec3};

/// Convert a state expressed relative to `from` into one relative to `to`
/// (both body-centred inertial), at the context's time. Exact and invertible.
pub fn rebase(states: &mut BodyStates<'_>, s: &StateVec, from: BodyId, to: BodyId) -> StateVec {
    let fs = states.absolute(from);
    let ts = states.absolute(to);
    StateVec {
        r: s.r + fs.r - ts.r,
        v: s.v + fs.v - ts.v,
    }
}

/// Absolute (root-centred) state from a dominant-relative one.
pub fn to_absolute(states: &mut BodyStates<'_>, s: &StateVec, dominant: BodyId) -> StateVec {
    let d = states.absolute(dominant);
    StateVec {
        r: s.r + d.r,
        v: s.v + d.v,
    }
}

/// A rotating frame co-orbiting with `secondary` about `primary` (CR3BP-style):
/// x̂ from primary to secondary, ẑ along the orbit normal. Returns the position
/// of an absolute point expressed in this rotating frame (origin at primary).
pub fn to_rotating(
    states: &mut BodyStates<'_>,
    primary: BodyId,
    secondary: BodyId,
    abs_point: Vec3,
) -> Vec3 {
    let p = states.absolute(primary);
    let s = states.absolute(secondary);
    let x_hat = (s.r - p.r).normalized();
    let z_hat = (s.r - p.r).cross(s.v - p.v).normalized();
    let y_hat = z_hat.cross(x_hat);
    let rel = abs_point - p.r;
    Vec3::new(rel.dot(x_hat), rel.dot(y_hat), rel.dot(z_hat))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bodies::Catalog;
    use std::collections::BTreeMap;

    #[test]
    fn rebase_round_trips_exactly() {
        let cat = Catalog::load_dir(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/astro"
        )))
        .unwrap();
        let motions = BTreeMap::new();
        let mut states = BodyStates::new(&cat, &motions, 12_345_000_000_000);
        let s = StateVec {
            r: Vec3::new(7.0e6, 1.0e5, -3.0e5),
            v: Vec3::new(10.0, 7400.0, -3.0),
        };
        let terra = crate::bodies::BodyId(1);
        let luna = crate::bodies::BodyId(2);
        let there = rebase(&mut states, &s, terra, luna);
        let back = rebase(&mut states, &there, luna, terra);
        // Round-trip exact to f64 cancellation at ~1e11 m magnitudes.
        assert!((back.r - s.r).norm() < 1e-2);
        assert!((back.v - s.v).norm() < 1e-6);
    }
}
