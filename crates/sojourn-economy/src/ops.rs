//! The finite ops/comms pool (data-model §9, R14, FR-EC-206). Active craft consume
//! a shared pool sized by ground-segment facilities; **oversubscription** degrades
//! outcomes (reduced data return, raised anomaly probability) — never silently
//! free. Light-time delay is a per-node/edge value.

use serde::{Deserialize, Serialize};

/// The ops/comms pool for a faction (SLICE/DERIVED).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct OpsPool {
    /// Total capacity (from ground-segment facilities).
    pub capacity: f64,
    /// Currently consumed by active craft.
    pub used: f64,
}

impl OpsPool {
    /// Whether the pool is oversubscribed.
    pub fn oversubscribed(&self) -> bool {
        self.used > self.capacity + 1e-9
    }

    /// Fraction over capacity (0 when within capacity).
    pub fn overage_fraction(&self) -> f64 {
        if self.capacity <= 1e-9 {
            if self.used > 0.0 { 1.0 } else { 0.0 }
        } else {
            ((self.used - self.capacity) / self.capacity).max(0.0)
        }
    }

    /// Data-return efficiency ∈ (0,1]: full within capacity, degrading beyond it.
    pub fn data_return_factor(&self) -> f64 {
        1.0 / (1.0 + self.overage_fraction())
    }

    /// Anomaly-probability multiplier ≥ 1, rising with oversubscription.
    pub fn anomaly_multiplier(&self) -> f64 {
        1.0 + self.overage_fraction()
    }
}
