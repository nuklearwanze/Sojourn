//! Policy & treaty levers (R6, FR-PEA-501…505): bounded levels with seeded drift and
//! lobbying, and a gating rule (a required approval is granted at/above the lever's
//! threshold). Pure functions over levels + `PolicyLever`; `module.rs` holds the
//! levels and applies these on commands/step.

use crate::params::PolicyLever;

/// True iff a required approval is granted at this level (≥ the grant threshold).
pub fn granted(level: f64, lever: &PolicyLever) -> bool {
    level >= lever.grant_threshold
}

/// Apply one seeded drift step to a level (nudge in [-drift, +drift]), clamped.
pub fn drift(level: f64, lever: &PolicyLever, u: f64) -> f64 {
    let nudge = lever.drift_per_period * (u * 2.0 - 1.0);
    (level + nudge).clamp(lever.min, lever.max)
}

/// Apply a lobby nudge in the given direction (+1 tighten / -1 relax), clamped.
pub fn lobby(level: f64, lever: &PolicyLever, direction: f64, u: f64) -> f64 {
    // The lobby outcome is seeded: effectiveness scales with a uniform draw.
    let step = lever.lobby_step * direction.signum() * (0.5 + 0.5 * u);
    (level + step).clamp(lever.min, lever.max)
}
