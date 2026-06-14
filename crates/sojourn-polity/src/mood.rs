//! Public/political mood (R3, FR-PEA-201…205): bounded mood with exponential decay
//! toward a baseline; loss-of-crew is severe and long-lasting; mood drives
//! appropriation/valuation modifiers and approval latency. Pure functions over mood
//! values + `MoodParams`; `module.rs` mutates the stored `FactionState`.

use crate::inputs::CrewFacts;
use crate::params::MoodParams;

/// Clamp a mood value to its bounds.
pub fn clamp(mood: f64, p: &MoodParams) -> f64 {
    mood.clamp(p.mood_min, p.mood_max)
}

/// The mood delta from a composed outcome (positive lifts, failures drop).
pub fn outcome_delta(c: &CrewFacts, p: &MoodParams) -> f64 {
    let mut d = 0.0;
    if c.world_first {
        d += p.world_first_delta;
    }
    if c.success {
        d += p.success_delta;
    }
    if c.routine_failure {
        d += p.routine_failure_delta;
    }
    if c.loss_of_crew {
        d += p.loss_of_crew_delta;
    }
    d
}

/// Exponential decay of mood toward the baseline over `dt_days`.
pub fn decay(mood: f64, baseline: f64, p: &MoodParams, dt_days: f64) -> f64 {
    let k = libm::exp(-p.decay_per_day * dt_days);
    baseline + (mood - baseline) * k
}

/// Appropriation modifier (agencies): `1 + slope × mood`.
pub fn appropriation_factor(mood: f64, p: &MoodParams) -> f64 {
    (1.0 + p.appropriation_slope * mood).max(0.0)
}

/// Valuation modifier (companies): `1 + slope × mood`.
pub fn valuation_factor(mood: f64, p: &MoodParams) -> f64 {
    (1.0 + p.valuation_slope * mood).max(0.0)
}

/// Major-program approval latency (days): lower when mood is high.
pub fn approval_latency_days(mood: f64, p: &MoodParams) -> f64 {
    (p.approval_base_days - mood * p.approval_mood_days).max(0.0)
}
