//! Foundational — the host + composition root build the full nine-module core and
//! advance it; the snapshot/clock stay consistent (FR-UI-1502/1503). This is the
//! end-to-end proof that the UI hosts the headless core in-process.

use sojourn_ui::host::UiHost;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../")
}

#[test]
fn the_host_builds_the_full_core_and_advances() {
    let mut host = UiHost::new_game(&repo_root(), 7).expect("the UI composes the full module set");
    let t0 = host.status().tick;

    // Advance a few days; the clock moves and stays consistent with what we read.
    host.advance(3 * 86_400).expect("advance");
    let s = host.status();
    assert!(s.tick > t0, "the host drives the core's clock forward");
    assert_eq!(s.tick, t0 + 3 * 86_400);

    // The event feed is readable (the same core the harness runs headless).
    let _events = host.recent_events(16);
}
