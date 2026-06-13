//! Capital facilities & ground segment (data-model §9, R14, FR-EC-701/702/703).
//! Owned/leased assets with capex/opex/capacity/level whose capacity **gates the
//! rate** of the activity they enable; ground-segment facilities size the finite
//! ops/comms pool.

use crate::ids::{FacilityId, FactionId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a facility does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FacilityKind {
    /// R&D laboratory.
    Lab,
    /// Engine/thermal-vac/structural test stand.
    TestStand,
    /// Manufacturing/integration line (gates build throughput).
    ProductionLine,
    /// Launch pad (gates launch cadence).
    Pad,
    /// Launch range (scheduling + political constraint).
    Range,
    /// Mission control (gates ops capacity).
    MissionControl,
    /// Deep-space network antennas (gates comms/ops capacity).
    DeepSpaceNetwork,
    /// Relay constellation (gates comms capacity).
    Relay,
}

impl FacilityKind {
    /// Whether this facility contributes to the ops/comms pool.
    pub fn is_ground_segment(self) -> bool {
        matches!(
            self,
            FacilityKind::MissionControl | FacilityKind::DeepSpaceNetwork | FacilityKind::Relay
        )
    }
}

/// A facility template (DATA, `facilities.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityDef {
    /// Stable id.
    pub id: String,
    /// What it does.
    pub kind: FacilityKind,
    /// Capital cost to build (Funds).
    pub capex: f64,
    /// Operating cost (Funds/day).
    pub opex_per_day: f64,
    /// Capacity per level (units of the gated activity).
    pub capacity: f64,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FacilitiesFile {
    facilities: Vec<FacilityDef>,
}

/// The loaded, validated facility templates (module data).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FacilityDefs {
    /// Templates by id.
    pub by_id: BTreeMap<String, FacilityDef>,
    /// Canonical source text (for the econ content hash).
    pub canonical: String,
}

impl FacilityDefs {
    /// Parse + validate `facilities.ron`.
    pub fn from_source(text: &str) -> Result<FacilityDefs, String> {
        let f: FacilitiesFile = ron::from_str(text).map_err(|e| format!("facilities.ron: {e}"))?;
        let mut by_id = BTreeMap::new();
        for d in f.facilities {
            if d.source.trim().is_empty() {
                return Err(format!("facility '{}' has empty source", d.id));
            }
            if d.capex < 0.0 || d.opex_per_day < 0.0 || d.capacity < 0.0 {
                return Err(format!("facility '{}': negative param", d.id));
            }
            if by_id.insert(d.id.clone(), d).is_some() {
                return Err("duplicate facility template".to_string());
            }
        }
        Ok(FacilityDefs {
            by_id,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// A template by id.
    pub fn get(&self, id: &str) -> Option<&FacilityDef> {
        self.by_id.get(id)
    }
}

/// A commissioned facility instance (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Facility {
    /// Identity.
    pub id: FacilityId,
    /// Owner.
    pub faction: FactionId,
    /// The template id.
    pub template: String,
    /// What it does.
    pub kind: FacilityKind,
    /// Level (≥ 1; capacity scales with it).
    pub level: u32,
    /// Capacity per level (from the template).
    pub capacity_per_level: f64,
}

impl Facility {
    /// Effective capacity (capacity_per_level × level).
    pub fn capacity(&self) -> f64 {
        self.capacity_per_level * f64::from(self.level)
    }
}
