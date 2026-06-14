//! The seeded, state-driven event scheduler (R5, FR-PEA-401…405). Each named source
//! is evaluated daily as a Bernoulli draw with a **multiplicative-hazard** probability
//! over composed state, so a less-mature faction earns a strictly higher technical-
//! event rate. The daily draws + emission live in `module.rs`; this owns the pure
//! probability.

use crate::hazard;
use crate::params::EventDef;

/// Per-faction technical-risk factor from the composed science-tide level: a lower
/// maturity (tide) earns a higher risk. Clamped to a sane band.
pub fn risk_factor(tide_level: f64) -> f64 {
    (2.0 - tide_level).clamp(0.5, 3.0)
}

/// True for event classes whose probability scales with technical maturity/risk.
fn technical(class: &str) -> bool {
    matches!(class, "anomaly" | "launch-failure")
}

/// The daily probability of an event of `def` for a faction at the given risk and
/// difficulty event-rate multiplier (multiplicative hazard, clamped).
pub fn event_prob(def: &EventDef, risk: f64, event_rate_mult: f64) -> f64 {
    let factors = if technical(&def.class) { [risk] } else { [1.0] };
    hazard::hazard(def.base_rate * event_rate_mult, &factors)
}
