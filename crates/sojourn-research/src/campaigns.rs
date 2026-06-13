//! Test campaigns (FR-RESP-204, R5): the seeded success/failure roll at a TRL-step
//! boundary. A failure costs schedule and **injects UL** (failure-that-teaches);
//! repeated failure without UL growth is the dead-end signal (raised in
//! `programs::advance`).

use crate::tree::Params;

/// Failure probability for a TRL step. Higher for immature (low-TRL) steps and for
/// seeded dead ends; rises with the program's risk index; trait overrun-variance
/// widens it.
pub fn fail_probability(
    params: &Params,
    target_trl: u8,
    risk_index: f64,
    is_dead_end: bool,
    overrun_mult: f64,
) -> f64 {
    // Earlier flight steps (5–7) are riskier than late qual (8–9).
    let trl_factor = match target_trl {
        ..=4 => 1.2,
        5 => 1.4,
        6 => 1.3,
        7 => 1.5, // flight demo
        8 => 0.8,
        _ => 0.5,
    };
    let dead_end_factor = if is_dead_end { 3.0 } else { 1.0 };
    let p = params.test_fail_base * trl_factor * dead_end_factor * (1.0 + risk_index) * overrun_mult;
    p.clamp(0.0, if is_dead_end { 0.98 } else { 0.9 })
}
