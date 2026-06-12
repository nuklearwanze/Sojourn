//! State foundation (FR-CORE-101, research R5): the `StateSlice` contract module
//! state implements, stable entity handles, published views, and the serializable
//! kernel state that — together with the module slices — forms the single
//! authoritative world state.

pub mod serde_support;

use crate::clock::SimClock;
use crate::command::CommandEnvelope;
use crate::event::interrupt::Interrupt;
use crate::event::policy::PausePolicy;
use crate::event::{EventStoreState, PendingEvent};
use crate::rng::RngRegistry;
use crate::watch::WatchState;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::BTreeMap;

/// Stable 64-bit entity handle. Modules build their own typed stores (slotmap
/// recommended) and exchange handles, never Rust references held across steps.
pub type Handle = u64;

/// A module's exclusively-owned state. The kernel drives serialization through the
/// owning module (`SimModule::save_slice`/`load_slice`), so slices stay plain Rust
/// types internally while remaining fully covered by saves and fingerprints.
pub trait StateSlice: Any {
    /// Downcast support (modules recover their concrete type in `step`).
    fn as_any(&self) -> &dyn Any;
    /// Mutable downcast support.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> StateSlice for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Identifier of a published view, e.g. `"kernel/status"` or `"synthetic/stats"`.
pub type ViewId = String;

/// A field value in a published view / event payload / watch comparison.
///
/// Deliberately small: enough for the kernel slice and the watch catalogue.
/// Content slices may extend this enum (additively) when their views need more.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ViewValue {
    /// Boolean flag.
    Bool(bool),
    /// Signed integer (also carries `SimTimeNs`).
    I64(i64),
    /// Unsigned integer (ticks, counts, handles).
    U64(u64),
    /// Float (SI quantities). Compared with strict IEEE semantics.
    F64(f64),
    /// Identifier-style string (never display text — FA-10 renders).
    Str(String),
}

/// A read-only snapshot of (part of) a module's state at a tick boundary.
/// The query surface, the watch binding target, and the inter-module read channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewSnapshot {
    /// View identity.
    pub id: ViewId,
    /// Deterministically ordered fields.
    pub fields: BTreeMap<String, ViewValue>,
}

/// Run lifecycle (data-model §1). Pause is NOT a lifecycle state: pausing is host
/// pacing (warp invariance, FR-CORE-204).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunLifecycle {
    /// Created, not yet stepped.
    Created,
    /// Advancing normally.
    Active,
    /// Horizon tick reached; end-of-run signalled; scoring (FA-09) reads this.
    HorizonReached,
    /// Player chose to continue past the horizon; play is flagged unscored.
    SandboxContinued,
}

/// Save-anywhere vs ironman (FR-CORE-603).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunMode {
    /// Manual saves at any pause; loading any save allowed.
    SaveAnywhere,
    /// Rolling autosave + journal only; integrity-verified; no save-scumming.
    Ironman,
}

/// Player-supplied run configuration (kernel tunables come from the `DataSet`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunConfig {
    /// Master seed: with the command log and pinned data, fully determines history.
    pub master_seed: u64,
    /// Scoring horizon in years from 2026 (25 / 50 / 100).
    pub horizon_years: u16,
    /// Save-anywhere or ironman.
    pub run_mode: RunMode,
    /// Opaque difficulty inputs passed through to modules (FR-XCU-010: modules may
    /// never let these alter physics).
    pub difficulty_inputs: BTreeMap<String, i64>,
}

impl RunConfig {
    /// Validate player-facing constraints.
    pub fn validate(&self) -> Result<(), crate::error::CoreError> {
        if ![25, 50, 100].contains(&self.horizon_years) {
            return Err(crate::error::CoreError::InvalidConfig {
                reason: format!(
                    "horizon_years must be 25, 50 or 100 (got {})",
                    self.horizon_years
                ),
            });
        }
        Ok(())
    }
}

/// Everything the kernel itself owns, as one serializable unit. Together with the
/// module slices (serialized via their owners) this is the complete world state:
/// nothing outside it influences outcomes (FR-CORE-101).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelState {
    /// Run id (derived from the master seed at creation).
    pub run_id: u64,
    /// Player configuration.
    pub config: RunConfig,
    /// Lifecycle.
    pub lifecycle: RunLifecycle,
    /// The clock.
    pub clock: SimClock,
    /// All random streams (serialized: saves capture the exact random future).
    pub rng: RngRegistry,
    /// Next ids (monotonic counters).
    pub next_event_id: u64,
    /// Next command id.
    pub next_command_id: u64,
    /// Next watch id.
    pub next_watch_id: u64,
    /// Next interrupt id.
    pub next_interrupt_id: u64,
    /// Commands accepted but not yet applied (apply at next boundary).
    pub commands_pending: Vec<CommandEnvelope>,
    /// Scheduled future events: tick → pending events in insertion order.
    pub scheduled: BTreeMap<u64, Vec<PendingEvent>>,
    /// Event history bookkeeping (recent window + spilled-tier metadata).
    pub events: EventStoreState,
    /// Registered watch conditions.
    pub watches: BTreeMap<u64, WatchState>,
    /// Pending (un-acknowledged) interrupts.
    pub interrupts: BTreeMap<u64, Interrupt>,
    /// Per-class pause policies (defaults from data, then journaled player changes).
    pub pause_policies: BTreeMap<String, PausePolicy>,
    /// Last published views (tick-boundary read surface), by view id.
    pub views: BTreeMap<ViewId, ViewSnapshot>,
    /// Whether the end-of-horizon event has been emitted (exactly once).
    pub horizon_signalled: bool,
}
