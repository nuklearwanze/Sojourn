//! Persistence integration tests (T047, SC-002): round-trip identity, save while
//! paused on an interrupt, missing pinned data error, corrupted save refused.

mod common;
use common::{TestModule, config, data};
use sojourn_core::{DataSet, MemResolver, SimCore, StepRequest, StopReason};

fn modules() -> Vec<Box<dyn sojourn_core::SimModule>> {
    vec![Box::new(TestModule {
        cadence: 1500,
        anomaly_at: vec![123_456],
        double_at: vec![],
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
fn save_load_continue_matches_unbroken_run() {
    let end = 200_000u64;
    let mut reference = SimCore::create(config(9), data(), modules()).unwrap();
    advance(&mut reference, end);
    let ref_fp = reference.fingerprint().unwrap();

    let mut run = SimCore::create(config(9), data(), modules()).unwrap();
    advance(&mut run, 80_000);
    let save_fp = run.fingerprint().unwrap();
    let bytes = run.save().unwrap();

    let resolver = MemResolver { sets: vec![data()] };
    let mut loaded = SimCore::load(&bytes, &resolver, modules()).unwrap();
    assert_eq!(
        loaded.fingerprint().unwrap(),
        save_fp,
        "load is bit-identical to save"
    );

    advance(&mut loaded, end);
    assert_eq!(
        loaded.fingerprint().unwrap(),
        ref_fp,
        "continuation matches unbroken run"
    );
}

#[test]
fn save_while_paused_restores_pending_interrupt() {
    let mut run = SimCore::create(config(10), data(), modules()).unwrap();
    let r = run.step(StepRequest::UntilInterrupt).unwrap();
    assert!(matches!(r.stopped, StopReason::Interrupted(_)));
    assert_eq!(run.interrupts().len(), 1);
    let bytes = run.save().unwrap();

    let resolver = MemResolver { sets: vec![data()] };
    let loaded = SimCore::load(&bytes, &resolver, modules()).unwrap();
    assert_eq!(
        loaded.interrupts().len(),
        1,
        "pending interrupt survives save/load"
    );
}

#[test]
fn missing_pinned_data_errors_actionably() {
    let mut run = SimCore::create(config(11), data(), modules()).unwrap();
    advance(&mut run, 10_000);
    let bytes = run.save().unwrap();
    // Resolver that has *different* data (empty set) cannot supply the pinned version.
    let empty = MemResolver { sets: vec![] };
    let err = SimCore::load(&bytes, &empty, modules())
        .map(|_| ())
        .unwrap_err();
    assert!(matches!(
        err,
        sojourn_core::CoreError::DataVersionUnavailable { .. }
    ));
}

#[test]
fn corrupted_save_is_refused() {
    let run = SimCore::create(config(12), data(), modules()).unwrap();
    let mut bytes = run.save().unwrap();
    // Flip a payload byte near the end.
    let n = bytes.len();
    bytes[n - 5] ^= 0xFF;
    let resolver = MemResolver { sets: vec![data()] };
    let err = SimCore::load(&bytes, &resolver, modules())
        .map(|_| ())
        .unwrap_err();
    assert!(matches!(
        err,
        sojourn_core::CoreError::IntegrityFailure { .. }
    ));
}

#[test]
fn data_version_is_stable() {
    // The same data files always hash to the same version (pinning is meaningful).
    assert_eq!(data().version(), data().version());
    let d2 = DataSet::from_sources(
        include_str!("../../../data/kernel/event-classes.ron"),
        include_str!("../../../data/kernel/watch-templates.ron"),
        include_str!("../../../data/kernel/config.ron"),
    )
    .unwrap();
    assert_eq!(data().version(), d2.version());
}
