//! Shared fixtures for the astro integration tests: kernel + astro module
//! assembly, spawn/advance/command helpers, snapshot access.
#![allow(dead_code)]

use sojourn_astro::{
    AstroCommand, AstroModule, AstroSnapshot, BodyId, StateVec, Vec3, astro_payload,
};
use sojourn_core::{DataSet, SimCore, StepRequest, StopReason};
use std::collections::BTreeMap;
use std::path::Path;

pub const TERRA: u32 = 1;
pub const LUNA: u32 = 2;
pub const ARES: u32 = 3;
pub const HEKLA: u32 = 4;

pub const MU_TERRA: f64 = 3.986004418e14;
pub const MU_LUNA: f64 = 4.9028e12;
pub const MU_SOL: f64 = 1.32712440018e20;

fn repo(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(path)
}

/// Kernel data set.
pub fn kernel_data() -> DataSet {
    DataSet::load_dir(&repo("data/kernel")).expect("kernel data")
}

/// Astro module with the repo data.
pub fn astro_module() -> AstroModule {
    AstroModule::load(&repo("data/astro")).expect("astro data")
}

/// Astro module with a tweaked config.
pub fn astro_module_with(f: impl FnOnce(&mut sojourn_astro::AstroConfig)) -> AstroModule {
    let mut m = astro_module();
    f(&mut m.config);
    m
}

/// A kernel core with one astro module installed.
pub fn core_with(module: AstroModule, seed: u64) -> (SimCore, AstroModule) {
    let cfg = sojourn_core::RunConfig {
        master_seed: seed,
        horizon_years: 100,
        run_mode: sojourn_core::RunMode::SaveAnywhere,
        difficulty_inputs: BTreeMap::new(),
    };
    // Keep a parallel module for snapshot extraction (same immutable data).
    let twin = AstroModule {
        catalog: module.catalog.clone(),
        engines: module.engines.clone(),
        config: module.config.clone(),
    };
    let core = SimCore::create(cfg, kernel_data(), vec![Box::new(module)]).expect("core");
    (core, twin)
}

/// Default core (repo config).
pub fn core(seed: u64) -> (SimCore, AstroModule) {
    core_with(astro_module(), seed)
}

/// Submit an astro command and apply it at the boundary.
pub fn astro(core: &mut SimCore, cmd: AstroCommand) {
    core.submit(astro_payload(&cmd)).expect("submit");
    let r = core.step(StepRequest::Ticks(0)).expect("apply");
    if let StopReason::Interrupted(ids) = r.stopped {
        for id in ids {
            core.acknowledge(id).expect("ack");
        }
        core.step(StepRequest::Ticks(0)).expect("apply ack");
    }
}

/// Advance to an absolute tick, acknowledging interrupts.
pub fn advance(core: &mut SimCore, to: u64) {
    loop {
        let now = core.status().tick;
        if now >= to {
            return;
        }
        let r = core.step(StepRequest::Ticks(to - now)).expect("step");
        if let StopReason::Interrupted(ids) = r.stopped {
            for id in ids {
                core.acknowledge(id).expect("ack");
            }
        }
    }
}

/// Snapshot the astro state.
pub fn snap(core: &SimCore, module: &AstroModule) -> AstroSnapshot {
    AstroSnapshot::from_core(core, module).expect("snapshot")
}

/// Spawn a craft in a circular orbit of radius `r` about a body (prograde, +z
/// orbit normal). Returns the craft id (sequential from 0).
pub fn spawn_circular(
    core: &mut SimCore,
    body: u32,
    mu: f64,
    r: f64,
    dry: f64,
    propellant: f64,
    engine: &str,
) -> u64 {
    let v = libm::sqrt(mu / r);
    spawn_state(
        core,
        body,
        StateVec {
            r: Vec3::new(r, 0.0, 0.0),
            v: Vec3::new(0.0, v, 0.0),
        },
        dry,
        propellant,
        engine,
    )
}

/// Spawn a craft with an explicit state.
pub fn spawn_state(
    core: &mut SimCore,
    body: u32,
    state: StateVec,
    dry: f64,
    propellant: f64,
    engine: &str,
) -> u64 {
    // Craft ids are sequential; read the count before spawning.
    let before = core.view("astro/status").expect("view");
    let id = match before.fields.get("craft_count") {
        Some(sojourn_core::ViewValue::U64(n)) => *n,
        _ => 0,
    };
    astro(
        core,
        AstroCommand::SpawnCraft {
            name_id: format!("craft-{id}"),
            dominant: body,
            state,
            dry_mass: dry,
            propellant,
            engine: engine.into(),
            available_power_w: 2300.0,
            drag_area_m2: 0.0,
            srp_area_m2: 0.0,
        },
    );
    id
}

/// Count events of a class.
pub fn count_events(core: &SimCore, class: &str) -> usize {
    core.events(&sojourn_core::EventFilter {
        classes: Some(vec![class.to_string()]),
        limit: 100_000,
        ..Default::default()
    })
    .expect("events")
    .events
    .len()
}

/// The craft's dominant-relative state from a snapshot.
pub fn craft_state(s: &AstroSnapshot, id: u64) -> (BodyId, StateVec) {
    let c = s.craft.get(&id).expect("craft");
    (c.dominant, c.state)
}
