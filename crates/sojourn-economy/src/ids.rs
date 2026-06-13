//! Identity types (data-model §1). String-keyed ids reference the world's
//! resource taxonomy and location catalogue without importing those crates
//! (dep core only, R1).

use serde::{Deserialize, Serialize};

/// Faction owner (aligns with the world/research/vehicle faction id).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

/// A tradable good. References FA-03 resource ids for raw materials.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommodityId(pub String);

/// A graph node identity (references an FA-03 world location id or Site key).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocationId(pub String);

/// Interned transport-graph node handle (stable within a save).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// Interned transport-graph edge handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EdgeId(pub u32);

/// A shipment handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShipmentId(pub u32);

/// An ISRU plant handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlantId(pub u32);

/// A contract handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractId(pub u32);

/// A facility handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FacilityId(pub u32);

/// A project handle (the Slice 7 seam).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProjectId(pub u32);

/// Launch-market orbit class (data-extensible label; the launch market prices
/// mass-to-orbit per class).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrbitClass(pub String);
