//! US1 — understanding before capability (FR-RESP-101/102/104).

mod common;
use common::*;
use sojourn_research::ResearchCommand::*;
use sojourn_research::{DomainId, FactionId, TechId};

#[test]
fn understanding_grows_and_gates_engineering() {
    let (mut core, m) = core_with_research(1);
    let a2 = DomainId("A2".into());
    let b15 = TechId("B1.5".into());
    // Before: A2 = 0 ⇒ B1.5 (needs A2≥60) unavailable.
    let s0 = snap(&core, &m);
    assert_eq!(s0.understanding(FactionId(0), &a2).unwrap().private_ul, 0.0);
    assert!(!s0.available_programs(FactionId(0)).contains(&b15));

    hire_scientists(&mut core, 0, "A2", 4.0, 5);
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(a2.clone(), 1.0)],
        program_splits: vec![],
        facilities: vec![],
    });
    advance_days(&mut core, 500);

    let s1 = snap(&core, &m);
    let u = s1.understanding(FactionId(0), &a2).unwrap();
    assert!(u.private_ul > 60.0, "UL grew past the floor ({:.1})", u.private_ul);
    assert!(u.private_ul <= 100.0);
    assert!(s1.available_programs(FactionId(0)).contains(&b15), "B1.5 now available");
}

#[test]
fn diminishing_returns_slow_growth_near_the_knee() {
    let (mut core, m) = core_with_research(2);
    let a2 = DomainId("A2".into());
    hire_scientists(&mut core, 0, "A2", 4.0, 5);
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(a2.clone(), 1.0)],
        program_splits: vec![],
        facilities: vec![],
    });
    advance_days(&mut core, 60);
    let early = snap(&core, &m).understanding(FactionId(0), &a2).unwrap().private_ul;
    advance_days(&mut core, 600);
    let mid = snap(&core, &m).understanding(FactionId(0), &a2).unwrap().private_ul;
    advance_days(&mut core, 600);
    let late = snap(&core, &m).understanding(FactionId(0), &a2).unwrap().private_ul;
    let gain_low = mid - early;
    let gain_high = late - mid;
    assert!(gain_high < gain_low, "diminishing returns: {gain_high:.2} (high) < {gain_low:.2} (low)");
}

#[test]
fn mission_injection_raises_named_domains() {
    let (mut core, m) = core_with_research(3);
    let a15 = DomainId("A15".into());
    research(&mut core, InjectUnderstanding {
        faction: 0,
        injections: vec![(a15.clone(), 30.0)],
    });
    let s = snap(&core, &m);
    assert!((s.understanding(FactionId(0), &a15).unwrap().private_ul - 30.0).abs() < 1e-9);
    // A second faction is unaffected (per-faction).
    assert_eq!(s.understanding(FactionId(1), &a15).unwrap().private_ul, 0.0);
}
