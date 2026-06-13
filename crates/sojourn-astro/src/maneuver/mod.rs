//! Manoeuvres: typed commands, nodes & plans, finite-burn state, seeded
//! execution error, station-keeping schedules, and the TCM targeter
//! (FR-ASTRO-301…307, contracts/astro-commands-events.md).

use crate::bodies::BodyId;
use crate::craft::{ArcEnd, CraftId, GuidanceLaw};
use crate::math::{StateVec, Vec3, prn_basis};
use serde::{Deserialize, Serialize};

/// The typed command set carried in `Command::ModulePayload { module: "astro" }`
/// (postcard-encoded). Structurally invalid payloads (decode failures, non-finite
/// values) are deterministic rejections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstroCommand {
    /// Create a craft (fixtures/scenarios; FA-04 will own production spawning).
    SpawnCraft {
        /// Identifier-style name.
        name_id: String,
        /// Dominant body the state is relative to.
        dominant: u32,
        /// Body-centred inertial state.
        state: StateVec,
        /// Dry mass (kg).
        dry_mass: f64,
        /// Propellant (kg).
        propellant: f64,
        /// Fixture engine id.
        engine: String,
        /// Available power (W).
        available_power_w: f64,
        /// Drag area (m²).
        drag_area_m2: f64,
        /// SRP area (m²).
        srp_area_m2: f64,
    },
    /// Remove a craft.
    DespawnCraft {
        /// Target craft.
        craft: u64,
    },
    /// Create a manoeuvre node.
    CreateNode {
        /// Craft.
        craft: u64,
        /// Burn epoch (tick).
        epoch_tick: u64,
        /// Δv in prograde/radial/normal components (m/s).
        dv_prn: Vec3,
    },
    /// Edit an uncommitted node.
    EditNode {
        /// Node id.
        node: u64,
        /// New epoch.
        epoch_tick: u64,
        /// New Δv.
        dv_prn: Vec3,
    },
    /// Delete an uncommitted node.
    DeleteNode {
        /// Node id.
        node: u64,
    },
    /// Commit a chain of nodes as the craft's plan (schedules kernel events).
    CommitPlan {
        /// Craft.
        craft: u64,
        /// Node ids in burn order (must belong to the craft).
        nodes: Vec<u64>,
        /// Optional aim point for reconciliation/TCM.
        aim: Option<AimPoint>,
    },
    /// Set commanded throttle.
    SetThrottle {
        /// Craft.
        craft: u64,
        /// Throttle [0, 1].
        throttle: f64,
    },
    /// Start a continuous-thrust guidance arc.
    SetGuidanceArc {
        /// Craft.
        craft: u64,
        /// Steering law.
        law: GuidanceLaw,
        /// End condition.
        end: ArcEnd,
    },
    /// Clear the active guidance arc.
    ClearGuidanceArc {
        /// Craft.
        craft: u64,
    },
    /// Schedule recurring station-keeping (FR-ASTRO-306).
    ScheduleStationKeeping {
        /// Craft.
        craft: u64,
        /// Burn cadence (ticks).
        cadence_ticks: u64,
        /// Δv budget per burn (m/s).
        budget_dv: f64,
        /// Hold reference in the rotating frame of (primary, secondary).
        rotating_pair: (u32, u32),
    },
    /// Cancel station-keeping.
    CancelStationKeeping {
        /// Craft.
        craft: u64,
    },
    /// Mark an intended aero pass (suppresses the entry handoff; FR-ASTRO-407).
    PlanAeroPass {
        /// Craft.
        craft: u64,
        /// Atmosphere body.
        body: u32,
        /// Window start (tick).
        start_tick: u64,
        /// Window end (tick).
        end_tick: u64,
    },
    /// Divert a small body off its rail with an applied impulse (FR-ASTRO-107).
    DivertBody {
        /// Divertible body.
        body: u32,
        /// Velocity impulse (m/s).
        impulse: Vec3,
    },
    /// Re-rail a diverted body on its current path.
    ReRailBody {
        /// Diverted body.
        body: u32,
    },
    /// Open/close a research gate (FA-05 input stand-in; FR-ASTRO-406).
    SetResearchGate {
        /// Gate id (e.g. "low-energy").
        gate: String,
        /// Open?
        open: bool,
    },
}

/// Reconciliation aim point: arrive near `body` at `epoch_tick` within
/// `target_radius_m` of its centre.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AimPoint {
    /// Target body.
    pub body: BodyId,
    /// Arrival epoch (tick).
    pub epoch_tick: u64,
    /// Acceptable closest-approach radius (m).
    pub target_radius_m: f64,
}

/// A manoeuvre node (impulsive plan; flown as a finite burn).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManeuverNode {
    /// Node id.
    pub id: u64,
    /// Owning craft.
    pub craft: CraftId,
    /// Burn epoch (tick).
    pub epoch_tick: u64,
    /// Δv in prograde/radial/normal components (m/s).
    pub dv_prn: Vec3,
    /// Committed into the craft's plan?
    pub committed: bool,
    /// Executed?
    pub executed: bool,
}

/// A committed plan: ordered nodes + optional aim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    /// Node ids in burn order.
    pub nodes: Vec<u64>,
    /// Aim point for reconciliation/TCM.
    pub aim: Option<AimPoint>,
}

/// An in-progress finite burn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveBurn {
    /// Node being flown.
    pub node: u64,
    /// World-frame unit thrust direction (fixed at ignition, incl. exec error).
    pub direction: Vec3,
    /// Δv still to deliver (m/s).
    pub remaining_dv: f64,
}

/// A station-keeping schedule (rotating-frame position hold).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StationKeeping {
    /// Cadence (ticks).
    pub cadence_ticks: u64,
    /// Δv budget per burn (m/s).
    pub budget_dv: f64,
    /// Rotating-frame pair (primary, secondary).
    pub rotating_pair: (BodyId, BodyId),
    /// Hold target in the rotating frame (m).
    pub reference_rot: Vec3,
    /// Next burn tick.
    pub next_tick: u64,
}

/// A planned aero pass window (entry-handoff exemption + violation tracking).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AeroPassPlan {
    /// Atmosphere body.
    pub body: BodyId,
    /// Window (ticks, inclusive).
    pub start_tick: u64,
    /// Window end.
    pub end_tick: u64,
    /// Minimum altitude seen during the pass (m; for violation detection).
    pub min_altitude_m: f64,
    /// Violation already reported?
    pub violated: bool,
}

/// World-frame Δv vector for a node given the craft state at ignition.
pub fn dv_world(node_dv_prn: Vec3, state_at_ignition: &StateVec) -> Vec3 {
    let (p, r, n) = prn_basis(state_at_ignition);
    p * node_dv_prn.x + r * node_dv_prn.y + n * node_dv_prn.z
}

/// Apply seeded execution error to an ignition (FR-ASTRO-304): magnitude factor
/// ~ N(1, σ_mag); pointing deflected by |N(0, σ_point)| at a uniform azimuth.
/// Draws exactly four values from the stream; zero-effect when disabled.
pub fn apply_execution_error(
    direction: Vec3,
    dv: f64,
    sigma_mag: f64,
    sigma_point: f64,
    enabled: bool,
    rng: &mut impl FnMut() -> f64, // uniform [0,1) draws from the module stream
) -> (Vec3, f64) {
    if !enabled {
        return (direction, dv);
    }
    // Box–Muller (libm): two gaussians from two uniforms.
    let (u1, u2) = (rng().max(1e-12), rng());
    let mag_g = libm::sqrt(-2.0 * libm::log(u1)) * libm::cos(2.0 * core::f64::consts::PI * u2);
    let (u3, u4) = (rng().max(1e-12), rng());
    let point_g = libm::sqrt(-2.0 * libm::log(u3)) * libm::cos(2.0 * core::f64::consts::PI * u4);

    let dv_out = (dv * (1.0 + sigma_mag * mag_g)).max(0.0);
    // Deflect by |point_g|·σ_point about an axis perpendicular to `direction`,
    // azimuth from u4 (reused uniformly — deterministic and documented).
    let ref_axis = if direction.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let perp = direction.cross(ref_axis).normalized();
    let azimuth = 2.0 * core::f64::consts::PI * u4;
    let axis = perp.rotate_about(direction, azimuth);
    let dir_out = direction.rotate_about(axis, libm::fabs(point_g) * sigma_point);
    (dir_out.normalized(), dv_out)
}

/// Differential-correction TCM targeter (FR-ASTRO-305, research R10): solve a
/// small burn at `burn_state` (conic-predicted craft state at the TCM epoch)
/// that moves the conic-predicted position at the aim epoch onto `target_pos`.
/// Pure and deterministic; bounded iterations. Returns the world-frame Δv.
pub fn solve_tcm_dv(
    mu: f64,
    burn_state: &StateVec,
    coast_s: f64,
    target_pos: Vec3,
    iterations: u32,
) -> Option<Vec3> {
    let mut dv = Vec3::ZERO;
    let predict = |dv: Vec3| -> Option<Vec3> {
        let s = StateVec {
            r: burn_state.r,
            v: burn_state.v + dv,
        };
        crate::math::universal_propagate(mu, &s, coast_s).map(|f| f.r)
    };
    for _ in 0..iterations {
        let p0 = predict(dv)?;
        let miss = target_pos - p0;
        if miss.norm() < 1.0 {
            break;
        }
        // Finite-difference sensitivity ∂(arrival)/∂(dv): 3 probes.
        let h = 0.01; // m/s probe
        let px = predict(dv + Vec3::new(h, 0.0, 0.0))?;
        let py = predict(dv + Vec3::new(0.0, h, 0.0))?;
        let pz = predict(dv + Vec3::new(0.0, 0.0, h))?;
        let jx = (px - p0) * (1.0 / h);
        let jy = (py - p0) * (1.0 / h);
        let jz = (pz - p0) * (1.0 / h);
        // Solve J · d = miss by Cramer's rule (columns jx, jy, jz).
        let det = jx.dot(jy.cross(jz));
        if libm::fabs(det) < 1e-12 {
            return None;
        }
        let d = Vec3::new(
            miss.dot(jy.cross(jz)) / det,
            miss.dot(jz.cross(jx)) / det,
            miss.dot(jx.cross(jy)) / det,
        );
        dv = dv + d;
        if d.norm() < 1e-6 {
            break;
        }
    }
    if dv.non_finite() { None } else { Some(dv) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_error_disabled_is_identity() {
        let mut rng = move || 0.5;
        let (d, dv) =
            apply_execution_error(Vec3::new(0.0, 1.0, 0.0), 100.0, 0.01, 0.01, false, &mut rng);
        assert_eq!(d, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(dv, 100.0);
    }

    #[test]
    fn tcm_targets_a_simple_intercept() {
        let mu = 3.986004418e14;
        let r0 = 7.0e6;
        let v0 = libm::sqrt(mu / r0);
        let s = StateVec {
            r: Vec3::new(r0, 0.0, 0.0),
            v: Vec3::new(0.0, v0, 0.0),
        };
        // Where would we be in 2000 s, and where do we WANT to be (50 km off)?
        let natural = crate::math::universal_propagate(mu, &s, 2000.0).unwrap();
        let target = natural.r + Vec3::new(5.0e4, 0.0, 0.0);
        let dv = solve_tcm_dv(mu, &s, 2000.0, target, 8).unwrap();
        let corrected = crate::math::universal_propagate(
            mu,
            &StateVec {
                r: s.r,
                v: s.v + dv,
            },
            2000.0,
        )
        .unwrap();
        assert!(
            (corrected.r - target).norm() < 100.0,
            "TCM lands within 100 m of target"
        );
        assert!(dv.norm() < 50.0, "correction is small");
    }
}
