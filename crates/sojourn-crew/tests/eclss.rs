//! US5 — ECLSS spares, maintenance & failure (Q1).

mod common;
use common::*;
use sojourn_crew::ids::AssetId;
use sojourn_crew::{CrewCommand, Sex, eclss};

#[test]
fn lower_maturity_fails_more_often() {
    let mut e = CrewH::new();
    // Asset 0: low TRL (5); asset 1: high TRL (9).
    e.cmd(CrewCommand::OccupyAsset {
        faction: 0,
        asset: 0,
        sizing: sizing(0.9, 1.0, false, true),
        env: env(0.6, "default", true),
        eclss_maturity: maturity(5, 0.8, 0),
        ops_oversub: 0.0,
        consumables_kg: 1.0e7,
    });
    e.cmd(CrewCommand::OccupyAsset {
        faction: 0,
        asset: 1,
        sizing: sizing(0.9, 1.0, false, true),
        env: env(0.6, "default", true),
        eclss_maturity: maturity(9, 0.99, 10),
        ops_oversub: 0.0,
        consumables_kg: 1.0e7,
    });
    let s = e.snap();
    assert!(
        s.eclss_risk(AssetId(0)).unwrap().failure_prob
            > s.eclss_risk(AssetId(1)).unwrap().failure_prob,
        "lower maturity ⇒ higher failure probability"
    );
}

#[test]
fn maintenance_lowers_the_failure_probability() {
    let p = module().params;
    let mut maintained = asset(sizing(0.9, 1.0, false, true), env(0.6, "default", true));
    let mut unmaintained = asset(sizing(0.9, 1.0, false, true), env(0.6, "default", true));
    for _ in 0..50 {
        eclss::accrue(
            &mut maintained.eclss,
            p.eclss.crew_hr_per_day,
            p.eclss.spares_per_day,
            &p.eclss,
        );
        eclss::accrue(&mut unmaintained.eclss, 0.0, 0.0, &p.eclss);
    }
    assert!(maintained.eclss.maintenance_deficit < unmaintained.eclss.maintenance_deficit);
    assert!(
        eclss::failure_prob(&maintained, &p.eclss) < eclss::failure_prob(&unmaintained, &p.eclss),
        "maintenance lowers the failure probability"
    );
}

#[test]
fn critical_failure_beyond_abort_reach_is_a_loss_of_crew_risk() {
    let _ = Sex::Male;
    let mut far = asset(
        sizing(0.9, 1.0, false, true),
        env(0.6, "mars-transit", false),
    );
    far.eclss.failed = true;
    assert!(
        eclss::critical(&far),
        "failed ECLSS beyond abort reach is critical"
    );
    let mut near = asset(sizing(0.9, 1.0, false, true), env(0.6, "leo", true));
    near.eclss.failed = true;
    assert!(!eclss::critical(&near), "near Earth an abort is an option");
}
