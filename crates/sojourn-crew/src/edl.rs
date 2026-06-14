//! EDL & aerocapture crew risk (R10, Q1, FR-LSC-601…603). A multiplicative hazard
//! over vehicle suitability, the target body's difficulty (the Mars gap) and crew
//! state; an EDL failure is a loss-of-crew consequence.

use crate::hazard;
use crate::inputs::EdlSuitability;
use crate::params::EdlParams;

/// The vehicle-suitability multiplier (no heat shield / can't land raise the risk).
fn suitability_factor(s: &EdlSuitability, p: &EdlParams) -> f64 {
    let hs = if s.has_heat_shield {
        1.0
    } else {
        p.no_heatshield_mult
    };
    let land = if s.can_land { 1.0 } else { p.cant_land_mult };
    hs * land
}

/// The per-body EDL difficulty (Mars ≫ airless; the modelled gap).
pub fn body_difficulty(body: &str, p: &EdlParams) -> f64 {
    p.body_difficulty
        .get(body)
        .copied()
        .or_else(|| p.body_difficulty.get("default").copied())
        .unwrap_or(1.0)
}

/// The EDL crew-loss probability (multiplicative hazard, R12): suitability × body ×
/// crew-state. Lower crew capability raises the risk.
pub fn crew_loss_prob(
    suitability: &EdlSuitability,
    body: &str,
    capability: f64,
    p: &EdlParams,
) -> f64 {
    let crew_state = 2.0 - capability.clamp(0.0, 1.0); // 1.0 at full capability → 2.0 at zero
    hazard::hazard(
        p.base_rate,
        &[
            body_difficulty(body, p),
            suitability_factor(suitability, p),
            crew_state,
        ],
    )
}
