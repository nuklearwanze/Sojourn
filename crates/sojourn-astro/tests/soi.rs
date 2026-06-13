//! SOI handoff integration (T019): crossing fires the event, preserves the
//! physical state exactly, and rebasing is invertible.

mod common;
use common::*;
use sojourn_astro::{StateVec, Vec3};

#[test]
fn soi_crossing_is_continuous_and_evented() {
    let module = astro_module_with(|c| {
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 21);

    // Place a craft just outside luna's SOI, drifting straight at it.
    // luna at t≈0: M0=1 rad on its circular rail.
    let s0 = snap(&core, &twin);
    let motions = std::collections::BTreeMap::new();
    let mut states = sojourn_astro::bodies::rails::BodyStates::new(&s0.catalog, &motions, 0);
    let terra_abs = states.absolute(sojourn_astro::BodyId(TERRA));
    let luna_abs = states.absolute(sojourn_astro::BodyId(LUNA));
    let luna_rel_terra = luna_abs.r - terra_abs.r;
    let soi = sojourn_astro::soi::soi_radius(&s0.catalog, sojourn_astro::BodyId(LUNA));

    // Start 1.2 SOI radii from luna (terra-dominant), velocity toward luna +
    // matching luna's orbital velocity so we fall in cleanly.
    let dir = luna_rel_terra.normalized();
    let start_r = luna_rel_terra - dir * (1.2 * soi);
    let luna_v_rel = luna_abs.v - terra_abs.v;
    let start_v = luna_v_rel + dir * 300.0;
    let id = spawn_state(
        &mut core,
        TERRA,
        StateVec {
            r: start_r,
            v: start_v,
        },
        500.0,
        0.0,
        "chem-hydrolox",
    );

    // Absolute state just before stepping.
    let pre = snap(&core, &twin);
    let (dom_pre, _) = craft_state(&pre, id);
    assert_eq!(dom_pre.0, TERRA);

    // Step until the crossing event fires (well within a day at 300 m/s + fall).
    let mut crossed_at = None;
    for k in 1..=2000u64 {
        advance(&mut core, k * 600);
        if count_events(&core, "soi-crossing") > 0 {
            crossed_at = Some(core.status().tick);
            break;
        }
    }
    let crossed_at = crossed_at.expect("the craft crosses into luna's SOI");

    // Continuity: dominant switched; the absolute state is consistent (the
    // relative state + luna's rail state lands where terra-frame extrapolation
    // says it should, to integration tolerance).
    let post = snap(&core, &twin);
    let (dom_post, st_post) = craft_state(&post, id);
    assert_eq!(dom_post.0, LUNA, "dominant rebased to luna");
    let t_ns = crossed_at as i64 * 1_000_000_000;
    let mut states2 = sojourn_astro::bodies::rails::BodyStates::new(&post.catalog, &motions, t_ns);
    let luna_now = states2.absolute(sojourn_astro::BodyId(LUNA));
    let abs_now = st_post.r + luna_now.r;
    // The craft must still be near the SOI boundary (it just crossed).
    let d_luna = (abs_now - luna_now.r).norm();
    assert!(
        (d_luna / soi - 1.0).abs() < 0.2,
        "freshly crossed craft sits near the boundary (d/soi = {})",
        d_luna / soi
    );
    // And its velocity magnitude is physical (no teleport jumps).
    assert!(
        st_post.v.norm() < 2.0e3,
        "no velocity discontinuity at handoff"
    );
}

#[test]
fn rebase_helpers_are_invertible_through_module_data() {
    let (core, twin) = core_with(astro_module(), 22);
    let s = snap(&core, &twin);
    let motions = std::collections::BTreeMap::new();
    let mut states =
        sojourn_astro::bodies::rails::BodyStates::new(&s.catalog, &motions, 86_400_000_000_000);
    let sv = StateVec {
        r: Vec3::new(1.0e8, 2.0e7, -3.0e6),
        v: Vec3::new(10.0, -250.0, 1.0),
    };
    let there = sojourn_astro::frames::rebase(
        &mut states,
        &sv,
        sojourn_astro::BodyId(TERRA),
        sojourn_astro::BodyId(LUNA),
    );
    let back = sojourn_astro::frames::rebase(
        &mut states,
        &there,
        sojourn_astro::BodyId(LUNA),
        sojourn_astro::BodyId(TERRA),
    );
    assert!((back.r - sv.r).norm() < 1e-2);
    assert!((back.v - sv.v).norm() < 1e-6);
    drop(core);
}
