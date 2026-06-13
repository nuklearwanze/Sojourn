//! In-crate math primitives (research R5): 3-vectors, Keplerian element
//! conversions, Kepler/universal-variable propagation kernels. All
//! transcendentals via `libm` (FA-01 float policy); no external math crates.

use serde::{Deserialize, Serialize};

/// 3-vector of f64 (SI).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec3 {
    /// x component.
    pub x: f64,
    /// y component.
    pub y: f64,
    /// z component.
    pub z: f64,
}

impl Vec3 {
    /// Zero vector.
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Construct.
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    /// Dot product.
    pub fn dot(self, o: Vec3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Cross product.
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    /// Squared norm.
    pub fn norm2(self) -> f64 {
        self.dot(self)
    }

    /// Norm.
    pub fn norm(self) -> f64 {
        libm::sqrt(self.norm2())
    }

    /// Unit vector (zero vector returns zero).
    pub fn normalized(self) -> Vec3 {
        let n = self.norm();
        if n == 0.0 {
            Vec3::ZERO
        } else {
            self * (1.0 / n)
        }
    }

    /// Any of the components non-finite?
    pub fn non_finite(self) -> bool {
        !(self.x.is_finite() && self.y.is_finite() && self.z.is_finite())
    }

    /// Rotate about a unit axis by an angle (Rodrigues).
    pub fn rotate_about(self, axis: Vec3, angle: f64) -> Vec3 {
        let (s, c) = (libm::sin(angle), libm::cos(angle));
        self * c + axis.cross(self) * s + axis * (axis.dot(self) * (1.0 - c))
    }
}

impl core::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl core::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl core::ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f64) -> Vec3 {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }
}
impl core::ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

/// Position + velocity state (SI, frame per context).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct StateVec {
    /// Position (m).
    pub r: Vec3,
    /// Velocity (m/s).
    pub v: Vec3,
}

/// Classical Keplerian elements (radians; SI).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Elements {
    /// Semi-major axis (m). Negative for hyperbolic.
    pub sma: f64,
    /// Eccentricity.
    pub ecc: f64,
    /// Inclination (rad).
    pub inc: f64,
    /// Right ascension of ascending node (rad).
    pub raan: f64,
    /// Argument of periapsis (rad).
    pub argp: f64,
    /// Mean anomaly at epoch (rad).
    pub mean_anomaly_at_epoch: f64,
    /// Epoch (ns since game epoch).
    pub epoch_ns: i64,
}

/// Solve Kepler's equation E − e·sinE = M (elliptic) by Newton iteration.
pub fn solve_kepler_elliptic(mean_anomaly: f64, ecc: f64) -> f64 {
    let m = rem_two_pi(mean_anomaly);
    let mut e_anom = if ecc < 0.8 { m } else { core::f64::consts::PI };
    for _ in 0..30 {
        let f = e_anom - ecc * libm::sin(e_anom) - m;
        let fp = 1.0 - ecc * libm::cos(e_anom);
        let d = f / fp;
        e_anom -= d;
        if libm::fabs(d) < 1e-14 {
            break;
        }
    }
    e_anom
}

/// Wrap an angle into [0, 2π).
pub fn rem_two_pi(a: f64) -> f64 {
    let two_pi = 2.0 * core::f64::consts::PI;
    let r = a - two_pi * libm::floor(a / two_pi);
    if r < 0.0 { r + two_pi } else { r }
}

/// Elements → state at time t (elliptic rails; e < 1 enforced by data validation).
pub fn elements_to_state(mu: f64, el: &Elements, t_ns: i64) -> StateVec {
    let n = libm::sqrt(mu / (el.sma * el.sma * el.sma)); // mean motion
    let dt = (t_ns - el.epoch_ns) as f64 * 1e-9;
    let m = el.mean_anomaly_at_epoch + n * dt;
    let e_anom = solve_kepler_elliptic(m, el.ecc);
    let (se, ce) = (libm::sin(e_anom), libm::cos(e_anom));
    // True anomaly and radius.
    let sq = libm::sqrt(1.0 - el.ecc * el.ecc);
    let nu = libm::atan2(sq * se, ce - el.ecc);
    let r_mag = el.sma * (1.0 - el.ecc * ce);
    // Perifocal position/velocity.
    let p = el.sma * (1.0 - el.ecc * el.ecc);
    let h = libm::sqrt(mu * p);
    let (snu, cnu) = (libm::sin(nu), libm::cos(nu));
    let r_pf = Vec3::new(r_mag * cnu, r_mag * snu, 0.0);
    let v_pf = Vec3::new(-(mu / h) * snu, (mu / h) * (el.ecc + cnu), 0.0);
    // Perifocal → inertial (3-1-3 rotation: raan, inc, argp).
    let rot = |v: Vec3| -> Vec3 {
        v.rotate_about(Vec3::new(0.0, 0.0, 1.0), el.argp)
            .rotate_about(Vec3::new(1.0, 0.0, 0.0), el.inc)
            .rotate_about(Vec3::new(0.0, 0.0, 1.0), el.raan)
    };
    StateVec {
        r: rot(r_pf),
        v: rot(v_pf),
    }
}

/// State → classical elements (any conic).
pub fn state_to_elements(mu: f64, s: &StateVec, t_ns: i64) -> Elements {
    let r = s.r.norm();
    let v2 = s.v.norm2();
    let h_vec = s.r.cross(s.v);
    let h = h_vec.norm();
    let e_vec = (s.v.cross(h_vec)) * (1.0 / mu) - s.r.normalized();
    let ecc = e_vec.norm();
    let energy = v2 / 2.0 - mu / r;
    let sma = if libm::fabs(energy) < 1e-12 {
        f64::INFINITY
    } else {
        -mu / (2.0 * energy)
    };
    let inc = libm::acos((h_vec.z / h).clamp(-1.0, 1.0));
    let n_vec = Vec3::new(0.0, 0.0, 1.0).cross(h_vec);
    let n = n_vec.norm();
    let raan = if n > 1e-12 {
        let mut o = libm::acos((n_vec.x / n).clamp(-1.0, 1.0));
        if n_vec.y < 0.0 {
            o = 2.0 * core::f64::consts::PI - o;
        }
        o
    } else {
        0.0
    };
    let argp = if n > 1e-12 && ecc > 1e-12 {
        let mut w = libm::acos((n_vec.dot(e_vec) / (n * ecc)).clamp(-1.0, 1.0));
        if e_vec.z < 0.0 {
            w = 2.0 * core::f64::consts::PI - w;
        }
        w
    } else if ecc > 1e-12 {
        libm::atan2(e_vec.y, e_vec.x)
    } else {
        0.0
    };
    // True anomaly → mean anomaly (elliptic only; hyperbolic uses its own flows).
    let mean_anomaly = if ecc < 1.0 {
        let cosnu = (e_vec.dot(s.r) / (ecc.max(1e-15) * r)).clamp(-1.0, 1.0);
        let mut nu = libm::acos(cosnu);
        if s.r.dot(s.v) < 0.0 {
            nu = 2.0 * core::f64::consts::PI - nu;
        }
        let e_anom = 2.0 * libm::atan(libm::sqrt((1.0 - ecc) / (1.0 + ecc)) * libm::tan(nu / 2.0));
        rem_two_pi(e_anom - ecc * libm::sin(e_anom))
    } else {
        0.0
    };
    Elements {
        sma,
        ecc,
        inc,
        raan,
        argp,
        mean_anomaly_at_epoch: mean_anomaly,
        epoch_ns: t_ns,
    }
}

/// Stumpff C(z).
pub fn stumpff_c(z: f64) -> f64 {
    if z > 1e-8 {
        (1.0 - libm::cos(libm::sqrt(z))) / z
    } else if z < -1e-8 {
        (libm::cosh(libm::sqrt(-z)) - 1.0) / (-z)
    } else {
        0.5 - z / 24.0
    }
}

/// Stumpff S(z).
pub fn stumpff_s(z: f64) -> f64 {
    if z > 1e-8 {
        let sz = libm::sqrt(z);
        (sz - libm::sin(sz)) / (sz * sz * sz)
    } else if z < -1e-8 {
        let sz = libm::sqrt(-z);
        (libm::sinh(sz) - sz) / (sz * sz * sz)
    } else {
        1.0 / 6.0 - z / 120.0
    }
}

/// Universal-variable two-body propagation: state after `dt` seconds (any conic).
/// Curtis Algorithm 3.4 (f and g functions). Returns None on non-convergence.
pub fn universal_propagate(mu: f64, s: &StateVec, dt: f64) -> Option<StateVec> {
    if dt == 0.0 {
        return Some(*s);
    }
    let r0 = s.r.norm();
    let vr0 = s.r.dot(s.v) / r0;
    let alpha = 2.0 / r0 - s.v.norm2() / mu; // 1/a
    let sqrt_mu = libm::sqrt(mu);
    // Initial guess per conic (Vallado, Fundamentals, Alg. 8 'kepler').
    let mut chi = if alpha > 1e-12 {
        // Elliptic.
        sqrt_mu * alpha * dt
    } else if alpha < -1e-12 {
        // Hyperbolic.
        let a = 1.0 / alpha; // negative
        let num = -2.0 * mu * alpha * dt;
        let den = s.r.dot(s.v) + libm::copysign(1.0, dt) * libm::sqrt(-mu * a) * (1.0 - r0 * alpha);
        if den != 0.0 && num / den > 0.0 {
            libm::copysign(1.0, dt) * libm::sqrt(-a) * libm::log(num / den)
        } else {
            sqrt_mu * dt / r0
        }
    } else {
        // Near-parabolic.
        sqrt_mu * dt / r0
    };
    if !chi.is_finite() || chi == 0.0 {
        chi = sqrt_mu * dt / r0;
    }
    let residual = |chi: f64| -> f64 {
        let z = alpha * chi * chi;
        let c = stumpff_c(z);
        let sf = stumpff_s(z);
        r0 * vr0 / sqrt_mu * chi * chi * c + (1.0 - alpha * r0) * chi * chi * chi * sf + r0 * chi
            - sqrt_mu * dt
    };
    let mut converged = false;
    for _ in 0..60 {
        let z = alpha * chi * chi;
        let c = stumpff_c(z);
        let sf = stumpff_s(z);
        let f = residual(chi);
        let fp = r0 * vr0 / sqrt_mu * chi * (1.0 - alpha * chi * chi * sf)
            + (1.0 - alpha * r0) * chi * chi * c
            + r0;
        let d = f / fp;
        chi -= d;
        // Relative convergence: an absolute bound on chi is meaningless across
        // scales (LEO chi ~1e3, heliocentric chi ~1e6).
        if libm::fabs(d) < 1e-12 * libm::fabs(chi).max(1.0) {
            converged = true;
            break;
        }
    }
    // Accept iteration-capped solutions whose time residual is negligible
    // (< 1 µs of flight time): Newton at machine precision oscillates in the
    // last ulp without ever meeting a step-size test.
    if !converged && libm::fabs(residual(chi)) < 1e-6 * sqrt_mu {
        converged = true;
    }
    if !converged || !chi.is_finite() {
        return None;
    }
    let z = alpha * chi * chi;
    let c = stumpff_c(z);
    let sf = stumpff_s(z);
    let f = 1.0 - chi * chi / r0 * c;
    let g = dt - chi * chi * chi / sqrt_mu * sf;
    let r_new = s.r * f + s.v * g;
    let rn = r_new.norm();
    let fdot = sqrt_mu / (rn * r0) * (alpha * chi * chi * chi * sf - chi);
    let gdot = 1.0 - chi * chi / rn * c;
    Some(StateVec {
        r: r_new,
        v: s.r * fdot + s.v * gdot,
    })
}

/// Prograde/radial/normal (`dv_prn`) basis from a state: returns (prograde,
/// radial, normal) unit vectors. prograde = v̂; normal = (r×v)̂; radial completes
/// the right-handed set pointing away from the central body.
pub fn prn_basis(s: &StateVec) -> (Vec3, Vec3, Vec3) {
    let prograde = s.v.normalized();
    let normal = s.r.cross(s.v).normalized();
    let radial = prograde.cross(normal);
    (prograde, radial, normal)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MU: f64 = 3.986004418e14;

    #[test]
    fn elements_state_round_trip() {
        let el = Elements {
            sma: 7.0e6,
            ecc: 0.1,
            inc: 0.5,
            raan: 1.0,
            argp: 2.0,
            mean_anomaly_at_epoch: 0.3,
            epoch_ns: 0,
        };
        let s = elements_to_state(MU, &el, 0);
        let back = state_to_elements(MU, &s, 0);
        assert!((back.sma - el.sma).abs() / el.sma < 1e-10);
        assert!((back.ecc - el.ecc).abs() < 1e-10);
        assert!((back.inc - el.inc).abs() < 1e-10);
        assert!((back.raan - el.raan).abs() < 1e-10);
        assert!((back.argp - el.argp).abs() < 1e-9);
        assert!((back.mean_anomaly_at_epoch - el.mean_anomaly_at_epoch).abs() < 1e-9);
    }

    #[test]
    fn universal_propagate_full_period_closes() {
        let el = Elements {
            sma: 7.0e6,
            ecc: 0.2,
            inc: 0.3,
            raan: 0.7,
            argp: 1.1,
            mean_anomaly_at_epoch: 0.0,
            epoch_ns: 0,
        };
        let s0 = elements_to_state(MU, &el, 0);
        let period = 2.0 * core::f64::consts::PI * libm::sqrt(el.sma.powi(3) / MU);
        let s1 = universal_propagate(MU, &s0, period).unwrap();
        assert!(
            (s1.r - s0.r).norm() < 1.0,
            "orbit closes to <1 m after one period"
        );
        assert!((s1.v - s0.v).norm() < 1e-3);
    }

    #[test]
    fn universal_propagate_hyperbolic() {
        // Hyperbolic departure: v > escape at r.
        let r = Vec3::new(7.0e6, 0.0, 0.0);
        let vesc = libm::sqrt(2.0 * MU / 7.0e6);
        let s0 = StateVec {
            r,
            v: Vec3::new(0.0, vesc * 1.2, 0.0),
        };
        let s1 = universal_propagate(MU, &s0, 86_400.0).unwrap();
        assert!(s1.r.norm() > 1.0e8, "escaping trajectory recedes");
        // Energy conserved.
        let e0 = s0.v.norm2() / 2.0 - MU / s0.r.norm();
        let e1 = s1.v.norm2() / 2.0 - MU / s1.r.norm();
        assert!((e1 - e0).abs() / e0.abs() < 1e-9);
    }

    #[test]
    fn prn_basis_is_orthonormal() {
        let s = StateVec {
            r: Vec3::new(7e6, 1e5, -2e5),
            v: Vec3::new(100.0, 7500.0, 50.0),
        };
        let (p, r, n) = prn_basis(&s);
        assert!((p.norm() - 1.0).abs() < 1e-12);
        assert!((r.norm() - 1.0).abs() < 1e-12);
        assert!((n.norm() - 1.0).abs() < 1e-12);
        assert!(p.dot(r).abs() < 1e-12);
        assert!(p.dot(n).abs() < 1e-12);
        assert!(r.dot(n).abs() < 1e-12);
    }
}
