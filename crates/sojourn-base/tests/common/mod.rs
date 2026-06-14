//! Shared fixtures for the base integration tests: a kernel core with the base
//! module installed, command/advance helpers, and snapshot construction over stub
//! composed-value inputs.
#![allow(dead_code)]

use sojourn_base::ids::{BaseId, ModuleId, SiteId};
use sojourn_base::{
    BaseCommand, BaseInputs, BaseModule, BaseSnapshot, PpCategory, SiteFacts, base_payload,
};
use sojourn_core::{DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DAY: u64 = 86_400;
pub const AU: f64 = 1.495_978_707e11;

pub fn repo(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(p)
}

pub fn module() -> BaseModule {
    BaseModule::load(&repo("data/base")).expect("base data")
}

/// A benign 1-AU, illuminated, unrestricted, low-radiation site.
pub fn benign_site() -> SiteFacts {
    SiteFacts {
        pp_category: PpCategory::Unrestricted,
        illumination: 0.9,
        thermal_k: 250.0,
        slope_deg: 3.0,
        comms_visible: true,
        hazard_level: 0.2,
        radiation_env_sv_yr: 0.3,
        resource_grade: 0.5,
        solar_distance_m: AU,
    }
}

/// `BaseInputs` with one site fact for the given site id.
pub fn inputs_for(site_id: &str, facts: SiteFacts) -> BaseInputs {
    let mut bi = BaseInputs::default();
    bi.sites.insert(SiteId(site_id.to_string()), facts);
    bi
}

/// A kernel core with the base module installed + a module copy for snapshots.
pub struct BaseH {
    pub core: SimCore,
    pub module: BaseModule,
}

impl BaseH {
    pub fn new() -> BaseH {
        let data = DataSet::load_dir(&repo("data/kernel")).expect("kernel data");
        let cfg = RunConfig {
            master_seed: 7,
            horizon_years: 100,
            run_mode: RunMode::SaveAnywhere,
            difficulty_inputs: BTreeMap::new(),
        };
        let core = SimCore::create(cfg, data, vec![Box::new(module())]).expect("core");
        BaseH {
            core,
            module: module(),
        }
    }

    fn drain(&mut self, stopped: StopReason) {
        if let StopReason::Interrupted(ids) = stopped {
            for id in ids {
                self.core.acknowledge(id).expect("ack");
            }
            let r = self.core.step(StepRequest::Ticks(0)).expect("ack-step");
            self.drain(r.stopped);
        }
    }

    pub fn cmd(&mut self, c: BaseCommand) {
        self.core.submit(base_payload(&c)).expect("submit");
        let r = self.core.step(StepRequest::Ticks(0)).expect("apply");
        self.drain(r.stopped);
    }

    pub fn advance(&mut self, ticks: u64) {
        let r = self.core.step(StepRequest::Ticks(ticks)).expect("advance");
        self.drain(r.stopped);
    }

    pub fn snap(&self, inputs: BaseInputs) -> BaseSnapshot {
        BaseSnapshot::from_core(&self.core, &self.module, inputs).expect("snapshot")
    }

    /// Found a base, add the modules, open construction and fully deliver every
    /// module so they all commission. Returns the base id.
    pub fn build(&mut self, site: &str, class: &str, modules: &[&str]) -> BaseId {
        self.cmd(BaseCommand::FoundBase {
            faction: 0,
            site: site.to_string(),
            class: class.to_string(),
        });
        let bid = 0u32; // first base
        for m in modules {
            self.cmd(BaseCommand::AddModule {
                base: bid,
                module_type: (*m).to_string(),
            });
        }
        self.cmd(BaseCommand::OpenConstruction { base: bid });
        for i in 0..modules.len() as u32 {
            self.cmd(BaseCommand::DeliverToBase {
                base: bid,
                module: i,
                mass_kg: 1.0e8,
                crew_time_hr: 1.0e7,
            });
        }
        BaseId(bid)
    }
}

/// The module id of the nth added module.
pub fn mid(n: u32) -> ModuleId {
    ModuleId(n)
}
