//! US8 — Grand Goals as win/scoring conditions (pass/fail from composed inputs + score).

mod common;
use common::*;
use sojourn_polity::goals::Verdict;
use sojourn_polity::ids::{CandidateId, FactionId, GoalKind};
use sojourn_polity::{EvidenceInput, EvidenceStage, PolityCommand};

fn full_chain(e: &mut PolityH, faction: u32, cand: CandidateId) {
    for stage in [
        EvidenceStage::OrbitalHint,
        EvidenceStage::InSitu,
        EvidenceStage::Microscopy,
        EvidenceStage::SampleReturn,
    ] {
        e.cmd(PolityCommand::CollectEvidence(EvidenceInput {
            faction: FactionId(faction),
            candidate: cand,
            stage,
            quality: 1.0,
        }));
    }
}

#[test]
fn seeker_passes_with_three_conclusive_worlds() {
    let mut e = PolityH::new();
    e.init(
        vec![faction(0, false)],
        vec![candidate(4, 1.0), candidate(111, 1.0), candidate(121, 1.0)],
        vec![],
    );
    e.cmd(PolityCommand::SelectGrandGoal {
        faction: FactionId(0),
        kind: GoalKind::Seeker,
    });
    for body in [4, 111, 121] {
        full_chain(&mut e, 0, cid(body));
    }
    e.advance(DAY);
    let g = e.snap().grand_goal(FactionId(0)).unwrap();
    assert_eq!(
        g.verdict,
        Verdict::Pass,
        "three conclusive worlds satisfy Seeker"
    );
}

#[test]
fn prospector_passes_with_enough_off_earth_tonnage() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![], vec![]);
    e.cmd(PolityCommand::SelectGrandGoal {
        faction: FactionId(0),
        kind: GoalKind::Prospector,
    });
    // Threshold is 100 t; record 200 t of profitable off-Earth tonnage.
    e.cmd(PolityCommand::RecordOutcome {
        crew: crew_facts(0, false, false, true, false),
        economy: econ(0, 1.0e9, 0.0, 200.0),
    });
    assert_eq!(
        e.snap().grand_goal(FactionId(0)).unwrap().verdict,
        Verdict::Pass
    );
}

#[test]
fn changing_goal_applies_a_penalty_to_the_score() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![], vec![]);
    e.cmd(PolityCommand::SelectGrandGoal {
        faction: FactionId(0),
        kind: GoalKind::Prospector,
    });
    e.cmd(PolityCommand::RecordOutcome {
        crew: crew_facts(0, false, false, true, false),
        economy: econ(0, 1.0e9, 0.0, 200.0),
    });
    let before = e.snap().score(FactionId(0)).unwrap().composite;
    e.cmd(PolityCommand::ChangeGrandGoal {
        faction: FactionId(0),
        kind: GoalKind::Prospector,
    });
    let after = e.snap().score(FactionId(0)).unwrap().composite;
    assert!(
        after < before,
        "changing Grand Goal applies a penalty ({after} < {before})"
    );
}
