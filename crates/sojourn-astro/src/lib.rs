//! # sojourn-astro — Astrodynamics & Flight (FA-02)
//!
//! The physical source of truth for where everything is and how it moves: an
//! authoritative deterministic propagator (multi-body gravity from railed
//! celestial bodies, J2, solar radiation pressure, drag, continuous low thrust)
//! plus the analytic planning tier (conic predictions, Lambert/porkchop, flyby
//! chaining, low-thrust estimates, research-gated low-energy routes, aerocapture
//! corridors) delivered through a read-only planning-query surface.
//!
//! Built as a `SimModule` on the FA-01 kernel: state-driven step tiering (warp
//! invariant), named random streams for execution error, single-writer slice
//! ownership, libm-only transcendentals. No top speed, no reactionless motion:
//! every state change follows from integrated forces or journaled commands.
//!
//! Contracts: `specs/003-astrodynamics/contracts/` (planning queries, propulsion
//! interface for FA-04, body catalogue for FA-03, commands & events).

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod bodies;
pub mod craft;
pub mod forces;
pub mod frames;
pub mod integrator;
pub mod maneuver;
pub mod math;
pub mod module;
pub mod planner;
pub mod propulsion;
pub mod soi;

pub use bodies::{AtmosphereDef, BodyDef, BodyId, BodyMotion, Catalog};
pub use craft::{CraftId, FlightStatus, MassState};
pub use maneuver::AstroCommand;
pub use math::{Elements, StateVec, Vec3};
pub use module::{AstroConfig, AstroModule, astro_payload};
pub use planner::query::{self as queries, AstroSnapshot};
pub use propulsion::{EngineDef, Engines, PropulsionEndpoint};
