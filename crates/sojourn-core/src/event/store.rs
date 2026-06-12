//! Disk-backed event-history tiering (T048b, FR-CORE-401, clarified 2026-06-12):
//! the full run history stays queryable; a recent window stays fast in memory;
//! older events spill to a tier sink.
//!
//! Determinism requirement: the *state bookkeeping* (window content, spilled
//! count, chained tier hash) is identical whether the tier bytes live in a file
//! or in memory — sink choice is a hosting concern and can never influence
//! outcomes or fingerprints. Saves embed the tier bytes, so a save is fully
//! self-contained and never depends on the old run directory.

use crate::error::CoreError;
use std::io::Write;
use std::path::PathBuf;

/// Where spilled tier bytes live.
#[derive(Debug)]
pub enum TierSink {
    /// In-memory buffer (transient runs, loaded saves, replay).
    Mem(Vec<u8>),
    /// `events.tier` file in a run directory (durable runs).
    File(PathBuf),
}

impl TierSink {
    /// Append frame bytes.
    pub fn append(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        match self {
            TierSink::Mem(buf) => {
                buf.extend_from_slice(bytes);
                Ok(())
            }
            TierSink::File(path) => {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&*path)
                    .map_err(|e| CoreError::io("opening events.tier", e))?;
                f.write_all(bytes)
                    .map_err(|e| CoreError::io("appending events.tier", e))
            }
        }
    }

    /// Read the whole tier (history queries, save embedding).
    pub fn read_all(&self) -> Result<Vec<u8>, CoreError> {
        match self {
            TierSink::Mem(buf) => Ok(buf.clone()),
            TierSink::File(path) => {
                if path.exists() {
                    std::fs::read(path).map_err(|e| CoreError::io("reading events.tier", e))
                } else {
                    Ok(Vec::new())
                }
            }
        }
    }
}
