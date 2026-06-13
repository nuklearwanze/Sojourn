//! US2 — propulsion is physics: endpoint conformance, power-limited EP, NEP mass.

mod common;
use common::*;
use sojourn_vehicle::design::MissionReqs;
use sojourn_vehicle::{DesignId, FactionId};

#[test]
fn engine_def_is_fa02_conformant_and_power_limited() {
    let m = vehicle_module();
    // EP on an RTG: 300 W available < 2300 W rated ⇒ thrust scales down.
    let d = design(
        "probe",
        vec![stage(
            &["struct-allium", "pwr-rtg", "eng-ion"],
            0.0,
            Some("eng-ion"),
        )],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let defs = s.engine_defs(DesignId(0));
    assert_eq!(defs.len(), 1);
    assert!(defs[0].power_limited, "ion engine is power-limited");
    assert!((defs[0].exhaust_velocity() - 3100.0 * 9.806_65).abs() < 1.0);

    let out = s.derive(FactionId(0), DesignId(0), AU).unwrap();
    assert!(
        out.max_thrust < 0.092,
        "power-limited thrust {} < max 0.092",
        out.max_thrust
    );
    assert!(
        (out.max_thrust - 0.092 * (300.0 / 2300.0)).abs() < 1e-4,
        "thrust scales with power"
    );
}

#[test]
fn nep_carries_reactor_and_radiator_mass() {
    let m = vehicle_module();
    let d = design(
        "tug",
        vec![stage(
            &["struct-allium", "pwr-reactor", "rad-panel", "eng-nep"],
            5000.0,
            Some("eng-nep"),
        )],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let out = s.derive(FactionId(0), DesignId(0), AU).unwrap();
    // dry = struct 500 + pwr-reactor 1500 + rad 200 + eng-nep (400 + reactor 2000) = 4600.
    assert!(
        (out.dry_mass - 4600.0).abs() < 1e-6,
        "NEP dry mass incl. reactor+radiator ({})",
        out.dry_mass
    );
    // radiator (200 kg × 500 W/kg = 100 kW) rejects the engine's 80 kW heat.
    assert!(
        out.thermal.margin_w > 0.0,
        "radiator sized for the waste heat"
    );
}
