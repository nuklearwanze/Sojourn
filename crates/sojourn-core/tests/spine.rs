//! Foundational spine integration test (T024): a run from the repo data set with
//! a registered module steps a year, applies and rejects commands, filters event
//! history, and reaches the horizon end-of-run signal.

mod common;
use common::{TestModule, config, data};
use sojourn_core::{Command, EventFilter, RunLifecycle, SimCore, StepRequest, StopReason};

fn modules() -> Vec<Box<dyn sojourn_core::SimModule>> {
    vec![Box::new(TestModule {
        cadence: 3600,
        ..Default::default()
    })]
}

#[test]
fn create_step_query_command() {
    let data = data();
    let mut core = SimCore::create(config(1), data, modules()).unwrap();

    // Step one simulated year. The quiet test module emits nothing on its own,
    // so the run completes with no events.
    let one_year = 365 * 86_400;
    let r = core.step(StepRequest::Ticks(one_year)).unwrap();
    assert_eq!(r.advanced_to, one_year);
    assert!(matches!(r.stopped, StopReason::Completed));
    assert_eq!(
        core.status().total_events,
        0,
        "no spontaneous events from a quiet module"
    );

    // A module command applies, is visible in state, and produces an event.
    core.submit(Command::ModuleCommand {
        module: "test".into(),
        key: "set".into(),
        value: 42,
    })
    .unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();
    let v = core.view("test/state").unwrap();
    assert_eq!(
        v.fields.get("last_value"),
        Some(&sojourn_core::ViewValue::I64(42))
    );
    assert!(
        core.status().total_events > 0,
        "the module-command produced an event"
    );

    // An invalid command is a structural error and never enters the journal.
    let err = core
        .submit(Command::ModuleCommand {
            module: "ghost".into(),
            key: "x".into(),
            value: 0,
        })
        .unwrap_err();
    assert!(matches!(
        err,
        sojourn_core::CoreError::CommandInvalid { .. }
    ));

    // Acknowledging a non-existent interrupt is a deterministic *rejection*
    // (journaled), surfaced as a command-rejected event.
    let before = core.status().total_events;
    core.submit(Command::AcknowledgeInterrupt { id: 999 })
        .unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();
    let page = core
        .events(&EventFilter {
            classes: Some(vec!["command-rejected".into()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(page.events.len(), 1, "one rejection recorded");
    assert!(core.status().total_events > before);
}

#[test]
fn horizon_signals_end_of_run_once() {
    let data = data();
    let mut core = SimCore::create(config(2), data, modules()).unwrap();
    // Run to the horizon (the quiet module raises no interrupts before it).
    let r = core.step(StepRequest::UntilInterrupt).unwrap();
    assert!(
        matches!(r.stopped, StopReason::HorizonReached),
        "reaching the horizon is reported as HorizonReached, got {:?}",
        r.stopped
    );
    assert_eq!(core.status().lifecycle, RunLifecycle::HorizonReached);
    let page = core
        .events(&EventFilter {
            classes: Some(vec!["end-of-horizon".into()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(page.events.len(), 1, "end-of-horizon fires exactly once");

    // The end-of-horizon event raised an interrupt; acknowledge it, then continue
    // into unscored sandbox.
    for i in core.interrupts() {
        core.acknowledge(i.id).unwrap();
    }
    core.submit(Command::ContinueSandbox).unwrap();
    core.step(StepRequest::Ticks(86_400)).unwrap();
    assert_eq!(core.status().lifecycle, RunLifecycle::SandboxContinued);
}
