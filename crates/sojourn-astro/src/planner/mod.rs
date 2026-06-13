//! The analytic planning tier (FR-ASTRO-401…409): instant, explicitly
//! approximate, regime-tagged predictions — reconciled against the propagated
//! truth, never replacing it. Everything here is pure (no mutation, no streams,
//! no journal participation; FR-ASTRO-409).

pub mod aero;
pub mod flyby;
pub mod lambert;
pub mod lowenergy;
pub mod lowthrust;
pub mod porkchop;
pub mod query;
pub mod reconcile;
pub mod twobody;

use serde::{Deserialize, Serialize};

/// Planner honesty tag (FR-ASTRO-402): predictions in regimes the conic tier
/// models poorly are flagged, never silently wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Regime {
    /// Two-body/patched-conic assumptions hold well here.
    WellConditioned,
    /// Multi-body / near-boundary territory: treat as an estimate.
    LowConfidence,
}
