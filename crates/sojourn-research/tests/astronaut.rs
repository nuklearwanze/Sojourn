//! US6 — the astronaut pipeline (FR-RESP-601/602): select → train → ready, and the
//! FA-08 `CrewFeedback` career update.

mod common;
use common::*;
use sojourn_research::ResearchCommand::*;
use sojourn_research::personnel::Role;
use sojourn_research::{DomainId, FactionId};

#[test]
fn pipeline_trains_to_ready_then_crew_feedback_retires_over_limit() {
    let (mut core, m) = core_with_research(15);
    // Select an astronaut candidate (person 0).
    research(
        &mut core,
        Hire {
            faction: 0,
            role: Role::Astronaut,
            discipline: DomainId("A13".into()),
            skill: 5.0,
            traits: vec![],
        },
    );
    assert_eq!(
        snap(&core, &m).personnel(FactionId(0))["AstronautReady"],
        0,
        "candidate is not yet ready"
    );

    // Train (730 days) → Ready.
    research(&mut core, Train { person: 0 });
    advance_days(&mut core, 800);
    assert_eq!(
        snap(&core, &m).personnel(FactionId(0))["AstronautReady"],
        1,
        "training completes → ready"
    );

    // FA-08 in-mission feedback over the career dose limit → leaves the ready pool.
    research(
        &mut core,
        CrewFeedback {
            faction: 0,
            astronaut: 0,
            dose_delta: 1100.0, // > career_dose_limit (1000)
            health_delta: 0.0,
            psych_delta: 0.0,
        },
    );
    assert_eq!(
        snap(&core, &m).personnel(FactionId(0))["AstronautReady"],
        0,
        "over the dose limit → retired"
    );
}

#[test]
fn training_requires_the_time_gate() {
    let (mut core, m) = core_with_research(16);
    research(
        &mut core,
        Hire {
            faction: 0,
            role: Role::Astronaut,
            discipline: DomainId("A13".into()),
            skill: 5.0,
            traits: vec![],
        },
    );
    research(&mut core, Train { person: 0 });
    advance_days(&mut core, 300); // < 730
    assert_eq!(
        snap(&core, &m).personnel(FactionId(0))["AstronautReady"],
        0,
        "not ready before the training gate"
    );
}
