//! # sojourn-base — Bases & Construction (FA-07)
//!
//! Settlement as the **sum of physical truths**: a base or orbital station is
//! composed from modules (habitat, power, ECLSS, greenhouse, ISRU host, science,
//! storage, manufacturing, shielding) sited at a world Site or dynamical
//! location, and its properties **emerge from physics** — power margin, ECLSS
//! closure fraction, population capacity, radiation shielding (a mass-attenuation
//! exponential model), the limiting-factor self-sufficiency index, hazard
//! exposure — never a hand-set level, every number traceable to sourced
//! module/site inputs (Principles II/VIII). A base is built by a **construction
//! project** whose modules commission only as delivered mass and crew-time land,
//! and **on-site regolith construction** turns local material into shielding that
//! is never launched — the path to self-sufficiency, tested by an analytic
//! resupply-embargo check (the Homestead condition).
//!
//! Built as a [`sojourn_core::SimModule`] depending **only on `sojourn-core`**:
//! Site facts, module-tech maturity, and construction-delivery/ISRU-output status
//! flow in as **composed values** ([`inputs`]) the host assembles, so the slice
//! is unit-testable with stubs and structurally cannot reach hidden truth or
//! instant-place. Ordered stores, libm-only transcendentals, zero random streams
//! (deterministic derivations), no wall-clock. Contracts:
//! `specs/008-bases-construction/contracts/`.

#![forbid(unsafe_code)]

pub mod base;
pub mod catalogue;
pub mod construction;
pub mod ids;
pub mod inputs;
pub mod lifesupport;
pub mod module;
pub mod power;
pub mod production;
pub mod query;
pub mod shielding;
pub mod siting;
pub mod sustainability;
pub mod trace;

pub use ids::{BaseId, FactionId, ModuleId, ModuleTypeId, ProjectId, SiteId};
pub use inputs::{BaseInputs, DeliveryStatus, IsruOutput, PpCategory, SiteFacts, TechMaturity};
pub use module::{BaseCommand, BaseModule, base_payload};
pub use query::BaseSnapshot;

/// Errors raised by the base slice.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BaseError {
    /// A referenced base does not exist.
    #[error("unknown base {0}")]
    UnknownBase(u32),
    /// A referenced module does not exist.
    #[error("unknown module {0} in base {1}")]
    UnknownModule(u32, u32),
    /// A referenced module type is not in the catalogue.
    #[error("unknown module type '{0}'")]
    UnknownModuleType(String),
}
