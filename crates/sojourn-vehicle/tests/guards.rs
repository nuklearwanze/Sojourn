//! US4 — the designer refuses the impossible (realism guards).

mod common;
use common::*;
use sojourn_vehicle::DesignId;
use sojourn_vehicle::design::MissionReqs;
use sojourn_vehicle::guards::Severity;

#[test]
fn lander_tw_below_local_gravity_is_hard_flagged() {
    let m = vehicle_module();
    // A tiny-thrust ion engine as an Earth-g lander ⇒ T/W ≪ 1.
    let d = design(
        "lander",
        vec![stage(
            &["struct-allium", "pwr-pv", "eng-ion", "gear-landing"],
            0.0,
            Some("eng-ion"),
        )],
        MissionReqs {
            target_body_g: Some(9.81),
            ..Default::default()
        },
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let flags = s.red_flags(DesignId(0), AU).unwrap();
    let f = flags
        .iter()
        .find(|f| f.constraint.contains("T/W"))
        .expect("lander T/W flag");
    assert_eq!(f.severity, Severity::Hard);
}

#[test]
fn negative_power_margin_is_hard_flagged() {
    let m = vehicle_module();
    // Payload + comms demand power; no generator ⇒ negative margin.
    let d = design(
        "probe",
        vec![stage(
            &["struct-allium", "payload-sci", "comms-ka"],
            0.0,
            None,
        )],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let flags = s.red_flags(DesignId(0), AU).unwrap();
    assert!(
        flags
            .iter()
            .any(|f| f.constraint.contains("power") && f.severity == Severity::Hard),
        "negative power margin hard-flagged: {flags:?}"
    );
}

#[test]
fn dv_shortfall_is_a_soft_buildable_flag() {
    let m = vehicle_module();
    let d = design(
        "tug",
        vec![stage(
            &["struct-allium", "tank-cryo", "eng-hydrolox"],
            1000.0,
            Some("eng-hydrolox"),
        )],
        MissionReqs {
            dv: Some(50_000.0), // far beyond what this design delivers
            ..Default::default()
        },
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let flags = s.red_flags(DesignId(0), AU).unwrap();
    let f = flags
        .iter()
        .find(|f| f.constraint.contains("Δv"))
        .expect("Δv shortfall flag");
    assert_eq!(
        f.severity,
        Severity::Soft,
        "Δv shortfall is a buildable (soft) gamble"
    );
}
