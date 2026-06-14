//! Shared fixtures for the crew integration tests: a kernel core with the crew
//! module installed, command/advance helpers, snapshot construction, and value
//! builders.
#![allow(dead_code)]

use sojourn_core::{DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use sojourn_crew::asset::{CrewedAsset, EclssState};
use sojourn_crew::ids::{AssetId, FactionId};
use sojourn_crew::{
    AssetSizing, AstronautFacts, CrewCommand, CrewModule, CrewSnapshot, EnvFacts, Sex,
    TechMaturity, crew_payload,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const DAY: u64 = 86_400;

pub fn repo(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(p)
}

pub fn module() -> CrewModule {
    CrewModule::load(&repo("data/crew")).expect("crew data")
}

/// A crewed sizing with the given closure/shield/spin and benign defaults.
pub fn sizing(closure: f64, shield: f64, spin: bool, crewed: bool) -> AssetSizing {
    AssetSizing {
        closure_capability: closure,
        shield_attenuation: shield,
        population_capacity: 6,
        spin_gravity: spin,
        habitat_volume_m3_per_crew: 30.0,
        consumables_capacity_kg: 100_000.0,
        crewed,
    }
}

/// An environment with the given GCR rate / body / abort reach.
pub fn env(gcr: f64, body: &str, abort: bool) -> EnvFacts {
    EnvFacts {
        gcr_rate_sv_yr: gcr,
        body: body.to_string(),
        comms_lag_s: 600.0,
        abort_reach: abort,
    }
}

pub fn maturity(trl: u8, reliability: f64, flight_units: u32) -> TechMaturity {
    TechMaturity {
        trl,
        reliability,
        flight_units,
    }
}

pub fn facts(age: f64, sex: Sex) -> AstronautFacts {
    AstronautFacts {
        age_years: age,
        sex,
        traits: vec![],
        training: 0.8,
    }
}

/// A bare `CrewedAsset` value (for pure-function tests).
pub fn asset(sizing: AssetSizing, env: EnvFacts) -> CrewedAsset {
    CrewedAsset {
        id: AssetId(0),
        faction: FactionId(0),
        mission: None,
        sizing,
        env,
        eclss_maturity: maturity(9, 0.99, 10),
        ops_oversub: 0.0,
        consumables_kg: 100_000.0,
        eclss: EclssState {
            reliability: 0.99,
            degradation: 0.0,
            maintenance_deficit: 0.0,
            failed: false,
        },
        sheltering: false,
        maint_crew_hr: 0.0,
        maint_spares_kg: 0.0,
        occupied_since_tick: 0,
        crew: BTreeSet::new(),
    }
}

/// A kernel core with the crew module installed + a module copy for snapshots.
pub struct CrewH {
    pub core: SimCore,
    pub module: CrewModule,
}

impl CrewH {
    pub fn new() -> CrewH {
        let data = DataSet::load_dir(&repo("data/kernel")).expect("kernel data");
        let cfg = RunConfig {
            master_seed: 7,
            horizon_years: 100,
            run_mode: RunMode::SaveAnywhere,
            difficulty_inputs: BTreeMap::new(),
        };
        let core = SimCore::create(cfg, data, vec![Box::new(module())]).expect("core");
        CrewH {
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

    pub fn cmd(&mut self, c: CrewCommand) {
        self.core.submit(crew_payload(&c)).expect("submit");
        let r = self.core.step(StepRequest::Ticks(0)).expect("apply");
        self.drain(r.stopped);
    }

    /// Advance exactly `ticks`, acknowledging crew interrupts (SPE storms,
    /// grounding, anomalies, loss-of-crew) as they fire and **resuming the
    /// remaining ticks** — the daily seeded step must run for the whole span,
    /// not just up to the first interrupt.
    pub fn advance(&mut self, ticks: u64) {
        let target = self.core.status().tick + ticks;
        loop {
            let remaining = target.saturating_sub(self.core.status().tick);
            let r = self
                .core
                .step(StepRequest::Ticks(remaining))
                .expect("advance");
            match r.stopped {
                StopReason::Interrupted(ids) => {
                    for id in ids {
                        self.core.acknowledge(id).expect("ack");
                    }
                }
                _ => break,
            }
        }
    }

    pub fn snap(&self) -> CrewSnapshot {
        CrewSnapshot::from_core(&self.core, &self.module).expect("snapshot")
    }

    /// Occupy an asset and seat the given crew (id, facts).
    pub fn occupy(
        &mut self,
        asset: u32,
        sizing: AssetSizing,
        env: EnvFacts,
        consumables_kg: f64,
        crew: &[(&str, AstronautFacts)],
    ) {
        self.cmd(CrewCommand::OccupyAsset {
            faction: 0,
            asset,
            sizing,
            env,
            eclss_maturity: maturity(9, 0.99, 10),
            ops_oversub: 0.0,
            consumables_kg,
        });
        for (id, f) in crew {
            self.cmd(CrewCommand::AssignCrew {
                asset,
                astronaut: (*id).to_string(),
                facts: f.clone(),
            });
        }
    }
}
