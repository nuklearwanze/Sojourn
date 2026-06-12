//! Kernel error taxonomy (contracts/core-api.md §Errors).
//!
//! Every variant is serializable so the same errors cross an IPC boundary unchanged.
//! `CommandInvalid` is an API-level failure (the command never entered the journal);
//! a *journaled* deterministic rejection is a [`crate::command::CommandOutcome`], not
//! an error.

use serde::{Deserialize, Serialize};

/// All fallible kernel operations return this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum CoreError {
    /// Run configuration failed validation.
    #[error("invalid run configuration: {reason}")]
    InvalidConfig {
        /// Human-actionable explanation.
        reason: String,
    },

    /// Kernel data files failed to load or validate.
    #[error("data set invalid: {reason}")]
    DataInvalid {
        /// Human-actionable explanation.
        reason: String,
    },

    /// A save pins a content-data version that the resolver cannot supply.
    #[error("pinned content-data version {pinned} unavailable: {hint}")]
    DataVersionUnavailable {
        /// Hex digest of the pinned `DataVersionId`.
        pinned: String,
        /// Where to look / what to install.
        hint: String,
    },

    /// Checksum or chain verification failed on a save or journal.
    #[error("integrity failure: {what}")]
    IntegrityFailure {
        /// What failed verification, and where.
        what: String,
    },

    /// Save format version is newer than this build supports.
    #[error("save format {found} unsupported (this build supports up to {supported})")]
    SaveFormatUnsupported {
        /// Version found in the header.
        found: u32,
        /// Maximum supported version.
        supported: u32,
    },

    /// A structural save migration failed.
    #[error("save migration failed: {reason}")]
    MigrationFailed {
        /// Human-actionable explanation.
        reason: String,
    },

    /// The journal is unreadable beyond a valid prefix.
    #[error("journal corrupt after tick {valid_prefix_tick}: {lost}")]
    JournalCorrupt {
        /// Last tick covered by intact frames.
        valid_prefix_tick: u64,
        /// Precise description of what was lost.
        lost: String,
    },

    /// A journal was produced by a different build and cannot be replayed here.
    #[error("journal build mismatch: journal from {journal_build}, this build is {this_build}")]
    BuildMismatch {
        /// Build that wrote the journal.
        journal_build: String,
        /// The running build.
        this_build: String,
    },

    /// API-level command failure: the command never entered the journal.
    #[error("command invalid: {reason}")]
    CommandInvalid {
        /// Why the command could not be accepted.
        reason: String,
    },

    /// Stepping refused: un-acknowledged interrupts are pending.
    #[error("cannot advance: {count} interrupt(s) pending acknowledgement")]
    InterruptPending {
        /// Number of pending interrupts.
        count: u64,
    },

    /// Module registration rejected (ownership overlap, unknown view, cycle…).
    #[error("module registration failed: {reason}")]
    RegistrationFailed {
        /// Which manifests conflicted and how.
        reason: String,
    },

    /// A query referenced an unknown view, watch, template or similar.
    #[error("unknown {kind}: {id}")]
    Unknown {
        /// What category of thing was looked up.
        kind: String,
        /// The identifier that failed to resolve.
        id: String,
    },

    /// Serialization plumbing failed (a kernel bug if it ever surfaces).
    #[error("internal serialization error: {reason}")]
    Serialization {
        /// Codec error detail.
        reason: String,
    },

    /// File I/O failed in the persistence layer.
    #[error("persistence I/O error during {during}: {reason}")]
    Io {
        /// Operation that failed.
        during: String,
        /// OS error detail.
        reason: String,
    },
}

impl CoreError {
    /// Wrap a postcard error.
    pub(crate) fn ser(e: impl core::fmt::Display) -> Self {
        CoreError::Serialization {
            reason: e.to_string(),
        }
    }

    /// Wrap an I/O error with operation context.
    pub(crate) fn io(during: &str, e: impl core::fmt::Display) -> Self {
        CoreError::Io {
            during: during.to_string(),
            reason: e.to_string(),
        }
    }
}
