//! # sojourn-world — World Data & Belief State (FA-03)
//!
//! Turns FA-02's idealised test fixture into the **real Solar System** and layers
//! the game's honesty contract on top of it:
//!
//! * the real catalogue (Sun, planets, dwarfs, moons, small bodies) is *data* the
//!   FA-02 propagator loads through the existing body-catalogue contract;
//! * **ground truth** (sourced + seeded) is held strictly apart from per-faction
//!   **belief** (estimate + uncertainty), refined only by journaled, seeded
//!   observation commands whose Gaussian precision-addition model guarantees
//!   information never decreases and converges to documented class floors —
//!   never to truth;
//! * surveyable **sites**, first-class **dynamical locations**, statistical
//!   **prospecting fields** that deterministically mint permanent new bodies, and
//!   the cited **Sojournal** complete the world.
//!
//! Everything reaches other slices and the future UI through a pure, read-only
//! **world-query surface** ([`query::WorldSnapshot`]) that is *structurally
//! incapable of carrying truth* (the honesty contract, Principle VIII).
//!
//! Built as a [`sojourn_core::SimModule`] above `sojourn-astro`: ordered stores,
//! libm-only transcendentals, named seeded streams, no wall-clock. Contracts:
//! `specs/004-world-data/contracts/`.

#![forbid(unsafe_code)]

pub mod belief;
pub mod catalogue;
pub mod ids;
pub mod locations;
pub mod module;
pub mod observe;
pub mod prospect;
pub mod query;
pub mod sites;
pub mod sojournal;
pub mod truth;

pub use ids::{GENERATED_ID_BASE, GeneratedIds};
pub use module::{WorldCommand, WorldModule, world_payload};
pub use query::WorldSnapshot;
