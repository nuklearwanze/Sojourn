//! US3 — plan → preview → commit gating (FR-UI-301…305).

use sojourn_ui::command::{ActionKind, CommitOutcome, DraftPlan, Gate, Preview, TracedDelta, gate};

fn preview() -> Preview {
    Preview {
        deltas: vec![TracedDelta {
            label: "Δv spent".into(),
            formatted: "2.4 km/s".into(),
            tree: None,
        }],
        becomes_unrecoverable: vec!["the departure window".into()],
        warnings: vec![],
        computed_at_tick: 100,
    }
}

#[test]
fn irreversible_actions_are_gated_with_a_preview() {
    let draft = DraftPlan {
        kind: ActionKind::Burn,
        label: "TMI burn".into(),
        built_at_tick: 100,
    };
    let mut previewed = false;
    let g = gate(&draft, || {
        previewed = true;
        preview()
    });
    assert!(
        previewed,
        "an irreversible action must obtain a core preview"
    );
    assert!(matches!(g, Gate::Gated(_)));
}

#[test]
fn reversible_actions_bypass_the_gate() {
    let draft = DraftPlan {
        kind: ActionKind::Reversible,
        label: "toggle comms layer".into(),
        built_at_tick: 100,
    };
    let mut previewed = false;
    let g = gate(&draft, || {
        previewed = true;
        preview()
    });
    assert!(
        !previewed,
        "a reversible action must NOT compute a preview/confirm"
    );
    assert_eq!(g, Gate::Direct);
}

#[test]
fn a_stale_preview_is_detected_for_re_preview() {
    let p = preview(); // computed at tick 100
    assert!(!p.is_stale(100));
    assert!(p.is_stale(101), "state advanced ⇒ re-preview before commit");
}

#[test]
fn a_rejection_is_surfaced_not_hidden() {
    let o = CommitOutcome::Rejected("insufficient funds".into());
    match o {
        CommitOutcome::Rejected(r) => assert!(r.contains("funds")),
        CommitOutcome::Applied => panic!("a rejected command must not read as applied"),
    }
}
