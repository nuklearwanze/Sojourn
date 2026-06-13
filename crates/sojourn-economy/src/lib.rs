//! # sojourn-economy — Economy & Logistics (FA-06)
//!
//! The slice where **felt scarcity is real**: a per-faction ledger of the six
//! currencies (Funds, Δv/propellant, mass-to-orbit, crew-time, ops capacity,
//! political capital) and physical resources held as **location-addressed
//! stocks**, moved over a directed transport graph **priced in delta-v and
//! time-of-flight**. Every balance change is a conserved, recorded transaction;
//! money is a proxy the player can trace back to mass × Δv (Principle VII).
//!
//! On top of the ledger and graph sit the two funding models (agency
//! appropriations / private cash-runway with bankruptcy), a P50/P80 cost model
//! with learning curves, ISRU break-even economics with scale-up, a
//! faction-agnostic markets & contracts layer, and capital facilities sizing the
//! finite ops/comms pool. ISRU pays off only when the physics and economics
//! close — never free fuel.
//!
//! Built as a [`sojourn_core::SimModule`] depending **only on `sojourn-core`**:
//! all cross-slice physics (astro edge prices, world grades, vehicle cost, tech
//! maturity) flows in as **composed values** ([`inputs`]) the host assembles, so
//! the slice is unit-testable with stubs and structurally cannot reach hidden
//! truth. Ordered stores, libm-only transcendentals, named seeded streams, no
//! wall-clock. Contracts: `specs/007-economy-logistics/contracts/`.

#![forbid(unsafe_code)]

pub mod commodity;
pub mod contract;
pub mod cost;
pub mod facility;
pub mod funding;
pub mod ids;
pub mod inputs;
pub mod isru;
pub mod ledger;
pub mod market;
pub mod module;
pub mod network;
pub mod ops;
pub mod project;
pub mod query;
pub mod shipment;
pub mod trace;

pub use ids::{
    CommodityId, ContractId, EdgeId, FacilityId, FactionId, LocationId, NodeId, OrbitClass,
    PlantId, ProjectId, ShipmentId,
};
pub use inputs::{EconomyInputs, EdgePrice, GradeBelief, TechMaturity, VehicleCost};
pub use module::{EconomyCommand, EconomyModule, economy_payload};
pub use query::EconomySnapshot;

/// Errors raised by the economy slice.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EconomyError {
    /// A currency debit would drive a balance negative.
    #[error("insufficient {currency} for faction {faction} (need {need}, have {have})")]
    InsufficientBalance {
        /// Faction.
        faction: u32,
        /// Currency name.
        currency: String,
        /// Amount needed (milli-units, for display).
        need: i64,
        /// Amount available.
        have: i64,
    },
    /// A stock debit would drive a location stock negative.
    #[error("insufficient '{commodity}' at '{location}' (need {need}, have {have})")]
    InsufficientStock {
        /// Commodity id.
        commodity: String,
        /// Location id.
        location: String,
        /// Amount needed.
        need: i64,
        /// Amount available.
        have: i64,
    },
}
