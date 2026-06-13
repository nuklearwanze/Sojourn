//! US3 — breakthroughs are seeded + earned and basic-science-weighted (FR-RESP-302).

mod common;
use common::*;
use sojourn_research::DomainId;
use sojourn_research::ResearchCommand::*;

fn breakthroughs_funding(domain: &str, seed: u64) -> usize {
    let (mut core, _m) = core_with_research(seed);
    hire_scientists(&mut core, 0, domain, 8.0, 10);
    research(
        &mut core,
        SetAllocation {
            faction: 0,
            domain_splits: vec![(DomainId(domain.into()), 1.0)],
            program_splits: vec![],
            facilities: vec![],
        },
    );
    advance_days(&mut core, 365 * 10);
    count_events(&core, "breakthrough")
}

#[test]
fn breakthroughs_are_basic_science_weighted() {
    // Heavy investment in a basic-science domain (A4 Nuclear Physics) breaks through;
    // the same effort in an applied domain (A15 Geosciences) is far rarer.
    let basic = breakthroughs_funding("A4", 9);
    let applied = breakthroughs_funding("A15", 9);
    assert!(
        basic >= 1,
        "sustained basic-science investment breaks through ({basic})"
    );
    assert!(
        basic > applied,
        "applied-only is rarer ({basic} basic vs {applied} applied)"
    );
}
