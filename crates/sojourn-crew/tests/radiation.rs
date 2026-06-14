//! US2 — radiation dose accumulation & the REID limit (Q2).

mod common;
use common::*;
use sojourn_crew::asset::CrewStatus;
use sojourn_crew::{CrewCommand, Sex, radiation};

#[test]
fn dose_accrues_and_reid_grounds_the_astronaut() {
    let mut e = CrewH::new();
    // No shielding (attenuation 1.0), young female ⇒ REID climbs fast.
    e.occupy(
        0,
        sizing(0.9, 1.0, false, true),
        env(0.6, "default", true),
        1.0e7,
        &[("alice", facts(30.0, Sex::Female))],
    );
    e.advance(300 * DAY);
    let s = e.snap();
    assert!(
        s.member("alice").unwrap().career_dose_sv > 0.4,
        "dose accrued"
    );
    assert!(s.reid("alice").unwrap() >= 3.0, "REID reached 3%");
    assert_eq!(s.member("alice").unwrap().status, CrewStatus::Grounded);
}

#[test]
fn sheltering_reduces_the_spe_dose() {
    let p = module().params;
    let mut a = asset(sizing(0.9, 1.0, false, true), env(0.6, "default", true));
    a.sheltering = false;
    let unsheltered = radiation::spe_dose(&a, &p.radiation);
    a.sheltering = true;
    let sheltered = radiation::spe_dose(&a, &p.radiation);
    assert!(
        sheltered < unsheltered,
        "storm shelter cuts the acute SPE dose ({sheltered} < {unsheltered})"
    );
}

#[test]
fn career_dose_carries_across_missions() {
    let mut e = CrewH::new();
    e.occupy(
        0,
        sizing(0.9, 1.0, false, true),
        env(0.6, "default", true),
        1.0e7,
        &[("bob", facts(40.0, Sex::Male))],
    );
    e.advance(100 * DAY);
    let dose1 = e.snap().member("bob").unwrap().career_dose_sv;
    assert!(dose1 > 0.0);

    // Vacate, then re-occupy a new asset and re-assign Bob — his career dose persists.
    e.cmd(CrewCommand::VacateAsset { asset: 0 });
    e.occupy(
        1,
        sizing(0.9, 1.0, false, true),
        env(0.6, "default", true),
        1.0e7,
        &[("bob", facts(40.0, Sex::Male))],
    );
    e.advance(100 * DAY);
    let dose2 = e.snap().member("bob").unwrap().career_dose_sv;
    assert!(
        dose2 > dose1 * 1.5,
        "career dose carried over + accrued more ({dose2} vs {dose1})"
    );
}
