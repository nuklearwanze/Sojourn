//! Composed-value integration seams (R1/R15, `contracts/integration-seams.md`).
//! Every cross-slice fact enters as a plain, serializable input the host composes
//! from the upstream query surfaces and **captures into the slice at command time**
//! (`InitWorld`/`UpdateWorld`/`RecordAchievement`/`RecordOutcome`/`CollectEvidence`/
//! `EvaluateContamination`) — the FA-06 `OperateIsru` / FA-08 `OccupyAsset` pattern.
//! No upstream gameplay crate is a dependency.

use crate::ids::{BodyId, CandidateId, FactionId, MilestoneId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How a faction is funded (sets which mood modifier applies).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FundingModel {
    /// Appropriation-funded national agency.
    Agency,
    /// Revenue-funded private company.
    Company,
}

/// A candidate world's astrobiology tier (from FA-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    /// Subsurface (e.g. Mars deep aquifers).
    Subsurface,
    /// Subglacial / subsurface ocean (Europa/Enceladus).
    Ocean,
    /// Atmospheric (Titan/Venus clouds).
    Atmospheric,
    /// Briny subsurface (Ceres).
    Brine,
}

/// A staged astrobiology evidence stage (orbital → in-situ → microscopy → return).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceStage {
    /// Orbital biosignature hint (weakest).
    OrbitalHint,
    /// In-situ chemistry (organics, disequilibria).
    InSitu,
    /// Microscopy / metabolism.
    Microscopy,
    /// Sample return & independent confirmation (strongest; conclusive-gating).
    SampleReturn,
}

/// A faction's initial political setup (captured at `InitWorld`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactionInit {
    /// Faction id.
    pub id: FactionId,
    /// Funding model.
    pub funding_model: FundingModel,
    /// Baseline mood ∈ [-1, 1] the faction decays toward.
    pub baseline_mood: f64,
    /// AI-controlled?
    pub ai: bool,
}

/// The composed facts proving a milestone's award conditions (FA-04/06/07/08).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AchievementFacts {
    /// Body involved (if any), as a catalogue id.
    pub body: Option<u32>,
    /// Crewed mission?
    pub crewed: bool,
    /// Fully-reusable vehicle?
    pub reusable: bool,
    /// Propellant sold from in-space production (kg).
    pub propellant_sold_kg: f64,
    /// Off-Earth-produced (ISRU origin)?
    pub isru_origin: bool,
    /// Free-form capability tags the conditions can match (e.g. "ntp-flight").
    pub tags: Vec<String>,
}

impl Default for AchievementFacts {
    fn default() -> AchievementFacts {
        AchievementFacts {
            body: None,
            crewed: false,
            reusable: false,
            propellant_sold_kg: 0.0,
            isru_origin: false,
            tags: Vec::new(),
        }
    }
}

/// A composed achievement claim (US1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Achievement {
    /// Claiming faction.
    pub faction: FactionId,
    /// The milestone claimed.
    pub milestone: MilestoneId,
    /// The proof.
    pub facts: AchievementFacts,
}

/// A candidate-world astrobiology prior, bridged from FA-03 `data/world/astrobiology.ron`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CandidatePrior {
    /// The candidate world.
    pub candidate: CandidateId,
    /// Sourced 2026 plausibility prior of life present.
    pub presence_prob: f64,
    /// Tier.
    pub tier: Tier,
}

/// A staged evidence collection (US3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceInput {
    /// Collecting faction.
    pub faction: FactionId,
    /// Candidate world.
    pub candidate: CandidateId,
    /// Evidence stage.
    pub stage: EvidenceStage,
    /// Instrument quality ∈ [0,1] (scales the evidence strength).
    pub quality: f64,
}

/// A body/site planetary-protection facts bundle, bridged from FA-03 `sites.ron`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SiteProtection {
    /// Body id.
    pub body: BodyId,
    /// COSPAR category I–V.
    pub category: u8,
    /// Special Region (strict forward-contamination rules)?
    pub special_region: bool,
    /// Bioburden limit (spores/m² or equivalent; composed units).
    pub bioburden_limit: f64,
}

/// The facts of a mission that may cause contamination (US6).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MissionFacts {
    /// Target body.
    pub body: BodyId,
    /// Reached a Special Region?
    pub special_region: bool,
    /// The lander's bioburden (composed; compared to the limit).
    pub lander_bioburden: f64,
    /// Was it a crash (uncontrolled impact)?
    pub crash: bool,
    /// A sample-return mission?
    pub sample_return: bool,
    /// A compliant back-contamination containment chain in place?
    pub containment_chain: bool,
}

/// Composed economy facts (FA-06); mood applies modifiers over these.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EconomyFacts {
    /// Faction.
    pub faction: FactionId,
    /// Base appropriation / budget (currency).
    pub base_appropriation: f64,
    /// Base valuation (companies).
    pub base_valuation: f64,
    /// Cumulative profitable off-Earth tonnage (t) — Prospector.
    pub off_earth_tonnage_profit: f64,
}

/// Composed outcome facts driving mood (FA-08 loss-of-crew among them).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CrewFacts {
    /// Faction.
    pub faction: FactionId,
    /// A loss-of-crew occurred.
    pub loss_of_crew: bool,
    /// A routine (robotic) failure occurred.
    pub routine_failure: bool,
    /// A routine success occurred.
    pub success: bool,
    /// A world-first was achieved (spectacular lift).
    pub world_first: bool,
}

/// Composed Homestead embargo-survival facts (FA-07/08).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HomesteadFacts {
    /// Faction.
    pub faction: FactionId,
    /// Embargo-survival index ∈ [0,1] (≥ threshold passes Homestead).
    pub embargo_survival_index: f64,
}

/// The composed global science tide (FA-05).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScienceTide {
    /// Global baseline tech level.
    pub global_level: f64,
    /// Per-faction tech level.
    pub per_faction_level: BTreeMap<FactionId, f64>,
}

impl Default for ScienceTide {
    fn default() -> ScienceTide {
        ScienceTide {
            global_level: 0.0,
            per_faction_level: BTreeMap::new(),
        }
    }
}

/// Difficulty knobs — political/economic harshness + AI tuning, **never physics**.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Difficulty {
    /// AI funding multiplier.
    pub ai_funding_mult: f64,
    /// AI competence multiplier.
    pub ai_competence_mult: f64,
    /// Event base-rate multiplier.
    pub event_rate_mult: f64,
}

impl Default for Difficulty {
    fn default() -> Difficulty {
        Difficulty {
            ai_funding_mult: 1.0,
            ai_competence_mult: 1.0,
            event_rate_mult: 1.0,
        }
    }
}
