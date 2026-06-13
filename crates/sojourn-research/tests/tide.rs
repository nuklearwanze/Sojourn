//! US4 — the global science tide (FR-RESP-401/402).

mod common;
use common::*;
use sojourn_research::ResearchCommand::*;
use sojourn_research::{DomainId, FactionId};

fn world_ul_after(publish: bool, seed: u64) -> f64 {
    let (mut core, m) = core_with_research(seed);
    hire_scientists(&mut core, 0, "A2", 6.0, 6);
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(DomainId("A2".into()), 1.0)],
        program_splits: vec![],
        facilities: vec![],
    });
    if publish {
        research(&mut core, SetPublishPolicy { faction: 0, domain: DomainId("A2".into()), publish: true });
    }
    advance_days(&mut core, 1000);
    snap(&core, &m).tide(&DomainId("A2".into())).unwrap()
}

#[test]
fn tide_advances_and_publishing_accelerates_it() {
    let w_pub = world_ul_after(true, 11);
    let w_hold = world_ul_after(false, 11);
    assert!(w_hold > 0.0, "tide advances from aggregate + baseline even when holding ({w_hold:.3})");
    assert!(w_pub > w_hold, "publishing accelerates World UL ({w_pub:.3} > {w_hold:.3})");
}

#[test]
fn publish_emits_a_prestige_eligible_event_and_private_leads_world() {
    let (mut core, m) = core_with_research(12);
    hire_scientists(&mut core, 0, "A2", 6.0, 6);
    research(&mut core, SetAllocation {
        faction: 0,
        domain_splits: vec![(DomainId("A2".into()), 1.0)],
        program_splits: vec![],
        facilities: vec![],
    });
    research(&mut core, SetPublishPolicy { faction: 0, domain: DomainId("A2".into()), publish: true });
    advance_days(&mut core, 600);
    assert!(count_events(&core, "publish") >= 1, "publish emits an event (FA-09 prestige hook)");
    let s = snap(&core, &m);
    let priv_ul = s.understanding(FactionId(0), &DomainId("A2".into())).unwrap().private_ul;
    let world = s.tide(&DomainId("A2".into())).unwrap();
    assert!(priv_ul >= world, "a faction's private UL = world + lead ({priv_ul:.2} ≥ {world:.2})");
}
