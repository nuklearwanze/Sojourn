//! US7 — the maturity/heritage/understanding query surface (FR-RESP-801): faction
//! privacy and flyability gating.

mod common;
use common::*;
use sojourn_research::ResearchCommand::*;
use sojourn_research::{DomainId, FactionId, TechId};

#[test]
fn queries_are_faction_scoped_and_flyability_gated() {
    let (mut core, m) = core_with_research(7);
    let a2 = DomainId("A2".into());
    // Faction 0 builds A2; faction 1 does nothing.
    hire_scientists(&mut core, 0, "A2", 5.0, 6);
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(a2.clone(), 1.0)],
        program_splits: vec![],
        facilities: vec![],
    });
    advance_days(&mut core, 400);
    let s = snap(&core, &m);

    // Per-faction privacy: faction 0 understands A2, faction 1 does not.
    assert!(s.understanding(FactionId(0), &a2).unwrap().private_ul > 50.0);
    assert_eq!(s.understanding(FactionId(1), &a2).unwrap().private_ul, 0.0);

    // A technology with no program: TRL 0, unflyable, reliability 0.
    let mat = s.maturity(FactionId(0), &TechId("B3.1".into())).unwrap();
    assert_eq!(mat.trl, 0);
    assert!(!mat.flyable);
    assert_eq!(mat.reliability, 0.0);

    // The tide (world UL) is the shared channel — present for a known domain.
    assert!(s.tide(&a2).is_some());
}
