//! Identity & enum types (data-model §1). Newtypes over `u32`/`String` so stores
//! are ordered and keys are explicit.

use serde::{Deserialize, Serialize};

/// A faction (organisation) id — one of the ten (player + nine AI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FactionId(pub u32);

/// A catalogued historic-first id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MilestoneId(pub u32);

/// A Solar-System body id (matches the FA-03 catalogue; e.g. Mars=4, Europa=111).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BodyId(pub u32);

/// A candidate-world astrobiology target (a `BodyId` carrying a prior).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CandidateId(pub BodyId);

/// A policy/treaty lever key (e.g. `"nuclear-launch"`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PolicyId(pub String);

/// An event-class / event-source key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventClassId(pub String);

/// The four selectable Grand Goals (win/scoring conditions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GoalKind {
    /// Accumulate exploration firsts & science.
    Pathfinder,
    /// A settlement that survives a 5-year resupply embargo.
    Homestead,
    /// An in-space economy: profitable off-Earth tonnage.
    Prospector,
    /// Resolve the astrobiology question for ≥ 3 candidate worlds.
    Seeker,
}
