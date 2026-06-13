//! Shared fixtures for the world integration tests: kernel + world module
//! assembly, command submission, snapshot access.
#![allow(dead_code)]

use sojourn_core::{DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use sojourn_world::{WorldCommand, WorldModule, WorldSnapshot, world_payload};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn repo(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(p)
}

/// Kernel data set (event classes incl. FA-03's).
pub fn kernel_data() -> DataSet {
    DataSet::load_dir(&repo("data/kernel")).expect("kernel data")
}

/// World module over the real `data/world`.
pub fn world_module() -> WorldModule {
    WorldModule::load(&repo("data/world")).expect("world data")
}

/// A kernel core with one world module installed (+ a twin for snapshots).
pub fn core_with_world(seed: u64) -> (SimCore, WorldModule) {
    let cfg = RunConfig {
        master_seed: seed,
        horizon_years: 100,
        run_mode: RunMode::SaveAnywhere,
        difficulty_inputs: BTreeMap::new(),
    };
    let core = SimCore::create(cfg, kernel_data(), vec![Box::new(world_module())]).expect("core");
    (core, world_module())
}

/// Submit a world command and apply it at the boundary.
pub fn world(core: &mut SimCore, cmd: WorldCommand) {
    core.submit(world_payload(&cmd)).expect("submit");
    let r = core.step(StepRequest::Ticks(0)).expect("apply");
    if let StopReason::Interrupted(ids) = r.stopped {
        for id in ids {
            core.acknowledge(id).expect("ack");
        }
        core.step(StepRequest::Ticks(0)).expect("apply ack");
    }
}

/// Advance the core to an absolute tick, acknowledging interrupts.
pub fn advance(core: &mut SimCore, to: u64) {
    loop {
        let now = core.status().tick;
        if now >= to {
            return;
        }
        if let StopReason::Interrupted(ids) = core
            .step(StepRequest::Ticks(to - now))
            .expect("step")
            .stopped
        {
            for id in ids {
                core.acknowledge(id).expect("ack");
            }
        }
    }
}

/// Snapshot the world state.
pub fn snap(core: &SimCore, m: &WorldModule) -> WorldSnapshot {
    WorldSnapshot::from_core(core, m).expect("snapshot")
}
