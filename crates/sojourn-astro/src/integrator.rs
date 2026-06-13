//! Fixed-step RK4 (research R1): deterministic, branch-free per step, accuracy
//! guaranteed by state-driven tier selection (research R2) plus the CI
//! validation gates. Mass flow is applied between substeps by the burn executor
//! (first-order mass handling; bounded by the propellant-accuracy tolerance).

use crate::math::{StateVec, Vec3};

/// One RK4 step of (r, v) under `accel(r, v) + thrust_accel`.
pub fn rk4_step(
    s: &StateVec,
    dt: f64,
    thrust_accel: Vec3,
    accel: &impl Fn(Vec3, Vec3) -> Vec3,
) -> StateVec {
    let f = |r: Vec3, v: Vec3| -> (Vec3, Vec3) { (v, accel(r, v) + thrust_accel) };
    let (k1r, k1v) = f(s.r, s.v);
    let (k2r, k2v) = f(s.r + k1r * (dt / 2.0), s.v + k1v * (dt / 2.0));
    let (k3r, k3v) = f(s.r + k2r * (dt / 2.0), s.v + k2v * (dt / 2.0));
    let (k4r, k4v) = f(s.r + k3r * dt, s.v + k3v * dt);
    StateVec {
        r: s.r + (k1r + k2r * 2.0 + k3r * 2.0 + k4r) * (dt / 6.0),
        v: s.v + (k1v + k2v * 2.0 + k3v * 2.0 + k4v) * (dt / 6.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MU: f64 = 3.986004418e14;

    fn central(r: Vec3, _v: Vec3) -> Vec3 {
        let rn = r.norm();
        r * (-MU / (rn * rn * rn))
    }

    #[test]
    fn rk4_converges_at_fourth_order() {
        // Error after a fixed span should drop ~16x when the step halves.
        let r0 = 7.0e6;
        let v0 = libm::sqrt(MU / r0);
        let s0 = StateVec {
            r: Vec3::new(r0, 0.0, 0.0),
            v: Vec3::new(0.0, v0, 0.0),
        };
        let span = 600.0;

        let run = |dt: f64| -> StateVec {
            let mut s = s0;
            let n = (span / dt) as usize;
            for _ in 0..n {
                s = rk4_step(&s, dt, Vec3::ZERO, &central);
            }
            s
        };
        let exact = crate::math::universal_propagate(MU, &s0, span).unwrap();
        let e1 = (run(10.0).r - exact.r).norm();
        let e2 = (run(5.0).r - exact.r).norm();
        assert!(
            e1 / e2 > 10.0,
            "fourth-order convergence (got ratio {})",
            e1 / e2
        );
    }

    #[test]
    fn circular_orbit_energy_stable_at_coast_tier() {
        // Coast tier (60 s steps) only applies beyond the encounter zone
        // (~20 body radii); test a representative high orbit there.
        let r0 = 3.0e8;
        let v0 = libm::sqrt(MU / r0);
        let mut s = StateVec {
            r: Vec3::new(r0, 0.0, 0.0),
            v: Vec3::new(0.0, v0, 0.0),
        };
        let e0 = s.v.norm2() / 2.0 - MU / s.r.norm();
        let period = 2.0 * core::f64::consts::PI * libm::sqrt(r0 * r0 * r0 / MU);
        let n = (period / 60.0) as usize;
        for _ in 0..n {
            s = rk4_step(&s, 60.0, Vec3::ZERO, &central);
        }
        let e1 = s.v.norm2() / 2.0 - MU / s.r.norm();
        assert!(
            ((e1 - e0) / e0).abs() < 1e-10,
            "energy drift per orbit within bound"
        );
    }

    #[test]
    fn leo_energy_stable_at_encounter_tier() {
        // Low orbits run at the encounter tier (1 s steps).
        let r0 = 7.0e6;
        let v0 = libm::sqrt(MU / r0);
        let mut s = StateVec {
            r: Vec3::new(r0, 0.0, 0.0),
            v: Vec3::new(0.0, v0, 0.0),
        };
        let e0 = s.v.norm2() / 2.0 - MU / s.r.norm();
        let period = 2.0 * core::f64::consts::PI * libm::sqrt(r0 * r0 * r0 / MU);
        let n = period as usize;
        for _ in 0..n {
            s = rk4_step(&s, 1.0, Vec3::ZERO, &central);
        }
        let e1 = s.v.norm2() / 2.0 - MU / s.r.norm();
        assert!(
            ((e1 - e0) / e0).abs() < 1e-11,
            "fine-tier energy drift per orbit"
        );
    }
}
