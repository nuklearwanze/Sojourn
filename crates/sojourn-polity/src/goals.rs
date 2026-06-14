//! Grand Goals (R9, FR-PEA-801/802): a deterministic **pass/fail** verdict per goal
//! computed from composed inputs. Pure functions over `GoalParams`; `module.rs`/
//! `query.rs` gather the composed inputs.

use crate::ids::GoalKind;
use crate::params::GoalParams;
use serde::{Deserialize, Serialize};

/// A Grand-Goal verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// The primary condition is met.
    Pass,
    /// The primary condition is not (yet) met at the horizon.
    Fail,
    /// Not yet resolved (before the horizon).
    Pending,
}

/// Inputs a Grand-Goal verdict reads (all composed/derived).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GoalInputs {
    /// Exploration firsts claimed (Pathfinder).
    pub firsts: u32,
    /// Embargo-survival index (Homestead).
    pub embargo_index: f64,
    /// Profitable off-Earth tonnage (Prospector).
    pub tonnage: f64,
    /// Conclusively-resolved candidate worlds (Seeker).
    pub conclusive_worlds: u32,
}

/// True iff the goal's primary condition is met by the inputs.
pub fn passes(goal: GoalKind, inp: &GoalInputs, p: &GoalParams) -> bool {
    match goal {
        GoalKind::Pathfinder => inp.firsts >= p.pathfinder_firsts,
        GoalKind::Homestead => inp.embargo_index >= p.homestead_index,
        GoalKind::Prospector => inp.tonnage >= p.prospector_tonnage,
        GoalKind::Seeker => inp.conclusive_worlds >= p.seeker_worlds,
    }
}

/// The verdict: `Pending` before the horizon; `Pass`/`Fail` at/after it.
pub fn verdict(goal: GoalKind, inp: &GoalInputs, p: &GoalParams, at_horizon: bool) -> Verdict {
    if passes(goal, inp, p) {
        Verdict::Pass
    } else if at_horizon {
        Verdict::Fail
    } else {
        Verdict::Pending
    }
}

/// Progress ∈ [0,1] toward the goal (for the composite score).
pub fn progress(goal: GoalKind, inp: &GoalInputs, p: &GoalParams) -> f64 {
    let r = match goal {
        GoalKind::Pathfinder => inp.firsts as f64 / p.pathfinder_firsts as f64,
        GoalKind::Homestead => inp.embargo_index / p.homestead_index,
        GoalKind::Prospector => inp.tonnage / p.prospector_tonnage,
        GoalKind::Seeker => inp.conclusive_worlds as f64 / p.seeker_worlds as f64,
    };
    r.clamp(0.0, 1.0)
}
