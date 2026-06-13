//! Shipment lifecycle + reusable logistics assets (data-model §4, R4/R6). A
//! shipment is a deterministic **timed transfer** over a graph edge: it waits for
//! a window, debits the right budget, and delivers after the time-of-flight.

use crate::ids::{CommodityId, FactionId, ShipmentId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What kind of vehicle is flying the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleKind {
    /// A launch vehicle (surface→orbit).
    Launcher,
    /// A reusable tug.
    Tug,
    /// A cycler on a fixed recurring path.
    Cycler,
    /// A generic expendable transfer stage.
    Generic,
}

/// A vehicle assigned to an edge (capacity basis carried from an FA-04 design).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vehicle {
    /// Propellant aboard (kg).
    pub propellant_kg: f64,
    /// Δv capacity (m/s).
    pub dv_capacity_mps: f64,
    /// Payload capacity (kg).
    pub payload_kg: f64,
    /// Reusable (returns to the tug pool on arrival)?
    pub reusable: bool,
    /// Kind.
    pub kind: VehicleKind,
}

/// Shipment lifecycle states (R4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipmentState {
    /// Created, awaiting feasibility/window.
    Ordered,
    /// Feasible but waiting for a transfer window.
    WaitingWindow,
    /// Departed, in flight.
    InTransit,
    /// Arrived and credited.
    Delivered,
    /// Rejected (insufficient Δv/capacity/window) — carries a reason.
    Rejected,
}

/// A cargo movement over one edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shipment {
    /// Identity.
    pub id: ShipmentId,
    /// Owner.
    pub faction: FactionId,
    /// The graph edge id traversed.
    pub edge: String,
    /// Destination node id (for crediting on arrival).
    pub to_node: String,
    /// Cargo (commodity, kg/units).
    pub cargo: Vec<(CommodityId, f64)>,
    /// The assigned vehicle.
    pub vehicle: Vehicle,
    /// Tick the shipment departs (set when leaving `WaitingWindow`).
    pub depart_tick: u64,
    /// Tick the shipment arrives.
    pub arrive_tick: u64,
    /// Current state.
    pub state: ShipmentState,
    /// A human reason when `Rejected`/`WaitingWindow`.
    pub note: String,
}

/// A buffer depot holding stocks at a node (decouples production from transport).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Depot {
    /// The node it sits at.
    pub node: String,
    /// What it holds.
    pub holds: BTreeMap<CommodityId, f64>,
}

/// A cycler following a fixed recurring path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cycler {
    /// Owner.
    pub faction: FactionId,
    /// Ordered (edge id, period in days) legs.
    pub legs: Vec<(String, f64)>,
}

impl Shipment {
    /// True once the shipment has reached a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            ShipmentState::Delivered | ShipmentState::Rejected
        )
    }
}
