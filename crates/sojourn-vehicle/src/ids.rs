//! Identity types. Designs are allocated `u32`s; components and technologies are
//! stable data keys (transparent strings) for natural data files + ordered keys.

use serde::{Deserialize, Serialize};

/// Opaque faction identity (FA-09 binds real factions later).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

/// Component data key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentId(pub String);

/// Design identity (allocated, monotonic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DesignId(pub u32);
