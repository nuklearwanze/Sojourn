//! Identity types (data-model §1). String-keyed ids reference the world's Site
//! catalogue and the research tech ids without importing those crates (dep core
//! only, R1).

use serde::{Deserialize, Serialize};

/// Faction owner (aligns with world/research/economy faction id).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

/// A base/station instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BaseId(pub u32);

/// A module instance within a base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ModuleId(pub u32);

/// A catalogue module-type id (transparent).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModuleTypeId(pub String);

/// A world Site or dynamical-location id (transparent; references FA-03).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SiteId(pub String);

/// A construction-project handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProjectId(pub u32);
