//! The generic project / resource-delivery accounting primitive (data-model §11,
//! R15, FR-EC-808) — the **Slice 7 seam**. FA-06 owns the delivery *accounting*
//! (what was delivered, what remains, completion); Slice 7 (Bases & Construction)
//! consumes it to assemble bases/stations from modules. No assembly logic here.

use crate::ids::{CommodityId, FactionId, ProjectId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Project lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectState {
    /// Opened, nothing delivered.
    Open,
    /// Some deliveries landed.
    InProgress,
    /// All requirements met.
    Complete,
}

/// A delivery-driven project (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Identity.
    pub id: ProjectId,
    /// Owner.
    pub faction: FactionId,
    /// Where it is built.
    pub target_node: String,
    /// Required commodities (id, amount).
    pub required: BTreeMap<CommodityId, f64>,
    /// Required crew-time (hours).
    pub crew_time_req: f64,
    /// Delivered so far.
    pub delivered: BTreeMap<CommodityId, f64>,
    /// Crew-time applied so far.
    pub crew_delivered: f64,
    /// State.
    pub state: ProjectState,
}

impl Project {
    /// Record a delivery and recompute state.
    pub fn deliver(&mut self, commodity: &CommodityId, amount: f64) {
        *self.delivered.entry(commodity.clone()).or_insert(0.0) += amount;
        self.recompute();
    }

    /// Apply crew-time and recompute state.
    pub fn apply_crew(&mut self, hours: f64) {
        self.crew_delivered += hours;
        self.recompute();
    }

    fn recompute(&mut self) {
        let materials_done = self
            .required
            .iter()
            .all(|(c, &need)| self.delivered.get(c).copied().unwrap_or(0.0) + 1e-9 >= need);
        let crew_done = self.crew_delivered + 1e-9 >= self.crew_time_req;
        self.state = if materials_done && crew_done {
            ProjectState::Complete
        } else if self.delivered.is_empty() && self.crew_delivered == 0.0 {
            ProjectState::Open
        } else {
            ProjectState::InProgress
        };
    }
}
