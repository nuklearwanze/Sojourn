//! Save container (FR-CORE-601…604, contracts/persistence-format.md §1):
//! versioned header, BLAKE3 integrity, pinned content-data version, atomic writes
//! that always preserve the previous good save.

pub mod migrate;
pub mod pin;

use crate::data::DataVersionId;
use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

/// Container magic.
pub const MAGIC: [u8; 4] = *b"SJRN";

/// Build identity stamped into saves and journals; replay binds to it
/// (clarified 2026-06-12: journals replay on their originating build).
pub fn build_id() -> String {
    concat!("sojourn-core/", env!("CARGO_PKG_VERSION")).to_string()
}

/// Save header (persisted via postcard after the magic + length prefix).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveHeader {
    /// Save-format (structure) version; migrations key off this.
    pub format_version: u32,
    /// Build that wrote the save.
    pub build_id: String,
    /// Pinned content-data version of the run.
    pub data_version: DataVersionId,
    /// Run id.
    pub run_id: u64,
    /// Tick at which the save was taken.
    pub tick: u64,
    /// BLAKE3 of the payload.
    pub payload_checksum: [u8; 32],
}

/// Assemble container bytes: MAGIC | u32 header_len | header | payload.
pub fn encode(header: &SaveHeader, payload: &[u8]) -> Result<Vec<u8>, CoreError> {
    debug_assert_eq!(header.payload_checksum, *blake3::hash(payload).as_bytes());
    let header_bytes = postcard::to_allocvec(header).map_err(CoreError::ser)?;
    let mut out = Vec::with_capacity(4 + 4 + header_bytes.len() + payload.len());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(
        &(u32::try_from(header_bytes.len()).map_err(CoreError::ser)?).to_le_bytes(),
    );
    out.extend_from_slice(&header_bytes);
    out.extend_from_slice(payload);
    Ok(out)
}

/// Parse and integrity-verify a container. Returns the (possibly migrated) payload.
pub fn decode(bytes: &[u8]) -> Result<(SaveHeader, Vec<u8>), CoreError> {
    if bytes.len() < 8 || bytes[..4] != MAGIC {
        return Err(CoreError::IntegrityFailure {
            what: "not a Sojourn save (bad magic)".into(),
        });
    }
    let header_len = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let header_end = 8 + header_len;
    if bytes.len() < header_end {
        return Err(CoreError::IntegrityFailure {
            what: "truncated save header".into(),
        });
    }
    let header: SaveHeader = postcard::from_bytes(&bytes[8..header_end]).map_err(CoreError::ser)?;
    if header.format_version > migrate::SAVE_FORMAT_VERSION {
        return Err(CoreError::SaveFormatUnsupported {
            found: header.format_version,
            supported: migrate::SAVE_FORMAT_VERSION,
        });
    }
    let payload = &bytes[header_end..];
    let checksum = *blake3::hash(payload).as_bytes();
    if checksum != header.payload_checksum {
        return Err(CoreError::IntegrityFailure {
            what: "save payload checksum mismatch (file corrupted or edited)".into(),
        });
    }
    let payload = migrate::migrate(payload.to_vec(), header.format_version)?;
    Ok((header, payload))
}

/// Atomic durable write that preserves the previous good file (FR-CORE-604, and
/// the ironman "+1 backup generation" retention of persistence-format §3):
/// write `.tmp` + fsync → rotate existing file to `.bak` → rename `.tmp` in.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CoreError> {
    let tmp = path.with_extension("tmp");
    let bak = path.with_extension("bak");
    {
        let mut f =
            std::fs::File::create(&tmp).map_err(|e| CoreError::io("creating save tmp", e))?;
        f.write_all(bytes)
            .map_err(|e| CoreError::io("writing save tmp", e))?;
        f.sync_data()
            .map_err(|e| CoreError::io("syncing save tmp", e))?;
    }
    if path.exists() {
        if bak.exists() {
            std::fs::remove_file(&bak).map_err(|e| CoreError::io("rotating save backup", e))?;
        }
        std::fs::rename(path, &bak).map_err(|e| CoreError::io("rotating save", e))?;
    }
    std::fs::rename(&tmp, path).map_err(|e| CoreError::io("activating save", e))?;
    Ok(())
}
