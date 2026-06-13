//! The read-only planning-query surface (FR-ASTRO-409, contracts/
//! planning-queries.md): pure functions over an [`AstroSnapshot`]. No mutation,
//! no random streams, no journal or fingerprint participation — the suite
//! asserts a query between steps leaves the state fingerprint unchanged.

use super::{Regime, aero, flyby, lambert, lowenergy, lowthrust, porkchop, reconcile, twobody};
use crate::bodies::{BodyId, BodyMotion, Catalog, rails::BodyStates};
use crate::craft::Craft;
use crate::maneuver::AimPoint;
use crate::math::{StateVec, Vec3, universal_propagate};
use crate::module::{AstroConfig, AstroSlice};
use crate::propulsion::Engines;
use crate::soi::soi_radius;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A cheap, owned snapshot of everything queries need (research R14): craft
/// states, body motion overrides, gates, config, plus catalogue/engine handles.
#[derive(Debug, Clone)]
pub struct AstroSnapshot {
    /// Snapshot tick.
    pub tick: u64,
    /// Snapshot time (ns).
    pub t_ns: i64,
    /// Craft by id.
    pub craft: BTreeMap<u64, Craft>,
    /// Small-body motion overrides.
    pub motions: BTreeMap<BodyId, BodyMotion>,
    /// Research gates.
    pub gates: BTreeMap<String, bool>,
    /// Catalogue (clone; immutable data).
    pub catalog: Catalog,
    /// Engines (clone; immutable data).
    pub engines: Engines,
    /// Config (clone).
    pub config: AstroConfig,
}

impl AstroSnapshot {
    /// Extract a snapshot from a running kernel at the current tick boundary
    /// (the host-facing constructor; uses the kernel's read-only slice access).
    pub fn from_core(
        core: &sojourn_core::SimCore,
        module: &crate::module::AstroModule,
    ) -> Result<AstroSnapshot, sojourn_core::CoreError> {
        let tick = core.status().tick;
        core.with_slice("astro", |slice| {
            let s = slice
                .as_any()
                .downcast_ref::<AstroSlice>()
                .expect("astro slice type");
            AstroSnapshot::extract(s, &module.catalog, &module.engines, &module.config, tick)
        })
    }

    /// Extract a snapshot from the slice at a completed tick boundary.
    pub fn extract(
        slice: &AstroSlice,
        catalog: &Catalog,
        engines: &Engines,
        config: &AstroConfig,
        tick: u64,
    ) -> AstroSnapshot {
        AstroSnapshot {
            tick,
            t_ns: tick as i64 * 1_000_000_000,
            craft: slice.craft.clone(),
            motions: slice.motions.clone(),
            gates: slice.gates.clone(),
            catalog: catalog.clone(),
            engines: engines.clone(),
            config: config.clone(),
        }
    }
}

/// One sampled trajectory point (planner-tier conic prediction).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrajectorySample {
    /// Time offset from the snapshot (s).
    pub t_offset_s: f64,
    /// Dominant-relative position (m).
    pub r: Vec3,
}

/// Remaining delta-v budget by the rocket equation (FR-ASTRO-302).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DvBudget {
    /// Total achievable Δv from current mass state (m/s).
    pub dv_total: f64,
    /// Exhaust velocity (m/s).
    pub exhaust_velocity: f64,
    /// Propellant (kg).
    pub propellant_kg: f64,
}

/// Node feasibility verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Feasibility {
    /// Affordable within the budget.
    Ok {
        /// Propellant the burn will consume (kg).
        propellant_kg: f64,
    },
    /// Exceeds the achievable Δv.
    InsufficientDeltaV {
        /// Requested (m/s).
        requested: f64,
        /// Achievable (m/s).
        achievable: f64,
    },
    /// Craft unknown or not propagating.
    Invalid {
        /// Reason.
        reason: String,
    },
}

/// Orbit summary for a craft (FR-ASTRO-401).
pub fn orbit_summary(snap: &AstroSnapshot, craft: u64) -> Option<twobody::OrbitSummary> {
    let c = snap.craft.get(&craft)?;
    let mu = snap.catalog.body(c.dominant)?.mu;
    let soi = soi_radius(&snap.catalog, c.dominant);
    Some(twobody::orbit_summary(mu, &c.state, snap.t_ns, soi))
}

/// Sampled conic trajectory prediction (dominant frame).
pub fn predict_trajectory(
    snap: &AstroSnapshot,
    craft: u64,
    span_s: f64,
    samples: u32,
) -> Vec<TrajectorySample> {
    let Some(c) = snap.craft.get(&craft) else {
        return Vec::new();
    };
    let Some(body) = snap.catalog.body(c.dominant) else {
        return Vec::new();
    };
    let n = samples.clamp(2, 4096);
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let t = span_s * f64::from(i) / f64::from(n - 1);
        if let Some(s) = universal_propagate(body.mu, &c.state, t) {
            out.push(TrajectorySample {
                t_offset_s: t,
                r: s.r,
            });
        }
    }
    out
}

/// Rocket-equation budget for a craft.
pub fn dv_budget(snap: &AstroSnapshot, craft: u64) -> Option<DvBudget> {
    let c = snap.craft.get(&craft)?;
    let def = snap.engines.get(&c.engine)?;
    let ve = def.exhaust_velocity();
    let m0 = c.mass.total();
    let m1 = c.mass.dry;
    Some(DvBudget {
        dv_total: ve * libm::log(m0 / m1.max(1e-9)),
        exhaust_velocity: ve,
        propellant_kg: c.mass.propellant,
    })
}

/// Feasibility of an additional Δv on a craft (after `already_planned` m/s).
pub fn node_feasibility(
    snap: &AstroSnapshot,
    craft: u64,
    dv_mps: f64,
    already_planned: f64,
) -> Feasibility {
    let Some(c) = snap.craft.get(&craft) else {
        return Feasibility::Invalid {
            reason: format!("unknown craft {craft}"),
        };
    };
    if c.status != crate::craft::FlightStatus::Propagating {
        return Feasibility::Invalid {
            reason: "craft is not propagating".into(),
        };
    }
    let Some(budget) = dv_budget(snap, craft) else {
        return Feasibility::Invalid {
            reason: "no engine".into(),
        };
    };
    let total = dv_mps + already_planned;
    if total > budget.dv_total {
        return Feasibility::InsufficientDeltaV {
            requested: total,
            achievable: budget.dv_total,
        };
    }
    // Propellant for this burn after the already-planned ones (rocket equation).
    let ve = budget.exhaust_velocity;
    let m_before = c.mass.total() * libm::exp(-already_planned / ve);
    let m_after = m_before * libm::exp(-dv_mps / ve);
    Feasibility::Ok {
        propellant_kg: m_before - m_after,
    }
}

/// Predict the dominant-frame state after a chain of (coast_s, dv_world) legs.
pub fn predict_with_burns(
    snap: &AstroSnapshot,
    craft: u64,
    legs: &[(f64, Vec3)],
) -> Option<(StateVec, Regime)> {
    let c = snap.craft.get(&craft)?;
    let body = snap.catalog.body(c.dominant)?;
    let soi = soi_radius(&snap.catalog, c.dominant);
    twobody::predict_chain(body.mu, &c.state, legs, soi)
}

/// Porkchop grid (FR-ASTRO-403).
pub fn porkchop_grid(snap: &AstroSnapshot, q: &porkchop::PorkchopQuery) -> porkchop::PorkchopGrid {
    porkchop::solve(
        &snap.catalog,
        &snap.motions,
        q,
        snap.config.lambert_max_iter,
        snap.config.lambert_tol,
    )
}

/// Departure Δv for a porkchop cell (world frame, at the departure body state).
pub fn porkchop_departure_dv(
    snap: &AstroSnapshot,
    q: &porkchop::PorkchopQuery,
    cell: &porkchop::PorkchopCell,
) -> Option<Vec3> {
    porkchop::cell_departure_dv(
        &snap.catalog,
        &snap.motions,
        q,
        cell,
        snap.config.lambert_max_iter,
        snap.config.lambert_tol,
    )
}

/// Single-encounter flyby solve (FR-ASTRO-404).
pub fn solve_flyby(
    snap: &AstroSnapshot,
    q: &flyby::FlybyQuery,
) -> Option<flyby::EncounterSolution> {
    flyby::solve(&snap.catalog, q)
}

/// Verify a manual assist chain: v∞-continuity + validity per leg.
pub fn verify_chain(
    snap: &AstroSnapshot,
    start_v_inf: Vec3,
    legs: &[flyby::FlybyLeg],
) -> flyby::ChainReport {
    let mut v_inf = start_v_inf;
    let mut out = Vec::with_capacity(legs.len());
    let mut all_valid = true;
    for leg in legs {
        let q = flyby::FlybyQuery {
            body: leg.body,
            v_inf_in: v_inf,
            periapsis_m: leg.periapsis_m,
            plane_normal: leg.plane_normal,
            turn_sign: leg.turn_sign,
        };
        match flyby::solve(&snap.catalog, &q) {
            Some(sol) => {
                out.push((v_inf.norm(), sol.turn_angle_rad, sol.valid));
                all_valid &= sol.valid;
                v_inf = sol.v_inf_out;
            }
            None => {
                out.push((v_inf.norm(), 0.0, false));
                all_valid = false;
            }
        }
    }
    flyby::ChainReport {
        legs: out,
        all_valid,
        regime: Regime::LowConfidence,
    }
}

/// Low-thrust spiral estimate for a craft between circular radii.
pub fn lowthrust_arc(
    snap: &AstroSnapshot,
    craft: u64,
    r_target: f64,
) -> Option<lowthrust::ArcEstimate> {
    let c = snap.craft.get(&craft)?;
    let def = snap.engines.get(&c.engine)?;
    let mu = snap.catalog.body(c.dominant)?.mu;
    let r1 = c.state.r.norm();
    let thrust = crate::propulsion::delivered_thrust(
        &crate::propulsion::EngineBinding {
            def,
            mass: &mut c.mass.clone(),
            throttle: c.throttle,
            available_power_w: c.available_power_w,
            drag_area_m2: c.drag_area_m2,
            srp_area_m2: c.srp_area_m2,
        },
        if c.throttle > 0.0 { c.throttle } else { 1.0 },
    );
    lowthrust::spiral_estimate(
        mu,
        r1,
        r_target,
        thrust,
        def.exhaust_velocity(),
        c.mass.total(),
    )
}

/// Research-gated low-energy routes (FR-ASTRO-406).
pub fn lowenergy_routes(
    snap: &AstroSnapshot,
    from: BodyId,
    to: BodyId,
) -> Vec<lowenergy::RouteEstimate> {
    lowenergy::routes(&snap.gates, from, to)
}

/// Aerocapture corridor (FR-ASTRO-407).
pub fn aero_corridor(snap: &AstroSnapshot, q: &aero::AeroQuery) -> Option<aero::AeroCorridor> {
    aero::corridor(&snap.catalog, q)
}

/// Reconciliation report for a craft against an aim point (FR-ASTRO-402).
pub fn reconcile_report(
    snap: &AstroSnapshot,
    craft: u64,
    aim: &AimPoint,
) -> Option<reconcile::ReconciliationReport> {
    let c = snap.craft.get(&craft)?;
    let mut states = BodyStates::new(&snap.catalog, &snap.motions, snap.t_ns);
    let dom = states.absolute(c.dominant);
    let abs = StateVec {
        r: c.state.r + dom.r,
        v: c.state.v + dom.v,
    };
    reconcile::reconcile(
        &snap.catalog,
        &snap.motions,
        &abs,
        snap.tick,
        aim,
        snap.config.divergence_threshold_m,
    )
}

/// TCM solve (FR-ASTRO-305): a world-frame correction Δv at the snapshot time
/// targeting the aim body (+ target radius along the current approach).
pub fn solve_tcm(snap: &AstroSnapshot, craft: u64, aim: &AimPoint) -> Option<Vec3> {
    let c = snap.craft.get(&craft)?;
    if aim.epoch_tick <= snap.tick {
        return None;
    }
    let mu = snap.catalog.body(snap.catalog.root())?.mu;
    let mut states = BodyStates::new(&snap.catalog, &snap.motions, snap.t_ns);
    let dom = states.absolute(c.dominant);
    let abs = StateVec {
        r: c.state.r + dom.r,
        v: c.state.v + dom.v,
    };
    let aim_ns = aim.epoch_tick as i64 * 1_000_000_000;
    let mut aim_states = BodyStates::new(&snap.catalog, &snap.motions, aim_ns);
    let target = aim_states.absolute(aim.body).r;
    let coast = (aim.epoch_tick - snap.tick) as f64;
    crate::maneuver::solve_tcm_dv(mu, &abs, coast, target, 10)
}

/// Named Lagrange-region locations for a pair (FR-ASTRO-203).
pub fn lagrange_points(
    snap: &AstroSnapshot,
    primary: BodyId,
    secondary: BodyId,
) -> Vec<reconcile::LagrangePoint> {
    reconcile::lagrange_points(&snap.catalog, primary, secondary)
}

/// Absolute position of a named Lagrange point at the snapshot time.
pub fn lagrange_position(
    snap: &AstroSnapshot,
    primary: BodyId,
    secondary: BodyId,
    point: &reconcile::LagrangePoint,
) -> Vec3 {
    reconcile::lagrange_position(
        &snap.catalog,
        &snap.motions,
        primary,
        secondary,
        point,
        snap.t_ns,
    )
}

/// Lambert solve between arbitrary heliocentric positions (exposed for tests
/// and future tooling).
pub fn lambert_solve(
    snap: &AstroSnapshot,
    r1: Vec3,
    r2: Vec3,
    tof_s: f64,
    revs: u8,
) -> Option<lambert::LambertSolution> {
    let mu = snap.catalog.body(snap.catalog.root())?.mu;
    lambert::solve(
        mu,
        r1,
        r2,
        tof_s,
        revs,
        snap.config.lambert_max_iter,
        snap.config.lambert_tol,
    )
}
