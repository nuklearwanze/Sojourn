//! Composed-value integration seams (R2, `contracts/integration-seams.md`). Every
//! cross-slice value enters as a plain, serializable input the host composes from
//! the upstream query surfaces. Per the implementation refinement these are
//! **captured into the slice at command time** (`OccupyAsset`/`AssignCrew`/
//! `UpdateEnv`/`EvaluateEdl`) so the daily step can evolve the stored state — the
//! FA-06 `OperateIsru` pattern.

use serde::{Deserialize, Serialize};

/// Biological sex (for the REID dose-limit model, Q2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    /// Male.
    Male,
    /// Female.
    Female,
}

/// A vehicle/base's static life-support sizing (from FA-04/FA-07).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AssetSizing {
    /// ECLSS air/water closure fraction ∈ [0,1].
    pub closure_capability: f64,
    /// Shielding dose-attenuation factor ∈ (0,1].
    pub shield_attenuation: f64,
    /// Crew the asset accommodates.
    pub population_capacity: u32,
    /// Has spin/artificial gravity?
    pub spin_gravity: bool,
    /// Habitat volume per crew (m³) — confinement.
    pub habitat_volume_m3_per_crew: f64,
    /// On-board consumables capacity (kg).
    pub consumables_capacity_kg: f64,
    /// Crewed (vs robotic — robotic carries no crew constraints).
    pub crewed: bool,
}

/// The asset's environment (from FA-03 / FA-06).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvFacts {
    /// Unshielded GCR dose rate (Sv/yr).
    pub gcr_rate_sv_yr: f64,
    /// Body id (for EDL difficulty + environment).
    pub body: String,
    /// One-way comms light-time (s).
    pub comms_lag_s: f64,
    /// Is an abort/resupply within reach?
    pub abort_reach: bool,
}

/// An astronaut's facts (from the FA-05 roster; age/sex feed REID, Q2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstronautFacts {
    /// Age (years).
    pub age_years: f64,
    /// Sex.
    pub sex: Sex,
    /// Trait tags.
    pub traits: Vec<String>,
    /// Training level ∈ [0,1].
    pub training: f64,
}

/// An ECLSS technology's maturity (from FA-05).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TechMaturity {
    /// Technology readiness level.
    pub trl: u8,
    /// Reliability ∈ [0,1].
    pub reliability: f64,
    /// Flight units accumulated.
    pub flight_units: u32,
}

/// A vehicle's static EDL suitability (from FA-04).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdlSuitability {
    /// Can it land propulsively (T/W ≥ 1)?
    pub can_land: bool,
    /// Carries a heat shield?
    pub has_heat_shield: bool,
    /// T/W at the target gravity.
    pub landing_tw: f64,
}
