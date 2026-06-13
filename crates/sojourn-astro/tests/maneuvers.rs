//! Manoeuvre stories (T031/T040/T043, US2/US5/US8): flown Hohmann vs analytic,
//! rocket-equation propellant honesty, exhaustion + invalidation, seeded
//! execution error, low-thrust spiral vs Edelbaum, power limiting, TCM
//! re-targeting, station-keeping at L1.

mod common;
use common::*;
use sojourn_astro::craft::{ArcEnd, GuidanceLaw};
use sojourn_astro::{AstroCommand, Vec3, queries};

const G0: f64 = 9.80665;

#[test]
fn hohmann_flown_matches_plan_and_rocket_equation() {
    let module = astro_module_with(|c| {
        c.enable_third_body = false;
        c.enable_j2 = false;
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 41);
    let (r1, r2) = (6.678e6, 4.2164e7);
    let id = spawn_circular(
        &mut core,
        TERRA,
        MU_TERRA,
        r1,
        500.0,
        1500.0,
        "chem-hydrolox",
    );

    // Analytic Hohmann (Curtis ch. 6).
    let v1 = libm::sqrt(MU_TERRA / r1);
    let a_t = (r1 + r2) / 2.0;
    let vp = libm::sqrt(MU_TERRA * (2.0 / r1 - 1.0 / a_t));
    let va = libm::sqrt(MU_TERRA * (2.0 / r2 - 1.0 / a_t));
    let v2 = libm::sqrt(MU_TERRA / r2);
    let (dv1, dv2) = (vp - v1, v2 - va);
    let transfer_t = core::f64::consts::PI * libm::sqrt(a_t * a_t * a_t / MU_TERRA);

    let t1 = 600u64;
    let t2 = t1 + transfer_t as u64;
    astro(
        &mut core,
        AstroCommand::CreateNode {
            craft: id,
            epoch_tick: t1,
            dv_prn: Vec3::new(dv1, 0.0, 0.0),
        },
    );
    astro(
        &mut core,
        AstroCommand::CreateNode {
            craft: id,
            epoch_tick: t2,
            dv_prn: Vec3::new(dv2, 0.0, 0.0),
        },
    );
    astro(
        &mut core,
        AstroCommand::CommitPlan {
            craft: id,
            nodes: vec![0, 1],
            aim: None,
        },
    );

    let m0 = snap(&core, &twin).craft.get(&id).unwrap().mass.total();
    advance(&mut core, t2 + 3000);
    let s = snap(&core, &twin);
    let c = s.craft.get(&id).unwrap();

    // Final orbit ≈ circular at r2 within finite-burn tolerance (1%).
    let el = sojourn_astro::math::state_to_elements(MU_TERRA, &c.state, 0);
    assert!(
        (el.sma - r2).abs() / r2 < 0.01,
        "flown Hohmann sma {} vs target {r2} (finite-burn tolerance)",
        el.sma
    );
    assert!(el.ecc < 0.02, "near-circular arrival (e = {})", el.ecc);

    // Propellant matches the rocket equation for the delivered Δv (SC-005).
    let m1 = c.mass.total();
    let ve = 450.5 * G0;
    let dv_from_mass = ve * libm::log(m0 / m1);
    let dv_planned = dv1 + dv2;
    assert!(
        (dv_from_mass - dv_planned).abs() / dv_planned < 1e-3,
        "rocket equation: delivered {dv_from_mass} vs planned {dv_planned}"
    );

    // Both maneuver-node pauses fired.
    assert_eq!(count_events(&core, "maneuver-node"), 2);
}

#[test]
fn unaffordable_plan_flags_then_exhausts_and_invalidates() {
    let (mut core, twin) = core_with(astro_module(), 42);
    let id = spawn_circular(
        &mut core,
        TERRA,
        MU_TERRA,
        7.0e6,
        500.0,
        50.0,
        "chem-hydrolox",
    );
    let s = snap(&core, &twin);

    // Budget: ve·ln(550/500) ≈ 421 m/s. Ask for 2000.
    match queries::node_feasibility(&s, id, 2000.0, 0.0) {
        queries::Feasibility::InsufficientDeltaV {
            requested,
            achievable,
        } => {
            assert!(requested > achievable, "flagged at planning time");
        }
        other => panic!("expected infeasibility flag, got {other:?}"),
    }

    // Fly it anyway: exhaustion mid-burn + plan invalidation (FR-ASTRO-307).
    astro(
        &mut core,
        AstroCommand::CreateNode {
            craft: id,
            epoch_tick: 700,
            dv_prn: Vec3::new(2000.0, 0.0, 0.0),
        },
    );
    astro(
        &mut core,
        AstroCommand::CommitPlan {
            craft: id,
            nodes: vec![0],
            aim: None,
        },
    );
    advance(&mut core, 5000);
    assert_eq!(
        count_events(&core, "propellant-exhausted"),
        1,
        "exhaustion event"
    );
    assert_eq!(
        count_events(&core, "plan-invalidated"),
        1,
        "dependent plan invalidated"
    );
    let c = snap(&core, &twin).craft.get(&id).unwrap().clone();
    assert!(c.mass.propellant < 1e-9, "tank empty");
}

#[test]
fn execution_error_is_seeded_and_reproducible() {
    let run = |seed: u64| {
        let module = astro_module_with(|c| c.exec_error.enabled = true);
        let (mut core, _twin) = core_with(module, seed);
        let id = spawn_circular(
            &mut core,
            TERRA,
            MU_TERRA,
            7.0e6,
            500.0,
            1500.0,
            "chem-hydrolox",
        );
        astro(
            &mut core,
            AstroCommand::CreateNode {
                craft: id,
                epoch_tick: 700,
                dv_prn: Vec3::new(100.0, 0.0, 0.0),
            },
        );
        astro(
            &mut core,
            AstroCommand::CommitPlan {
                craft: id,
                nodes: vec![0],
                aim: None,
            },
        );
        advance(&mut core, 3000);
        core.fingerprint().unwrap()
    };
    assert_eq!(run(7), run(7), "same seed ⇒ identical dispersed burn");
    assert_ne!(run(7), run(8), "different seed ⇒ different dispersion");
}

#[test]
fn lowthrust_spiral_matches_edelbaum_within_tolerance() {
    let module = astro_module_with(|c| {
        c.enable_third_body = false;
        c.enable_j2 = false;
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 44);
    let (r1, r2) = (7.0e6, 9.0e6);
    let id = spawn_circular(&mut core, TERRA, MU_TERRA, r1, 50.0, 10.0, "ep-ion");
    let s0 = snap(&core, &twin);
    let est = queries::lowthrust_arc(&s0, id, r2).expect("estimate");

    astro(
        &mut core,
        AstroCommand::SetGuidanceArc {
            craft: id,
            law: GuidanceLaw::Tangential { sign: 1 },
            end: ArcEnd::TargetRadius(r2),
        },
    );
    // Fly until the arc clears (estimate duration + 30% margin).
    let mut done_at = None;
    let deadline = (est.duration_s * 1.3) as u64;
    let mut t = core.status().tick;
    while t < deadline {
        t = (t + 6_000).min(deadline);
        advance(&mut core, t);
        let s = snap(&core, &twin);
        if s.craft.get(&id).unwrap().guidance.is_none() {
            done_at = Some(core.status().tick);
            break;
        }
    }
    let done_at = done_at.expect("spiral completes") as f64;
    let s1 = snap(&core, &twin);
    let used = 10.0 - s1.craft.get(&id).unwrap().mass.propellant;

    assert!(
        (done_at - est.duration_s).abs() / est.duration_s < 0.05,
        "duration {done_at} vs Edelbaum {} (SC-004 ≤5%)",
        est.duration_s
    );
    assert!(
        (used - est.propellant_kg).abs() / est.propellant_kg < 0.05,
        "propellant {used} vs Edelbaum {}",
        est.propellant_kg
    );
}

#[test]
fn power_limited_thrust_scales_with_available_power() {
    let module = astro_module_with(|c| {
        c.enable_third_body = false;
        c.enable_j2 = false;
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 45);
    // Full power vs half power: energy gain over a day should halve.
    let full = spawn_circular(&mut core, TERRA, MU_TERRA, 7.0e6, 50.0, 10.0, "ep-ion");
    astro(
        &mut core,
        AstroCommand::SpawnCraft {
            name_id: "half-power".into(),
            dominant: TERRA,
            state: sojourn_astro::StateVec {
                r: Vec3::new(7.0e6, 0.0, 0.0),
                v: Vec3::new(0.0, libm::sqrt(MU_TERRA / 7.0e6), 0.0),
            },
            dry_mass: 50.0,
            propellant: 10.0,
            engine: "ep-ion".into(),
            inline_engine: None,
            available_power_w: 1150.0, // half the 2300 W rating
            drag_area_m2: 0.0,
            srp_area_m2: 0.0,
        },
    );
    let half = full + 1;
    for id in [full, half] {
        astro(
            &mut core,
            AstroCommand::SetGuidanceArc {
                craft: id,
                law: GuidanceLaw::Tangential { sign: 1 },
                end: ArcEnd::AtTick(86_400),
            },
        );
    }
    advance(&mut core, 86_400);
    let s = snap(&core, &twin);
    let used_full = 10.0 - s.craft.get(&full).unwrap().mass.propellant;
    let used_half = 10.0 - s.craft.get(&half).unwrap().mass.propellant;
    let ratio = used_half / used_full;
    assert!(
        (ratio - 0.5).abs() < 0.05,
        "half power ⇒ half thrust ⇒ half mass flow (ratio {ratio})"
    );
}

#[test]
fn tcm_retargets_a_dispersed_transfer() {
    let module = astro_module_with(|c| {
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 46);

    // Heliocentric transfer to ares via Lambert, with a deliberate 20 m/s error.
    let s0 = snap(&core, &twin);
    let depart_tick = 600u64;
    let tof = 220.0 * 86_400.0;
    let q = sojourn_astro::planner::porkchop::PorkchopQuery {
        from: sojourn_astro::BodyId(TERRA),
        to: sojourn_astro::BodyId(ARES),
        depart_span: (depart_tick, depart_tick),
        tof_span: (tof, tof),
        grid: (1, 1),
        max_revs: 0,
    };
    let grid = queries::porkchop_grid(&s0, &q);
    let cell = *grid.best().expect("solvable");
    let dv = queries::porkchop_departure_dv(&s0, &q, &cell).expect("dv");

    // Spawn heliocentric at terra's state + transfer velocity + error.
    let motions = std::collections::BTreeMap::new();
    let mut rails = sojourn_astro::bodies::rails::BodyStates::new(
        &s0.catalog,
        &motions,
        depart_tick as i64 * 1_000_000_000,
    );
    let terra = rails.absolute(sojourn_astro::BodyId(TERRA));
    // Wait at tick 0; spawn directly at the departure state (outside terra's SOI
    // concerns by jumping ahead of the geometry: dominant = sol).
    let err = Vec3::new(12.0, -16.0, 0.0); // ~20 m/s injection error
    let id = spawn_state(
        &mut core,
        0, // sol-dominant
        sojourn_astro::StateVec {
            r: terra.r + (terra.r.normalized() * 2.0e9),
            v: terra.v + dv + err,
        },
        500.0,
        300.0,
        "chem-hydrolox",
    );
    let arrival_tick = depart_tick + tof as u64;
    let aim = sojourn_astro::maneuver::AimPoint {
        body: sojourn_astro::BodyId(ARES),
        epoch_tick: arrival_tick,
        target_radius_m: 0.0,
    };

    // Mid-course (40 days in): solve and apply a TCM.
    advance(&mut core, depart_tick + 40 * 86_400);
    let s_mid = snap(&core, &twin);
    let pre = queries::reconcile_report(&s_mid, id, &aim).expect("report");
    let dv_tcm = queries::solve_tcm(&s_mid, id, &aim).expect("tcm solves");
    // Convert world Δv to PRN at (approximately) the burn state.
    let c = s_mid.craft.get(&id).unwrap();
    let (p, r, n) = sojourn_astro::math::prn_basis(&c.state);
    let dv_prn = Vec3::new(dv_tcm.dot(p), dv_tcm.dot(r), dv_tcm.dot(n));
    let now = core.status().tick;
    astro(
        &mut core,
        AstroCommand::CreateNode {
            craft: id,
            epoch_tick: now + 120,
            dv_prn,
        },
    );
    let s2 = snap(&core, &twin);
    let node_id = s2.craft.len() as u64; // first node of this run is id 0
    let _ = node_id;
    astro(
        &mut core,
        AstroCommand::CommitPlan {
            craft: id,
            nodes: vec![0],
            aim: Some(aim),
        },
    );

    // Fly to arrival; the conic-predicted miss must shrink dramatically.
    advance(&mut core, arrival_tick - 5 * 86_400);
    let s_end = snap(&core, &twin);
    let post = queries::reconcile_report(&s_end, id, &aim).expect("report");
    assert!(
        post.miss_m < pre.miss_m * 0.2,
        "TCM re-targets: miss {0:.3e} → {1:.3e}",
        pre.miss_m,
        post.miss_m
    );
    assert!(
        post.miss_m < 5.0e8,
        "arrival within 500,000 km of the aim body"
    );
}

#[test]
fn station_keeping_holds_l1_and_unkept_craft_departs() {
    let module = astro_module_with(|c| {
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 47);
    let s0 = snap(&core, &twin);

    // L1 of (terra, luna) at t=0.
    let pts = queries::lagrange_points(
        &s0,
        sojourn_astro::BodyId(TERRA),
        sojourn_astro::BodyId(LUNA),
    );
    let l1 = pts.iter().find(|p| p.name == "L1").expect("L1");
    let l1_abs = queries::lagrange_position(
        &s0,
        sojourn_astro::BodyId(TERRA),
        sojourn_astro::BodyId(LUNA),
        l1,
    );

    // Rotating-frame-stationary initial state: v = ω × r (about terra).
    let motions = std::collections::BTreeMap::new();
    let mut rails = sojourn_astro::bodies::rails::BodyStates::new(&s0.catalog, &motions, 0);
    let terra = rails.absolute(sojourn_astro::BodyId(TERRA));
    let luna = rails.absolute(sojourn_astro::BodyId(LUNA));
    let omega = (luna.r - terra.r).cross(luna.v - terra.v) * (1.0 / (luna.r - terra.r).norm2());
    let v_abs = terra.v + omega.cross(l1_abs - terra.r);

    let spawn = |core: &mut sojourn_core::SimCore| -> u64 {
        spawn_state(
            core,
            TERRA,
            sojourn_astro::StateVec {
                r: l1_abs - terra.r,
                v: v_abs - terra.v,
            },
            500.0,
            100.0,
            "chem-hydrolox",
        )
    };
    let kept = spawn(&mut core);
    let free = spawn(&mut core);
    astro(
        &mut core,
        AstroCommand::ScheduleStationKeeping {
            craft: kept,
            cadence_ticks: 86_400,
            budget_dv: 10.0,
            rotating_pair: (TERRA, LUNA),
        },
    );

    advance(&mut core, 90 * 86_400);
    let s = snap(&core, &twin);
    let t_ns = core.status().tick as i64 * 1_000_000_000;
    let mut rails2 = sojourn_astro::bodies::rails::BodyStates::new(&s.catalog, &s.motions, t_ns);
    let pts_now = queries::lagrange_points(
        &s,
        sojourn_astro::BodyId(TERRA),
        sojourn_astro::BodyId(LUNA),
    );
    let l1_now = pts_now.iter().find(|p| p.name == "L1").unwrap();
    let l1_abs_now = queries::lagrange_position(
        &s,
        sojourn_astro::BodyId(TERRA),
        sojourn_astro::BodyId(LUNA),
        l1_now,
    );

    let abs_of = |id: u64, rails: &mut sojourn_astro::bodies::rails::BodyStates<'_>| {
        let c = s.craft.get(&id).unwrap();
        let dom = rails.absolute(c.dominant);
        c.state.r + dom.r
    };
    let kept_dist = (abs_of(kept, &mut rails2) - l1_abs_now).norm();
    let free_dist = (abs_of(free, &mut rails2) - l1_abs_now).norm();

    assert!(
        kept_dist < 1.0e7,
        "station-kept craft holds within 10,000 km of L1 (got {kept_dist:.3e} m)"
    );
    assert!(
        free_dist > 3.0 * kept_dist.max(1.0e6),
        "unkept craft departs the unstable region (kept {kept_dist:.3e}, free {free_dist:.3e})"
    );
    // Station-keeping consumed real propellant.
    let kept_prop = s.craft.get(&kept).unwrap().mass.propellant;
    assert!(kept_prop < 100.0, "SK burns consumed propellant");
}
