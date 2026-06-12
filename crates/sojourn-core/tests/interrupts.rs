//! Interrupt-and-pause integration tests (T040, SC-005/SC-014): exact-tick halt
//! before consequences, simultaneous events all surfaced, log-only continues,
//! full API usable while paused, watch firing including true-at-registration.

mod common;
use common::{TestModule, config, data};
use sojourn_core::{
    Command, EventFilter, PausePolicy, SimCore, StepRequest, StopReason, ViewValue, WatchExpr,
    WatchSpec,
};

#[test]
fn anomaly_interrupts_at_exact_tick_before_consequences() {
    let at = 50_000u64;
    let module = TestModule {
        cadence: 1000,
        anomaly_at: vec![at],
        double_at: vec![],
    };
    let mut core = SimCore::create(config(1), data(), vec![Box::new(module)]).unwrap();

    let r = core.step(StepRequest::UntilInterrupt).unwrap();
    assert_eq!(r.advanced_to, at, "paused exactly at the anomaly tick");
    assert!(matches!(r.stopped, StopReason::Interrupted(_)));
    assert_eq!(core.status().tick, at);
    assert_eq!(core.interrupts().len(), 1);

    // Stepping refuses to advance while the interrupt is pending.
    let err = core.step(StepRequest::Ticks(1000)).unwrap_err();
    assert!(matches!(
        err,
        sojourn_core::CoreError::InterruptPending { .. }
    ));

    // The full API is usable while paused: query and submit.
    assert!(core.view("test/state").is_ok());
    core.acknowledge(core.interrupts()[0].id).unwrap();
    let r = core.step(StepRequest::Ticks(1000)).unwrap();
    assert!(matches!(r.stopped, StopReason::Completed));
}

#[test]
fn simultaneous_events_all_surface() {
    let at = 40_000u64;
    let module = TestModule {
        cadence: 1000,
        anomaly_at: vec![],
        double_at: vec![at],
    };
    let mut core = SimCore::create(config(2), data(), vec![Box::new(module)]).unwrap();
    let r = core.step(StepRequest::UntilInterrupt).unwrap();
    assert_eq!(r.advanced_to, at);
    // Two anomalies on one tick → two interrupts, none lost.
    assert_eq!(
        core.interrupts().len(),
        2,
        "both simultaneous anomalies surfaced"
    );
    let page = core
        .events(&EventFilter {
            classes: Some(vec!["anomaly".into()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(page.events.len(), 2);
    assert!(page.events.iter().all(|e| e.tick == at));
}

#[test]
fn log_only_policy_does_not_interrupt() {
    let at = 30_000u64;
    let module = TestModule {
        cadence: 1000,
        anomaly_at: vec![at],
        double_at: vec![],
    };
    let mut core = SimCore::create(config(3), data(), vec![Box::new(module)]).unwrap();
    // Make anomalies log-only.
    core.submit(Command::SetPausePolicy {
        class: "anomaly".into(),
        policy: PausePolicy::LogOnly,
    })
    .unwrap();
    let r = core.step(StepRequest::Ticks(60_000)).unwrap();
    assert_eq!(r.advanced_to, 60_000, "no interrupt; warp ran through");
    assert!(matches!(r.stopped, StopReason::Completed));
    assert!(core.interrupts().is_empty());
    let page = core
        .events(&EventFilter {
            classes: Some(vec!["anomaly".into()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(page.events.len(), 1, "the anomaly was still logged");
}

#[test]
fn watch_fires_at_first_truth_and_true_at_registration() {
    let mut core =
        SimCore::create(config(4), data(), vec![Box::new(TestModule::default())]).unwrap();
    // Watch: kernel tick ≥ 100000.
    core.submit(Command::RegisterWatch {
        spec: WatchSpec {
            expr: WatchExpr::Leaf {
                template: "tick-reached".into(),
                value: ViewValue::U64(100_000),
            },
        },
    })
    .unwrap();
    let r = core.step(StepRequest::UntilInterrupt).unwrap();
    assert_eq!(
        r.advanced_to, 100_000,
        "watch fires at the exact threshold tick"
    );
    let fired = core
        .events(&EventFilter {
            classes: Some(vec!["watch-fired".into()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(fired.events.len(), 1);
    core.acknowledge(core.interrupts()[0].id).unwrap();

    // A watch already true at registration fires immediately (at the current tick).
    core.submit(Command::RegisterWatch {
        spec: WatchSpec {
            expr: WatchExpr::Leaf {
                template: "tick-reached".into(),
                value: ViewValue::U64(1),
            },
        },
    })
    .unwrap();
    let r = core.step(StepRequest::Ticks(0)).unwrap();
    assert!(
        matches!(r.stopped, StopReason::Interrupted(_)),
        "true-at-registration fires now"
    );
}

#[test]
fn composed_and_or_watch() {
    let at = 70_000u64;
    let module = TestModule {
        cadence: 1000,
        anomaly_at: vec![],
        double_at: vec![],
    };
    let mut core = SimCore::create(config(5), data(), vec![Box::new(module)]).unwrap();
    // (tick ≥ 70000) AND (tick ≥ 50000) → fires at 70000.
    core.submit(Command::RegisterWatch {
        spec: WatchSpec {
            expr: WatchExpr::And(vec![
                WatchExpr::Leaf {
                    template: "tick-reached".into(),
                    value: ViewValue::U64(at),
                },
                WatchExpr::Leaf {
                    template: "tick-reached".into(),
                    value: ViewValue::U64(50_000),
                },
            ]),
        },
    })
    .unwrap();
    let r = core.step(StepRequest::UntilInterrupt).unwrap();
    assert_eq!(r.advanced_to, at);
}
