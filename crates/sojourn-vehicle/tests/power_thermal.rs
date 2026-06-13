//! US5 — power & thermal balance.

mod common;
use common::*;
use sojourn_vehicle::design::MissionReqs;
use sojourn_vehicle::{DesignId, FactionId};

#[test]
fn pv_generation_falls_with_solar_distance() {
    let m = vehicle_module();
    let d = design(
        "probe",
        vec![stage(&["struct-allium", "pwr-pv"], 0.0, None)],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let near = s
        .derive(FactionId(0), DesignId(0), AU)
        .unwrap()
        .power
        .generation_w;
    let far = s
        .derive(FactionId(0), DesignId(0), 5.0 * AU)
        .unwrap()
        .power
        .generation_w;
    assert!((near - 30_000.0).abs() < 1.0, "PV full at 1 AU ({near})");
    assert!(
        (far - 30_000.0 / 25.0).abs() < 10.0,
        "PV ~1/25 at 5 AU ({far})"
    );
    assert!(far < near, "generation falls with solar distance");
}

#[test]
fn radiator_shortfall_is_flagged() {
    let m = vehicle_module();
    // A NEP engine (80 kW waste heat) with no radiator ⇒ thermal margin negative.
    let d = design(
        "tug",
        vec![stage(
            &["struct-allium", "pwr-reactor", "eng-nep"],
            1000.0,
            Some("eng-nep"),
        )],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let out = s.derive(FactionId(0), DesignId(0), AU).unwrap();
    assert!(out.thermal.margin_w < 0.0, "heat exceeds rejection");
    let flags = s.red_flags(DesignId(0), AU).unwrap();
    assert!(
        flags.iter().any(|f| f.constraint.contains("radiator")),
        "radiator shortfall flagged: {flags:?}"
    );
}
