//! Craft state (the slice's primary store): dominant-frame state vectors, mass
//! accounting, propulsion binding, flight status. Invariants (research R12):
//! craft exert no gravity, never collide with each other, and never change state
//! without an integrated force or a journaled command (FR-ASTRO-104).

use crate::bodies::BodyId;
use crate::math::StateVec;
use serde::{Deserialize, Serialize};

/// Stable craft handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CraftId(pub u64);

/// Mass accounting (kg). Total = dry + propellant; both non-negative always.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MassState {
    /// Dry mass (kg).
    pub dry: f64,
    /// Propellant mass (kg).
    pub propellant: f64,
}

impl MassState {
    /// Total mass.
    pub fn total(&self) -> f64 {
        self.dry + self.propellant
    }
}

/// Terminal-or-active flight status. `Impacted`/`EntryHandoff` are terminal in
/// this slice — the vehicle/EDL slice consumes the handoff (clarified 2026-06-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlightStatus {
    /// Being propagated.
    Propagating,
    /// Intersected a body's surface (terminal here).
    Impacted {
        /// Body hit.
        body: BodyId,
        /// Tick of impact.
        tick: u64,
    },
    /// Crossed an atmosphere interface outside a planned aero pass (EDL handoff).
    EntryHandoff {
        /// Body whose atmosphere was entered.
        body: BodyId,
        /// Tick of entry.
        tick: u64,
    },
}

/// Integration tier (research R2), derived from state each step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    /// Quiet coasting at module cadence.
    Coast,
    /// Near a body / inside an atmosphere: 1-tick steps.
    Encounter,
    /// Impulsive burn in progress: 1-tick steps with fine substeps.
    Burn,
}

/// An active continuous-thrust guidance arc (FR-ASTRO-405).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuidanceArc {
    /// Steering law.
    pub law: GuidanceLaw,
    /// End condition.
    pub end: ArcEnd,
}

/// Steering laws (data-extensible enum).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GuidanceLaw {
    /// Thrust along the velocity vector (spiral out) or against it (inward).
    Tangential {
        /// +1 prograde, −1 retrograde.
        sign: i8,
    },
}

/// Arc termination conditions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArcEnd {
    /// Stop when the dominant-relative radius reaches this value (m).
    TargetRadius(f64),
    /// Stop at this tick.
    AtTick(u64),
}

/// One propagated craft.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Craft {
    /// Identity.
    pub id: CraftId,
    /// Identifier-style name.
    pub name_id: String,
    /// Dominant body (SOI owner); `state` is relative to it.
    pub dominant: BodyId,
    /// Body-centred inertial state.
    pub state: StateVec,
    /// Mass accounting.
    pub mass: MassState,
    /// Fixture engine id (FA-04 endpoint binding later).
    pub engine: String,
    /// Commanded throttle [0, 1].
    pub throttle: f64,
    /// Available electrical power (W) for power-limited engines.
    pub available_power_w: f64,
    /// Drag reference area (m²).
    pub drag_area_m2: f64,
    /// SRP reference area (m²).
    pub srp_area_m2: f64,
    /// Active guidance arc, if any.
    pub guidance: Option<GuidanceArc>,
    /// Flight status.
    pub status: FlightStatus,
}
