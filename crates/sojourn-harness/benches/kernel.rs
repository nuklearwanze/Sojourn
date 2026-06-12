//! Kernel performance budgets (SC-006, T065): step-loop throughput under the
//! synthetic full-game load profile (FR-CORE-704). Budget acceptance runs on the
//! reference machine record results in specs/002-sim-core/perf-results.md.

// The bench re-includes harness source modules; not every item is exercised here.
#![allow(dead_code)]

use criterion::{Criterion, criterion_group, criterion_main};
use sojourn_core::{DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use std::collections::BTreeMap;
use std::path::Path;

#[path = "../src/mutation/mod.rs"]
mod mutation;
#[path = "../src/synthetic/mod.rs"]
mod synthetic;

use synthetic::{SyntheticConfig, SyntheticModule};

fn full_load_core() -> SimCore {
    let data = DataSet::load_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/kernel")
            .as_path(),
    )
    .expect("kernel data");
    let cfg = RunConfig {
        master_seed: 99,
        horizon_years: 100,
        run_mode: RunMode::SaveAnywhere,
        difficulty_inputs: BTreeMap::new(),
    };
    // FR-CORE-704 profile: 3,200 entity stand-ins (≥3,000 catalogued bodies plus
    // ~200 craft), ~10k events/sim-year, hourly cadence.
    let synth = SyntheticModule::new(SyntheticConfig {
        entities: 3200,
        events_per_year: 10_000,
        cadence_ticks: 3600,
        milestone_every: 1000,
        anomaly_every: 0, // log-only flows for throughput measurement
    });
    SimCore::create(cfg, data, vec![Box::new(synth)]).expect("core")
}

fn bench_step_year(c: &mut Criterion) {
    let mut group = c.benchmark_group("kernel");
    group.sample_size(10);
    // One simulated month per iteration under full load.
    group.bench_function("step_month_full_load", |b| {
        let mut core = full_load_core();
        b.iter(|| {
            let r = core.step(StepRequest::Ticks(30 * 86_400)).expect("step");
            if let StopReason::Interrupted(ids) = r.stopped {
                for id in ids {
                    core.acknowledge(id).expect("ack");
                }
            }
        });
    });
    group.bench_function("fingerprint_full_load", |b| {
        let mut core = full_load_core();
        let _ = core.step(StepRequest::Ticks(86_400)).expect("step");
        b.iter(|| core.fingerprint().expect("fp"));
    });
    group.finish();
}

criterion_group!(benches, bench_step_year);
criterion_main!(benches);
