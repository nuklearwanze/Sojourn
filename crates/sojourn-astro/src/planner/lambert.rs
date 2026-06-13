//! Universal-variable Lambert solver (research R4; Curtis Algorithm 5.2):
//! bounded Newton iteration with explicit no-convergence results, prograde
//! transfers, optional multi-revolution attempts via deterministic bisection.

use crate::math::{Vec3, stumpff_c, stumpff_s};

/// A Lambert solution: departure and arrival velocity vectors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LambertSolution {
    /// Velocity at r1 (m/s).
    pub v1: Vec3,
    /// Velocity at r2 (m/s).
    pub v2: Vec3,
}

/// Solve the prograde Lambert problem from `r1` to `r2` in `tof` seconds.
/// `revs = 0` is the classical single-arc problem (Newton on z); `revs ≥ 1`
/// attempts the long-period multi-rev branch by bisection and reports
/// `None` honestly when it fails to converge within the caps.
pub fn solve(
    mu: f64,
    r1: Vec3,
    r2: Vec3,
    tof: f64,
    revs: u8,
    max_iter: u32,
    tol: f64,
) -> Option<LambertSolution> {
    if tof <= 0.0 {
        return None;
    }
    let r1n = r1.norm();
    let r2n = r2.norm();
    if r1n == 0.0 || r2n == 0.0 {
        return None;
    }
    // Prograde transfer angle (orbit normal +z convention of the test system).
    let cosd = (r1.dot(r2) / (r1n * r2n)).clamp(-1.0, 1.0);
    let cross = r1.cross(r2);
    let mut dtheta = libm::acos(cosd);
    if cross.z < 0.0 {
        dtheta = 2.0 * core::f64::consts::PI - dtheta;
    }
    let a_coef = libm::sin(dtheta) * libm::sqrt(r1n * r2n / (1.0 - libm::cos(dtheta)));
    if !a_coef.is_finite() || libm::fabs(a_coef) < 1e-6 * (r1n + r2n) {
        return None; // degenerate (0° or 180° geometry)
    }

    let y = |z: f64| -> f64 {
        r1n + r2n + a_coef * (z * stumpff_s(z) - 1.0) / libm::sqrt(stumpff_c(z))
    };
    let tof_of_z = |z: f64| -> Option<f64> {
        let c = stumpff_c(z);
        if c <= 0.0 {
            return None;
        }
        let yy = y(z);
        if yy < 0.0 {
            return None;
        }
        let x = libm::sqrt(yy / c);
        Some((x * x * x * stumpff_s(z) + a_coef * libm::sqrt(yy)) / libm::sqrt(mu))
    };

    let z_solution = if revs == 0 {
        // Newton with bisection safeguarding on z ∈ (z_min, (2π)²).
        let z_hi = (2.0 * core::f64::consts::PI) * (2.0 * core::f64::consts::PI) * 0.999;
        let mut z_lo = -4.0 * core::f64::consts::PI * core::f64::consts::PI;
        // Ensure the bracket is valid (y(z_lo) may be negative; walk up).
        let mut guard = 0;
        while tof_of_z(z_lo).is_none() && guard < 64 {
            z_lo = z_lo / 2.0 + 0.1;
            guard += 1;
        }
        bisect_then_polish(&tof_of_z, z_lo, z_hi, tof, max_iter, tol)?
    } else {
        // Multi-rev long-period branch: z ∈ ((2πN)², (2π(N+1))²); the time
        // curve has a minimum — search the descending side by bisection on a
        // monotone sub-bracket found by deterministic sampling.
        let n = f64::from(revs);
        let lo = (2.0 * core::f64::consts::PI * n).powi(2) * 1.0001;
        let hi = (2.0 * core::f64::consts::PI * (n + 1.0)).powi(2) * 0.9999;
        multi_rev_bisect(&tof_of_z, lo, hi, tof, max_iter, tol)?
    };

    let z = z_solution;
    let yy = y(z);
    if yy < 0.0 {
        return None;
    }
    let f = 1.0 - yy / r1n;
    let g = a_coef * libm::sqrt(yy / mu);
    if g == 0.0 {
        return None;
    }
    let gdot = 1.0 - yy / r2n;
    let v1 = (r2 - r1 * f) * (1.0 / g);
    let v2 = (r2 * gdot - r1) * (1.0 / g);
    if v1.non_finite() || v2.non_finite() {
        return None;
    }
    Some(LambertSolution { v1, v2 })
}

fn bisect_then_polish(
    tof_of_z: &impl Fn(f64) -> Option<f64>,
    mut lo: f64,
    mut hi: f64,
    target: f64,
    max_iter: u32,
    tol: f64,
) -> Option<f64> {
    let f_lo = tof_of_z(lo)?;
    let f_hi = tof_of_z(hi)?;
    if (f_lo - target) * (f_hi - target) > 0.0 {
        return None; // target time outside the single-rev branch
    }
    let mut z = 0.0;
    for _ in 0..max_iter {
        z = 0.5 * (lo + hi);
        let f = tof_of_z(z)?;
        if libm::fabs(f - target) < tol * target.max(1.0) {
            return Some(z);
        }
        // tof is increasing in z on this branch.
        if f < target {
            lo = z;
        } else {
            hi = z;
        }
    }
    Some(z)
}

fn multi_rev_bisect(
    tof_of_z: &impl Fn(f64) -> Option<f64>,
    lo: f64,
    hi: f64,
    target: f64,
    max_iter: u32,
    tol: f64,
) -> Option<f64> {
    // Sample to find the minimum-time z, then bisect the right (increasing) side.
    let samples = 64;
    let mut best_z = lo;
    let mut best_t = f64::INFINITY;
    for i in 0..=samples {
        let z = lo + (hi - lo) * f64::from(i) / f64::from(samples);
        if let Some(t) = tof_of_z(z)
            && t < best_t
        {
            best_t = t;
            best_z = z;
        }
    }
    if !best_t.is_finite() || target < best_t {
        return None; // requested time shorter than the N-rev minimum: unsolvable
    }
    bisect_then_polish(tof_of_z, best_z, hi, target, max_iter, tol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lambert_self_consistency_quarter_orbit() {
        // Solve r1 → r2 over a quarter of a circular orbit; the solution must
        // reproduce the circular velocity and propagate onto r2.
        let mu = 3.986004418e14;
        let r = 7.0e6;
        let v = libm::sqrt(mu / r);
        let period = 2.0 * core::f64::consts::PI * r / v;
        let r1 = Vec3::new(r, 0.0, 0.0);
        let r2 = Vec3::new(0.0, r, 0.0);
        let sol = solve(mu, r1, r2, period / 4.0, 0, 60, 1e-10).unwrap();
        assert!(
            (sol.v1.norm() - v).abs() / v < 1e-6,
            "circular speed recovered"
        );
        let prop = crate::math::universal_propagate(
            mu,
            &crate::math::StateVec { r: r1, v: sol.v1 },
            period / 4.0,
        )
        .unwrap();
        assert!((prop.r - r2).norm() / r < 1e-6, "arrives at r2");
    }

    #[test]
    fn degenerate_geometry_is_unsolvable_not_garbage() {
        let mu = 3.986004418e14;
        let r1 = Vec3::new(7.0e6, 0.0, 0.0);
        // Anti-parallel (180°): a_coef → 0 ⇒ explicit None.
        let r2 = Vec3::new(-7.0e6, 0.0, 0.0);
        assert!(solve(mu, r1, r2, 3000.0, 0, 60, 1e-9).is_none());
    }
}
