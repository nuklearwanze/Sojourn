//! Kernel-managed scheduling (FR-CORE-104/402, clarified 2026-06-12): module
//! cadences and fine-step escalation are declared in manifests; the kernel derives
//! the resolution schedule from simulation state alone and advances efficiently
//! through quiet spans — "increment until something matters" — without ever
//! skipping a due occurrence or a time-based watch threshold.

pub mod cadence;

pub use cadence::{escalated, next_active_tick, next_due_tick};
