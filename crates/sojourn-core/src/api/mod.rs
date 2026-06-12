//! The public boundary (contracts/core-api.md): `SimCore` — create, step, query,
//! submit, persist, replay. Owned serde DTOs only; queries answer between step
//! calls at completed tick boundaries; the host owns pacing (FR-CORE-105/204).

use crate::clock::{SimClock, SimTimeNs, Tick, calendar};
use crate::command::{Command, CommandEnvelope, CommandId, CommandOutcome, CommandReceipt};
use crate::data::{DataResolver, DataSet, WatchTemplateDef};
use crate::error::CoreError;
use crate::event::interrupt::Interrupt;
use crate::event::policy::PausePolicy;
use crate::event::{EventFilter, EventId, EventPage, EventRecord, EventSource, PendingEvent};
use crate::hash::diff::DiffReport;
use crate::hash::{StateFingerprint, fingerprint};
use crate::journal::{
    CheckpointBody, FrameKind, HeaderBody, Journal, OutcomeBody, recover::TornTail,
};
use crate::module::ctx::{InitCtx, StepCtx};
use crate::module::{KERNEL_STATUS_VIEW, Registry, SimModule};
use crate::sched;
use crate::state::serde_support::SaveBody;
use crate::state::{
    KernelState, RunConfig, RunLifecycle, RunMode, StateSlice, ViewId, ViewSnapshot, ViewValue,
};
use crate::watch::{WatchState, eval as watch_eval};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};

/// Sandbox continuation is capped 150 years past the epoch (i64-ns headroom).
const SANDBOX_CAP_YEAR: u16 = 2176;

/// How far to advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepRequest {
    /// Up to this many ticks.
    Ticks(u64),
    /// Until simulated time (first tick at-or-after).
    UntilSimTime(SimTimeNs),
    /// Until the next interrupt (or horizon).
    UntilInterrupt,
}

/// Why stepping stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopReason {
    /// The requested target was reached.
    Completed,
    /// Interrupt(s) were raised; the world is paused at `advanced_to`.
    Interrupted(Vec<u64>),
    /// The horizon was reached (continue via `Command::ContinueSandbox`).
    HorizonReached,
}

/// Result of one `step` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepResult {
    /// Tick the world is now at.
    pub advanced_to: u64,
    /// Why we stopped.
    pub stopped: StopReason,
    /// Events raised during this call (bodies via `events`).
    pub new_events: Vec<EventId>,
}

/// Snapshot of run status (the `kernel/status` data, plus identity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunStatus {
    /// Run id.
    pub run_id: u64,
    /// Current tick.
    pub tick: u64,
    /// Exact simulated time.
    pub sim_time_ns: i64,
    /// Civil date-time.
    pub date: calendar::DateTime,
    /// Lifecycle.
    pub lifecycle: RunLifecycle,
    /// Run mode.
    pub mode: RunMode,
    /// Pending (un-acknowledged) interrupts.
    pub pending_interrupts: Vec<Interrupt>,
    /// Pinned content-data version (hex).
    pub data_version: String,
    /// Build identity.
    pub build_id: String,
    /// Total events recorded so far.
    pub total_events: u64,
}

/// Crash-recovery report (FR-CORE-303).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryReport {
    /// Tick the run recovered to.
    pub recovered_tick: u64,
    /// Number of commands replayed from the journal tail.
    pub commands_replayed: u64,
    /// Torn-tail loss, if the journal was damaged.
    pub torn_tail: Option<TornTail>,
}

/// Replay verification report (SC-003/SC-009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayReport {
    /// Tick the replay reached.
    pub final_tick: u64,
    /// Commands re-applied.
    pub commands_applied: u64,
    /// In verify mode: None = perfect match; Some = first divergence description.
    pub first_divergence: Option<String>,
}

/// The simulation kernel handle — the authoritative seam between simulation and
/// presentation. No UI, Tauri, webview or rendering dependency exists behind it.
pub struct SimCore {
    kernel: KernelState,
    slices: Vec<Box<dyn StateSlice>>,
    registry: Registry,
    data: DataSet,
    journal: Journal,
    tier: crate::event::store::TierSink,
    run_dir: Option<PathBuf>,
    available_views: Vec<ViewId>,
}

impl SimCore {
    // ------------------------------------------------------------------ create

    /// Create a new run with the module set that defines this game's systems
    /// (in-memory journal; use [`SimCore::create_persistent`] for a durable run).
    pub fn create(
        config: RunConfig,
        data: DataSet,
        modules: Vec<Box<dyn SimModule>>,
    ) -> Result<SimCore, CoreError> {
        Self::create_inner(config, data, modules, None)
    }

    /// Create a new durable run: journal + autosaves + event tiers live in
    /// `run_dir` (created; must not already contain a journal).
    pub fn create_persistent(
        config: RunConfig,
        data: DataSet,
        modules: Vec<Box<dyn SimModule>>,
        run_dir: &Path,
    ) -> Result<SimCore, CoreError> {
        Self::create_inner(config, data, modules, Some(run_dir.to_path_buf()))
    }

    fn create_inner(
        config: RunConfig,
        data: DataSet,
        modules: Vec<Box<dyn SimModule>>,
        run_dir: Option<PathBuf>,
    ) -> Result<SimCore, CoreError> {
        config.validate()?;
        data.validate()?;
        let registry = Registry::build(modules, &data)?;

        let run_id = derive_run_id(config.master_seed);
        let clock = SimClock::new(data.config.base_tick_ns, config.horizon_years);
        let mut kernel = KernelState {
            run_id,
            config,
            lifecycle: RunLifecycle::Created,
            clock,
            rng: crate::rng::RngRegistry::new(0), // replaced below (needs config seed)
            next_event_id: 0,
            next_command_id: 0,
            next_watch_id: 0,
            next_interrupt_id: 0,
            commands_pending: Vec::new(),
            scheduled: BTreeMap::new(),
            events: crate::event::EventStoreState::new(),
            watches: BTreeMap::new(),
            interrupts: BTreeMap::new(),
            pause_policies: data
                .event_classes
                .iter()
                .map(|c| (c.id.clone(), c.default_pause))
                .collect(),
            views: BTreeMap::new(),
            horizon_signalled: false,
        };
        kernel.rng = crate::rng::RngRegistry::new(kernel.config.master_seed);

        // Init module slices (seeded via declared streams only).
        let mut slices: Vec<Box<dyn StateSlice>> = Vec::with_capacity(registry.modules.len());
        for (module, manifest) in registry.modules.iter().zip(&registry.manifests) {
            let mut ctx = InitCtx {
                manifest,
                rng: &mut kernel.rng,
                config: &kernel.config,
            };
            slices.push(module.init(&mut ctx));
        }

        // Initial view publication (tick 0 boundary).
        for (i, module) in registry.modules.iter().enumerate() {
            publish_module(
                &mut kernel.views,
                &registry.manifests[i],
                module.as_ref(),
                slices[i].as_ref(),
            );
        }
        publish_kernel_view(&mut kernel);

        let header = HeaderBody {
            format_version: crate::journal::JOURNAL_FORMAT_VERSION,
            run_id,
            master_seed: kernel.config.master_seed,
            config: kernel.config.clone(),
            data_version: data.version(),
            build_id: crate::save::build_id(),
        };
        let (journal, tier) = match &run_dir {
            Some(dir) => (
                Journal::in_dir(dir, &header, data.config.journal_flush_wall_ms)?,
                crate::event::store::TierSink::File(dir.join("events.tier")),
            ),
            None => (
                Journal::in_memory(&header)?,
                crate::event::store::TierSink::Mem(Vec::new()),
            ),
        };

        let available_views = registry.available_views();
        Ok(SimCore {
            kernel,
            slices,
            registry,
            data,
            journal,
            tier,
            run_dir,
            available_views,
        })
    }

    // ----------------------------------------------------------------- queries

    /// Run status at the current tick boundary.
    pub fn status(&self) -> RunStatus {
        RunStatus {
            run_id: self.kernel.run_id,
            tick: self.kernel.clock.tick.0,
            sim_time_ns: self.kernel.clock.now().0,
            date: self.kernel.clock.date(),
            lifecycle: self.kernel.lifecycle,
            mode: self.kernel.config.run_mode,
            pending_interrupts: self.kernel.interrupts.values().cloned().collect(),
            data_version: self.data.version().hex(),
            build_id: crate::save::build_id(),
            total_events: self.kernel.events.total(),
        }
    }

    /// All view ids available in this run (kernel + registered producers).
    pub fn views(&self) -> Vec<ViewId> {
        self.available_views.clone()
    }

    /// A published view at the last completed tick boundary.
    pub fn view(&self, id: &str) -> Result<ViewSnapshot, CoreError> {
        self.kernel
            .views
            .get(id)
            .cloned()
            .ok_or_else(|| CoreError::Unknown {
                kind: "view".into(),
                id: id.into(),
            })
    }

    /// Query the full run history (FR-CORE-401), paged and filterable.
    pub fn events(&self, filter: &EventFilter) -> Result<EventPage, CoreError> {
        self.kernel.events.query(filter, &self.tier)
    }

    /// Pending interrupts.
    pub fn interrupts(&self) -> Vec<Interrupt> {
        self.kernel.interrupts.values().cloned().collect()
    }

    /// Registered watches.
    pub fn watches(&self) -> Vec<WatchState> {
        self.kernel.watches.values().cloned().collect()
    }

    /// Per-class pause policies.
    pub fn pause_policies(&self) -> BTreeMap<String, PausePolicy> {
        self.kernel.pause_policies.clone()
    }

    /// Watch templates available in this run (bindings resolved at creation —
    /// templates referencing views no registered module publishes are excluded).
    pub fn watch_templates(&self) -> Vec<WatchTemplateDef> {
        self.data
            .watch_templates
            .iter()
            .filter(|t| self.available_views.contains(&t.view))
            .cloned()
            .collect()
    }

    /// Calendar service (FR-CORE-103).
    pub fn calendar(&self, t: SimTimeNs) -> calendar::DateTime {
        calendar::datetime_at(t)
    }

    // ---------------------------------------------------------------- commands

    /// Validate and accept a command; it applies at the current tick boundary,
    /// at the start of the next `step` call (FR-CORE-301/304).
    pub fn submit(&mut self, cmd: Command) -> Result<CommandReceipt, CoreError> {
        self.validate_command(&cmd)?;
        let id = CommandId(self.kernel.next_command_id);
        self.kernel.next_command_id += 1;
        let env = CommandEnvelope {
            id,
            submitted_at: self.kernel.clock.tick.0,
            payload: cmd,
        };
        self.journal.command(&env)?;
        self.kernel.commands_pending.push(env);
        Ok(CommandReceipt {
            id,
            applies_at: self.kernel.clock.tick.0,
        })
    }

    /// Acknowledge a pending interrupt (sugar for `submit`).
    pub fn acknowledge(&mut self, id: u64) -> Result<CommandReceipt, CoreError> {
        self.submit(Command::AcknowledgeInterrupt { id })
    }

    /// Structural validation at submission: malformed commands never enter the
    /// journal (state validity is checked deterministically at application).
    fn validate_command(&self, cmd: &Command) -> Result<(), CoreError> {
        match cmd {
            Command::RegisterWatch { spec } | Command::ModifyWatch { spec, .. } => {
                watch_eval::validate_spec(spec, &self.data, &self.available_views)
            }
            Command::SetPausePolicy { class, .. } => {
                if self.data.event_class(class).is_none() {
                    return Err(CoreError::CommandInvalid {
                        reason: format!("unknown event class '{class}'"),
                    });
                }
                Ok(())
            }
            Command::ModuleCommand { module, .. } => {
                if !self.registry.manifests.iter().any(|m| &m.id == module) {
                    return Err(CoreError::CommandInvalid {
                        reason: format!("unknown module '{module}'"),
                    });
                }
                Ok(())
            }
            Command::RemoveWatch { .. }
            | Command::AcknowledgeInterrupt { .. }
            | Command::ContinueSandbox => Ok(()),
        }
    }

    // ---------------------------------------------------------------- stepping

    /// Advance the world (FR-CORE-101/402/404). Pending commands apply first (so
    /// acknowledgements clear pauses); stepping then refuses while interrupts
    /// remain pending; otherwise the kernel jumps active tick to active tick
    /// until the target, an interrupt, or the horizon stops it.
    pub fn step(&mut self, req: StepRequest) -> Result<StepResult, CoreError> {
        let mut new_events: Vec<EventId> = Vec::new();

        // 1. Apply pending commands at the current boundary.
        self.apply_pending_commands(&mut new_events)?;

        // 2. A reached horizon is terminal until the player continues into
        //    sandbox (its end-of-horizon interrupt may still be pending).
        if self.kernel.lifecycle == RunLifecycle::HorizonReached {
            return Ok(StepResult {
                advanced_to: self.kernel.clock.tick.0,
                stopped: StopReason::HorizonReached,
                new_events,
            });
        }

        // 3. Refuse to advance while interrupts are pending.
        if !self.kernel.interrupts.is_empty() {
            if new_events.is_empty() {
                return Err(CoreError::InterruptPending {
                    count: self.kernel.interrupts.len() as u64,
                });
            }
            // Commands just applied raised fresh interrupts (e.g. a watch that was
            // already true): report rather than error.
            return Ok(StepResult {
                advanced_to: self.kernel.clock.tick.0,
                stopped: StopReason::Interrupted(self.kernel.interrupts.keys().copied().collect()),
                new_events,
            });
        }

        if self.kernel.lifecycle == RunLifecycle::Created {
            self.kernel.lifecycle = RunLifecycle::Active;
        }

        let sandbox_cap = self
            .kernel
            .clock
            .tick_at_or_after(SimTimeNs(calendar::ns_at_year_start(SANDBOX_CAP_YEAR)))
            .0;
        let target = match req {
            StepRequest::Ticks(n) => self.kernel.clock.tick.0.saturating_add(n),
            StepRequest::UntilSimTime(t) => self.kernel.clock.tick_at_or_after(t).0,
            StepRequest::UntilInterrupt => u64::MAX,
        }
        .min(sandbox_cap);

        // 3. Active-tick loop: jump → process → check.
        while self.kernel.clock.tick.0 < target {
            let next =
                sched::next_active_tick(&self.kernel, &self.registry.manifests, &self.data, target);
            let prev = self.kernel.clock.tick.0;
            self.kernel.clock.tick = Tick(next);
            self.run_tick(prev, &mut new_events)?;

            // The horizon is the more significant terminal condition: report it
            // even when the end-of-horizon event also raised an interrupt (that
            // interrupt is still pending and acknowledgeable). Persist either way.
            let interrupted = !self.kernel.interrupts.is_empty();
            if interrupted && self.run_dir.is_some() {
                self.autosave()?;
            }
            if self.kernel.lifecycle == RunLifecycle::HorizonReached {
                return Ok(StepResult {
                    advanced_to: self.kernel.clock.tick.0,
                    stopped: StopReason::HorizonReached,
                    new_events,
                });
            }
            if interrupted {
                return Ok(StepResult {
                    advanced_to: self.kernel.clock.tick.0,
                    stopped: StopReason::Interrupted(
                        self.kernel.interrupts.keys().copied().collect(),
                    ),
                    new_events,
                });
            }
        }

        Ok(StepResult {
            advanced_to: self.kernel.clock.tick.0,
            stopped: StopReason::Completed,
            new_events,
        })
    }

    /// Process one active tick in the documented order (event/order.rs).
    fn run_tick(&mut self, prev_tick: u64, new_events: &mut Vec<EventId>) -> Result<(), CoreError> {
        let tick = self.kernel.clock.tick.0;
        let mut queue: VecDeque<PendingEvent> = VecDeque::new();

        // Step 2: scheduled events due at this tick (insertion order).
        if let Some(due) = self.kernel.scheduled.remove(&tick) {
            queue.extend(due);
        }

        // Step 3: module steps (derived order; cadence/escalation, FR-CORE-104).
        let mut touched = vec![false; self.registry.modules.len()];
        // Index loop: simultaneous disjoint borrows of registry/slices/kernel fields.
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.registry.modules.len() {
            let manifest = &self.registry.manifests[i];
            let esc = sched::escalated(manifest, &self.kernel.views);
            let due = if esc {
                true
            } else {
                tick.is_multiple_of(manifest.cadence_ticks.max(1))
            };
            if !due {
                continue;
            }
            touched[i] = true;
            let mut ctx = StepCtx {
                manifest,
                clock: &self.kernel.clock,
                rng: &mut self.kernel.rng,
                views: &self.kernel.views,
                emitted: Vec::new(),
                scheduled: Vec::new(),
            };
            self.registry.modules[i].step(self.slices[i].as_mut(), &mut ctx);
            let (emitted, scheduled) = (ctx.emitted, ctx.scheduled);
            queue.extend(emitted);
            for (at, ev) in scheduled {
                self.kernel.scheduled.entry(at).or_default().push(ev);
            }
        }

        // Step 4: deliver the queue (FIFO; reactions append).
        self.drain_queue(&mut queue, &mut touched, new_events)?;

        // Step 5: publish views of touched modules + refresh the kernel view, so
        // watch evaluation (step 6) sees THIS tick's state, not the previous one.
        self.publish_touched(&touched);
        publish_kernel_view(&mut self.kernel);

        // Step 6: one watch pass (WatchId order), then deliver what fired.
        let fired = self.evaluate_watches();
        if !fired.is_empty() {
            for pe in fired {
                queue.push_back(pe);
            }
            let mut reacted = vec![false; self.registry.modules.len()];
            self.drain_queue(&mut queue, &mut reacted, new_events)?;
            self.publish_touched(&reacted);
        }

        // Step 7: horizon (exactly once).
        if !self.kernel.horizon_signalled && tick >= self.kernel.clock.horizon_tick.0 {
            self.kernel.horizon_signalled = true;
            self.kernel.lifecycle = RunLifecycle::HorizonReached;
            queue.push_back(PendingEvent {
                class: "end-of-horizon".into(),
                source: EventSource::Kernel,
                payload: BTreeMap::from([(
                    "horizon_years".to_string(),
                    ViewValue::U64(u64::from(self.kernel.config.horizon_years)),
                )]),
            });
            let mut reacted = vec![false; self.registry.modules.len()];
            self.drain_queue(&mut queue, &mut reacted, new_events)?;
            self.publish_touched(&reacted);
        }
        publish_kernel_view(&mut self.kernel);

        // Persistence cadences (boundary-crossing rule works with tick jumps).
        let cfg = &self.data.config;
        if crossed(prev_tick, tick, cfg.hash_mark_interval_ticks) {
            let fp = self.fingerprint()?;
            self.journal.hashmark(tick, &fp)?;
        }
        if self.run_dir.is_some() && crossed(prev_tick, tick, cfg.autosave_interval_ticks) {
            self.autosave()?;
        }
        Ok(())
    }

    /// FIFO event finalization + delivery. Assigns ids in queue order, stores,
    /// journals, applies pause policy, and delivers to subscribers (whose
    /// reactions append to the same queue).
    fn drain_queue(
        &mut self,
        queue: &mut VecDeque<PendingEvent>,
        touched: &mut [bool],
        new_events: &mut Vec<EventId>,
    ) -> Result<(), CoreError> {
        while let Some(pe) = queue.pop_front() {
            let id = EventId(self.kernel.next_event_id);
            self.kernel.next_event_id += 1;
            let record = EventRecord {
                id,
                tick: self.kernel.clock.tick.0,
                class: pe.class.clone(),
                source: pe.source.clone(),
                payload: pe.payload.clone(),
            };
            let policy = self
                .kernel
                .pause_policies
                .get(&record.class)
                .copied()
                .unwrap_or(PausePolicy::LogOnly);
            let interrupting = policy == PausePolicy::Interrupt;
            self.kernel.events.push(
                record.clone(),
                self.data.config.event_memory_window,
                &mut self.tier,
            )?;
            self.journal.event(&record, interrupting)?;
            new_events.push(id);
            if interrupting {
                let iid = self.kernel.next_interrupt_id;
                self.kernel.next_interrupt_id += 1;
                self.kernel.interrupts.insert(
                    iid,
                    Interrupt {
                        id: iid,
                        raised_by: id,
                        raised_at: record.tick,
                        class: record.class.clone(),
                    },
                );
            }
            // Delivery: module-command events go only to their target module;
            // everything else to declared subscribers, in module order.
            // Index loop: simultaneous disjoint borrows of registry/slices/kernel.
            #[allow(clippy::needless_range_loop)]
            for i in 0..self.registry.modules.len() {
                let manifest = &self.registry.manifests[i];
                let deliver = if record.class == "module-command" {
                    matches!(record.payload.get("module"),
                             Some(ViewValue::Str(m)) if m == &manifest.id)
                } else {
                    manifest.subscribes.iter().any(|c| c == &record.class)
                };
                if !deliver {
                    continue;
                }
                touched[i] = true;
                let mut ctx = StepCtx {
                    manifest,
                    clock: &self.kernel.clock,
                    rng: &mut self.kernel.rng,
                    views: &self.kernel.views,
                    emitted: Vec::new(),
                    scheduled: Vec::new(),
                };
                self.registry.modules[i].on_event(self.slices[i].as_mut(), &record, &mut ctx);
                let (emitted, scheduled) = (ctx.emitted, ctx.scheduled);
                queue.extend(emitted);
                for (at, ev) in scheduled {
                    self.kernel.scheduled.entry(at).or_default().push(ev);
                }
            }
        }
        Ok(())
    }

    fn publish_touched(&mut self, touched: &[bool]) {
        for (i, &t) in touched.iter().enumerate() {
            if t {
                publish_module(
                    &mut self.kernel.views,
                    &self.registry.manifests[i],
                    self.registry.modules[i].as_ref(),
                    self.slices[i].as_ref(),
                );
            }
        }
    }

    /// One watch pass: armed watches in WatchId order; fire on truth; re-arm on
    /// falsehood (documented edge semantics in watch/mod.rs).
    fn evaluate_watches(&mut self) -> Vec<PendingEvent> {
        let mut fired = Vec::new();
        for (id, w) in self.kernel.watches.iter_mut() {
            let truth = watch_eval::evaluate(&w.spec.expr, &self.data, &self.kernel.views);
            if w.armed && truth {
                w.armed = false;
                fired.push(PendingEvent {
                    class: "watch-fired".into(),
                    source: EventSource::Watch(*id),
                    payload: BTreeMap::from([("watch".to_string(), ViewValue::U64(*id))]),
                });
            } else if !w.armed && !truth {
                w.armed = true;
            }
        }
        fired
    }

    /// Apply pending commands at the current boundary (step 1 of the tick order).
    fn apply_pending_commands(&mut self, new_events: &mut Vec<EventId>) -> Result<(), CoreError> {
        if self.kernel.commands_pending.is_empty() {
            return Ok(());
        }
        let pending = std::mem::take(&mut self.kernel.commands_pending);
        let tick = self.kernel.clock.tick.0;
        let mut queue: VecDeque<PendingEvent> = VecDeque::new();

        for env in pending {
            let outcome = self.apply_one(&env, &mut queue);
            self.journal.outcome(
                tick,
                &OutcomeBody {
                    command: env.id,
                    outcome: outcome.clone(),
                },
            )?;
            if let CommandOutcome::Rejected(reason) = outcome {
                queue.push_back(PendingEvent {
                    class: "command-rejected".into(),
                    source: EventSource::Kernel,
                    payload: BTreeMap::from([
                        ("command".to_string(), ViewValue::U64(env.id.0)),
                        ("reason".to_string(), ViewValue::Str(reason)),
                    ]),
                });
            }
        }
        let mut reacted = vec![false; self.registry.modules.len()];
        self.drain_queue(&mut queue, &mut reacted, new_events)?;
        self.publish_touched(&reacted);
        publish_kernel_view(&mut self.kernel);
        Ok(())
    }

    /// Deterministic application of one command (state validity → Rejected).
    fn apply_one(
        &mut self,
        env: &CommandEnvelope,
        queue: &mut VecDeque<PendingEvent>,
    ) -> CommandOutcome {
        match &env.payload {
            Command::RegisterWatch { spec } => {
                let id = self.kernel.next_watch_id;
                self.kernel.next_watch_id += 1;
                self.kernel.watches.insert(
                    id,
                    WatchState {
                        id,
                        spec: spec.clone(),
                        armed: true,
                    },
                );
                self.immediate_watch_check(id, queue);
                CommandOutcome::Applied
            }
            Command::ModifyWatch { id, spec } => {
                let Some(w) = self.kernel.watches.get_mut(id) else {
                    return CommandOutcome::Rejected(format!("watch {id} does not exist"));
                };
                w.spec = spec.clone();
                w.armed = true;
                self.immediate_watch_check(*id, queue);
                CommandOutcome::Applied
            }
            Command::RemoveWatch { id } => {
                if self.kernel.watches.remove(id).is_none() {
                    return CommandOutcome::Rejected(format!("watch {id} does not exist"));
                }
                CommandOutcome::Applied
            }
            Command::SetPausePolicy { class, policy } => {
                self.kernel.pause_policies.insert(class.clone(), *policy);
                CommandOutcome::Applied
            }
            Command::AcknowledgeInterrupt { id } => {
                if self.kernel.interrupts.remove(id).is_none() {
                    return CommandOutcome::Rejected(format!("interrupt {id} is not pending"));
                }
                CommandOutcome::Applied
            }
            Command::ContinueSandbox => {
                if self.kernel.lifecycle != RunLifecycle::HorizonReached {
                    return CommandOutcome::Rejected(
                        "the run has not reached its horizon".to_string(),
                    );
                }
                self.kernel.lifecycle = RunLifecycle::SandboxContinued;
                CommandOutcome::Applied
            }
            Command::ModuleCommand { module, key, value } => {
                queue.push_back(PendingEvent {
                    class: "module-command".into(),
                    source: EventSource::Kernel,
                    payload: BTreeMap::from([
                        ("module".to_string(), ViewValue::Str(module.clone())),
                        ("key".to_string(), ViewValue::Str(key.clone())),
                        ("value".to_string(), ViewValue::I64(*value)),
                    ]),
                });
                CommandOutcome::Applied
            }
        }
    }

    /// A watch already true at registration/modification fires at this tick
    /// (FR-CORE-406 edge semantics).
    fn immediate_watch_check(&mut self, id: u64, queue: &mut VecDeque<PendingEvent>) {
        let Some(w) = self.kernel.watches.get_mut(&id) else {
            return;
        };
        if w.armed && watch_eval::evaluate(&w.spec.expr, &self.data, &self.kernel.views) {
            w.armed = false;
            queue.push_back(PendingEvent {
                class: "watch-fired".into(),
                source: EventSource::Watch(id),
                payload: BTreeMap::from([("watch".to_string(), ViewValue::U64(id))]),
            });
        }
    }

    // ------------------------------------------------------------- persistence

    /// Canonical body of the complete world state (FR-CORE-504). Self-contained:
    /// embeds the spilled event tier, so a save never depends on its run dir.
    fn body(&self) -> Result<SaveBody, CoreError> {
        let mut slices = Vec::with_capacity(self.slices.len());
        for (i, module) in self.registry.modules.iter().enumerate() {
            slices.push((
                self.registry.manifests[i].id.clone(),
                module.save_slice(self.slices[i].as_ref())?,
            ));
        }
        Ok(SaveBody {
            kernel: self.kernel.clone(),
            tier: self.tier.read_all()?,
            slices,
        })
    }

    /// Canonical state fingerprint at the current tick boundary (FR-CORE-203).
    pub fn fingerprint(&self) -> Result<StateFingerprint, CoreError> {
        fingerprint(&self.body()?)
    }

    /// Structural comparison against another save (divergence diagnosis).
    pub fn fingerprint_diff(&self, other_save: &[u8]) -> Result<DiffReport, CoreError> {
        let (_, payload) = crate::save::decode(other_save)?;
        let other = SaveBody::decode(&payload)?;
        crate::hash::diff::diff(&self.body()?, &other)
    }

    /// Complete save (FR-CORE-601): bit-identical restoration, including pending
    /// interrupts, queues, RNG streams and watch states.
    pub fn save(&self) -> Result<Vec<u8>, CoreError> {
        let payload = self.body()?.encode()?;
        let header = crate::save::SaveHeader {
            format_version: crate::save::migrate::SAVE_FORMAT_VERSION,
            build_id: crate::save::build_id(),
            data_version: self.data.version(),
            run_id: self.kernel.run_id,
            tick: self.kernel.clock.tick.0,
            payload_checksum: *blake3::hash(&payload).as_bytes(),
        };
        crate::save::encode(&header, &payload)
    }

    /// Restore from a save: pinned data resolved (never substituted), structure
    /// migrated if needed, state restored bit-identically (FR-CORE-601/602).
    /// The same module set used at creation must be supplied.
    pub fn load(
        save_bytes: &[u8],
        resolver: &dyn DataResolver,
        modules: Vec<Box<dyn SimModule>>,
    ) -> Result<SimCore, CoreError> {
        let (header, payload) = crate::save::decode(save_bytes)?;
        let data = crate::save::pin::resolve_pinned(resolver, header.data_version)?;
        let body = SaveBody::decode(&payload)?;
        Self::from_body(body, data, modules, None)
    }

    fn from_body(
        body: SaveBody,
        data: DataSet,
        modules: Vec<Box<dyn SimModule>>,
        run_dir: Option<PathBuf>,
    ) -> Result<SimCore, CoreError> {
        let registry = Registry::build(modules, &data)?;
        let saved_ids: Vec<&String> = body.slices.iter().map(|(id, _)| id).collect();
        let reg_ids: Vec<&String> = registry.manifests.iter().map(|m| &m.id).collect();
        if saved_ids != reg_ids {
            return Err(CoreError::RegistrationFailed {
                reason: format!(
                    "module set mismatch: save has [{}], registered [{}]",
                    saved_ids
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    reg_ids
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }
        let mut slices = Vec::with_capacity(body.slices.len());
        for (i, (_, bytes)) in body.slices.iter().enumerate() {
            slices.push(registry.modules[i].load_slice(bytes)?);
        }
        let kernel = body.kernel;
        if body.tier.len() as u64 != kernel.events.spilled_len {
            return Err(CoreError::IntegrityFailure {
                what: format!(
                    "embedded event tier length {} does not match recorded {}",
                    body.tier.len(),
                    kernel.events.spilled_len
                ),
            });
        }
        let tier = crate::event::store::TierSink::Mem(body.tier);
        let header = HeaderBody {
            format_version: crate::journal::JOURNAL_FORMAT_VERSION,
            run_id: kernel.run_id,
            master_seed: kernel.config.master_seed,
            config: kernel.config.clone(),
            data_version: data.version(),
            build_id: crate::save::build_id(),
        };
        let journal = Journal::in_memory(&header)?;
        let available_views = registry.available_views();
        Ok(SimCore {
            kernel,
            slices,
            registry,
            data,
            journal,
            tier,
            run_dir,
            available_views,
        })
    }

    /// Autosave into the run directory + CHECKPOINT frame (FR-CORE-603).
    fn autosave(&mut self) -> Result<(), CoreError> {
        let Some(dir) = self.run_dir.clone() else {
            return Ok(());
        };
        let bytes = self.save()?;
        let name = "autosave.sjs".to_string();
        crate::save::write_atomic(&dir.join(&name), &bytes)?;
        let fp = self.fingerprint()?;
        self.journal.checkpoint(
            self.kernel.clock.tick.0,
            &CheckpointBody {
                save_file: name,
                fingerprint: fp,
            },
        )
    }

    /// Export the journal bytes (seed + commands + verification material).
    pub fn export_journal(&mut self) -> Result<Vec<u8>, CoreError> {
        self.journal.export()
    }

    // ------------------------------------------------------- recovery & replay

    /// Recover a crashed durable run (FR-CORE-303): scan the journal — strictly
    /// (chain-verified) in ironman, leniently otherwise to salvage the longest
    /// intact prefix — then reconstruct by full replay of seed + intact command
    /// frames, reporting precisely what a torn tail lost. The recovered core
    /// journals in memory; the host re-persists explicitly.
    ///
    /// Recovery reconstructs from the journal header rather than the newest
    /// checkpoint: the replay path is the verified one (SC-003), and a 100-year
    /// command log is human-scale to replay. Checkpoint-accelerated recovery is a
    /// future optimization that must reproduce this exact result.
    ///
    /// `modules` is a factory so callers need not pre-clone the (owned) module set.
    pub fn recover(
        run_dir: &Path,
        resolver: &dyn DataResolver,
        modules: &dyn Fn() -> Vec<Box<dyn SimModule>>,
    ) -> Result<(SimCore, RecoveryReport), CoreError> {
        let bytes = std::fs::read(run_dir.join(crate::journal::JOURNAL_FILE))
            .map_err(|e| CoreError::io("reading journal", e))?;
        // Peek the header to learn the run mode (ironman ⇒ strict verification).
        let peek = crate::journal::recover::scan(&bytes, false)?;
        let strict = peek.header.config.run_mode == RunMode::Ironman;
        let scanned = if strict {
            crate::journal::recover::scan(&bytes, true)?
        } else {
            peek
        };

        let data = crate::save::pin::resolve_pinned(resolver, scanned.header.data_version)?;
        let mut core = SimCore::create(scanned.header.config.clone(), data, modules())?;

        let tail: Vec<(u64, CommandEnvelope)> = scanned
            .frames
            .iter()
            .filter(|f| f.kind == FrameKind::Command)
            .map(|f| {
                postcard::from_bytes::<CommandEnvelope>(&f.body)
                    .map(|env| (f.tick, env))
                    .map_err(CoreError::ser)
            })
            .collect::<Result<_, _>>()?;
        let replayed = tail.len() as u64;
        drive_commands(&mut core, &tail)?;
        // Advance to the last intact frame's tick (events after the last command).
        if let Some(last_tick) = scanned.frames.iter().map(|f| f.tick).max() {
            advance_to(&mut core, last_tick)?;
        }

        let report = RecoveryReport {
            recovered_tick: core.kernel.clock.tick.0,
            commands_replayed: replayed,
            torn_tail: scanned.torn_tail,
        };
        Ok((core, report))
    }

    /// Replay a journal from the beginning (FR-CORE-302, SC-003): reconstruct
    /// history from seed + commands; in verify mode, compare regenerated events,
    /// outcomes and hashmarks against the recorded frames and report the first
    /// divergence. Replay binds to the originating build.
    pub fn replay(
        journal_bytes: &[u8],
        resolver: &dyn DataResolver,
        modules: Vec<Box<dyn SimModule>>,
        until: Option<u64>,
        verify: bool,
    ) -> Result<(SimCore, ReplayReport), CoreError> {
        let scanned = crate::journal::recover::scan(journal_bytes, true)?;
        let this_build = crate::save::build_id();
        if scanned.header.build_id != this_build {
            return Err(CoreError::BuildMismatch {
                journal_build: scanned.header.build_id,
                this_build,
            });
        }
        let data = crate::save::pin::resolve_pinned(resolver, scanned.header.data_version)?;
        let mut core = SimCore::create(scanned.header.config.clone(), data, modules)?;

        let cmds: Vec<(u64, CommandEnvelope)> = scanned
            .frames
            .iter()
            .filter(|f| f.kind == FrameKind::Command)
            .map(|f| {
                postcard::from_bytes::<CommandEnvelope>(&f.body)
                    .map(|env| (f.tick, env))
                    .map_err(CoreError::ser)
            })
            .collect::<Result<_, _>>()?;
        let applied = cmds.len() as u64;
        let cmds: Vec<(u64, CommandEnvelope)> = match until {
            Some(t) => cmds.into_iter().filter(|(tick, _)| *tick <= t).collect(),
            None => cmds,
        };
        drive_commands(&mut core, &cmds)?;
        let final_tick =
            until.unwrap_or_else(|| scanned.frames.iter().map(|f| f.tick).max().unwrap_or(0));
        advance_to(&mut core, final_tick)?;

        let first_divergence = if verify {
            compare_journals(journal_bytes, &core.journal.export()?, until)?
        } else {
            None
        };
        let final_tick = core.kernel.clock.tick.0;
        Ok((
            core,
            ReplayReport {
                final_tick,
                commands_applied: applied,
                first_divergence,
            },
        ))
    }
}

// ------------------------------------------------------------------- internals

/// Deterministic run id from the master seed.
fn derive_run_id(seed: u64) -> u64 {
    let h = blake3::hash(&[&seed.to_le_bytes()[..], b"run_id"].concat());
    u64::from_le_bytes(h.as_bytes()[..8].try_into().expect("8 bytes"))
}

fn publish_module(
    views: &mut BTreeMap<ViewId, ViewSnapshot>,
    manifest: &crate::module::ModuleManifest,
    module: &dyn SimModule,
    slice: &dyn StateSlice,
) {
    let published = module.publish(slice);
    let ids: Vec<&ViewId> = published.iter().map(|v| &v.id).collect();
    for declared in &manifest.publishes {
        assert!(
            ids.contains(&declared),
            "module '{}' failed to publish declared view '{declared}' (FR-CORE-501)",
            manifest.id
        );
    }
    for v in published {
        assert!(
            manifest.publishes.contains(&v.id),
            "module '{}' published undeclared view '{}' (FR-CORE-501)",
            manifest.id,
            v.id
        );
        views.insert(v.id.clone(), v);
    }
}

fn publish_kernel_view(kernel: &mut KernelState) {
    let date = kernel.clock.date();
    let snapshot = ViewSnapshot {
        id: KERNEL_STATUS_VIEW.to_string(),
        fields: BTreeMap::from([
            ("tick".to_string(), ViewValue::U64(kernel.clock.tick.0)),
            (
                "sim_time_ns".to_string(),
                ViewValue::I64(kernel.clock.now().0),
            ),
            ("year".to_string(), ViewValue::I64(i64::from(date.year))),
            ("month".to_string(), ViewValue::U64(u64::from(date.month))),
            ("day".to_string(), ViewValue::U64(u64::from(date.day))),
            (
                "lifecycle".to_string(),
                ViewValue::Str(format!("{:?}", kernel.lifecycle)),
            ),
            (
                "pending_interrupts".to_string(),
                ViewValue::U64(kernel.interrupts.len() as u64),
            ),
        ]),
    };
    kernel
        .views
        .insert(KERNEL_STATUS_VIEW.to_string(), snapshot);
}

/// Boundary-crossing cadence rule (works with active-tick jumps): true when the
/// span (prev, current] contains a multiple of `interval`.
fn crossed(prev: u64, current: u64, interval: u64) -> bool {
    interval > 0 && current / interval > prev / interval
}

/// Drive a journal's command list through a core, faithfully reproducing the
/// original submit/apply interleaving. Each command is advanced-to, submitted,
/// then applied with a zero-tick step *before the next command is submitted* —
/// because commands at the same tick (e.g. a milestone interrupt followed by its
/// acknowledgement) depend on each other's effects having materialized. Stepping
/// toward a command never has to cross an un-acknowledged interrupt: the original
/// could not have either, so its acknowledgement is an earlier command frame.
fn drive_commands(core: &mut SimCore, cmds: &[(u64, CommandEnvelope)]) -> Result<(), CoreError> {
    for (tick, env) in cmds {
        replay_advance_to(core, *tick)?;
        if core.kernel.clock.tick.0 != *tick {
            return Err(CoreError::JournalCorrupt {
                valid_prefix_tick: core.kernel.clock.tick.0,
                lost: format!(
                    "replay diverged: reached tick {} but the journal's next command is at \
                     tick {tick}",
                    core.kernel.clock.tick.0
                ),
            });
        }
        core.submit(env.payload.clone())?;
        core.step(StepRequest::Ticks(0))?; // apply this command at the boundary
    }
    Ok(())
}

/// Advance toward a target tick during replay, stopping at un-acknowledged
/// interrupts and the horizon (the journal's acknowledgement/continue commands
/// drive past them when their turn comes).
fn replay_advance_to(core: &mut SimCore, target: u64) -> Result<(), CoreError> {
    while core.kernel.clock.tick.0 < target {
        if !core.interrupts().is_empty() {
            break; // an ack command later in the journal will clear it
        }
        let remaining = target - core.kernel.clock.tick.0;
        let r = core.step(StepRequest::Ticks(remaining))?;
        match r.stopped {
            StopReason::Completed => {}
            StopReason::Interrupted(_) | StopReason::HorizonReached => break,
        }
    }
    Ok(())
}

/// Advance to a final tick after all commands, stopping cleanly at pauses the
/// original run ended on (un-acknowledged interrupts, horizon).
fn advance_to(core: &mut SimCore, target: u64) -> Result<(), CoreError> {
    replay_advance_to(core, target)
}

/// Compare two journals' verification material (COMMAND/OUTCOME/EVENT/HASHMARK;
/// CHECKPOINT frames are persistence artifacts and excluded).
fn compare_journals(
    original: &[u8],
    replayed: &[u8],
    until: Option<u64>,
) -> Result<Option<String>, CoreError> {
    let a = crate::journal::recover::scan(original, true)?;
    let b = crate::journal::recover::scan(replayed, true)?;
    let pick = |frames: &[crate::journal::Frame]| -> Vec<(FrameKind, u64, Vec<u8>)> {
        frames
            .iter()
            .filter(|f| f.kind != FrameKind::Checkpoint)
            .filter(|f| until.is_none_or(|t| f.tick <= t))
            .map(|f| (f.kind, f.tick, f.body.clone()))
            .collect()
    };
    let fa = pick(&a.frames);
    let fb = pick(&b.frames);
    for (i, (x, y)) in fa.iter().zip(fb.iter()).enumerate() {
        if x != y {
            return Ok(Some(format!(
                "frame {i}: original {:?}@tick {} vs replay {:?}@tick {}",
                x.0, x.1, y.0, y.1
            )));
        }
    }
    if fa.len() != fb.len() {
        return Ok(Some(format!(
            "frame count differs: original {}, replay {}",
            fa.len(),
            fb.len()
        )));
    }
    Ok(None)
}
