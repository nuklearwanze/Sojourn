//! The abstracted AI world (R8, FR-PEA-701…704): a lightweight, seeded heuristic over
//! composed capability estimates — **not** a mirror of FA-04…08. Capability is clamped
//! to the plausibility envelope (no impossible tech); difficulty tunes funding/
//! competence only. Pure functions; `module.rs` drives the per-faction daily step.

use crate::params::AiParams;

/// An AI faction's capability from its composed tide level + difficulty, **clamped to
/// the plausibility envelope** (difficulty never lifts it past the cap).
pub fn capability(tide_level: f64, competence_mult: f64, p: &AiParams) -> f64 {
    let raw = p.base_competence * competence_mult + tide_level;
    raw.clamp(0.0, p.envelope_cap)
}

/// True iff the AI faction is capable enough to pursue/claim a milestone.
pub fn pursues(capability: f64, p: &AiParams) -> bool {
    capability >= p.claim_threshold
}

/// The science-tide advance contributed by one AI faction over `dt_days`.
pub fn tide_advance(funding_mult: f64, p: &AiParams, dt_days: f64) -> f64 {
    p.tide_advance_per_day * funding_mult * dt_days
}
