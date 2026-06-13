//! Divert lifecycle (T017b, FR-ASTRO-107): continuity, eligibility, budget,
//! re-rail, planning-target equivalence, determinism.

mod common;
use common::*;
use sojourn_astro::{AstroCommand, Vec3, astro_payload, queries};
use sojourn_core::StepRequest;

#[test]
fn divert_preserves_state_continuity_and_rerails() {
    let (mut core, twin) = core_with(astro_module(), 31);
    // Railed hekla state at the divert tick.
    advance(&mut core, 600);
    let s_pre = snap(&core, &twin);
    let motions_pre = s_pre.motions.clone();
    let t_ns = core.status().tick as i64 * 1_000_000_000;
    let mut rails =
        sojourn_astro::bodies::rails::BodyStates::new(&s_pre.catalog, &motions_pre, t_ns);
    let railed = rails.absolute(sojourn_astro::BodyId(HEKLA));

    let impulse = Vec3::new(0.0, 50.0, 0.0);
    astro(
        &mut core,
        AstroCommand::DivertBody {
            body: HEKLA,
            impulse,
        },
    );
    let s_post = snap(&core, &twin);
    let mut rails2 =
        sojourn_astro::bodies::rails::BodyStates::new(&s_post.catalog, &s_post.motions, t_ns);
    let diverted = rails2.absolute(sojourn_astro::BodyId(HEKLA));

    assert!(
        (diverted.r - railed.r).norm() < 1.0,
        "position continuous at diversion"
    );
    assert!(
        ((diverted.v - railed.v) - impulse).norm() < 1e-6,
        "velocity changed by exactly the impulse"
    );

    // Propagate a while, then re-rail; state continuous again.
    let target = core.status().tick + 30 * 86_400;
    advance(&mut core, target);
    let t2_ns = core.status().tick as i64 * 1_000_000_000;
    let s_mid = snap(&core, &twin);
    let mut rails3 =
        sojourn_astro::bodies::rails::BodyStates::new(&s_mid.catalog, &s_mid.motions, t2_ns);
    let before_rerail = rails3.absolute(sojourn_astro::BodyId(HEKLA));

    astro(&mut core, AstroCommand::ReRailBody { body: HEKLA });
    let s_rr = snap(&core, &twin);
    assert!(matches!(
        s_rr.motions.get(&sojourn_astro::BodyId(HEKLA)),
        Some(sojourn_astro::BodyMotion::ReRailed { .. })
    ));
    let mut rails4 =
        sojourn_astro::bodies::rails::BodyStates::new(&s_rr.catalog, &s_rr.motions, t2_ns);
    let after_rerail = rails4.absolute(sojourn_astro::BodyId(HEKLA));
    assert!(
        (after_rerail.r - before_rerail.r).norm() < 1.0,
        "re-rail is continuity-exact in position"
    );
    assert!(
        (after_rerail.v - before_rerail.v).norm() < 1e-3,
        "and velocity"
    );
}

#[test]
fn eligibility_and_budget_are_enforced() {
    // Budget 0: even an eligible body is rejected.
    let module = astro_module_with(|c| c.diversion_budget = 0);
    let (mut core, _twin) = core_with(module, 32);
    core.submit(astro_payload(&AstroCommand::DivertBody {
        body: HEKLA,
        impulse: Vec3::new(0.0, 10.0, 0.0),
    }))
    .unwrap();
    core.step(StepRequest::Ticks(0)).unwrap();
    assert_eq!(
        count_events(&core, "command-rejected"),
        1,
        "budget rejection journaled"
    );

    // Non-divertible body (luna) is rejected regardless of budget.
    let (mut core2, _twin2) = core_with(astro_module(), 33);
    core2
        .submit(astro_payload(&AstroCommand::DivertBody {
            body: LUNA,
            impulse: Vec3::new(0.0, 10.0, 0.0),
        }))
        .unwrap();
    core2.step(StepRequest::Ticks(0)).unwrap();
    assert_eq!(
        count_events(&core2, "command-rejected"),
        1,
        "eligibility rejection"
    );

    // Re-rail of a body that is not diverted is rejected.
    core2
        .submit(astro_payload(&AstroCommand::ReRailBody { body: HEKLA }))
        .unwrap();
    core2.step(StepRequest::Ticks(0)).unwrap();
    assert_eq!(
        count_events(&core2, "command-rejected"),
        2,
        "re-rail state validity"
    );
}

#[test]
fn diverted_body_remains_a_planning_target() {
    let (mut core, twin) = core_with(astro_module(), 34);
    astro(
        &mut core,
        AstroCommand::DivertBody {
            body: HEKLA,
            impulse: Vec3::new(0.0, 200.0, 0.0),
        },
    );
    advance(&mut core, 86_400);
    let s = snap(&core, &twin);
    // A porkchop to the diverted body still solves.
    let q = sojourn_astro::planner::porkchop::PorkchopQuery {
        from: sojourn_astro::BodyId(TERRA),
        to: sojourn_astro::BodyId(HEKLA),
        depart_span: (2 * 86_400, 40 * 86_400),
        tof_span: (100.0 * 86_400.0, 400.0 * 86_400.0),
        grid: (8, 8),
        max_revs: 0,
    };
    let grid = queries::porkchop_grid(&s, &q);
    assert!(
        grid.best().is_some(),
        "diverted body solves as a transfer target"
    );
}

#[test]
fn diversion_is_deterministic_across_runs() {
    let run = || {
        let (mut core, _twin) = core_with(astro_module(), 35);
        astro(
            &mut core,
            AstroCommand::DivertBody {
                body: HEKLA,
                impulse: Vec3::new(5.0, 80.0, 1.0),
            },
        );
        advance(&mut core, 45 * 86_400);
        core.fingerprint().unwrap()
    };
    assert_eq!(run(), run(), "diversions are bit-identical per seed");
}
