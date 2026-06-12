//! Frame checksum chaining (FR-CORE-305): each frame's BLAKE3 covers the previous
//! frame's checksum, its own header and its body — interior edits, reordering and
//! truncation are all detectable, and an intact prefix is provably intact.

/// Chained checksum for one frame.
pub fn chain(prev: &[u8; 32], head: &[u8], body: &[u8]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(prev);
    h.update(head);
    h.update(body);
    *h.finalize().as_bytes()
}
