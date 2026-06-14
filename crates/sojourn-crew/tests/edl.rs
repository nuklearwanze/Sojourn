//! US6 — entry/descent/landing & aerocapture crew-risk (the Mars EDL gap, Q1).

mod common;
use common::*;
use sojourn_crew::{CrewCommand, EdlSuitability, Sex};

/// A well-suited lander (heat shield + propulsive landing).
fn suit_good() -> EdlSuitability {
    EdlSuitability {
        can_land: true,
        has_heat_shield: true,
        landing_tw: 1.5,
    }
}

#[test]
fn mars_is_harder_than_airless() {
    let p = module().params;
    let mars = sojourn_crew::edl::crew_loss_prob(&suit_good(), "mars", 1.0, &p.edl);
    let moon = sojourn_crew::edl::crew_loss_prob(&suit_good(), "moon", 1.0, &p.edl);
    assert!(
        mars > moon,
        "Mars EDL is materially harder than an airless body ({mars} > {moon})"
    );
    // The Moon is the airless baseline (difficulty 1.0).
    assert!(
        (moon - p.edl.base_rate).abs() < 1e-12,
        "airless baseline = base rate"
    );
}

#[test]
fn edl_failure_loses_crew() {
    let mut e = CrewH::new();
    // A crewed lander attempting Mars with no heat shield and unable to land:
    // 0.02 × 5(mars) × 3(no shield) × 5(can't land) × 1(full capability) ⇒ clamps to 1.0.
    e.occupy(
        0,
        sizing(0.9, 1.0, false, true),
        env(0.6, "mars", false),
        1.0e7,
        &[("bob", facts(40.0, Sex::Male))],
    );
    e.cmd(CrewCommand::EvaluateEdl {
        asset: 0,
        suitability: EdlSuitability {
            can_land: false,
            has_heat_shield: false,
            landing_tw: 0.4,
        },
    });
    assert!(
        e.snap().loss_of_crew("bob"),
        "a certain-failure EDL is a loss-of-crew"
    );
    // A well-suited descent onto an airless body is survivable: the same crew
    // would not certainly be lost (probability ≪ 1).
    let well = sojourn_crew::edl::crew_loss_prob(&suit_good(), "moon", 1.0, &module().params.edl);
    assert!(
        well < 1.0,
        "a good airless landing is not a certain loss ({well})"
    );
}
