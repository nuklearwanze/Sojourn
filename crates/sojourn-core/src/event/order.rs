//! The documented deterministic intra-tick total order (FR-CORE-205/404).
//!
//! Within one tick, all outcome-affecting work happens in this fixed sequence, and
//! event ids are assigned in exactly this order:
//!
//! 1. **Command application** — pending commands in `CommandId` order; their
//!    outcome events (e.g. `command-rejected`) enter the tick queue immediately.
//! 2. **Scheduled events due at this tick** — in original scheduling order
//!    (insertion order within the tick's bucket).
//! 3. **Module steps** — modules due this tick, in the registry's derived
//!    deterministic order; each module's emissions in its own emission order.
//! 4. **Queue delivery** — the tick queue is processed FIFO; subscriber
//!    `on_event` reactions may append further events, which are processed in
//!    append order (bounded by the tick's natural fan-out).
//! 5. **View publication** — stepped/reacting modules publish; kernel view updates.
//! 6. **Watch evaluation** — armed watches in `WatchId` order; `watch-fired`
//!    events enter the queue and are delivered (one watch pass per tick).
//! 7. **Horizon check** — `end-of-horizon` (exactly once) enters the queue last.
//!
//! Ties cannot occur: every producer has a unique position in this sequence, and
//! within a producer the emission sequence is itself deterministic. No platform,
//! hash or insertion-address ordering participates anywhere.
//!
//! A decision and an event that would invalidate it landing on the same tick
//! resolve per step 1: commands apply first, so the event sees the post-command
//! world; a command whose target vanished in a *previous* tick resolves as a
//! deterministic `Rejected` outcome (FR-CORE-304).

/// Rank of a producer in the documented order (used in diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProducerRank {
    /// Step 1.
    Command = 1,
    /// Step 2.
    Scheduled = 2,
    /// Steps 3–4.
    Module = 3,
    /// Step 6.
    Watch = 4,
    /// Step 7.
    Kernel = 5,
}
