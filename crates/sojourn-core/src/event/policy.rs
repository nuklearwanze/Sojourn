//! Per-class pause policies (FR-CORE-403). Defaults come from the data registry;
//! player changes are `SetPausePolicy` commands (journaled).

use serde::{Deserialize, Serialize};

/// What an event class does to time-warp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PausePolicy {
    /// Raise an interrupt: stepping halts at the event's tick until acknowledged.
    Interrupt,
    /// Record and surface without halting.
    LogOnly,
}
