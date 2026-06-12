//! Save-format migration framework (FR-CORE-602): structure-only, sequential,
//! version-keyed. Migrations never substitute a pinned run's content values.

use crate::error::CoreError;

/// Current save-format (structure) version.
pub const SAVE_FORMAT_VERSION: u32 = 1;

/// Migrate a payload written at `from` up to the current version, one step at a
/// time. Each step is a pure `Vec<u8> -> Vec<u8>` transformation of the canonical
/// body, registered here when a release changes the structure.
pub fn migrate(payload: Vec<u8>, from: u32) -> Result<Vec<u8>, CoreError> {
    if from == SAVE_FORMAT_VERSION {
        return Ok(payload);
    }
    // Sequential chain: when a release changes the structure, add a
    // `from => migrate_vN_to_vN+1(payload)` step here and recurse.
    Err(CoreError::MigrationFailed {
        reason: format!("no migration registered from save format {from}"),
    })
}
