//! Determinism integration tests (T030, SC-001): double-run identity, pacing
//! invariance, and one-decision divergence.

mod common;
use common::{TestModule, config, data};
use sojourn_core::{Command, SimCore, StepRequest, StopReason};

fn modules() -> Vec<Box<dyn sojourn_core::SimModule>> {
    vec![Box::new(TestModule {
        cadence: 1800,
        ..Default::default()
    })]
}

/// Run `ticks` in strides of `chunk`, acknowledging interrupts, return fingerprint.
fn run(
    seed: u64,
    ticks: u64,
    chunk: u64,
    cmds: &[(u64, Command)],
) -> sojourn_core::StateFingerprint {
    let mut core = SimCore::create(config(seed), data(), modules()).unwrap();
    let mut cmd_i = 0;
    while core.status().tick < ticks {
        let now = core.status().tick;
        while cmd_i < cmds.len() && cmds[cmd_i].0 == now {
            core.submit(cmds[cmd_i].1.clone()).unwrap();
            cmd_i += 1;
        }
        let next_cmd = cmds.get(cmd_i).map(|(t, _)| *t).unwrap_or(ticks);
        let stride = (next_cmd.min(ticks) - now).min(chunk).max(1);
        let r = core.step(StepRequest::Ticks(stride)).unwrap();
        if let StopReason::Interrupted(ids) = r.stopped {
            for id in ids {
                core.acknowledge(id).unwrap();
            }
        }
    }
    core.fingerprint().unwrap()
}

#[test]
fn double_run_is_bit_identical_across_pacing() {
    let ticks = 90 * 86_400; // ~3 months
    let a = run(7, ticks, ticks, &[]); // one big stride
    let b = run(7, ticks, 5000, &[]); // many strides
    let c = run(7, ticks, 9973, &[]); // odd strides
    assert_eq!(a, b, "pacing must not affect outcomes (FR-CORE-204)");
    assert_eq!(a, c);
}

#[test]
fn different_seeds_diverge() {
    let ticks = 30 * 86_400;
    assert_ne!(run(1, ticks, 4000, &[]), run(2, ticks, 4000, &[]));
}

#[test]
fn one_decision_changes_history() {
    let ticks = 30 * 86_400;
    let base = run(5, ticks, 4000, &[]);
    let with_cmd = run(
        5,
        ticks,
        4000,
        &[(
            86_400,
            Command::ModuleCommand {
                module: "test".into(),
                key: "set".into(),
                value: 1,
            },
        )],
    );
    assert_ne!(
        base, with_cmd,
        "a decision must change the resulting history"
    );
}
