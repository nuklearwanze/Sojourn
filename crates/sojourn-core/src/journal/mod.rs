//! Append-only run journal (FR-CORE-301…305, contracts/persistence-format.md §2).
//!
//! Frame layout: `len u32 | kind u8 | seq u64 | tick u64 | body … | checksum [32]`.
//! Checksums are **chained** (each covers the previous frame's checksum), so
//! interior edits and truncation are detectable (ironman integrity, FR-CORE-305).
//! The replay determinant is HEADER + ordered COMMAND frames; EVENT/OUTCOME/
//! HASHMARK frames are verification material; CHECKPOINT frames accelerate
//! recovery. Warp rates are never journaled (FR-CORE-204).

pub mod durability;
pub mod integrity;
pub mod recover;

use crate::command::{CommandEnvelope, CommandId, CommandOutcome};
use crate::data::DataVersionId;
use crate::error::CoreError;
use crate::event::EventRecord;
use crate::hash::StateFingerprint;
use crate::state::RunConfig;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Journal format (structure) version.
pub const JOURNAL_FORMAT_VERSION: u32 = 1;

/// Frame kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FrameKind {
    /// Run identity: seed, config, data version, build.
    Header = 0,
    /// A player decision — replay input.
    Command = 1,
    /// Deterministic application result — verification material.
    Outcome = 2,
    /// An occurrence — verification material.
    Event = 3,
    /// Autosave reference — recovery acceleration.
    Checkpoint = 4,
    /// Fingerprint anchor — divergence diagnosis.
    HashMark = 5,
}

/// HEADER frame body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderBody {
    /// Journal format version.
    pub format_version: u32,
    /// Run id.
    pub run_id: u64,
    /// Master seed.
    pub master_seed: u64,
    /// Player configuration.
    pub config: RunConfig,
    /// Pinned content-data version.
    pub data_version: DataVersionId,
    /// Build that wrote the journal (replay binds to it).
    pub build_id: String,
}

/// CHECKPOINT frame body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointBody {
    /// Autosave file name within the run directory.
    pub save_file: String,
    /// Fingerprint at the checkpoint tick.
    pub fingerprint: StateFingerprint,
}

/// OUTCOME frame body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeBody {
    /// Which command.
    pub command: CommandId,
    /// What happened.
    pub outcome: CommandOutcome,
}

/// A decoded frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// Kind.
    pub kind: FrameKind,
    /// Strictly monotonic sequence number.
    pub seq: u64,
    /// Tick the frame belongs to.
    pub tick: u64,
    /// Postcard-encoded body (kind-specific).
    pub body: Vec<u8>,
}

/// Where journal bytes go.
enum Sink {
    /// In-memory (tests, transient runs); fully exportable.
    Mem(Vec<u8>),
    /// File in the run directory, with the durability policy attached.
    File {
        file: std::fs::File,
        path: PathBuf,
        policy: durability::FlushPolicy,
    },
}

/// The append-only journal writer.
pub struct Journal {
    sink: Sink,
    next_seq: u64,
    last_checksum: [u8; 32],
}

/// File name within a run directory.
pub const JOURNAL_FILE: &str = "journal.sjl";

impl Journal {
    /// In-memory journal (exportable; no durability guarantees — used by tests
    /// and by replay reconstruction).
    pub fn in_memory(header: &HeaderBody) -> Result<Journal, CoreError> {
        let mut j = Journal {
            sink: Sink::Mem(Vec::new()),
            next_seq: 0,
            last_checksum: [0; 32],
        };
        j.append(
            FrameKind::Header,
            0,
            &postcard::to_allocvec(header).map_err(CoreError::ser)?,
        )?;
        Ok(j)
    }

    /// File journal in a run directory, flushed per the durability policy.
    pub fn in_dir(
        dir: &Path,
        header: &HeaderBody,
        flush_wall_ms: u64,
    ) -> Result<Journal, CoreError> {
        std::fs::create_dir_all(dir).map_err(|e| CoreError::io("creating run dir", e))?;
        let path = dir.join(JOURNAL_FILE);
        let file = std::fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .map_err(|e| CoreError::io("creating journal", e))?;
        let mut j = Journal {
            sink: Sink::File {
                file,
                path,
                policy: durability::FlushPolicy::new(flush_wall_ms),
            },
            next_seq: 0,
            last_checksum: [0; 32],
        };
        j.append(
            FrameKind::Header,
            0,
            &postcard::to_allocvec(header).map_err(CoreError::ser)?,
        )?;
        j.sync()?;
        Ok(j)
    }

    /// Append a COMMAND frame (replay input). Synced immediately (FR-CORE-303).
    pub fn command(&mut self, env: &CommandEnvelope) -> Result<(), CoreError> {
        let body = postcard::to_allocvec(env).map_err(CoreError::ser)?;
        self.append(FrameKind::Command, env.submitted_at, &body)?;
        self.sync()
    }

    /// Append an OUTCOME frame.
    pub fn outcome(&mut self, tick: u64, body: &OutcomeBody) -> Result<(), CoreError> {
        let body = postcard::to_allocvec(body).map_err(CoreError::ser)?;
        self.append(FrameKind::Outcome, tick, &body)?;
        self.maybe_sync()
    }

    /// Append an EVENT frame. Interrupt-raising events sync immediately.
    pub fn event(&mut self, ev: &EventRecord, interrupting: bool) -> Result<(), CoreError> {
        let body = postcard::to_allocvec(ev).map_err(CoreError::ser)?;
        self.append(FrameKind::Event, ev.tick, &body)?;
        if interrupting {
            self.sync()
        } else {
            self.maybe_sync()
        }
    }

    /// Append a CHECKPOINT frame (synced).
    pub fn checkpoint(&mut self, tick: u64, body: &CheckpointBody) -> Result<(), CoreError> {
        let body = postcard::to_allocvec(body).map_err(CoreError::ser)?;
        self.append(FrameKind::Checkpoint, tick, &body)?;
        self.sync()
    }

    /// Append a HASHMARK frame.
    pub fn hashmark(&mut self, tick: u64, fp: &StateFingerprint) -> Result<(), CoreError> {
        let body = postcard::to_allocvec(fp).map_err(CoreError::ser)?;
        self.append(FrameKind::HashMark, tick, &body)?;
        self.maybe_sync()
    }

    /// Export the full journal bytes (Mem sink reads its buffer; File sink reads
    /// back the synced file).
    pub fn export(&mut self) -> Result<Vec<u8>, CoreError> {
        self.sync()?;
        match &self.sink {
            Sink::Mem(bytes) => Ok(bytes.clone()),
            Sink::File { path, .. } => {
                std::fs::read(path).map_err(|e| CoreError::io("exporting journal", e))
            }
        }
    }

    fn append(&mut self, kind: FrameKind, tick: u64, body: &[u8]) -> Result<(), CoreError> {
        let seq = self.next_seq;
        self.next_seq += 1;
        let mut head = Vec::with_capacity(17);
        head.push(kind as u8);
        head.extend_from_slice(&seq.to_le_bytes());
        head.extend_from_slice(&tick.to_le_bytes());
        let checksum = integrity::chain(&self.last_checksum, &head, body);
        let len = u32::try_from(1 + 8 + 8 + body.len() + 32).map_err(CoreError::ser)?;
        let mut frame = Vec::with_capacity(4 + len as usize);
        frame.extend_from_slice(&len.to_le_bytes());
        frame.extend_from_slice(&head);
        frame.extend_from_slice(body);
        frame.extend_from_slice(&checksum);
        self.last_checksum = checksum;
        match &mut self.sink {
            Sink::Mem(buf) => buf.extend_from_slice(&frame),
            Sink::File { file, .. } => {
                file.write_all(&frame)
                    .map_err(|e| CoreError::io("appending journal", e))?;
            }
        }
        Ok(())
    }

    /// Force durability now.
    pub fn sync(&mut self) -> Result<(), CoreError> {
        if let Sink::File { file, policy, .. } = &mut self.sink {
            file.sync_data()
                .map_err(|e| CoreError::io("syncing journal", e))?;
            policy.mark_synced();
        }
        Ok(())
    }

    fn maybe_sync(&mut self) -> Result<(), CoreError> {
        let due = match &mut self.sink {
            Sink::File { policy, .. } => policy.sync_due(),
            Sink::Mem(_) => false,
        };
        if due { self.sync() } else { Ok(()) }
    }
}
