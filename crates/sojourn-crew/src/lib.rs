//! # sojourn-crew — Life Support & Crew (FA-08)
//!
//! The slice that makes **crewed missions materially harder than robotic ones** —
//! the dynamic, time-evolving model FA-07 deferred to it. A crewed asset (a
//! vehicle in transit or an occupied base) consumes O₂/water/food over time
//! against its ECLSS closure (air/water recycled; food open-loop); each crew
//! member accumulates radiation dose (continuous GCR + seeded SPE storms,
//! shielding-attenuated) with a **REID** career limit; micro-g deconditioning and
//! psychological load build, reducing a **multiplicative** crew-capability metric;
//! ECLSS hardware degrades and can fail; and EDL carries a seeded crew-risk roll.
//! Every stochastic outcome is a **multiplicative hazard** (`base × ∏ factors`),
//! so a low-maturity, under-maintained, high-psych craft *earns* its failure
//! probability. **Loss-of-crew is a real, modelled consequence**; its political
//! fallout is FA-09's.
//!
//! The project's first genuinely **dynamic, seeded-stream slice**: per-crew/
//! per-asset health state is **stored** slice state evolved on the daily step with
//! named seeded streams; derived figures (REID, capability, viability) are pure
//! functions over that state. Depends **only on `sojourn-core`**: vehicle/base
//! sizing, the crew roster + age/sex, ECLSS-tech maturity and ops/light-time are
//! **captured into the slice at command time** as composed values (the
//! FA-04/06/07 decoupling). Ordered stores, libm-only, no wall-clock. Contracts:
//! `specs/009-life-support-crew/contracts/`.

#![forbid(unsafe_code)]

pub mod asset;
pub mod consumables;
pub mod eclss;
pub mod edl;
pub mod hazard;
pub mod ids;
pub mod inputs;
pub mod module;
pub mod params;
pub mod physiology;
pub mod psychology;
pub mod query;
pub mod radiation;
pub mod trace;

pub use ids::{AssetId, AstronautId, FactionId, MissionId};
pub use inputs::{AssetSizing, AstronautFacts, EdlSuitability, EnvFacts, Sex, TechMaturity};
pub use module::{CrewCommand, CrewModule, crew_payload};
pub use query::CrewSnapshot;

/// Errors raised by the crew slice.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CrewError {
    /// A referenced asset does not exist.
    #[error("unknown crewed asset {0}")]
    UnknownAsset(u32),
    /// A referenced astronaut is not seated on the asset.
    #[error("unknown astronaut '{0}'")]
    UnknownAstronaut(String),
}
