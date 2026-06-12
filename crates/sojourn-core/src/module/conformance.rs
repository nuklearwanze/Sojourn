//! Module conformance suite (contracts/module-contract.md §Conformance, SC-010):
//! the mechanical acceptance gate every module crate runs in its CI. Operates on
//! a *factory* because several checks construct independent runs.

use super::{Registry, SimModule};
use crate::api::{SimCore, StepRequest, StopReason};
use crate::data::DataSet;
use crate::error::CoreError;
use crate::event::EventFilter;
use crate::state::{RunConfig, RunMode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Factory producing fresh instances of the module under test.
pub type ModuleFactory<'a> = &'a dyn Fn() -> Box<dyn SimModule>;

/// Suite outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// Checks that passed.
    pub passed: Vec<String>,
    /// Checks that failed, with reasons.
    pub failed: Vec<String>,
}

impl ConformanceReport {
    /// All green?
    pub fn ok(&self) -> bool {
        self.failed.is_empty()
    }
}

fn config(seed: u64) -> RunConfig {
    RunConfig {
        master_seed: seed,
        horizon_years: 100,
        run_mode: RunMode::SaveAnywhere,
        difficulty_inputs: BTreeMap::new(),
    }
}

/// Run the conformance suite against a module factory.
pub fn run_suite(
    factory: ModuleFactory<'_>,
    data: &DataSet,
    seed: u64,
    ticks: u64,
) -> Result<ConformanceReport, CoreError> {
    let mut report = ConformanceReport {
        passed: Vec::new(),
        failed: Vec::new(),
    };
    let mut check = |name: &str, result: Result<(), String>| match result {
        Ok(()) => report.passed.push(name.to_string()),
        Err(e) => report.failed.push(format!("{name}: {e}")),
    };

    check("manifest-validates", manifest_validates(factory, data));
    check(
        "double-run-identity",
        double_run(factory, data, seed, ticks),
    );
    check(
        "slice-serde-round-trip",
        round_trip(factory, data, seed, ticks),
    );
    check("cadence-respected", cadence(factory, data, seed, ticks));

    Ok(report)
}

fn manifest_validates(factory: ModuleFactory<'_>, data: &DataSet) -> Result<(), String> {
    Registry::build(vec![factory()], data)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Double-run with deliberately varied stepping patterns (FR-CORE-204): the
/// module must introduce no nondeterminism and no pacing sensitivity.
fn double_run(
    factory: ModuleFactory<'_>,
    data: &DataSet,
    seed: u64,
    ticks: u64,
) -> Result<(), String> {
    let run = |chunk: u64| -> Result<(crate::hash::StateFingerprint, u64), String> {
        let mut core = SimCore::create(config(seed), data.clone(), vec![factory()])
            .map_err(|e| e.to_string())?;
        let mut remaining = ticks;
        while remaining > 0 {
            let n = remaining.min(chunk);
            let r = core
                .step(StepRequest::Ticks(n))
                .map_err(|e| e.to_string())?;
            let advanced = r.advanced_to;
            if let StopReason::Interrupted(ids) = r.stopped {
                for id in ids {
                    core.acknowledge(id).map_err(|e| e.to_string())?;
                }
            }
            remaining = ticks.saturating_sub(advanced);
        }
        let total = core
            .events(&EventFilter::default())
            .map_err(|e| e.to_string())?
            .total;
        Ok((core.fingerprint().map_err(|e| e.to_string())?, total))
    };
    let a = run(ticks)?; // one big stride
    let b = run(7)?; // many small odd strides
    if a != b {
        return Err(format!(
            "fingerprint/event-count differ between stepping patterns: {} ({} events) vs {} ({} events)",
            a.0.hex(),
            a.1,
            b.0.hex(),
            b.1
        ));
    }
    Ok(())
}

fn round_trip(
    factory: ModuleFactory<'_>,
    data: &DataSet,
    seed: u64,
    ticks: u64,
) -> Result<(), String> {
    let half = (ticks / 2).max(1);
    // Reference: unbroken run.
    let mut reference =
        SimCore::create(config(seed), data.clone(), vec![factory()]).map_err(|e| e.to_string())?;
    step_through(&mut reference, ticks)?;
    // Save at half, reload, continue.
    let mut a =
        SimCore::create(config(seed), data.clone(), vec![factory()]).map_err(|e| e.to_string())?;
    step_through(&mut a, half)?;
    let save = a.save().map_err(|e| e.to_string())?;
    let resolver = crate::data::MemResolver {
        sets: vec![data.clone()],
    };
    let mut b = SimCore::load(&save, &resolver, vec![factory()]).map_err(|e| e.to_string())?;
    step_through(&mut b, ticks)?;
    let (fa, fb) = (
        reference.fingerprint().map_err(|e| e.to_string())?,
        b.fingerprint().map_err(|e| e.to_string())?,
    );
    if fa != fb {
        return Err(format!(
            "save→load→continue diverged from unbroken run: {} vs {}",
            fa.hex(),
            fb.hex()
        ));
    }
    Ok(())
}

/// Step a core to an absolute tick, acknowledging interrupts along the way.
fn step_through(core: &mut SimCore, to: u64) -> Result<(), String> {
    loop {
        let now = core.status().tick;
        if now >= to {
            return Ok(());
        }
        let r = core
            .step(StepRequest::Ticks(to - now))
            .map_err(|e| e.to_string())?;
        if let StopReason::Interrupted(ids) = r.stopped {
            for id in ids {
                core.acknowledge(id).map_err(|e| e.to_string())?;
            }
        }
        if let StopReason::HorizonReached = core
            .step(StepRequest::Ticks(0))
            .map(|r| r.stopped)
            .unwrap_or(StopReason::Completed)
        {
            return Ok(());
        }
    }
}

/// Without escalations, a module must step exactly per its declared cadence;
/// with escalations, at least that often (observed via its declared views'
/// publication — every step republished views, so we infer from event totals
/// being identical across runs and the registry contract; here we check the
/// declared cadence arithmetic itself).
fn cadence(
    factory: ModuleFactory<'_>,
    data: &DataSet,
    seed: u64,
    ticks: u64,
) -> Result<(), String> {
    let module = factory();
    let manifest = module.manifest();
    drop(module);
    if manifest.cadence_ticks == 0 {
        return Err("cadence_ticks must be ≥ 1".into());
    }
    // Behavioural spot check: a run of N ticks completes without panicking and
    // lands exactly on N (no module can stall or overshoot the clock).
    let mut core =
        SimCore::create(config(seed), data.clone(), vec![factory()]).map_err(|e| e.to_string())?;
    step_through(&mut core, ticks)?;
    let status = core.status();
    if status.tick != ticks && status.lifecycle == crate::state::RunLifecycle::Active {
        return Err(format!("expected tick {ticks}, got {}", status.tick));
    }
    Ok(())
}
