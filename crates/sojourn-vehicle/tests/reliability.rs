//! US3 — reliability is earned: the reliability-block-diagram over FA-05 maturity.

mod common;
use common::*;
use sojourn_vehicle::DesignId;
use sojourn_vehicle::design::MissionReqs;

#[test]
fn series_and_redundancy_compose_per_block_diagram() {
    let m = vehicle_module();
    // Three distinct-tech components at r = 0.9 in series ⇒ 0.9³ = 0.729.
    let d = design(
        "tug",
        vec![stage(
            &["struct-allium", "tank-cryo", "eng-hydrolox"],
            100.0,
            Some("eng-hydrolox"),
        )],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
    let r = s.reliability(DesignId(0)).unwrap();
    assert!(
        (r.composed - 0.729).abs() < 1e-6,
        "series composition {}",
        r.composed
    );
    assert!(r.sub_trl6.is_empty());

    // Make struct + tank a redundancy block ⇒ parallel (1 − 0.1·0.1) = 0.99; ×0.9 = 0.891.
    let d2 = design(
        "tug",
        vec![stage(
            &["struct-allium", "tank-cryo", "eng-hydrolox"],
            100.0,
            Some("eng-hydrolox"),
        )],
        MissionReqs::default(),
        vec![redundancy(&["struct-allium", "tank-cryo"])],
    );
    let s2 = snapshot(m.catalogue.clone(), d2, maturity_for(&m, 0.9, 9));
    let r2 = s2.reliability(DesignId(0)).unwrap();
    assert!(
        r2.composed > r.composed,
        "redundancy raises reliability ({} > {})",
        r2.composed,
        r.composed
    );
    assert!(
        (r2.composed - 0.891).abs() < 1e-6,
        "block-diagram parallel ({})",
        r2.composed
    );
}

#[test]
fn sub_trl6_components_are_flagged() {
    let m = vehicle_module();
    let d = design(
        "tug",
        vec![stage(
            &["struct-allium", "eng-hydrolox"],
            100.0,
            Some("eng-hydrolox"),
        )],
        MissionReqs::default(),
        vec![],
    );
    let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.5, 4));
    let r = s.reliability(DesignId(0)).unwrap();
    assert!(!r.sub_trl6.is_empty(), "immature components surfaced");
    assert!(
        r.composed < 0.5,
        "immature parts compose to low reliability ({})",
        r.composed
    );
}
