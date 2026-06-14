//! The in-process UI host (R3, `contracts/ui-host.md`). `UiHost` owns a `SimCore`
//! with the full module set, drives time-warp stepping, exposes the event/interrupt
//! feed and typed snapshot pulls, and submits typed journalled commands. It holds
//! **no game logic** and **never blocks or alters** the core's deterministic stepping;
//! the same `SimCore` runs headless in the harness.

use crate::command::CommitOutcome;
use crate::modules::build_modules;
use sojourn_core::{
    Command, CommandOutcome, CoreError, DataSet, EventFilter, RunConfig, RunMode, SimCore,
    StepRequest,
};
use std::collections::BTreeMap;
use std::path::Path;

/// A compact, player-facing run status (the clock every snapshot matches).
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    /// Current tick.
    pub tick: u64,
    /// Civil date (year, month, day).
    pub date: (i32, u8, u8),
    /// Total events emitted so far.
    pub total_events: u64,
    /// Pending (un-acknowledged) interrupts.
    pub pending_interrupts: u64,
}

/// A compact view of one event for the feed (FR-UI-1001).
#[derive(Debug, Clone, PartialEq)]
pub struct EventView {
    /// Sequential id (for acknowledge / linking).
    pub id: u64,
    /// Tick.
    pub tick: u64,
    /// Event class.
    pub class: String,
}

/// The in-process host: a `SimCore` plus the read/command façade.
pub struct UiHost {
    core: SimCore,
}

impl UiHost {
    /// Start a fresh game: build the full module set under `data_root` and create the
    /// core. `data_root` is the repo root (holds `data/kernel`, `data/world`, …).
    pub fn new_game(data_root: &Path, seed: u64) -> Result<UiHost, String> {
        let data = DataSet::load_dir(&data_root.join("data/kernel"))
            .map_err(|e| format!("kernel data: {e}"))?;
        let cfg = RunConfig {
            master_seed: seed,
            horizon_years: 100,
            run_mode: RunMode::SaveAnywhere,
            difficulty_inputs: BTreeMap::new(),
        };
        let modules = build_modules(data_root)?;
        let core = SimCore::create(cfg, data, modules).map_err(|e| format!("core: {e}"))?;
        Ok(UiHost { core })
    }

    /// Wrap an already-built core (tests / embedding).
    pub fn from_core(core: SimCore) -> UiHost {
        UiHost { core }
    }

    /// Borrow the core for a typed in-process snapshot pull (e.g. `WorldSnapshot::
    /// from_core`, `CrewSnapshot::from_core`, or a `with_slice` read). The UI consumes
    /// the slices' read-only surfaces directly (FR-UI-1502).
    pub fn core(&self) -> &SimCore {
        &self.core
    }

    /// The current run status (the clock every snapshot matches, FR-UI-1503).
    pub fn status(&self) -> Status {
        let s = self.core.status();
        Status {
            tick: s.tick,
            date: (s.date.year, s.date.month, s.date.day),
            total_events: 0,
            pending_interrupts: s.pending_interrupts.len() as u64,
        }
    }

    /// The most recent events (newest-first), for the feed.
    pub fn recent_events(&self, limit: u64) -> Vec<EventView> {
        let page = self.core.events(&EventFilter {
            limit,
            ..Default::default()
        });
        match page {
            Ok(p) => p
                .events
                .into_iter()
                .map(|e| EventView {
                    id: e.id.0,
                    tick: e.tick,
                    class: e.class,
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Advance the core by `ticks` (the warp-appropriate stride); returns the number
    /// of interrupts now pending. Never blocks beyond this call (FR-UI-1503).
    pub fn advance(&mut self, ticks: u64) -> Result<u64, CoreError> {
        self.core.step(StepRequest::Ticks(ticks))?;
        Ok(self.core.status().pending_interrupts.len() as u64)
    }

    /// Acknowledge a pending interrupt (resume).
    pub fn acknowledge(&mut self, id: u64) -> Result<(), CoreError> {
        self.core.acknowledge(id)?;
        Ok(())
    }

    /// Submit a typed journalled command; surface the outcome (rejection reason
    /// included, never hidden — FR-UI-305).
    pub fn submit(&mut self, cmd: Command) -> CommitOutcome {
        match self.core.submit(cmd) {
            Ok(_) => {
                // apply the command (zero-tick step) so its outcome is realised.
                match self.core.step(StepRequest::Ticks(0)) {
                    Ok(_) => CommitOutcome::Applied,
                    Err(e) => CommitOutcome::Rejected(e.to_string()),
                }
            }
            Err(e) => CommitOutcome::Rejected(e.to_string()),
        }
    }
}

/// Map a core `CommandOutcome` to the UI `CommitOutcome`.
pub fn outcome(o: CommandOutcome) -> CommitOutcome {
    match o {
        CommandOutcome::Applied => CommitOutcome::Applied,
        CommandOutcome::Rejected(r) => CommitOutcome::Rejected(r),
    }
}
