//! Seeded randomness with named hierarchical sub-streams (FR-CORE-202, research R2).
//!
//! Every stream's seed is `blake3_keyed(key(master_seed), path)` — identity is the
//! *name*, never the position, so adding a consumer can never shift an existing
//! stream (edge case "stream exhaustion / new consumers"). Stream states serialize
//! with the world state, so saves capture the exact random future.

use rand_chacha::ChaCha12Rng;
use rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Canonical stream name, e.g. `"synthetic/churn"` or `"research/breakthrough/A4"`.
pub type StreamPath = String;

/// All random streams of a run. Owned by the kernel; modules reach their declared
/// streams through `StepCtx::rng` only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RngRegistry {
    master_seed: u64,
    streams: BTreeMap<StreamPath, ChaCha12Rng>,
}

impl RngRegistry {
    /// New registry for a run.
    pub fn new(master_seed: u64) -> Self {
        RngRegistry {
            master_seed,
            streams: BTreeMap::new(),
        }
    }

    /// The run's master seed.
    pub fn master_seed(&self) -> u64 {
        self.master_seed
    }

    /// Borrow (lazily creating) the named stream.
    ///
    /// Lazy creation is deterministic: the stream's state depends only on
    /// (master seed, name, draws made on *this* stream).
    pub fn stream(&mut self, path: &str) -> &mut ChaCha12Rng {
        if !self.streams.contains_key(path) {
            self.streams
                .insert(path.to_string(), derive_stream(self.master_seed, path));
        }
        self.streams.get_mut(path).expect("just inserted")
    }
}

/// Derive a stream from the master seed and the stream's canonical name.
fn derive_stream(master_seed: u64, path: &str) -> ChaCha12Rng {
    // Key the hash with a digest of the master seed, then hash the name:
    // 256 bits of name-stable, seed-dependent stream identity.
    let key = blake3::hash(&master_seed.to_le_bytes());
    let digest = blake3::keyed_hash(key.as_bytes(), path.as_bytes());
    ChaCha12Rng::from_seed(*digest.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::RngCore;

    #[test]
    fn streams_are_name_stable() {
        let mut a = RngRegistry::new(42);
        let mut b = RngRegistry::new(42);
        // Create streams in different orders; values must be identical.
        let x1 = a.stream("alpha").next_u64();
        let _ = a.stream("beta").next_u64();
        let _ = b.stream("beta").next_u64();
        let x2 = b.stream("alpha").next_u64();
        assert_eq!(x1, x2, "stream identity is the name, not creation order");
    }

    #[test]
    fn streams_are_isolated() {
        let mut a = RngRegistry::new(7);
        let mut b = RngRegistry::new(7);
        // Draw heavily from one stream in `a` only; the other stream is unaffected.
        for _ in 0..1000 {
            a.stream("noisy").next_u64();
        }
        assert_eq!(
            a.stream("quiet").next_u64(),
            b.stream("quiet").next_u64(),
            "one consumer's draw count never shifts another's values"
        );
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = RngRegistry::new(1);
        let mut b = RngRegistry::new(2);
        assert_ne!(a.stream("s").next_u64(), b.stream("s").next_u64());
    }

    #[test]
    fn state_serde_round_trip_continues_identically() {
        let mut a = RngRegistry::new(99);
        for _ in 0..17 {
            a.stream("s").next_u64();
        }
        let bytes = postcard::to_allocvec(&a).unwrap();
        let mut restored: RngRegistry = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(a.stream("s").next_u64(), restored.stream("s").next_u64());
    }
}
