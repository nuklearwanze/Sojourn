//! US2 — FA-02 flies a designer-built engine carried inline at spawn (T020).

mod common;
use common::*;
use sojourn_astro::{
    AstroCommand, AstroModule, AstroSnapshot, BodyId, StateVec, Vec3, astro_payload,
};
use sojourn_core::{DataSet, RunConfig, RunMode, SimCore, StepRequest, StopReason};
use sojourn_vehicle::DesignId;
use sojourn_vehicle::design::MissionReqs;
use std::collections::BTreeMap;

#[test]
fn fa02_flies_a_designer_built_engine() {
    let m = vehicle_module();
    let d = design(
        "tug",
        vec![stage(
            &["struct-allium", "tank-cryo", "eng-hydrolox"],
            18000.0,
            Some("eng-hydrolox"),
        )],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let edef = s
        .engine_defs(DesignId(0))
        .into_iter()
        .next()
        .expect("designer engine");

    // An astro core over the fixture catalogue (terra = id 1).
    let astro = AstroModule::load(&repo("data/astro")).unwrap();
    let twin = AstroModule::load(&repo("data/astro")).unwrap();
    let kernel = DataSet::load_dir(&repo("data/kernel")).unwrap();
    let mu = astro.catalog.body(BodyId(1)).unwrap().mu;
    let cfg = RunConfig {
        master_seed: 1,
        horizon_years: 100,
        run_mode: RunMode::SaveAnywhere,
        difficulty_inputs: BTreeMap::new(),
    };
    let mut core = SimCore::create(cfg, kernel, vec![Box::new(astro)]).unwrap();

    let r0 = 7.0e6;
    let v = libm::sqrt(mu / r0);
    core.submit(astro_payload(&AstroCommand::SpawnCraft {
        name_id: "designer-built".into(),
        dominant: 1,
        state: StateVec {
            r: Vec3::new(r0, 0.0, 0.0),
            v: Vec3::new(0.0, v, 0.0),
        },
        dry_mass: 1000.0,
        propellant: 18000.0,
        engine: "designer".into(), // ignored — inline engine wins
        inline_engine: Some(edef),
        available_power_w: 0.0,
        drag_area_m2: 0.0,
        srp_area_m2: 0.0,
    }))
    .unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();

    // Coast one day — a designer-built engine flies exactly like a fixture engine.
    let to = 86_400u64;
    loop {
        let now = core.status().tick;
        if now >= to {
            break;
        }
        if let StopReason::Interrupted(ids) =
            core.step(StepRequest::Ticks(to - now)).unwrap().stopped
        {
            for id in ids {
                core.acknowledge(id).unwrap();
            }
        }
    }
    let snap = AstroSnapshot::from_core(&core, &twin).unwrap();
    let c = snap.craft.get(&0).expect("craft");
    assert!(
        (c.state.r.norm() - r0).abs() / r0 < 0.02,
        "the designer-engine craft coasts on its circular orbit (r {:.0} vs {:.0})",
        c.state.r.norm(),
        r0
    );
}
