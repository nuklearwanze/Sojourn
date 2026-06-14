//! The base composition + module commissioning state (SLICE, data-model §4). A
//! base owns only its modules + commissioning; emergent properties are pure
//! query-time derivations (R3).

use crate::ids::{BaseId, FactionId, ModuleId, ModuleTypeId, ProjectId, SiteId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A module instance within a base.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleInstance {
    /// Identity.
    pub id: ModuleId,
    /// Catalogue type.
    pub type_id: ModuleTypeId,
    /// Operational (contributes to emergent properties)?
    pub commissioned: bool,
    /// Mass delivered so far (kg).
    pub delivered_mass_kg: f64,
    /// Crew-time applied so far (hours).
    pub crew_time_hr: f64,
    /// Mass satisfied by on-site production (vs import).
    pub built_local: bool,
}

/// A base or orbital station (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Base {
    /// Identity.
    pub id: BaseId,
    /// Owner.
    pub faction: FactionId,
    /// The world Site / dynamical location it sits at.
    pub site: SiteId,
    /// Class-template id.
    pub class: String,
    /// Modules by id.
    pub modules: BTreeMap<ModuleId, ModuleInstance>,
    /// Next module id.
    pub next_module: u32,
    /// The construction project (if open).
    pub project: Option<ProjectId>,
}

impl Base {
    /// Commissioned modules' catalogue type ids.
    pub fn commissioned_types(&self) -> impl Iterator<Item = &ModuleTypeId> {
        self.modules
            .values()
            .filter(|m| m.commissioned)
            .map(|m| &m.type_id)
    }
}
