//! The shared multiplicative-hazard primitive (R5, FR-PEA-402) — the FA-08 shape.
//! Every seeded event probability composes as `clamp(base × ∏ factors, 0, 1)`, so a
//! riskier configuration strictly earns a higher probability.

/// A multiplicative-hazard probability: `clamp(base × ∏ factors, 0, 1)`.
pub fn hazard(base: f64, factors: &[f64]) -> f64 {
    let p = factors.iter().fold(base, |acc, f| acc * f);
    p.clamp(0.0, 1.0)
}
