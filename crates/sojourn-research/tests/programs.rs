//! US2 — maturing a technology through TRL (FR-RESP-201/202/203/703).

mod common;
use common::*;
use sojourn_research::ResearchCommand::*;
use sojourn_research::{DomainId, FactionId, ProgramId, TechId};

/// Bring faction 0's A2 understanding above the B1.5 floor and staff it.
fn ready_a2(core: &mut sojourn_core::SimCore, facilities: Vec<String>) {
    hire_scientists(core, 0, "A2", 5.0, 6);
    hire_engineers(core, 0, "A2", 5.0, 8);
    research(core, SetAllocation {
        faction: 0,
        domain_splits: vec![(DomainId("A2".into()), 1.0)],
        program_splits: vec![],
        facilities: facilities.clone(),
    });
    advance_days(core, 400); // A2 past 60
}

#[test]
fn technology_matures_through_trl_with_reliability() {
    let (mut core, m) = core_with_research(5);
    let b15 = TechId("B1.5".into());
    ready_a2(&mut core, vec!["test-stand".into()]);
    research(&mut core, StartProgram { faction: 0, tech: b15.clone(), lead: None });
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(DomainId("A2".into()), 0.2)],
        program_splits: vec![(0, 1.0)],
        facilities: vec!["test-stand".into()],
    });

    let m0 = snap(&core, &m).maturity(FactionId(0), &b15).unwrap();
    assert_eq!(m0.trl, 6, "B1.5 starts at TRL 6");
    assert!(m0.flyable && m0.reliability > 0.0);

    advance_days(&mut core, 365 * 7);
    let s = snap(&core, &m);
    let mat = s.maturity(FactionId(0), &b15).unwrap();
    assert!(mat.trl > 6, "advanced past TRL 6 (got {})", mat.trl);
    assert!(mat.reliability > 0.0 && mat.reliability <= 0.99);
    // P80 carries the overrun margin above P50.
    let st = s.program_status(FactionId(0), ProgramId(0)).unwrap();
    assert!(st.p80_days > st.p50_days, "P80 {:.0} > P50 {:.0}", st.p80_days, st.p50_days);
}

#[test]
fn below_floor_start_is_rejected() {
    // Without A2 understanding, B1.5 cannot start (UL floor unmet).
    let (mut core, _m) = core_with_research(6);
    let out = core
        .submit(sojourn_research::research_payload(&StartProgram {
            faction: 0,
            tech: TechId("B1.5".into()),
            lead: None,
        }))
        .unwrap();
    let _ = out;
    // Apply: the command is rejected (a command-rejected event is journaled).
    core.step(sojourn_core::StepRequest::Ticks(0)).unwrap();
    let rejects = count_events(&core, "command-rejected");
    assert!(rejects >= 1, "starting below the UL floor is rejected");
}

#[test]
fn facility_gate_blocks_a_step() {
    // With A2 ready but NO test-stand, B1.5's TRL-7 step cannot complete.
    let (mut core, m) = core_with_research(7);
    let b15 = TechId("B1.5".into());
    ready_a2(&mut core, vec![]); // no test-stand
    research(&mut core, StartProgram { faction: 0, tech: b15.clone(), lead: None });
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(DomainId("A2".into()), 0.2)],
        program_splits: vec![(0, 1.0)],
        facilities: vec![], // still no test-stand
    });
    advance_days(&mut core, 365 * 3);
    assert_eq!(snap(&core, &m).maturity(FactionId(0), &b15).unwrap().trl, 6, "blocked at TRL 6 without the facility");

    // Provide the facility → it advances.
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(DomainId("A2".into()), 0.2)],
        program_splits: vec![(0, 1.0)],
        facilities: vec!["test-stand".into()],
    });
    advance_days(&mut core, 365 * 3);
    assert!(snap(&core, &m).maturity(FactionId(0), &b15).unwrap().trl > 6, "advances once the facility is available");
}

#[test]
fn heritage_raises_reliability_and_enables_a_derivative() {
    let (mut core, m) = core_with_research(8);
    let b15 = TechId("B1.5".into());
    let b16 = TechId("B1.6".into());
    ready_a2(&mut core, vec!["test-stand".into()]);
    research(&mut core, StartProgram { faction: 0, tech: b15.clone(), lead: None });
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(DomainId("A2".into()), 0.2)],
        program_splits: vec![(0, 1.0)],
        facilities: vec!["test-stand".into()],
    });
    advance_days(&mut core, 365 * 8); // mature toward 9

    let before = snap(&core, &m).maturity(FactionId(0), &b15).unwrap().reliability;
    research(&mut core, RegisterHeritage { faction: 0, tech: b15.clone(), units: 6 });
    let after = snap(&core, &m).maturity(FactionId(0), &b15).unwrap().reliability;
    assert!(after > before, "heritage raised reliability ({before:.3} → {after:.3})");

    // The derivative B1.6 (requires the proven B1.5 product) can now start at its
    // derivative TRL.
    research(&mut core, StartProgram { faction: 0, tech: b16.clone(), lead: None });
    let mat = snap(&core, &m).maturity(FactionId(0), &b16).unwrap();
    assert!(mat.trl >= 7, "derivative starts partway up the ladder (TRL {})", mat.trl);
}
