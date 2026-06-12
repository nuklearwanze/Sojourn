//! Canonical state fingerprint (FR-CORE-203, research R3): BLAKE3 over the
//! canonical `SaveBody` encoding — the same bytes saves persist, so fingerprint
//! coverage equals save coverage by construction.

pub mod diff;

use crate::error::CoreError;
use crate::state::serde_support::SaveBody;
use serde::{Deserialize, Serialize};

/// 256-bit canonical fingerprint of the full world state at a tick boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateFingerprint(pub [u8; 32]);

impl StateFingerprint {
    /// Hex form for logs, hashmarks and golden scenario values.
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// Fingerprint a canonical body.
pub fn fingerprint(body: &SaveBody) -> Result<StateFingerprint, CoreError> {
    let bytes = body.encode()?;
    Ok(StateFingerprint(*blake3::hash(&bytes).as_bytes()))
}
