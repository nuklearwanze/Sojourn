//! Shared fixtures for the research integration tests: kernel + research module
//! assembly, command submission, snapshot access.
#![allow(dead_code)]

use sojourn_core::{DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use sojourn_research::personnel::Role;
use sojourn_research::{DomainId, ResearchCommand, ResearchModule, ResearchSnapshot};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn repo(p: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(p)
}

pub fn kernel_data() -> DataSet {
    DataSet::load_dir(&repo("data/kernel")).expect("kernel data")
}

pub fn research_module() -> ResearchModule {
    ResearchModule::load(&repo("data")).expect("research data")
}

pub fn core_with_research(seed: u64) -> (SimCore, ResearchModule) {
    let cfg = RunConfig {
        master_seed: seed,
        horizon_years: 100,
        run_mode: RunMode::SaveAnywhere,
        difficulty_inputs: BTreeMap::new(),
    };
    let core = SimCore::create(cfg, kernel_data(), vec![Box::new(research_module())]).expect("core");
    (core, research_module())
}

/// Submit a research command and apply it at the boundary.
pub fn research(core: &mut SimCore, cmd: ResearchCommand) {
    core.submit(sojourn_research::research_payload(&cmd)).expect("submit");
    let r = core.step(StepRequest::Ticks(0)).expect("apply");
    if let StopReason::Interrupted(ids) = r.stopped {
        for id in ids {
            core.acknowledge(id).expect("ack");
        }
        core.step(StepRequest::Ticks(0)).expect("apply ack");
    }
}

/// Hire `n` scientists of a discipline at a skill.
pub fn hire_scientists(core: &mut SimCore, faction: u32, discipline: &str, skill: f64, n: u32) {
    for _ in 0..n {
        research(
            core,
            ResearchCommand::Hire {
                faction,
                role: Role::Scientist,
                discipline: DomainId(discipline.into()),
                skill,
                traits: vec![],
            },
        );
    }
}

/// Hire `n` engineers of a discipline at a skill.
pub fn hire_engineers(core: &mut SimCore, faction: u32, discipline: &str, skill: f64, n: u32) {
    for _ in 0..n {
        research(
            core,
            ResearchCommand::Hire {
                faction,
                role: Role::Engineer,
                discipline: DomainId(discipline.into()),
                skill,
                traits: vec![],
            },
        );
    }
}

/// Advance to an absolute tick (days × 86400), acknowledging interrupts.
pub fn advance_days(core: &mut SimCore, days: u64) {
    let to = core.status().tick + days * 86_400;
    loop {
        let now = core.status().tick;
        if now >= to {
            return;
        }
        if let StopReason::Interrupted(ids) = core.step(StepRequest::Ticks(to - now)).expect("step").stopped {
            for id in ids {
                core.acknowledge(id).expect("ack");
            }
        }
    }
}

pub fn snap(core: &SimCore, m: &ResearchModule) -> ResearchSnapshot {
    ResearchSnapshot::from_core(core, m).expect("snapshot")
}

/// Count events of a class.
pub fn count_events(core: &SimCore, class: &str) -> usize {
    core.events(&sojourn_core::EventFilter {
        classes: Some(vec![class.to_string()]),
        limit: 1_000_000,
        ..Default::default()
    })
    .expect("events")
    .events
    .len()
}
