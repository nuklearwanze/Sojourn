//! Identity types. Faction/program/person ids are allocated u32s; domain and tech
//! ids are stable data keys (e.g. `"A1"`, `"B1.5"`), kept as transparent strings so
//! data files read naturally and `BTreeMap` keys stay ordered (determinism).

use serde::{Deserialize, Serialize};

/// Opaque faction identity (FA-09 binds real factions later; callers supply ids).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

/// Knowledge-domain data key (`"A1"`…`"A17"`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DomainId(pub String);

/// Technology data key (`"B1.5"`, `"H3"`…).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TechId(pub String);

/// Engineering-program identity (allocated, monotonic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProgramId(pub u32);

/// Personnel identity (allocated, monotonic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PersonId(pub u32);
