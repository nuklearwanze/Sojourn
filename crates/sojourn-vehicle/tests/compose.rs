//! US1 — compose & derive: analytic rocket equation, mass, traceability, gating.

mod common;
use common::*;
use sojourn_vehicle::design::MissionReqs;
use sojourn_vehicle::{ComponentId, DesignId, FactionId};

fn minimal() -> sojourn_vehicle::design::VehicleDesign {
    design(
        "tug",
        vec![stage(
            &["struct-allium", "tank-cryo", "eng-hydrolox"],
            18000.0,
            Some("eng-hydrolox"),
        )],
        MissionReqs::default(),
        vec![],
    )
}

#[test]
fn rocket_equation_and_mass_derive_analytically() {
    let m = vehicle_module();
    let s = snapshot(m.catalogue.clone(), minimal(), maturity_for(&m, 0.95, 9));
    let out = s.derive(FactionId(0), DesignId(0), AU).unwrap();
    // dry = 500 + 200 + 300 = 1000; wet = 1000 + 18000 = 19000.
    assert!((out.dry_mass - 1000.0).abs() < 1e-6, "dry {}", out.dry_mass);
    assert!(
        (out.wet_mass - 19000.0).abs() < 1e-6,
        "wet {}",
        out.wet_mass
    );
    // Δv = v_e·ln(m0/m1) = 450·g0·ln(19).
    let ve = 450.0 * 9.806_65;
    let expected = ve * libm::log(19000.0 / 1000.0);
    assert!(
        (out.total_dv - expected).abs() < 1.0,
        "Δv {} vs analytic {}",
        out.total_dv,
        expected
    );
}

#[test]
fn traceability_resolves_to_sourced_leaves() {
    let m = vehicle_module();
    let s = snapshot(m.catalogue.clone(), minimal(), maturity_for(&m, 0.95, 9));
    let mass_trace = s.trace_dry_mass(DesignId(0)).unwrap();
    assert!(
        mass_trace.all_leaves_sourced(),
        "dry-mass trace leaves are all sourced"
    );
    assert!((mass_trace.value() - 1000.0).abs() < 1e-6);
    assert!(mass_trace.leaf_count() >= 3);
    let dv_trace = s.trace_stage_dv(DesignId(0), 0).unwrap();
    assert!(
        dv_trace.all_leaves_sourced(),
        "Δv trace leaves are all sourced"
    );
}

#[test]
fn unresearched_component_is_unavailable() {
    let m = vehicle_module();
    // Empty maturity map ⇒ every technology unavailable.
    let s = snapshot(
        m.catalogue.clone(),
        minimal(),
        std::collections::BTreeMap::new(),
    );
    let (available, tech, _m) = s.availability(&ComponentId("eng-hydrolox".into())).unwrap();
    assert!(!available, "an unresearched component is unavailable");
    assert_eq!(tech, "B1.4", "gating technology reported");
}
