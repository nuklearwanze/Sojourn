//! The stored dynamic crew/asset state (SLICE, data-model §4) — this slice's
//! defining trait (R3). Composed sizing/env/maturity are **captured** onto the
//! asset at command time so the daily step can evolve the stored state.

use crate::ids::{AssetId, AstronautId, FactionId, MissionId};
use crate::inputs::{AssetSizing, AstronautFacts, EnvFacts, TechMaturity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Micro-gravity deconditioning indices (each ∈ [0,1], 0 = baseline).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Deconditioning {
    /// Bone density loss.
    pub bone: f64,
    /// Muscle loss.
    pub muscle: f64,
    /// Cardiovascular deconditioning.
    pub cardio: f64,
    /// Vision (SANS) impairment.
    pub vision: f64,
}

impl Deconditioning {
    /// Mean of the four indices.
    pub fn mean(&self) -> f64 {
        (self.bone + self.muscle + self.cardio + self.vision) / 4.0
    }
}

/// A crew member's status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrewStatus {
    /// On active duty.
    Active,
    /// Grounded (REID ≥ 3%).
    Grounded,
    /// Lost (loss-of-crew).
    Lost,
}

/// A crew member's stored dynamic health record (per FA-05 astronaut id).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewMember {
    /// Identity (FA-05 roster).
    pub id: AstronautId,
    /// The asset they crew.
    pub asset: AssetId,
    /// Captured roster facts (age/sex/traits/training).
    pub facts: AstronautFacts,
    /// Accumulated career radiation dose (Sv).
    pub career_dose_sv: f64,
    /// Deconditioning indices.
    pub decon: Deconditioning,
    /// Psychological load ∈ [0, ∞) (normalised in the capability curve).
    pub psych_load: f64,
    /// Status.
    pub status: CrewStatus,
}

/// ECLSS dynamic state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EclssState {
    /// Captured reliability (from maturity/heritage).
    pub reliability: f64,
    /// Accumulated degradation.
    pub degradation: f64,
    /// Maintenance deficit (rises without crew-time + spares).
    pub maintenance_deficit: f64,
    /// Has failed?
    pub failed: bool,
}

/// A crewed asset (vehicle in transit or occupied base) — SLICE.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewedAsset {
    /// Identity.
    pub id: AssetId,
    /// Owner.
    pub faction: FactionId,
    /// Mission context.
    pub mission: Option<MissionId>,
    /// Captured static sizing (FA-04/FA-07).
    pub sizing: AssetSizing,
    /// Captured environment (FA-03/FA-06).
    pub env: EnvFacts,
    /// Captured ECLSS-tech maturity (FA-05).
    pub eclss_maturity: TechMaturity,
    /// Captured ops oversubscription (FA-06).
    pub ops_oversub: f64,
    /// Consumables stock (kg).
    pub consumables_kg: f64,
    /// ECLSS state.
    pub eclss: EclssState,
    /// Crew sheltering against an SPE?
    pub sheltering: bool,
    /// Maintenance applied today (crew-hr, spares) — reset each step.
    pub maint_crew_hr: f64,
    /// Spares applied today (kg).
    pub maint_spares_kg: f64,
    /// Tick occupied.
    pub occupied_since_tick: u64,
    /// Seated crew.
    pub crew: BTreeSet<AstronautId>,
}
