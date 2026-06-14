//! Identity types (data-model §1). The astronaut id is string-keyed to the FA-05
//! roster without importing that crate (dep core only, R1).

use serde::{Deserialize, Serialize};

/// Faction owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

/// A crewed asset (vehicle in transit or occupied base).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AssetId(pub u32);

/// A crew member (references the FA-05 astronaut roster).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AstronautId(pub String);

/// A mission / occupancy context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MissionId(pub u32);
