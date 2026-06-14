//! The shared multiplicative-hazard primitive (R12, FR-LSC-808) and the capability
//! product (R11, Q3). Every seeded event probability (ECLSS failure, anomaly, EDL
//! crew-risk) composes as `clamp(base × ∏ factors)`; the crew-capability metric is
//! `clamp(∏ per-state factors)`.

/// A multiplicative-hazard probability: `clamp(base × ∏ factors, 0, 1)`.
pub fn hazard(base: f64, factors: &[f64]) -> f64 {
    let p = factors.iter().fold(base, |acc, f| acc * f);
    p.clamp(0.0, 1.0)
}

/// The capability product: `clamp(∏ factors, 0, 1)`.
pub fn capability(factors: &[f64]) -> f64 {
    factors.iter().fold(1.0, |acc, f| acc * f).clamp(0.0, 1.0)
}
