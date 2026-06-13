//! The vehicle design: a composition of components into stages (each with a
//! propulsion mode), optional redundancy blocks, stated mission requirements, and
//! a derivative lineage. Slice state (FR-VD-701/702).

use crate::ids::{ComponentId, DesignId, FactionId};
use serde::{Deserialize, Serialize};

/// One stage: components, its propellant load and its engine (the propulsion mode).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    /// Component ids in this stage (incl. structure/tanks/power/etc.).
    pub components: Vec<ComponentId>,
    /// Propellant load (kg).
    pub propellant_kg: f64,
    /// The engine component providing this stage's mode (None ⇒ inert stage).
    pub engine: Option<ComponentId>,
}

/// A declared redundancy block: these components are parallel-redundant for
/// reliability (block-diagram, FR-VD-501).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedundancyBlock {
    /// The parallel-redundant component ids.
    pub components: Vec<ComponentId>,
}

/// Stated mission requirements the realism guards check against (optional inputs —
/// the guards never invent a requirement).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct MissionReqs {
    /// Required Δv (m/s).
    pub dv: Option<f64>,
    /// Required endurance (days, crewed).
    pub endurance_days: Option<f64>,
    /// Mission duration (days) for endurance/dose checks.
    pub mission_days: Option<f64>,
    /// Target-body surface gravity (m/s²) for the lander T/W check.
    pub target_body_g: Option<f64>,
    /// Crew dose limit over the mission.
    pub dose_limit: Option<f64>,
    /// Expected mission dose (for the shield check).
    pub expected_dose: Option<f64>,
    /// Crew count required.
    pub crew: Option<u32>,
}

/// A vehicle design (slice state).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VehicleDesign {
    /// Identity.
    pub id: DesignId,
    /// Owner.
    pub faction: FactionId,
    /// Class-template id.
    pub class: String,
    /// Stages (ordered; stage 0 fires first / carries the rest).
    pub stages: Vec<Stage>,
    /// Declared redundancy blocks.
    pub redundancy: Vec<RedundancyBlock>,
    /// Stated mission requirements.
    pub mission: MissionReqs,
    /// Derivative parent (heritage lineage).
    pub parent: Option<DesignId>,
    /// Cumulative units produced (learning curve).
    pub production_count: u32,
}

impl VehicleDesign {
    /// All component ids across stages, in order.
    pub fn all_components(&self) -> impl Iterator<Item = &ComponentId> {
        self.stages.iter().flat_map(|s| s.components.iter())
    }
}
