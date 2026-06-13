//! Commands — the only mutation pathway into the simulation (FR-CORE-301/304).
//!
//! Structure is validated at submission (malformed commands are API-level
//! `CommandInvalid` errors and never enter the journal). State validity is checked
//! at application: a command whose target is stale resolves into a deterministic,
//! journaled `Rejected` outcome — never a crash, partial application or silent drop.

use crate::event::policy::PausePolicy;
use crate::watch::WatchSpec;
use serde::{Deserialize, Serialize};

/// Sequential command id per run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommandId(pub u64);

/// Kernel command set for this slice. Content slices route their gameplay commands
/// through `ModuleCommand` (delivered as a `module-command` event to the target
/// module) until they extend the kernel registry with typed commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Register a watch condition (FR-CORE-406). Fires immediately if already true.
    RegisterWatch {
        /// The condition.
        spec: WatchSpec,
    },
    /// Replace an existing watch's condition (re-arms it).
    ModifyWatch {
        /// Watch to modify.
        id: u64,
        /// New condition.
        spec: WatchSpec,
    },
    /// Remove a watch.
    RemoveWatch {
        /// Watch to remove.
        id: u64,
    },
    /// Change a class's pause policy (FR-CORE-403).
    SetPausePolicy {
        /// Event class id.
        class: String,
        /// New policy.
        policy: PausePolicy,
    },
    /// Acknowledge a pending interrupt (FR-CORE-404).
    AcknowledgeInterrupt {
        /// Interrupt to acknowledge.
        id: u64,
    },
    /// Continue past the horizon into unscored sandbox play (FR-CORE-102).
    ContinueSandbox,
    /// Deliver a key/value instruction to a module as a `module-command` event.
    /// The harness's synthetic commands use this; content slices may too.
    ModuleCommand {
        /// Target module id.
        module: String,
        /// Instruction key (module-defined).
        key: String,
        /// Instruction value (module-defined).
        value: i64,
    },
    /// Typed module command: an opaque, module-defined payload routed to
    /// [`crate::module::SimModule::on_command`] at application time. The kernel
    /// never decodes `payload` (no domain logic in the kernel, FR-CORE-505);
    /// malformed payloads are deterministic `Rejected` outcomes, journaled like
    /// any command. Added for FA-02 (specs/003-astrodynamics, research R11).
    ModulePayload {
        /// Target module id (validated to exist at submission).
        module: String,
        /// Module-defined command kind (diagnostics/filtering; not interpreted).
        kind: String,
        /// Module-defined encoded command (postcard by convention).
        payload: Vec<u8>,
    },
}

/// A validated, accepted command awaiting application at the next tick boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandEnvelope {
    /// Sequential id.
    pub id: CommandId,
    /// Tick at which it was accepted (== application tick: acceptance happens at a
    /// tick boundary, application at the start of the next `step` call, before any
    /// advance — see contracts/core-api.md).
    pub submitted_at: u64,
    /// The command.
    pub payload: Command,
}

/// Deterministic application result — journaled as an OUTCOME frame, and surfaced
/// as a `command-rejected` event when rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandOutcome {
    /// Applied successfully.
    Applied,
    /// Deterministically rejected at the application tick.
    Rejected(String),
}

/// Receipt returned by `SimCore::submit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandReceipt {
    /// Assigned command id.
    pub id: CommandId,
    /// Tick at which the command will apply.
    pub applies_at: u64,
}
