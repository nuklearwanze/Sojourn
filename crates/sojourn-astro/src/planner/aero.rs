//! Aerocapture/aerobraking corridors (FR-ASTRO-407): computed by bounded pure
//! mini-propagation with the SAME drag physics the propagator flies — the
//! planner can't lie about the atmosphere because it uses the atmosphere.

use super::Regime;
use crate::bodies::{BodyId, Catalog};
use crate::integrator::rk4_step;
use crate::math::{StateVec, Vec3};
use serde::{Deserialize, Serialize};

/// Corridor query: an arrival hyperbola at an atmosphere body.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AeroQuery {
    /// Atmosphere body.
    pub body: BodyId,
    /// Arrival v∞ magnitude (m/s).
    pub v_inf: f64,
    /// Target vacuum periapsis altitude (m) for the pass.
    pub periapsis_alt_m: f64,
    /// Craft drag area / mass (m²/kg).
    pub drag_area_over_mass: f64,
    /// Drag coefficient.
    pub cd: f64,
    /// Depth/load floor (m altitude) — below this is a violation.
    pub floor_alt_m: f64,
}

/// One simulated pass outcome.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PassOutcome {
    /// Captured into a bound orbit?
    pub captured: bool,
    /// Apoapsis after the pass (m; meaningful when captured).
    pub exit_apoapsis_m: f64,
    /// Minimum altitude reached (m).
    pub min_altitude_m: f64,
    /// Below the floor at any point?
    pub violates_floor: bool,
}

/// Corridor result.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AeroCorridor {
    /// Outcome at the target periapsis.
    pub at_target: PassOutcome,
    /// Shallowest capturing periapsis altitude found (m).
    pub shallow_limit_alt_m: f64,
    /// Steepest non-violating periapsis altitude (m).
    pub steep_limit_alt_m: f64,
    /// Honesty tag (single-pass drag prediction is well-posed; we computed it
    /// with the real physics).
    pub regime: Regime,
}

/// Simulate one pass through the atmosphere from SOI-edge approach geometry,
/// pure and bounded (fixed 1 s steps, capped step count). Two-body + drag.
pub fn simulate_pass(
    catalog: &Catalog,
    q: &AeroQuery,
    periapsis_alt_m: f64,
) -> Option<PassOutcome> {
    let body = catalog.body(q.body)?;
    let atmo = body.atmosphere.as_ref()?;
    let mu = body.mu;
    let rp = body.radius + periapsis_alt_m;
    // Build the inbound hyperbola from v∞ and vacuum periapsis: specific energy
    // ε = v∞²/2 ⇒ v(r) and angular momentum h = rp·vp.
    let eps = q.v_inf * q.v_inf / 2.0;
    let vp = libm::sqrt(2.0 * (eps + mu / rp));
    let h = rp * vp;
    // Start above all drag, inbound.
    let r0 = body.radius + (4.0 * atmo.interface_alt_m).max(1.2 * atmo.drag_alt_m);
    let v0 = libm::sqrt(2.0 * (eps + mu / r0));
    // Flight-path angle from h = r·v·cosγ (inbound ⇒ γ < 0).
    let cosg = (h / (r0 * v0)).clamp(-1.0, 1.0);
    let gamma = -libm::acos(cosg);
    let state0 = StateVec {
        r: Vec3::new(r0, 0.0, 0.0),
        v: Vec3::new(v0 * libm::sin(gamma), v0 * libm::cos(gamma), 0.0),
    };

    let accel = |r: Vec3, v: Vec3| -> Vec3 {
        let rn = r.norm();
        let mut a = r * (-mu / (rn * rn * rn));
        let alt = rn - body.radius;
        if alt < atmo.drag_alt_m {
            let rho = atmo.rho0 * libm::exp(-alt.max(0.0) / atmo.scale_height_m);
            let vn = v.norm();
            if vn > 0.0 {
                a = a + v * (-0.5 * rho * q.cd * q.drag_area_over_mass * vn);
            }
        }
        a
    };

    let mut s = state0;
    let mut min_alt = f64::INFINITY;
    let max_steps = 200_000u32; // bounded purity
    let dt = 1.0;
    let mut exited = false;
    for _ in 0..max_steps {
        s = rk4_step(&s, dt, Vec3::ZERO, &accel);
        let rn = s.r.norm();
        let alt = rn - body.radius;
        min_alt = min_alt.min(alt);
        if alt <= 0.0 {
            // Surface impact: deepest possible violation.
            return Some(PassOutcome {
                captured: false,
                exit_apoapsis_m: 0.0,
                min_altitude_m: 0.0,
                violates_floor: true,
            });
        }
        if rn > r0 && s.r.dot(s.v) > 0.0 {
            exited = true;
            break;
        }
    }
    if !exited {
        return None; // didn't exit within bounds (degenerate query)
    }
    let energy = s.v.norm2() / 2.0 - mu / s.r.norm();
    let captured = energy < 0.0;
    let exit_apo = if captured {
        let sma = -mu / (2.0 * energy);
        let h_out = s.r.cross(s.v).norm();
        let ecc = libm::sqrt((1.0 - h_out * h_out / (mu * sma)).max(0.0));
        sma * (1.0 + ecc)
    } else {
        f64::INFINITY
    };
    Some(PassOutcome {
        captured,
        exit_apoapsis_m: exit_apo,
        min_altitude_m: min_alt,
        violates_floor: min_alt < q.floor_alt_m,
    })
}

/// Compute the corridor by scanning periapsis altitudes (bounded, deterministic).
pub fn corridor(catalog: &Catalog, q: &AeroQuery) -> Option<AeroCorridor> {
    let body = catalog.body(q.body)?;
    let atmo = body.atmosphere.as_ref()?;
    let at_target = simulate_pass(catalog, q, q.periapsis_alt_m)?;

    // Scan from the floor to the interface for the capture band.
    let n = 24;
    let mut shallow = f64::NAN;
    let mut steep = f64::NAN;
    for i in 0..=n {
        let alt =
            q.floor_alt_m + (atmo.interface_alt_m - q.floor_alt_m) * f64::from(i) / f64::from(n);
        if let Some(p) = simulate_pass(catalog, q, alt)
            && p.captured
            && !p.violates_floor
        {
            shallow = if shallow.is_nan() {
                alt
            } else {
                shallow.max(alt)
            };
            steep = if steep.is_nan() { alt } else { steep.min(alt) };
        }
    }
    if shallow.is_nan() {
        return None; // no capturing corridor at this v∞
    }
    Some(AeroCorridor {
        at_target,
        shallow_limit_alt_m: shallow,
        steep_limit_alt_m: steep,
        regime: Regime::WellConditioned,
    })
}
