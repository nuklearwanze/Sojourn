//! US5 — policy & treaties with real consequences (gating, lobbying bounds, PP source).

mod common;
use common::*;
use sojourn_polity::PolityCommand;
use sojourn_polity::ids::FactionId;
use sojourn_polity::query::Approval;

#[test]
fn a_mission_lacking_nuclear_launch_approval_is_gated() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![], vec![]);
    // Default nuclear-launch level (0.3) is below the grant threshold (0.5).
    assert_eq!(e.snap().policy_granted("nuclear-launch"), Some(false));
    assert_eq!(
        e.snap().approval(FactionId(0), "nuclear-launch"),
        Some(Approval::Denied)
    );

    // Raising the lever past the threshold grants it (no longer denied).
    e.cmd(PolityCommand::SetPolicy {
        id: "nuclear-launch".into(),
        level: 0.6,
    });
    assert_eq!(e.snap().policy_granted("nuclear-launch"), Some(true));
    assert_ne!(
        e.snap().approval(FactionId(0), "nuclear-launch"),
        Some(Approval::Denied),
        "a satisfying mission proceeds"
    );
}

#[test]
fn lobbying_and_drift_stay_within_bounds() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![], vec![]);
    for _ in 0..50 {
        e.cmd(PolityCommand::Lobby {
            faction: FactionId(0),
            id: "export-control".into(),
            direction: 1.0,
        });
    }
    let lvl = e.snap().policy("export-control").unwrap();
    assert!(
        (0.0..=1.0).contains(&lvl),
        "lobbying never pushes a lever out of bounds ({lvl})"
    );
    assert!(
        lvl > 0.4,
        "tightening lobbying did raise the level above the 0.4 default"
    );
}

#[test]
fn pp_stringency_lever_is_present_as_the_single_source() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![], vec![]);
    assert!(
        e.snap().policy("pp-stringency").is_some(),
        "the PP-stringency lever exists (US6 reads it)"
    );
}
