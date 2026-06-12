//! # sojourn-core — the deterministic, headless simulation kernel (FA-01)
//!
//! The authoritative seam between simulation and presentation for Sojourn.
//! No UI, Tauri, webview or rendering dependency exists anywhere behind this API
//! (FR-CORE-702; enforced by the CI dependency audit).
//!
//! Guarantees (specs/002-sim-core/spec.md):
//! - **Determinism**: seed + pinned content data + ordered command log fully
//!   determine history, bit-identically, per platform and build (FR-CORE-201).
//! - **Warp invariance**: pacing and warp-rate choices never alter outcomes;
//!   warp is pure playback speed and is never journaled (FR-CORE-204).
//! - **Interrupt-and-pause**: the kernel refuses to advance past un-acknowledged
//!   interrupts; the world is fully queryable and commandable while paused.
//! - **Exact persistence**: saves round-trip bit-identically and pin the content
//!   data version their run started with; the journal replays from the start.
//!
//! See `specs/002-sim-core/contracts/` for the API, module and persistence
//! contracts, and `crates/sojourn-harness` for the determinism suite.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod api;
pub mod clock;
pub mod command;
pub mod data;
pub mod error;
pub mod event;
pub mod hash;
pub mod journal;
pub mod module;
pub mod rng;
pub mod save;
pub mod sched;
pub mod state;
pub mod watch;

pub use api::{
    RecoveryReport, ReplayReport, RunStatus, SimCore, StepRequest, StepResult, StopReason,
};
pub use clock::{SimTimeNs, Tick, calendar};
pub use command::{Command, CommandId, CommandOutcome, CommandReceipt};
pub use data::{DataResolver, DataSet, DataVersionId, DirResolver, MemResolver, WatchTemplateDef};
pub use error::CoreError;
pub use event::policy::PausePolicy;
pub use event::{EventFilter, EventId, EventPage, EventRecord, EventSource};
pub use hash::{StateFingerprint, diff::DiffReport};
pub use module::conformance::{ConformanceReport, run_suite};
pub use module::ctx::{InitCtx, StepCtx};
pub use module::{EscalationDecl, ModuleManifest, SimModule};
pub use state::{RunConfig, RunLifecycle, RunMode, StateSlice, ViewId, ViewSnapshot, ViewValue};
pub use watch::{CmpOp, ParamKind, WatchExpr, WatchSpec, WatchState};

// Re-exported for module crates so the whole workspace shares one version of the
// deterministic building blocks (research R5/R7).
pub use libm;
pub use slotmap;
