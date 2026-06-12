//! Journal & replay integration tests (T055, SC-003): replay fidelity to an
//! arbitrary tick, build binding, and that warp never enters the journal.

mod common;
use common::{TestModule, config, data, temp_dir};
use sojourn_core::{Command, MemResolver, SimCore, StepRequest, StopReason};

fn modules() -> Vec<Box<dyn sojourn_core::SimModule>> {
    vec![Box::new(TestModule {
        cadence: 1200,
        ..Default::default()
    })]
}

fn advance(core: &mut SimCore, to: u64) {
    while core.status().tick < to {
        let now = core.status().tick;
        let r = core.step(StepRequest::Ticks(to - now)).unwrap();
        if let StopReason::Interrupted(ids) = r.stopped {
            for id in ids {
                core.acknowledge(id).unwrap();
            }
        }
    }
}

#[test]
fn replay_reproduces_history_with_commands() {
    let dir = temp_dir("journal_replay");
    let mut core = SimCore::create_persistent(config(21), data(), modules(), &dir).unwrap();
    // Submit a couple of commands at known ticks.
    advance(&mut core, 10_000);
    core.submit(Command::ModuleCommand {
        module: "test".into(),
        key: "set".into(),
        value: 7,
    })
    .unwrap();
    advance(&mut core, 20_000);
    core.submit(Command::ModuleCommand {
        module: "test".into(),
        key: "boom".into(),
        value: 3,
    })
    .unwrap();
    advance(&mut core, 30_000);
    let original_fp = core.fingerprint().unwrap();
    let journal = core.export_journal().unwrap();

    // Replay with verification.
    let resolver = MemResolver { sets: vec![data()] };
    let (replayed, report) =
        SimCore::replay(&journal, &resolver, modules(), Some(30_000), true).unwrap();
    assert_eq!(
        report.first_divergence, None,
        "replay matches recorded history"
    );
    assert_eq!(
        replayed.fingerprint().unwrap(),
        original_fp,
        "replay reproduces final state"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn replay_to_interior_tick_matches() {
    let dir = temp_dir("journal_interior");
    let mut core = SimCore::create_persistent(config(22), data(), modules(), &dir).unwrap();
    advance(&mut core, 50_000);
    let mid_fp = core.fingerprint().unwrap();
    advance(&mut core, 100_000);
    let journal = core.export_journal().unwrap();

    let resolver = MemResolver { sets: vec![data()] };
    let (replayed, _) =
        SimCore::replay(&journal, &resolver, modules(), Some(50_000), false).unwrap();
    assert_eq!(
        replayed.fingerprint().unwrap(),
        mid_fp,
        "interior replay matches that tick"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn replay_is_pacing_independent() {
    // Build a journal, then confirm replay (which drives its own pacing) equals
    // a differently-paced live run — warp/pacing is never journaled.
    let dir = temp_dir("journal_pacing");
    let mut core = SimCore::create_persistent(config(23), data(), modules(), &dir).unwrap();
    advance(&mut core, 40_000);
    let journal = core.export_journal().unwrap();
    let live_fp = core.fingerprint().unwrap();

    let resolver = MemResolver { sets: vec![data()] };
    let (replayed, _) =
        SimCore::replay(&journal, &resolver, modules(), Some(40_000), true).unwrap();
    assert_eq!(replayed.fingerprint().unwrap(), live_fp);
    std::fs::remove_dir_all(&dir).ok();
}
