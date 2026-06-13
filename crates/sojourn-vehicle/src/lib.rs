//! # sojourn-vehicle — Vehicle Designer & Propulsion (FA-04)
//!
//! The tyranny of mass and Δv, felt at design time. A component-composition
//! designer builds vehicles from researched parts and **derives** every number
//! from physics — total/dry/propellant mass, per-stage/mode Δv (the rocket
//! equation), thrust and T/W, power generation/demand, thermal/radiator balance,
//! composed reliability, static life-support sizing, EDL suitability and a physical
//! cost estimate — **with full traceability** to sourced data and **no per-design
//! magic numbers** (Principle II).
//!
//! The propulsion families produce the [`sojourn_astro::EngineDef`] the FA-02
//! propagator flies (electric power-limited; nuclear-electric radiators as
//! first-class mass). Component reliability enters through a **maturity provider**
//! (the real FA-05 `maturity()` in production; a stub in tests) — so the crate
//! depends only on the kernel + astro and is unit-testable in isolation.
//!
//! Built as a [`sojourn_core::SimModule`]: the design library is the only slice
//! state; flight-time craft state stays in FA-02 (a craft is spawned from a design
//! by carrying its engine params inline). Derived outputs are pure query-time
//! computations. Contracts: `specs/006-vehicle-designer/contracts/`.

#![forbid(unsafe_code)]

pub mod catalogue;
pub mod cost;
pub mod deltav;
pub mod design;
pub mod edl;
pub mod guards;
pub mod ids;
pub mod lifesupport;
pub mod mass;
pub mod module;
pub mod power;
pub mod propulsion;
pub mod query;
pub mod reliability;
pub mod thermal;
pub mod trace;

pub use ids::{ComponentId, DesignId, FactionId};
pub use module::{VehicleCommand, VehicleModule, vehicle_payload};
pub use query::{ComponentMaturity, DesignSnapshot};
