//! US6 — planetary protection: graded forward contamination + back-contamination chain.

mod common;
use common::*;
use sojourn_polity::ids::BodyId;
use sojourn_polity::{MissionFacts, PolityCommand};

fn mission(
    body: u32,
    special: bool,
    bioburden: f64,
    crash: bool,
    sample_return: bool,
    chain: bool,
) -> MissionFacts {
    MissionFacts {
        body: BodyId(body),
        special_region: special,
        lander_bioburden: bioburden,
        crash,
        sample_return,
        containment_chain: chain,
    }
}

#[test]
fn forward_contamination_is_graded_by_overage() {
    let mut e = PolityH::new();
    // No explicit protections → the default COSPAR catalogue seeds Mars (body 4, limit 30, Special).
    e.init(vec![faction(0, false)], vec![candidate(4, 0.1)], vec![]);

    // A compliant (under-limit) lander degrades nothing.
    e.cmd(PolityCommand::EvaluateContamination(mission(
        4, true, 10.0, false, false, false,
    )));
    assert_eq!(
        e.snap().protection(4).unwrap().pristine_value,
        1.0,
        "a compliant lander degrades nothing"
    );

    // An over-limit crash degrades the pristine value (and confounds future evidence).
    e.cmd(PolityCommand::EvaluateContamination(mission(
        4, true, 120.0, true, false, false,
    )));
    let pv = e.snap().protection(4).unwrap().pristine_value;
    assert!(
        pv < 1.0,
        "an over-limit crash in a Special Region degrades pristine value ({pv})"
    );
    assert_eq!(
        e.snap().evidence_value(cid(4)),
        Some(pv),
        "the candidate's evidence value is confounded too"
    );
}

#[test]
fn forward_severity_is_monotone_in_overage() {
    let p = module().params;
    let cp = &p.protection.contamination;
    use sojourn_polity::protection::forward_severity;
    let small = forward_severity(40.0, 10.0, false, cp); // overage 3
    let large = forward_severity(110.0, 10.0, false, cp); // overage 10
    assert!(
        large > small && small > 0.0,
        "severity rises with bioburden overage ({large} > {small})"
    );
    assert!(
        forward_severity(110.0, 10.0, true, cp) >= large,
        "a crash is at least as severe as a soft landing"
    );
}

#[test]
fn back_contamination_is_gated_without_a_containment_chain() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![candidate(121, 0.1)], vec![]);
    e.cmd(PolityCommand::EvaluateContamination(mission(
        121, false, 1.0, false, true, false,
    )));
    assert!(
        e.snap().contamination_count(121) >= 1,
        "a sample return without a containment chain incurs a back-contamination penalty"
    );
}
