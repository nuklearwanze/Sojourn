//! Final scoring (R9, FR-PEA-803/804): the Grand-Goal **pass/fail** verdict is the
//! primary result; a **secondary composite score** (prestige + milestones + goal
//! progress) ranks the run and breaks ties. Soft-fail states are flags, not game-over.

use crate::params::GoalParams;
use serde::{Deserialize, Serialize};

/// A soft-fail state (the run continues in observer/rebuild mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftFail {
    /// Agency budget collapsed to caretaker level.
    AgencyGutted,
    /// Private faction went bankrupt.
    Bankrupt,
    /// Crewed-program loss-of-crew spiral.
    LocSpiral,
}

/// The secondary composite score: sourced-weighted blend of prestige + milestone
/// total + Grand-Goal progress, normalised.
pub fn composite(
    prestige: f64,
    milestone_total: f64,
    goal_progress: f64,
    p: &GoalParams,
    normalisation: f64,
) -> f64 {
    let raw = p.prestige_weight * prestige
        + p.milestone_weight * milestone_total
        + p.goal_weight * goal_progress * normalisation;
    raw / normalisation.max(1.0)
}

/// Apply the goal-change penalty (a fraction off the composite) `changes` times.
pub fn apply_change_penalty(score: f64, changes: u32, p: &GoalParams) -> f64 {
    let factor = libm::pow(1.0 - p.change_penalty.clamp(0.0, 1.0), changes as f64);
    score * factor
}
