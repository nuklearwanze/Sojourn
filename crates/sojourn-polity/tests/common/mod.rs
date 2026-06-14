//! Shared fixtures for the polity integration tests: a kernel core with the polity
//! module installed, command/advance helpers, snapshot construction, and value
//! builders.
#![allow(dead_code)]

use sojourn_core::{DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use sojourn_polity::ids::{BodyId, CandidateId, FactionId};
use sojourn_polity::{
    CandidatePrior, CrewFacts, Difficulty, EconomyFacts, FactionInit, FundingModel, PolityCommand,
    PolityModule, ScienceTide, SiteProtection, Tier, WorldSnapshot, polity_payload,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DAY: u64 = 86_400;

pub fn repo(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(p)
}

pub fn module() -> PolityModule {
    PolityModule::load(&repo("data/polity")).expect("polity data")
}

/// A faction-init value.
pub fn faction(id: u32, ai: bool) -> FactionInit {
    FactionInit {
        id: FactionId(id),
        funding_model: if id.is_multiple_of(2) {
            FundingModel::Agency
        } else {
            FundingModel::Company
        },
        baseline_mood: 0.0,
        ai,
    }
}

/// A candidate-world prior.
pub fn candidate(body: u32, prob: f64) -> CandidatePrior {
    CandidatePrior {
        candidate: CandidateId(BodyId(body)),
        presence_prob: prob,
        tier: Tier::Ocean,
    }
}

/// A site planetary-protection facts bundle.
pub fn protection(body: u32, special: bool, limit: f64) -> SiteProtection {
    SiteProtection {
        body: BodyId(body),
        category: if special { 4 } else { 2 },
        special_region: special,
        bioburden_limit: limit,
    }
}

pub fn cid(body: u32) -> CandidateId {
    CandidateId(BodyId(body))
}

/// A kernel core with the polity module installed + a module copy for snapshots.
pub struct PolityH {
    pub core: SimCore,
    pub module: PolityModule,
}

impl PolityH {
    pub fn new() -> PolityH {
        PolityH::with_seed(7)
    }

    pub fn with_seed(seed: u64) -> PolityH {
        let data = DataSet::load_dir(&repo("data/kernel")).expect("kernel data");
        let cfg = RunConfig {
            master_seed: seed,
            horizon_years: 100,
            run_mode: RunMode::SaveAnywhere,
            difficulty_inputs: BTreeMap::new(),
        };
        let core = SimCore::create(cfg, data, vec![Box::new(module())]).expect("core");
        PolityH {
            core,
            module: module(),
        }
    }

    pub fn cmd(&mut self, c: PolityCommand) {
        self.core.submit(polity_payload(&c)).expect("submit");
        let r = self.core.step(StepRequest::Ticks(0)).expect("apply");
        self.drain(r.stopped);
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

    /// Advance exactly `ticks`, acknowledging polity interrupts as they fire and
    /// **resuming the remaining ticks** (the daily seeded step must run the whole span).
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

    pub fn snap(&self) -> WorldSnapshot {
        WorldSnapshot::from_core(&self.core, &self.module).expect("snapshot")
    }

    /// Initialise the world with the given factions / candidates / protections.
    pub fn init(
        &mut self,
        factions: Vec<FactionInit>,
        candidates: Vec<CandidatePrior>,
        protections: Vec<SiteProtection>,
    ) {
        self.cmd(PolityCommand::InitWorld {
            factions,
            candidates,
            protections,
            difficulty: Difficulty::default(),
        });
    }

    /// Set the composed science tide (per-faction maturity levels) + difficulty.
    pub fn set_tide(&mut self, per_faction: &[(u32, f64)], global: f64, difficulty: Difficulty) {
        let mut m = BTreeMap::new();
        for (id, lvl) in per_faction {
            m.insert(FactionId(*id), *lvl);
        }
        self.cmd(PolityCommand::UpdateWorld {
            science_tide: ScienceTide {
                global_level: global,
                per_faction_level: m,
            },
            difficulty,
        });
    }
}

/// A crew-facts outcome bundle.
pub fn crew_facts(
    faction: u32,
    loss: bool,
    routine_fail: bool,
    success: bool,
    world_first: bool,
) -> CrewFacts {
    CrewFacts {
        faction: FactionId(faction),
        loss_of_crew: loss,
        routine_failure: routine_fail,
        success,
        world_first,
    }
}

/// An economy-facts bundle.
pub fn econ(faction: u32, appropriation: f64, valuation: f64, tonnage: f64) -> EconomyFacts {
    EconomyFacts {
        faction: FactionId(faction),
        base_appropriation: appropriation,
        base_valuation: valuation,
        off_earth_tonnage_profit: tonnage,
    }
}
