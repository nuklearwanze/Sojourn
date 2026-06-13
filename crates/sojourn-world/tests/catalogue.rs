//! US1 — the real Solar System loads, validates, and reproduces reality (T013/T014).

mod common;
use common::*;
use sojourn_astro::BodyId;
use sojourn_astro::bodies::rails::BodyStates;
use std::collections::BTreeMap;

const AU: f64 = 1.495_978_707e11;

#[test]
fn real_catalogue_loads_with_sources_and_counts() {
    let m = world_module();
    // Sun + 8 planets + Pluto + Ceres present.
    for id in 0..=10u32 {
        assert!(m.catalog.body(BodyId(id)).is_some(), "major body {id} present");
    }
    let (mut total, mut moons, mut small) = (0, 0, 0);
    for b in m.catalog.bodies() {
        total += 1;
        assert!(!b.source.trim().is_empty(), "{} carries a source", b.name_id);
        assert!(b.mu > 0.0 && b.radius > 0.0, "{} has positive mu/radius", b.name_id);
        match b.parent {
            Some(p) if p != BodyId(0) => moons += 1,
            Some(_) if b.id.0 >= 1000 => small += 1,
            _ => {}
        }
    }
    // Curated real subset (the ≥140 moons / ≥2,800 small-body production target is the
    // documented offline data-build step, FR-WORLD-106; see data/world/sources/README.md).
    assert!(total >= 20, "catalogue size {total}");
    assert!(moons >= 9, "moons {moons}");
    assert!(small >= 5, "small bodies {small}");
}

#[test]
fn divertibility_is_flag_driven_and_consistent() {
    let m = world_module();
    for b in m.catalog.bodies() {
        // The body-catalogue contract: a divertible body never gravitates far-field.
        assert!(!(b.divertible && b.gravitating), "{} divertible⇒!gravitating", b.name_id);
    }
    // Real NEAs are divertible; planets are not.
    assert!(m.catalog.body(BodyId(1002)).unwrap().divertible, "Bennu divertible");
    assert!(!m.catalog.body(BodyId(3)).unwrap().divertible, "Earth not divertible");
}

#[test]
fn heliocentric_distances_match_reality() {
    // An independent reality check (FR-WORLD-103 spirit): the rails place planets at
    // their real heliocentric distances. |r| lies within a(1±e) of the Sun.
    let m = world_module();
    let motions = BTreeMap::new();
    let mut st = BodyStates::new(&m.catalog, &motions, 0);
    let check = |st: &mut BodyStates, id: u32, a_au: f64, e: f64| {
        let r = st.absolute(BodyId(id)).r.norm() / AU;
        let lo = a_au * (1.0 - e) - 0.02;
        let hi = a_au * (1.0 + e) + 0.02;
        assert!(r > lo && r < hi, "body {id} at {r:.3} AU, expected {a_au}±e");
    };
    check(&mut st, 3, 1.0, 0.017); // Earth
    check(&mut st, 4, 1.524, 0.093); // Mars
    check(&mut st, 5, 5.203, 0.048); // Jupiter
    check(&mut st, 10, 2.766, 0.078); // Ceres
}

#[test]
fn catalogue_loads_under_the_propagator_unchanged() {
    // T015: the FA-02 propagator flies on the real catalogue. Spawn a circular LEO
    // craft about the real Earth and coast a day — radius must be ~conserved.
    use sojourn_astro::{AstroCommand, AstroModule, AstroSnapshot, StateVec, Vec3, astro_payload};
    use sojourn_core::{RunConfig, RunMode, SimCore, StepRequest, StopReason};

    let astro = AstroModule::load(&repo("data/world")).expect("astro on real catalogue");
    let twin = AstroModule::load(&repo("data/world")).expect("twin");
    let mu_earth = astro.catalog.body(BodyId(3)).unwrap().mu;
    let cfg = RunConfig {
        master_seed: 1,
        horizon_years: 100,
        run_mode: RunMode::SaveAnywhere,
        difficulty_inputs: std::collections::BTreeMap::new(),
    };
    let mut core =
        SimCore::create(cfg, kernel_data(), vec![Box::new(astro)]).expect("core");

    let r0 = 6.771e6; // ~400 km LEO
    let v = libm::sqrt(mu_earth / r0);
    core.submit(astro_payload(&AstroCommand::SpawnCraft {
        name_id: "probe".into(),
        dominant: 3,
        state: StateVec {
            r: Vec3::new(r0, 0.0, 0.0),
            v: Vec3::new(0.0, v, 0.0),
        },
        dry_mass: 1000.0,
        propellant: 0.0,
        engine: "chem-hydrolox".into(),
        available_power_w: 0.0,
        drag_area_m2: 0.0,
        srp_area_m2: 0.0,
    }))
    .unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();
    // Coast one day.
    let mut to = 86_400u64;
    loop {
        let now = core.status().tick;
        if now >= to {
            break;
        }
        if let StopReason::Interrupted(ids) = core.step(StepRequest::Ticks(to - now)).unwrap().stopped
        {
            for id in ids {
                core.acknowledge(id).unwrap();
            }
            to = to.max(core.status().tick);
        }
    }
    let s = AstroSnapshot::from_core(&core, &twin).unwrap();
    let c = s.craft.get(&0).expect("craft");
    let r = c.state.r.norm();
    assert!((r - r0).abs() / r0 < 0.02, "circular orbit radius ~conserved ({r:.0} vs {r0:.0})");
}
