//! US2 — the politics of money and approval (mood, modifiers, loss-of-crew severity).

mod common;
use common::*;
use sojourn_polity::PolityCommand;
use sojourn_polity::ids::FactionId;

#[test]
fn world_first_lifts_mood_then_it_decays() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![], vec![]);
    e.cmd(PolityCommand::RecordOutcome {
        crew: crew_facts(0, false, false, false, true),
        economy: econ(0, 1.0e9, 0.0, 0.0),
    });
    let lifted = e.snap().mood(FactionId(0)).unwrap();
    assert!(lifted > 0.0, "a world-first lifts mood");
    assert!(
        e.snap()
            .modifiers(FactionId(0))
            .unwrap()
            .appropriation_factor
            > 1.0,
        "high mood improves the budget modifier"
    );

    e.advance(200 * DAY);
    let decayed = e.snap().mood(FactionId(0)).unwrap();
    assert!(
        decayed < lifted && decayed >= 0.0,
        "mood decays back toward the baseline ({decayed} < {lifted})"
    );
}

#[test]
fn loss_of_crew_is_deeper_and_freezes_approval() {
    // Two identical factions: one suffers a routine failure, one a loss-of-crew.
    let mut e = PolityH::new();
    e.init(vec![faction(0, false), faction(1, false)], vec![], vec![]);
    e.cmd(PolityCommand::RecordOutcome {
        crew: crew_facts(0, false, true, false, false),
        economy: econ(0, 1.0e9, 0.0, 0.0),
    });
    e.cmd(PolityCommand::RecordOutcome {
        crew: crew_facts(1, true, false, false, false),
        economy: econ(1, 1.0e9, 0.0, 0.0),
    });
    let s = e.snap();
    let routine = s.mood(FactionId(0)).unwrap();
    let loc = s.mood(FactionId(1)).unwrap();
    assert!(
        loc < routine,
        "loss-of-crew is a deeper mood hit than a routine failure ({loc} < {routine})"
    );
    assert!(
        s.modifiers(FactionId(1)).unwrap().crewed_frozen,
        "loss-of-crew freezes crewed-flight approval"
    );
    assert_eq!(
        s.approval(FactionId(1), "crewed"),
        Some(sojourn_polity::query::Approval::Denied)
    );
    assert!(
        !s.modifiers(FactionId(0)).unwrap().crewed_frozen,
        "a routine failure does not freeze approval"
    );

    // The freeze persists for years, not days.
    e.advance(365 * DAY);
    assert!(
        e.snap().modifiers(FactionId(1)).unwrap().crewed_frozen,
        "the freeze is multi-year"
    );
}
