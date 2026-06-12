//! Module execution contexts — the *whole* world a module sees (FR-CORE-502,
//! contracts/module-contract.md §StepCtx). Undeclared access is a loud defect:
//! the kernel panics with the module id and the violation, in every build profile,
//! because a contract violation is a programming bug whose continued execution
//! would silently corrupt determinism guarantees.

use super::ModuleManifest;
use crate::clock::{SimClock, SimTimeNs, Tick, calendar};
use crate::event::{EventSource, PendingEvent};
use crate::rng::RngRegistry;
use crate::state::{RunConfig, ViewId, ViewSnapshot, ViewValue};
use rand_chacha::ChaCha12Rng;
use std::collections::BTreeMap;

/// Context for `SimModule::init` — seeded slice construction.
pub struct InitCtx<'a> {
    pub(crate) manifest: &'a ModuleManifest,
    pub(crate) rng: &'a mut RngRegistry,
    pub(crate) config: &'a RunConfig,
}

impl InitCtx<'_> {
    /// The run configuration (difficulty inputs are opaque pass-through).
    pub fn config(&self) -> &RunConfig {
        self.config
    }

    /// A declared random stream.
    pub fn rng(&mut self, path: &str) -> &mut ChaCha12Rng {
        assert_declared_stream(self.manifest, path);
        self.rng.stream(path)
    }
}

/// Context for `SimModule::step` / `on_event` — clock, declared views, declared
/// streams, declared emissions, future scheduling. Nothing else: no filesystem,
/// no network, no wall-clock, no foreign state.
pub struct StepCtx<'a> {
    pub(crate) manifest: &'a ModuleManifest,
    pub(crate) clock: &'a SimClock,
    pub(crate) rng: &'a mut RngRegistry,
    pub(crate) views: &'a BTreeMap<ViewId, ViewSnapshot>,
    pub(crate) emitted: Vec<PendingEvent>,
    pub(crate) scheduled: Vec<(u64, PendingEvent)>,
}

impl StepCtx<'_> {
    /// Current tick.
    pub fn tick(&self) -> Tick {
        self.clock.tick
    }

    /// Exact simulated time.
    pub fn sim_time(&self) -> SimTimeNs {
        self.clock.now()
    }

    /// Civil calendar date-time (FR-CORE-103 calendar service).
    pub fn date(&self) -> calendar::DateTime {
        self.clock.date()
    }

    /// A declared read view, as of the last completed tick boundary. `None` when
    /// the producer has not published yet (first ticks of a run).
    pub fn view(&self, id: &str) -> Option<&ViewSnapshot> {
        let declared = id == super::KERNEL_STATUS_VIEW
            || self.manifest.reads.iter().any(|r| r == id)
            || self.manifest.publishes.iter().any(|p| p == id);
        assert!(
            declared,
            "module '{}' read undeclared view '{id}' — declare it in manifest.reads \
             (contract violation, FR-CORE-501b)",
            self.manifest.id
        );
        self.views.get(id)
    }

    /// A declared random stream.
    pub fn rng(&mut self, path: &str) -> &mut ChaCha12Rng {
        assert_declared_stream(self.manifest, path);
        self.rng.stream(path)
    }

    /// Emit an event of a declared class at the current tick. Enters the
    /// deterministic tick queue (event/order.rs) and the journal.
    pub fn emit(&mut self, class: &str, payload: BTreeMap<String, ViewValue>) {
        assert!(
            self.manifest.emits.iter().any(|c| c == class),
            "module '{}' emitted undeclared event class '{class}' — declare it in \
             manifest.emits (contract violation, FR-CORE-501c)",
            self.manifest.id
        );
        self.emitted.push(PendingEvent {
            class: class.to_string(),
            source: EventSource::Module(self.manifest.id.clone()),
            payload,
        });
    }

    /// Schedule a declared-class event for a future tick (known-time occurrence:
    /// manoeuvre node due, review date…). Fires in step 2 of the tick order.
    pub fn schedule(&mut self, at: Tick, class: &str, payload: BTreeMap<String, ViewValue>) {
        assert!(
            self.manifest.emits.iter().any(|c| c == class),
            "module '{}' scheduled undeclared event class '{class}' (FR-CORE-501c)",
            self.manifest.id
        );
        assert!(
            at > self.clock.tick,
            "module '{}' scheduled an event at tick {} which is not in the future (now {})",
            self.manifest.id,
            at.0,
            self.clock.tick.0
        );
        self.scheduled.push((
            at.0,
            PendingEvent {
                class: class.to_string(),
                source: EventSource::Module(self.manifest.id.clone()),
                payload,
            },
        ));
    }
}

fn assert_declared_stream(manifest: &ModuleManifest, path: &str) {
    assert!(
        manifest.streams.iter().any(|s| s == path),
        "module '{}' drew from undeclared stream '{path}' — declare it in manifest.streams \
         (contract violation, FR-CORE-501e / FR-CORE-202)",
        manifest.id
    );
}
