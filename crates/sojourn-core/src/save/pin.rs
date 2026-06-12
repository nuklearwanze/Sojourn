//! Content-data pinning on load (FR-CORE-602; umbrella clarification: in-progress
//! runs keep the data they started with — never silent substitution).

use crate::data::{DataResolver, DataSet, DataVersionId};
use crate::error::CoreError;

/// Resolve a pinned version or fail actionably.
pub fn resolve_pinned(
    resolver: &dyn DataResolver,
    pinned: DataVersionId,
) -> Result<DataSet, CoreError> {
    resolver
        .resolve(pinned)
        .ok_or_else(|| CoreError::DataVersionUnavailable {
            pinned: pinned.hex(),
            hint: "this save pins the content-data version its run started with; ensure that \
               data version is installed (game updates keep prior data versions side by side)"
                .to_string(),
        })
}
