//! US5 — people make it happen (FR-RESP-502/503): roster transitions and the
//! tacit-knowledge effective-UL recompute.

mod common;
use common::*;
use sojourn_research::ResearchCommand::*;
use sojourn_research::personnel::Role;
use sojourn_research::{DomainId, FactionId};

#[test]
fn tacit_knowledge_lowers_effective_ul_without_touching_ground_ul() {
    let (mut core, m) = core_with_research(13);
    let a2 = DomainId("A2".into());
    // One A2 scientist (person 0) builds understanding.
    research(&mut core, Hire { faction: 0, role: Role::Scientist, discipline: a2.clone(), skill: 6.0, traits: vec![] });
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(a2.clone(), 1.0)],
        program_splits: vec![],
        facilities: vec![],
    });
    advance_days(&mut core, 400);
    let u1 = snap(&core, &m).understanding(FactionId(0), &a2).unwrap();
    assert!(u1.private_ul > 30.0);
    assert!((u1.private_ul - u1.effective_ul).abs() < 1e-9, "specialist present ⇒ no tacit penalty");

    // Lose the last A2 scientist → effective UL drops, ground UL unchanged.
    research(&mut core, Retire { person: 0 });
    let u2 = snap(&core, &m).understanding(FactionId(0), &a2).unwrap();
    assert_eq!(u2.private_ul, u1.private_ul, "ground UL is not mutated");
    assert!(u2.effective_ul < u2.private_ul, "tacit-knowledge loss lowers effective UL");
    assert!((u2.private_ul - u2.effective_ul - 12.0).abs() < 1e-6, "penalty = tacit_per_head (12)");

    // Re-hire restores it (recompute, not a destroyed value).
    research(&mut core, Hire { faction: 0, role: Role::Scientist, discipline: a2.clone(), skill: 6.0, traits: vec![] });
    let u3 = snap(&core, &m).understanding(FactionId(0), &a2).unwrap();
    assert!((u3.private_ul - u3.effective_ul).abs() < 1e-9, "re-hiring restores effective UL");
}

#[test]
fn roster_transitions_are_deterministic() {
    let (mut core, m) = core_with_research(14);
    hire_scientists(&mut core, 0, "A4", 5.0, 3);
    hire_engineers(&mut core, 0, "A2", 5.0, 2);
    let counts = snap(&core, &m).personnel(FactionId(0));
    assert_eq!(counts.get("Scientist").copied().unwrap_or(0), 3);
    assert_eq!(counts.get("Engineer").copied().unwrap_or(0), 2);
    research(&mut core, Retire { person: 0 });
    assert_eq!(snap(&core, &m).personnel(FactionId(0)).get("Scientist").copied().unwrap_or(0), 2);
}
