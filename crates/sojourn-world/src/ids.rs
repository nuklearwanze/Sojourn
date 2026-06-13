//! BodyId partitioning + the collision-free generated-id allocator (R12,
//! FR-WORLD-502). The `BodyId` (u32) space is split: real catalogued bodies take
//! ids strictly below [`GENERATED_ID_BASE`] (assigned by the build tool from
//! designation); prospecting-generated bodies take ids at or above it from a
//! persisted monotonic counter. Disjoint ranges + monotonic allocation ⇒ no
//! collisions across saves and replays, with no runtime lookup.

use serde::{Deserialize, Serialize};
use sojourn_astro::BodyId;

/// First `BodyId` reserved for prospecting-generated bodies. Real catalogued
/// bodies MUST take ids strictly below this (the build tool enforces it).
pub const GENERATED_ID_BASE: u32 = 1 << 31;

/// Persisted monotonic allocator for generated-body ids (lives in the world
/// slice; serialized, so replay reproduces identical ids).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneratedIds {
    /// Next offset above [`GENERATED_ID_BASE`] to assign.
    pub next: u32,
}

impl GeneratedIds {
    /// Allocate the next collision-free generated id.
    pub fn alloc(&mut self) -> BodyId {
        let id = BodyId(GENERATED_ID_BASE.saturating_add(self.next));
        self.next += 1;
        id
    }

    /// Is this id in the generated range?
    pub fn is_generated(id: BodyId) -> bool {
        id.0 >= GENERATED_ID_BASE
    }
}
