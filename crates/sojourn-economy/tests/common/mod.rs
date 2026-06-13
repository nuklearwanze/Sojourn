//! Shared fixtures for the economy integration tests: a kernel core with the
//! economy module installed, command/advance helpers, and snapshot construction
//! over stub composed-value inputs.
#![allow(dead_code)]

use sojourn_core::{Command, DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use sojourn_economy::{
    EconomyCommand, EconomyInputs, EconomyModule, EconomySnapshot, economy_payload,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DAY: u64 = 86_400;

/// Repo-relative path (tests run with CWD = crate dir).
pub fn repo(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(p)
}

/// A freshly-loaded economy module (for snapshots; the core owns its own copy).
pub fn module() -> EconomyModule {
    EconomyModule::load(&repo("data/econ")).expect("econ data")
}

/// A kernel core with the economy module installed + a module copy for snapshots.
pub struct Econ {
    pub core: SimCore,
    pub module: EconomyModule,
}

impl Econ {
    /// Build a fresh core.
    pub fn new() -> Econ {
        let data = DataSet::load_dir(&repo("data/kernel")).expect("kernel data");
        let cfg = RunConfig {
            master_seed: 7,
            horizon_years: 100,
            run_mode: RunMode::SaveAnywhere,
            difficulty_inputs: BTreeMap::new(),
        };
        let core = SimCore::create(cfg, data, vec![Box::new(module())]).expect("core");
        Econ {
            core,
            module: module(),
        }
    }

    fn drain_interrupts(&mut self, stopped: StopReason) {
        if let StopReason::Interrupted(ids) = stopped {
            for id in ids {
                self.core.acknowledge(id).expect("ack");
            }
            let r = self.core.step(StepRequest::Ticks(0)).expect("ack-step");
            self.drain_interrupts(r.stopped);
        }
    }

    /// Submit + apply one economy command (acknowledging any interrupts).
    pub fn cmd(&mut self, c: EconomyCommand) {
        self.core.submit(economy_payload(&c)).expect("submit");
        let r = self.core.step(StepRequest::Ticks(0)).expect("apply");
        self.drain_interrupts(r.stopped);
    }

    /// Submit a raw kernel command.
    pub fn raw(&mut self, c: Command) {
        self.core.submit(c).expect("submit");
        let r = self.core.step(StepRequest::Ticks(0)).expect("apply");
        self.drain_interrupts(r.stopped);
    }

    /// Advance the simulation by `ticks` (acknowledging interrupts along the way).
    pub fn advance(&mut self, ticks: u64) {
        let r = self.core.step(StepRequest::Ticks(ticks)).expect("advance");
        self.drain_interrupts(r.stopped);
    }

    /// Current tick.
    pub fn tick(&self) -> u64 {
        self.core.status().tick
    }

    /// A snapshot over the given composed inputs.
    pub fn snap(&self, inputs: EconomyInputs) -> EconomySnapshot {
        EconomySnapshot::from_core(&self.core, &self.module, inputs).expect("snapshot")
    }

    /// A snapshot over empty inputs.
    pub fn snap0(&self) -> EconomySnapshot {
        self.snap(EconomyInputs::default())
    }
}
