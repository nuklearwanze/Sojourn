//! Journal flush policy (FR-CORE-303): commands and interrupt-raising events sync
//! immediately; everything else is bounded by a wall-clock cadence ≤ 5 s so an
//! abrupt kill loses at most that much progress (SC-004).
//!
//! This file is the kernel's **single documented exception** to the no-wall-clock
//! rule (research R10): the timer below lives entirely in the persistence layer,
//! after state transitions are complete — it can bound durability but can never
//! influence simulation outcomes.

use std::time::Instant;

/// Wall-clock-bounded sync decider.
pub struct FlushPolicy {
    flush_wall_ms: u64,
    // The documented persistence-layer exception (clippy.toml disallows this
    // everywhere else in the workspace).
    #[allow(clippy::disallowed_types)]
    last_sync: Option<Instant>,
}

impl FlushPolicy {
    /// Policy with the configured bound (validated ≤ 5000 in `DataSet::validate`).
    pub fn new(flush_wall_ms: u64) -> Self {
        FlushPolicy {
            flush_wall_ms,
            last_sync: None,
        }
    }

    /// Should we sync now?
    pub fn sync_due(&mut self) -> bool {
        match self.last_sync {
            None => true,
            Some(t) => t.elapsed().as_millis() as u64 >= self.flush_wall_ms,
        }
    }

    /// Record that a sync just happened.
    pub fn mark_synced(&mut self) {
        #[allow(clippy::disallowed_methods)]
        {
            self.last_sync = Some(Instant::now());
        }
    }
}
