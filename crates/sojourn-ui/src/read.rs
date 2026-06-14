//! The read-sync layer (R3, FR-UI-1503). The UI stays in sync by **events + pulled
//! snapshots**: it tracks the last-seen tick and decides when to re-pull (on a tick
//! change, on navigation, or on a throttle), guaranteeing a **consistent snapshot +
//! matching clock** — never a half-updated mix. The actual snapshot reads are typed,
//! in-process calls into the slices via `UiHost::core()`.

/// Tracks when a re-pull is needed so the renderer doesn't recompute every frame.
#[derive(Debug, Clone, Default)]
pub struct ReadSync {
    last_pulled_tick: u64,
    navigated: bool,
}

impl ReadSync {
    /// A fresh sync (forces a first pull).
    pub fn new() -> ReadSync {
        ReadSync {
            last_pulled_tick: u64::MAX,
            navigated: true,
        }
    }

    /// Mark that the player navigated (force a re-pull on the next check).
    pub fn on_navigate(&mut self) {
        self.navigated = true;
    }

    /// Should the UI re-pull snapshots? True on a tick change or after navigation.
    /// The renderer calls this each frame (throttled) and only rebuilds view-models
    /// when it returns true.
    pub fn should_pull(&self, current_tick: u64) -> bool {
        self.navigated || current_tick != self.last_pulled_tick
    }

    /// Record that a consistent pull was taken at `current_tick`.
    pub fn pulled(&mut self, current_tick: u64) {
        self.last_pulled_tick = current_tick;
        self.navigated = false;
    }
}
