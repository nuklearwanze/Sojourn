//! Interrupt-and-pause (FR-CORE-404/405). "Pause" is mechanical: the kernel
//! refuses to advance past a tick that raised un-acknowledged interrupts. The host
//! (warp UI, harness) regains control; queries and commands stay fully available;
//! acknowledgement is itself a journaled command.

use serde::{Deserialize, Serialize};

/// A pause demand raised by an interrupt-policy event. Survives save/load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interrupt {
    /// Interrupt id (acknowledgement target).
    pub id: u64,
    /// Event that raised it.
    pub raised_by: super::EventId,
    /// Tick at which it was raised (the world is paused AT this tick).
    pub raised_at: u64,
    /// Event class, for display/filtering without a history lookup.
    pub class: String,
}
