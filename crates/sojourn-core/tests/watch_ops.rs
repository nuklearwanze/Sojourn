//! SC-008 coverage closure: ModifyWatch, RemoveWatch and StepRequest::UntilSimTime,
//! which the scenario fixtures did not otherwise exercise.

mod common;
use common::{TestModule, config, data};
use sojourn_core::{
    Command, SimCore, SimTimeNs, StepRequest, StopReason, ViewValue, WatchExpr, WatchSpec,
};

fn modules() -> Vec<Box<dyn sojourn_core::SimModule>> {
    vec![Box::new(TestModule::default())]
}

fn leaf(template: &str, v: u64) -> WatchSpec {
    WatchSpec {
        expr: WatchExpr::Leaf {
            template: template.into(),
            value: ViewValue::U64(v),
        },
    }
}

#[test]
fn until_sim_time_advances_to_instant() {
    let mut core = SimCore::create(config(1), data(), modules()).unwrap();
    // 2 hours of simulated time = 7200 s ticks.
    let two_hours = SimTimeNs(7200 * 1_000_000_000);
    let r = core.step(StepRequest::UntilSimTime(two_hours)).unwrap();
    assert!(matches!(r.stopped, StopReason::Completed));
    assert_eq!(core.status().tick, 7200);
    assert_eq!(core.status().sim_time_ns, two_hours.0);
}

#[test]
fn modify_watch_changes_threshold() {
    let mut core = SimCore::create(config(2), data(), modules()).unwrap();
    // Register a far watch, then modify it to a near threshold before it fires.
    core.submit(Command::RegisterWatch {
        spec: leaf("tick-reached", 1_000_000),
    })
    .unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();
    let id = core.watches()[0].id;
    core.submit(Command::ModifyWatch {
        id,
        spec: leaf("tick-reached", 50_000),
    })
    .unwrap();
    let r = core.step(StepRequest::UntilInterrupt).unwrap();
    assert_eq!(r.advanced_to, 50_000, "modified threshold takes effect");
    assert!(matches!(r.stopped, StopReason::Interrupted(_)));
}

#[test]
fn remove_watch_prevents_firing() {
    let mut core = SimCore::create(config(3), data(), modules()).unwrap();
    core.submit(Command::RegisterWatch {
        spec: leaf("tick-reached", 40_000),
    })
    .unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();
    let id = core.watches()[0].id;
    core.submit(Command::RemoveWatch { id }).unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();
    assert!(core.watches().is_empty());
    // Step well past the old threshold: no interrupt.
    let r = core.step(StepRequest::Ticks(100_000)).unwrap();
    assert_eq!(r.advanced_to, 100_000);
    assert!(matches!(r.stopped, StopReason::Completed));
    assert!(core.interrupts().is_empty());
}

#[test]
fn remove_nonexistent_watch_is_rejected() {
    let mut core = SimCore::create(config(4), data(), modules()).unwrap();
    core.submit(Command::RemoveWatch { id: 123 }).unwrap();
    let before = core.status().total_events;
    core.step(StepRequest::Ticks(0)).unwrap();
    // A rejection is journaled and surfaced as a command-rejected event.
    assert!(core.status().total_events > before);
}
