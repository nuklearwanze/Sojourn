//! Hyperbolic-encounter solving and manual chain verification (FR-ASTRO-404).
//! The chain representation is the contract automated sequence search builds on
//! later (clarified 2026-06-13).

use super::Regime;
use crate::bodies::{BodyId, Catalog};
use crate::math::Vec3;
use serde::{Deserialize, Serialize};

/// A flyby query: inbound v∞ and target periapsis at a body; the encounter
/// plane is set by `plane_normal` (unit; defaults to +z geometry when zero).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FlybyQuery {
    /// Encounter body.
    pub body: BodyId,
    /// Inbound hyperbolic excess velocity (m/s, body-relative).
    pub v_inf_in: Vec3,
    /// Periapsis radius (m, from body centre).
    pub periapsis_m: f64,
    /// Encounter plane normal (turn axis); zero ⇒ +z.
    pub plane_normal: Vec3,
    /// Turn direction sign (+1 / −1 about the normal).
    pub turn_sign: i8,
}

/// A solved encounter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EncounterSolution {
    /// Outbound v∞ (body-relative).
    pub v_inf_out: Vec3,
    /// Eccentricity of the hyperbola.
    pub eccentricity: f64,
    /// Turn angle δ (rad).
    pub turn_angle_rad: f64,
    /// Valid (periapsis above surface + atmosphere interface)?
    pub valid: bool,
    /// Honesty tag (encounters are two-body within the SOI: well conditioned
    /// unless the periapsis is invalid).
    pub regime: Regime,
}

/// Solve the hyperbolic encounter: δ = 2·asin(1/e), e = 1 + rp·v∞²/μ
/// (Curtis ch. 8). |v∞_out| = |v∞_in|; direction turned by δ about the normal.
pub fn solve(catalog: &Catalog, q: &FlybyQuery) -> Option<EncounterSolution> {
    let body = catalog.body(q.body)?;
    let v_inf = q.v_inf_in.norm();
    if v_inf <= 0.0 || q.periapsis_m <= 0.0 {
        return None;
    }
    let e = 1.0 + q.periapsis_m * v_inf * v_inf / body.mu;
    let delta = 2.0 * libm::asin((1.0 / e).clamp(-1.0, 1.0));
    let floor = body.radius
        + body
            .atmosphere
            .as_ref()
            .map(|a| a.interface_alt_m)
            .unwrap_or(0.0);
    let valid = q.periapsis_m > floor;
    let normal = if q.plane_normal.norm2() > 0.0 {
        q.plane_normal.normalized()
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    };
    let v_out = q
        .v_inf_in
        .rotate_about(normal, delta * f64::from(q.turn_sign.signum().max(-1)));
    Some(EncounterSolution {
        v_inf_out: v_out,
        eccentricity: e,
        turn_angle_rad: delta,
        valid,
        regime: if valid {
            Regime::WellConditioned
        } else {
            Regime::LowConfidence
        },
    })
}

/// One leg of a manual assist chain: a heliocentric coast into an encounter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FlybyLeg {
    /// Encounter at this body…
    pub body: BodyId,
    /// …at this tick.
    pub encounter_tick: u64,
    /// Flyby periapsis (m).
    pub periapsis_m: f64,
    /// Plane normal.
    pub plane_normal: Vec3,
    /// Turn sign.
    pub turn_sign: i8,
}

/// Chain verification report: per-leg v∞ continuity residuals (the composed
/// prediction check; flown divergence is measured by reconciliation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainReport {
    /// Per-leg: (inbound v∞ magnitude, turn angle, valid).
    pub legs: Vec<(f64, f64, bool)>,
    /// All legs valid?
    pub all_valid: bool,
    /// Honesty tag (chains are inherently patched-conic).
    pub regime: Regime,
}
