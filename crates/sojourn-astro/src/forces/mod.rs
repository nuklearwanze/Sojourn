//! Force models (research R7, FR-ASTRO-102): point gravity over the gravitating
//! set (dominant-relative differential formulation), J2, cannonball SRP with
//! cylindrical shadow, exponential drag, and thrust. Each term independently
//! enableable. Bodies are frozen per substep block at the block's midpoint time
//! (documented approximation; validation tolerances bound it).

use crate::bodies::{BodyId, Catalog, rails::BodyStates};
use crate::math::Vec3;
use crate::module::AstroConfig;

/// A body's frozen contribution context.
#[derive(Debug, Clone, Copy)]
pub struct ThirdBody {
    /// Gravitational parameter.
    pub mu: f64,
    /// Position relative to the dominant body at the frozen time.
    pub r_rel: Vec3,
}

/// Frozen environment for one craft over one substep block (research R7).
#[derive(Debug, Clone)]
pub struct FrozenEnv {
    /// Dominant body μ.
    pub mu_dom: f64,
    /// Dominant J2 + radius, when enabled and present.
    pub j2: Option<(f64, f64)>,
    /// Third bodies (gravitating flag, excluding the dominant).
    pub third: Vec<ThirdBody>,
    /// Star position relative to dominant + SRP constants, when enabled.
    pub srp: Option<SrpEnv>,
    /// Atmosphere of the dominant, when enabled and present:
    /// (drag ceiling alt, rho0, H, body radius).
    pub drag: Option<(f64, f64, f64, f64)>,
    /// Drag Cd.
    pub cd: f64,
}

/// SRP environment.
#[derive(Debug, Clone, Copy)]
pub struct SrpEnv {
    /// Star position relative to the dominant body.
    pub star_rel: Vec3,
    /// Solar flux at the reference distance (W/m²).
    pub flux_ref: f64,
    /// Reference distance (m).
    pub ref_dist: f64,
    /// Speed of light (m/s).
    pub c: f64,
    /// Reflectivity coefficient.
    pub cr: f64,
    /// Occulting body radius (the dominant; 0 when dominant is the star).
    pub occult_radius: f64,
}

/// Build the frozen environment for a craft around `dominant` at time `t_ns`.
pub fn freeze(
    catalog: &Catalog,
    states: &mut BodyStates<'_>,
    config: &AstroConfig,
    dominant: BodyId,
) -> FrozenEnv {
    let dom_def = catalog.body(dominant).expect("dominant");
    let dom_abs = states.absolute(dominant);
    let root = catalog.root();

    let mut third = Vec::new();
    if config.enable_third_body {
        for b in catalog.bodies() {
            if b.id == dominant || !b.gravitating {
                continue;
            }
            let s = states.absolute(b.id);
            third.push(ThirdBody {
                mu: b.mu,
                r_rel: s.r - dom_abs.r,
            });
        }
    }

    let j2 = if config.enable_j2 {
        dom_def.j2.map(|j2| (j2, dom_def.radius))
    } else {
        None
    };

    let srp = if config.enable_srp {
        let star_abs = states.absolute(root);
        Some(SrpEnv {
            star_rel: star_abs.r - dom_abs.r,
            flux_ref: config.solar_flux_w_m2,
            ref_dist: config.flux_reference_m,
            c: config.light_speed_m_s,
            cr: config.srp_cr,
            occult_radius: if dominant == root {
                0.0
            } else {
                dom_def.radius
            },
        })
    } else {
        None
    };

    let drag = if config.enable_drag {
        dom_def
            .atmosphere
            .as_ref()
            .map(|a| (a.drag_alt_m, a.rho0, a.scale_height_m, dom_def.radius))
    } else {
        None
    };

    FrozenEnv {
        mu_dom: dom_def.mu,
        j2,
        third,
        srp,
        drag,
        cd: config.drag_cd,
    }
}

/// Total non-thrust acceleration on a craft at dominant-relative (r, v).
/// `area_over_mass_*` are the cannonball ratios for SRP/drag.
pub fn acceleration(
    env: &FrozenEnv,
    r: Vec3,
    v: Vec3,
    srp_area_over_mass: f64,
    drag_area_over_mass: f64,
) -> Vec3 {
    let rn = r.norm();
    // Central gravity.
    let mut a = r * (-env.mu_dom / (rn * rn * rn));

    // Third-body differential (dominant-relative formulation).
    for tb in &env.third {
        let d = tb.r_rel - r; // craft → body
        let dn = d.norm();
        let bn = tb.r_rel.norm();
        a = a + d * (tb.mu / (dn * dn * dn)) - tb.r_rel * (tb.mu / (bn * bn * bn));
    }

    // J2 of the dominant (spin axis = global +Z; test-catalogue idealisation).
    if let Some((j2, radius)) = env.j2 {
        let k = 1.5 * j2 * env.mu_dom * radius * radius / libm::pow(rn, 5.0);
        let z2_r2 = (r.z * r.z) / (rn * rn);
        a = a + Vec3::new(
            k * r.x * (5.0 * z2_r2 - 1.0),
            k * r.y * (5.0 * z2_r2 - 1.0),
            k * r.z * (5.0 * z2_r2 - 3.0),
        );
    }

    // Solar radiation pressure (cannonball, cylindrical shadow).
    if let Some(srp) = &env.srp
        && srp_area_over_mass > 0.0
    {
        let from_star = r - srp.star_rel; // star → craft
        let d = from_star.norm();
        if d > 0.0 && !in_cylindrical_shadow(srp, r) {
            let flux = srp.flux_ref * (srp.ref_dist / d) * (srp.ref_dist / d);
            let mag = flux / srp.c * srp.cr * srp_area_over_mass;
            a = a + from_star.normalized() * mag;
        }
    }

    // Atmospheric drag (exponential, non-rotating atmosphere idealisation).
    if let Some((drag_ceiling, rho0, scale_h, radius)) = env.drag {
        let alt = rn - radius;
        if alt < drag_ceiling && drag_area_over_mass > 0.0 {
            let rho = rho0 * libm::exp(-alt.max(0.0) / scale_h);
            let vn = v.norm();
            if vn > 0.0 {
                a = a + v * (-0.5 * rho * env.cd * drag_area_over_mass * vn);
            }
        }
    }

    a
}

/// Is a dominant-relative point inside the dominant body's cylindrical shadow?
fn in_cylindrical_shadow(srp: &SrpEnv, r: Vec3) -> bool {
    if srp.occult_radius <= 0.0 {
        return false;
    }
    let sun_dir = srp.star_rel.normalized(); // dominant → star
    let along = r.dot(sun_dir);
    if along >= 0.0 {
        return false; // on the sunlit side of the occulting body
    }
    let radial = (r - sun_dir * along).norm();
    radial < srp.occult_radius
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_central(mu: f64) -> FrozenEnv {
        FrozenEnv {
            mu_dom: mu,
            j2: None,
            third: Vec::new(),
            srp: None,
            drag: None,
            cd: 2.2,
        }
    }

    #[test]
    fn central_gravity_inverse_square() {
        let env = env_central(3.986004418e14);
        let a1 = acceleration(&env, Vec3::new(7.0e6, 0.0, 0.0), Vec3::ZERO, 0.0, 0.0);
        let a2 = acceleration(&env, Vec3::new(1.4e7, 0.0, 0.0), Vec3::ZERO, 0.0, 0.0);
        assert!(
            (a1.norm() / a2.norm() - 4.0).abs() < 1e-9,
            "inverse-square ratio"
        );
        assert!(a1.x < 0.0, "points inward");
    }

    #[test]
    fn third_body_differential_vanishes_at_origin() {
        let mut env = env_central(3.986004418e14);
        env.third.push(ThirdBody {
            mu: 4.9e12,
            r_rel: Vec3::new(3.84e8, 0.0, 0.0),
        });
        // At the dominant body's centre the differential term is exactly zero.
        let a = acceleration(&env, Vec3::new(1e-6, 0.0, 0.0), Vec3::ZERO, 0.0, 0.0);
        let central_only = acceleration(
            &env_central(3.986004418e14),
            Vec3::new(1e-6, 0.0, 0.0),
            Vec3::ZERO,
            0.0,
            0.0,
        );
        assert!((a - central_only).norm() < 1e-9);
    }

    #[test]
    fn drag_only_below_interface_and_opposes_velocity() {
        let mut env = env_central(3.986004418e14);
        env.drag = Some((1.2e5, 1.225, 8500.0, 6.378137e6));
        let v = Vec3::new(0.0, 7800.0, 0.0);
        let low = acceleration(&env, Vec3::new(6.378137e6 + 8.0e4, 0.0, 0.0), v, 0.0, 0.01);
        let high = acceleration(&env, Vec3::new(6.378137e6 + 5.0e5, 0.0, 0.0), v, 0.0, 0.01);
        let central_low = acceleration(
            &env_central(3.986004418e14),
            Vec3::new(6.378137e6 + 8.0e4, 0.0, 0.0),
            v,
            0.0,
            0.0,
        );
        let drag_acc = low - central_low;
        assert!(drag_acc.dot(v) < 0.0, "drag opposes velocity");
        let central_high = acceleration(
            &env_central(3.986004418e14),
            Vec3::new(6.378137e6 + 5.0e5, 0.0, 0.0),
            v,
            0.0,
            0.0,
        );
        assert!(
            (high - central_high).norm() < 1e-12,
            "no drag above the interface"
        );
    }

    #[test]
    fn srp_pushes_away_from_star_and_shadow_blocks() {
        let mut env = env_central(3.986004418e14);
        env.srp = Some(SrpEnv {
            star_rel: Vec3::new(-1.495978707e11, 0.0, 0.0),
            flux_ref: 1361.0,
            ref_dist: 1.495978707e11,
            c: 299792458.0,
            cr: 1.3,
            occult_radius: 6.378137e6,
        });
        // Sunlit side (between star and body): pushed away from star (+ -x → push +x? star at -x ⇒ push toward +x).
        let lit = acceleration(&env, Vec3::new(-7.0e6, 0.0, 0.0), Vec3::ZERO, 0.02, 0.0);
        let central = acceleration(
            &env_central(3.986004418e14),
            Vec3::new(-7.0e6, 0.0, 0.0),
            Vec3::ZERO,
            0.0,
            0.0,
        );
        let srp_acc = lit - central;
        assert!(srp_acc.x > 0.0, "SRP pushes anti-starward");
        // Behind the body (shadow cylinder): no SRP.
        let shadowed = acceleration(&env, Vec3::new(7.0e6, 0.0, 0.0), Vec3::ZERO, 0.02, 0.0);
        let central2 = acceleration(
            &env_central(3.986004418e14),
            Vec3::new(7.0e6, 0.0, 0.0),
            Vec3::ZERO,
            0.0,
            0.0,
        );
        assert!(
            (shadowed - central2).norm() < 1e-15,
            "cylindrical shadow blocks SRP"
        );
    }
}
