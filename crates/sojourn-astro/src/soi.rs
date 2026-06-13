//! Sphere-of-influence structure (FR-ASTRO-202): derived from the catalogue,
//! used for dominant-body bookkeeping, planner patching, and the gravitating-set
//! rule. Crossings preserve physical state exactly (research R6).

use crate::bodies::{BodyId, Catalog, rails::BodyStates};
use crate::math::Vec3;

/// Laplace SOI radius of a body about its parent (root: infinite).
pub fn soi_radius(catalog: &Catalog, id: BodyId) -> f64 {
    let body = catalog.body(id).expect("known body");
    match (body.parent, &body.elements) {
        (Some(parent), Some(el)) => {
            let parent_mu = catalog.body(parent).expect("parent").mu;
            el.sma * libm::pow(body.mu / parent_mu, 0.4)
        }
        _ => f64::INFINITY,
    }
}

/// The dominant body (SOI owner) of an absolute position: walk the tree from
/// the root, descending into any child whose SOI contains the point.
/// Deterministic: children are visited in id order; first containing child wins
/// (catalogue geometry keeps sibling SOIs disjoint).
pub fn dominant_of(catalog: &Catalog, states: &mut BodyStates<'_>, point: Vec3) -> BodyId {
    let mut current = catalog.root();
    loop {
        let mut descended = false;
        let children: Vec<BodyId> = catalog.children(current).map(|b| b.id).collect();
        for child in children {
            let cs = states.absolute(child);
            let r = soi_radius(catalog, child);
            if (point - cs.r).norm2() < r * r {
                current = child;
                descended = true;
                break;
            }
        }
        if !descended {
            return current;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bodies::Catalog;
    use std::collections::BTreeMap;

    fn catalog() -> Catalog {
        Catalog::load_dir(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/astro"
        )))
        .unwrap()
    }

    #[test]
    fn terra_soi_is_about_a_million_km() {
        let cat = catalog();
        // Earth-like SOI ≈ 0.92e9 m with our idealised circular 1 au orbit.
        let r = soi_radius(&cat, crate::bodies::BodyId(1));
        assert!((8.0e8..1.1e9).contains(&r), "terra SOI ≈ 0.9e9 m, got {r}");
    }

    #[test]
    fn dominance_transitions_with_distance() {
        let cat = catalog();
        let motions = BTreeMap::new();
        let mut states = BodyStates::new(&cat, &motions, 0);
        let terra = states.absolute(crate::bodies::BodyId(1));
        // Near terra: dominated by terra.
        let near = terra.r + Vec3::new(1.0e7, 0.0, 0.0);
        assert_eq!(
            dominant_of(&cat, &mut states, near),
            crate::bodies::BodyId(1)
        );
        // Far from everything: the star.
        let far = terra.r + Vec3::new(5.0e9, 0.0, 0.0);
        assert_eq!(
            dominant_of(&cat, &mut states, far),
            crate::bodies::BodyId(0)
        );
        // Near luna: luna.
        let luna = states.absolute(crate::bodies::BodyId(2));
        let near_luna = luna.r + Vec3::new(1.0e6, 0.0, 0.0);
        assert_eq!(
            dominant_of(&cat, &mut states, near_luna),
            crate::bodies::BodyId(2)
        );
    }
}
