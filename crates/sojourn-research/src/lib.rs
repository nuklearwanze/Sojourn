//! # sojourn-research — Research & Personnel (FA-05)
//!
//! Research as a *modelled process, not a purchase*. Two interacting tracks:
//!
//! * **Track A — Science**: continuous **Understanding Levels** (0–100) across the
//!   A1–A17 knowledge domains, with diminishing returns and cross-domain synergy.
//!   UL gates engineering and sets its risk floor.
//! * **Track B — Engineering**: **Technologies** advanced through **TRL 1–9** via
//!   domain-UL-gated **test campaigns** with cost/schedule uncertainty and overruns.
//!
//! On that spine: per-seed **dead ends** (reachability guaranteed by construction —
//! no seed bricks a capability category), **failures-that-teach**, rare seeded-and-
//! earned **breakthroughs**, **leapfrogging**, the global science **tide**, the
//! **personnel** roster (with traits, recruitment, training, tacit-knowledge) and
//! the **astronaut career pipeline**. It exposes a pure, read-only **maturity /
//! heritage / understanding** query surface — the contract FA-04 consumes for a
//! scalar per-use reliability, FA-06 prices, FA-09 turns into prestige.
//!
//! Built as a [`sojourn_core::SimModule`] depending only on the kernel (research is
//! physics-independent): ordered stores, libm-only transcendentals, named seeded
//! streams, time-stepped at a fixed warp-invariant cadence. Contracts:
//! `specs/005-research-personnel/contracts/`.

#![forbid(unsafe_code)]

pub mod astronaut;
pub mod breakthrough;
pub mod campaigns;
pub mod domains;
pub mod ids;
pub mod module;
pub mod personnel;
pub mod programs;
pub mod query;
pub mod reliability;
pub mod rp_de;
pub mod seeding;
pub mod tide;
pub mod tree;

pub use ids::{DomainId, FactionId, PersonId, ProgramId, TechId};
pub use module::{ResearchCommand, ResearchModule, research_payload};
pub use query::ResearchSnapshot;
