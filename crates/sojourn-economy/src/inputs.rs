//! Composed-value integration seams (R2, `contracts/integration-seams.md`). Every
//! cross-slice physics value enters as a plain, serializable input the host
//! assembles from the upstream query surfaces — the FA-04 C1 decoupling. The
//! economy never imports the FA-02/03/04/05 crates; upstream ids are **local
//! aliases** here (U1).

use crate::ids::{CommodityId, FactionId, LocationId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Local alias for an FA-04 design id (the economy never imports the vehicle crate).
pub type DesignId = u32;
/// Local alias for an FA-05 technology id.
pub type TechId = String;

/// A transport edge's price, from the FA-02 analytic planner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdgePrice {
    /// Δv to traverse the edge (m/s).
    pub dv_mps: f64,
    /// Time-of-flight (s).
    pub tof_s: f64,
    /// Tick at which the next transfer window opens.
    pub next_window_tick: u64,
    /// Is a window open at the time the price was taken?
    pub window_open: bool,
}

/// A faction's surveyed belief about a resource's grade, from FA-03 (never truth).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradeBelief {
    /// Resource taxonomy id.
    pub resource_id: String,
    /// Believed grade (fraction ∈ [0,1] or process-specific units).
    pub believed_grade: f64,
    /// Certainty ∈ [0,1].
    pub certainty: f64,
}

/// A design's physical cost & capacity basis, from FA-04 (learning already applied).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VehicleCost {
    /// Unit cost basis (Funds).
    pub unit_cost_basis: f64,
    /// Build time basis (days).
    pub build_days_basis: f64,
    /// Payload capacity (kg).
    pub payload_kg: f64,
    /// Propellant capacity (kg).
    pub propellant_kg: f64,
    /// Δv capacity (m/s).
    pub dv_mps: f64,
}

/// A technology's maturity, from FA-05 (gates ISRU/facility tech & IP licensing).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TechMaturity {
    /// Technology readiness level.
    pub trl: u8,
    /// Domain understanding ∈ [0,1].
    pub understanding: f64,
    /// Flyable (TRL ≥ 6)?
    pub flyable: bool,
}

/// The bundle of composed values the host assembles per `integration-seams.md`.
/// Tests pass stubs; the harness fills from the real upstream modules.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EconomyInputs {
    /// Edge prices keyed by (from, to) location.
    pub edge_prices: BTreeMap<(LocationId, LocationId), EdgePrice>,
    /// Surveyed grades keyed by (faction, location).
    pub grades: BTreeMap<(FactionId, LocationId), GradeBelief>,
    /// Vehicle cost bases keyed by design id.
    pub vehicle_costs: BTreeMap<DesignId, VehicleCost>,
    /// Tech maturities keyed by (faction, tech).
    pub maturities: BTreeMap<(FactionId, TechId), TechMaturity>,
}

impl EconomyInputs {
    /// An edge price for a (from, to) pair, if the host supplied one.
    pub fn edge_price(&self, from: &LocationId, to: &LocationId) -> Option<&EdgePrice> {
        self.edge_prices.get(&(from.clone(), to.clone()))
    }

    /// A faction's believed grade at a location.
    pub fn grade(&self, faction: FactionId, loc: &LocationId) -> Option<&GradeBelief> {
        self.grades.get(&(faction, loc.clone()))
    }

    /// A design's cost/capacity basis.
    pub fn vehicle_cost(&self, design: DesignId) -> Option<&VehicleCost> {
        self.vehicle_costs.get(&design)
    }

    /// A faction's maturity for a technology.
    pub fn maturity(&self, faction: FactionId, tech: &str) -> Option<&TechMaturity> {
        self.maturities.get(&(faction, tech.to_string()))
    }
}

/// Mark a tradable commodity. Re-exported for ergonomic construction in tests.
pub use crate::ids::CommodityId as Commodity;

/// A (commodity, location) stock coordinate helper used across the slice.
pub fn stock_key(commodity: &str, location: &str) -> (CommodityId, LocationId) {
    (
        CommodityId(commodity.to_string()),
        LocationId(location.to_string()),
    )
}
