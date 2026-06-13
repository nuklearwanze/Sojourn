//! US6 — every vehicle from one component system: archetypes build with
//! class-appropriate outputs (crewed life-support sizing, lander EDL suitability),
//! and `compare()` diffs two designs.

mod common;
use common::*;
use sojourn_vehicle::design::{MissionReqs, VehicleDesign};
use sojourn_vehicle::{DesignId, FactionId};
use std::collections::BTreeMap;

#[test]
fn crew_lander_sizes_life_support_and_edl_from_shared_system() {
    let m = vehicle_module();
    // A crewed lander: structure + tank + a high-thrust engine able to lift off the
    // target body, ECLSS (crew + shield), a heat shield and landing gear.
    let d = design(
        "crew-lander",
        vec![stage(
            &[
                "struct-allium",
                "tank-cryo",
                "eng-methalox",
                "ls-eclss",
                "edl-shield",
                "gear-landing",
                "pwr-pv",
            ],
            8000.0,
            Some("eng-methalox"),
        )],
        MissionReqs {
            target_body_g: Some(1.62), // lunar surface gravity
            mission_days: Some(14.0),
            crew: Some(4),
            ..Default::default()
        },
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let out = s.derive(FactionId(0), DesignId(0), AU).unwrap();

    // Crewed-class output: ECLSS accommodates its crew and carries shielding + consumables.
    assert_eq!(
        out.life_support.crew_capacity, 4,
        "ECLSS crew accommodation"
    );
    assert!(
        out.life_support.shield_kg > 0.0,
        "radiation shield mass carried"
    );
    assert!(
        out.life_support.consumables_kg > 0.0,
        "open-loop consumables sized for the mission"
    );

    // Lander-class output: EDL suitability is computed against the target gravity,
    // the methalox engine out-thrusts lunar gravity, and a heat shield is present.
    let edl = out.edl.expect("lander derives EDL suitability");
    assert!(
        edl.landing_tw > 1.0,
        "T/W exceeds lunar gravity: {}",
        edl.landing_tw
    );
    assert!(edl.can_land, "propulsive landing feasible");
    assert!(edl.has_heat_shield, "carries a heat shield");
}

#[test]
fn compare_diffs_two_designs() {
    let m = vehicle_module();
    // Two tugs that differ only in propellant load ⇒ different Δv.
    let mut light = design(
        "tug",
        vec![stage(
            &["struct-allium", "tank-cryo", "eng-hydrolox"],
            2000.0,
            Some("eng-hydrolox"),
        )],
        MissionReqs::default(),
        vec![],
    );
    light.id = DesignId(0);
    let mut heavy = VehicleDesign {
        id: DesignId(1),
        stages: vec![stage(
            &["struct-allium", "tank-cryo", "eng-hydrolox"],
            18000.0,
            Some("eng-hydrolox"),
        )],
        ..light.clone()
    };
    heavy.id = DesignId(1);

    let mut designs = BTreeMap::new();
    designs.insert(light.id, light);
    designs.insert(heavy.id, heavy);
    let s = sojourn_vehicle::DesignSnapshot::new(
        designs,
        m.catalogue.clone(),
        maturity_for(&m, 0.9, 9),
    );

    let (a, b) = s.compare(DesignId(0), DesignId(1), AU).unwrap();
    assert!(
        b.total_dv > a.total_dv,
        "more propellant ⇒ more Δv ({} > {})",
        b.total_dv,
        a.total_dv
    );
    assert!(b.wet_mass > a.wet_mass, "heavier wet mass");
}
