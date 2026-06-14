//! Stored dynamic slice sub-state (data-model §4): faction political state, the
//! milestone ledger, candidate-world astrobiology state (incl. the **hidden** ground
//! truth), planetary-protection state, AI behaviour state and event records. The
//! top-level `PolitySlice` lives in `module.rs`.

use crate::ids::{FactionId, GoalKind};
use crate::inputs::FundingModel;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A faction's political state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactionState {
    /// Funding model (sets which mood modifier applies).
    pub funding_model: FundingModel,
    /// Pairwise relationships (cooperation ↔ rivalry) ∈ [-1, 1].
    pub relationships: BTreeMap<FactionId, f64>,
    /// Partnership/consortium membership.
    pub partnership: BTreeSet<FactionId>,
    /// Accrued prestige.
    pub prestige: f64,
    /// Current public/political mood ∈ [mood_min, mood_max].
    pub mood: f64,
    /// Baseline mood it decays toward.
    pub mood_baseline: f64,
    /// Tick until which crewed-flight approval is frozen (0 = not frozen).
    pub loc_freeze_until_tick: u64,
    /// AI-controlled?
    pub ai: bool,
    /// Selected Grand Goal.
    pub grand_goal: Option<GoalKind>,
    /// Number of goal changes (each takes the penalty).
    pub goal_changes: u32,
}

/// The world-/faction-first claim ledger for one milestone.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MilestoneClaim {
    /// World-first claimant + the tick it was awarded.
    pub world_first: Option<(FactionId, u64)>,
    /// Per-faction faction-first claim ticks.
    pub faction_firsts: BTreeMap<FactionId, u64>,
}

/// A conclusive astrobiology resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    /// Life conclusively present (requires positive ground truth).
    Positive,
    /// Life conclusively absent.
    Negative,
}

/// A candidate world's astrobiology state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateState {
    /// Tier (subsurface/ocean/atmospheric/brine).
    pub tier: crate::inputs::Tier,
    /// The seeded hidden ground truth — **never query-exposed**.
    pub ground_truth: bool,
    /// Per-faction posteriors in **log-odds** (0 = prior 0.5 after the prior offset).
    pub posteriors: BTreeMap<FactionId, f64>,
    /// Whether any SampleReturn-tier evidence has been collected (conclusive gate).
    pub has_sample_return: bool,
    /// The strongest abiotic-competitor pressure accrued (attenuates pre-conclusive).
    pub abiotic_strength: f64,
    /// The prestige-weighted community consensus probability (cached).
    pub consensus: f64,
    /// Conclusive resolution, once reached.
    pub conclusive: Option<Resolution>,
    /// Pristine astrobiology value ∈ [0,1] (degraded by forward contamination).
    pub pristine_value: f64,
}

/// A body's planetary-protection state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProtectionState {
    /// COSPAR category I–V.
    pub category: u8,
    /// Special Region?
    pub special_region: bool,
    /// Bioburden limit.
    pub bioburden_limit: f64,
    /// Pristine value ∈ [0,1] (shared with the candidate when applicable).
    pub pristine_value: f64,
}

/// A forward-/back-contamination record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContaminationRecord {
    /// Body id.
    pub body: u32,
    /// Forward (true) or back (false) contamination.
    pub forward: bool,
    /// Severity ∈ [0,1].
    pub severity: f64,
    /// Cause description.
    pub cause: String,
    /// Tick.
    pub tick: u64,
}

/// An AI faction's abstracted behaviour state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiState {
    /// Capability ∈ [0, envelope_cap].
    pub capability: f64,
    /// Pending milestone targets (ids).
    pub targets: BTreeSet<u32>,
}

/// An emitted-event record (the feed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    /// Event class.
    pub class: String,
    /// Tick.
    pub tick: u64,
    /// Faction (if attributable).
    pub faction: Option<FactionId>,
    /// Interrupt vs log.
    pub interrupt: bool,
}
