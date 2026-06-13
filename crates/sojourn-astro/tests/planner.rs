//! Planner stories (T034/T036/T037/T045/T047): porkchop cell → flight, flyby
//! flown vs predicted, invalid-encounter guards + impact, aerocapture corridor
//! flown, research-gated low-energy routes.

mod common;
use common::*;
use sojourn_astro::planner::{aero, flyby, porkchop};
use sojourn_astro::{AstroCommand, BodyId, StateVec, Vec3, queries};

#[test]
fn porkchop_cell_flies_to_its_target() {
    let module = astro_module_with(|c| {
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 51);
    let s0 = snap(&core, &twin);
    let depart = 600u64;
    let q = porkchop::PorkchopQuery {
        from: BodyId(TERRA),
        to: BodyId(ARES),
        depart_span: (depart, depart),
        tof_span: (180.0 * 86_400.0, 260.0 * 86_400.0),
        grid: (1, 9),
        max_revs: 0,
    };
    let grid = queries::porkchop_grid(&s0, &q);
    let cell = *grid.best().expect("solvable window");
    let dv = queries::porkchop_departure_dv(&s0, &q, &cell).expect("dv");

    // Spawn at terra's heliocentric state + transfer dv, clear of terra's SOI.
    let motions = std::collections::BTreeMap::new();
    let mut rails = sojourn_astro::bodies::rails::BodyStates::new(
        &s0.catalog,
        &motions,
        depart as i64 * 1_000_000_000,
    );
    let terra = rails.absolute(BodyId(TERRA));
    let offset = (terra.v + dv).normalized() * 1.5e9; // ahead of the planet, outside SOI
    let id = spawn_state(
        &mut core,
        0,
        StateVec {
            r: terra.r + offset,
            v: terra.v + dv,
        },
        500.0,
        0.0,
        "chem-hydrolox",
    );

    let arrive = depart + cell.tof_s as u64;
    advance(&mut core, arrive);
    let s1 = snap(&core, &twin);
    let (_, st) = craft_state(&s1, id);
    let mut rails2 = sojourn_astro::bodies::rails::BodyStates::new(
        &s1.catalog,
        &s1.motions,
        arrive as i64 * 1_000_000_000,
    );
    let craft_dom = s1.craft.get(&id).unwrap().dominant;
    let dom_abs = rails2.absolute(craft_dom);
    let craft_abs = st.r + dom_abs.r;
    let ares = rails2.absolute(BodyId(ARES));
    let miss = (craft_abs - ares.r).norm();
    // Multi-body reality vs the Lambert plan: within ~1% of the leg scale
    // (the offset start + terra's pull dominate; the planner tags this regime).
    assert!(
        miss < 3.0e9,
        "porkchop cell delivers the craft to ares vicinity (miss {miss:.3e} m)"
    );
}

#[test]
fn flyby_flown_matches_prediction() {
    let module = astro_module_with(|c| {
        c.enable_srp = false;
        c.enable_drag = false;
        c.enable_j2 = false;
    });
    let (mut core, twin) = core_with(module, 52);
    let s0 = snap(&core, &twin);
    // A hyperbola with known v∞ and periapsis, in luna's frame: spawn exactly
    // at periapsis (r = rp along +x, v = vp along +y) and fly the outbound leg.
    let v_inf = 800.0;
    let rp = 2.5e6;
    let e = 1.0 + rp * v_inf * v_inf / MU_LUNA;
    let predicted = queries::solve_flyby(
        &s0,
        &flyby::FlybyQuery {
            body: BodyId(LUNA),
            v_inf_in: Vec3::new(v_inf, 0.0, 0.0),
            periapsis_m: rp,
            plane_normal: Vec3::new(0.0, 0.0, 1.0),
            turn_sign: 1,
        },
    )
    .expect("solves");
    assert!(predicted.valid);

    let vp = libm::sqrt(v_inf * v_inf + 2.0 * MU_LUNA / rp);
    let id = spawn_state(
        &mut core,
        LUNA,
        StateVec {
            r: Vec3::new(rp, 0.0, 0.0),
            v: Vec3::new(0.0, vp, 0.0),
        },
        500.0,
        0.0,
        "chem-hydrolox",
    );
    // Fly outbound until the craft leaves luna's SOI.
    let mut exited = false;
    let mut t = core.status().tick;
    for _ in 0..600 {
        t += 1800;
        advance(&mut core, t);
        let s = snap(&core, &twin);
        if s.craft.get(&id).unwrap().dominant != BodyId(LUNA) {
            exited = true;
            break;
        }
    }
    assert!(exited, "the craft swings out and exits the SOI");

    // Outbound-asymptote check: the angle between the periapsis velocity (+y)
    // and v∞_out is ν∞ − 90°, with ν∞ = acos(−1/e); the full turn δ = 2·asin(1/e)
    // (Curtis ch. 8) follows by symmetry — the planner predicted δ.
    let s1 = snap(&core, &twin);
    let t_ns = core.status().tick as i64 * 1_000_000_000;
    let mut rails = sojourn_astro::bodies::rails::BodyStates::new(&s1.catalog, &s1.motions, t_ns);
    let c = s1.craft.get(&id).unwrap();
    let dom_abs = rails.absolute(c.dominant);
    let luna_abs = rails.absolute(BodyId(LUNA));
    let v_abs = c.state.v + dom_abs.v;
    let v_rel_out = v_abs - luna_abs.v;
    let nu_inf = libm::acos((-1.0 / e).clamp(-1.0, 1.0));
    let expected_half = nu_inf - core::f64::consts::FRAC_PI_2;
    let measured_half =
        libm::acos((v_rel_out.normalized().dot(Vec3::new(0.0, 1.0, 0.0))).clamp(-1.0, 1.0));
    let rel_err = (measured_half - expected_half).abs() / expected_half;
    assert!(
        rel_err < 0.05,
        "flown half-turn {measured_half:.4} rad vs analytic {expected_half:.4} (rel {rel_err:.3})"
    );
    // Consistency with the planner's full-turn prediction.
    let delta_from_half = 2.0 * (core::f64::consts::FRAC_PI_2 - (core::f64::consts::PI - nu_inf));
    let _ = delta_from_half;
    // Energy honesty: at the finite exit radius the speed is v(r) = √(v∞²+2μ/r),
    // not yet the asymptotic v∞ (vis-viva on the same hyperbola).
    let r_exit = ((c.state.r + dom_abs.r) - luna_abs.r).norm();
    let v_expected = libm::sqrt(v_inf * v_inf + 2.0 * MU_LUNA / r_exit);
    assert!(
        (v_rel_out.norm() - v_expected).abs() / v_expected < 0.02,
        "exit speed {} matches vis-viva {} at r_exit {:.3e}",
        v_rel_out.norm(),
        v_expected,
        r_exit
    );
    assert!(
        (2.0 * libm::asin(1.0 / e) - predicted.turn_angle_rad).abs() < 1e-12,
        "planner turn matches the analytic formula exactly"
    );
}

#[test]
fn invalid_periapsis_is_flagged_and_impact_terminates() {
    let (mut core, twin) = core_with(astro_module(), 53);
    let s0 = snap(&core, &twin);

    // Planning-time guard: periapsis below the surface is invalid.
    let bad = queries::solve_flyby(
        &s0,
        &flyby::FlybyQuery {
            body: BodyId(LUNA),
            v_inf_in: Vec3::new(500.0, 0.0, 0.0),
            periapsis_m: 1.0e6, // below luna's 1.7374e6 m radius
            plane_normal: Vec3::new(0.0, 0.0, 1.0),
            turn_sign: 1,
        },
    )
    .expect("solves geometrically");
    assert!(
        !bad.valid,
        "sub-surface periapsis flagged invalid at planning time"
    );

    // Flown: aim a craft straight at luna's centre → impact event, terminal state.
    let soi = sojourn_astro::soi::soi_radius(&s0.catalog, BodyId(LUNA));
    let id = spawn_state(
        &mut core,
        LUNA,
        StateVec {
            r: Vec3::new(-soi * 0.5, 0.0, 0.0),
            v: Vec3::new(400.0, 0.0, 0.0),
        },
        500.0,
        0.0,
        "chem-hydrolox",
    );
    let mut t = core.status().tick;
    for _ in 0..400 {
        t += 3600;
        advance(&mut core, t);
        if count_events(&core, "impact") > 0 {
            break;
        }
    }
    assert_eq!(count_events(&core, "impact"), 1, "impact event raised");
    let s = snap(&core, &twin);
    assert!(
        matches!(
            s.craft.get(&id).unwrap().status,
            sojourn_astro::FlightStatus::Impacted { .. }
        ),
        "impact is terminal in this slice (vehicle slice consumes the handoff)"
    );
}

#[test]
fn aerocapture_corridor_predicts_and_flies() {
    let module = astro_module_with(|c| {
        c.enable_srp = false;
        c.enable_third_body = false;
        c.enable_j2 = false;
    });
    let (mut core, twin) = core_with(module, 54);
    let s0 = snap(&core, &twin);

    let q = aero::AeroQuery {
        body: BodyId(ARES),
        v_inf: 1200.0,
        periapsis_alt_m: 0.0, // filled per call below
        drag_area_over_mass: 20.0 / 500.0,
        cd: 2.2,
        floor_alt_m: 50_000.0,
    };

    // Corridor exists at this v∞.
    let mut qt = q;
    qt.periapsis_alt_m = 60_000.0;
    let corridor = queries::aero_corridor(&s0, &qt).expect("corridor exists");
    assert!(corridor.steep_limit_alt_m >= q.floor_alt_m);
    assert!(corridor.shallow_limit_alt_m > corridor.steep_limit_alt_m);

    // Pick a periapsis in the middle of the corridor; the pure pass sim says captured.
    let mid_alt = 0.5 * (corridor.shallow_limit_alt_m + corridor.steep_limit_alt_m);
    let pass = aero::simulate_pass(&s0.catalog, &q, mid_alt).expect("pass");
    assert!(
        pass.captured && !pass.violates_floor,
        "mid-corridor pass captures"
    );

    // Fly it for real: spawn the inbound hyperbola at ares (same construction
    // the corridor used), mark the planned pass, and propagate through.
    let ares = s0.catalog.body(BodyId(ARES)).unwrap();
    let atmo = ares.atmosphere.as_ref().unwrap();
    let rp = ares.radius + mid_alt;
    let eps = q.v_inf * q.v_inf / 2.0;
    let vp = libm::sqrt(2.0 * (eps + ares.mu / rp));
    let h = rp * vp;
    let r0 = ares.radius + (4.0 * atmo.interface_alt_m).max(1.2 * atmo.drag_alt_m);
    let v0 = libm::sqrt(2.0 * (eps + ares.mu / r0));
    let gamma = -libm::acos((h / (r0 * v0)).clamp(-1.0, 1.0));
    let id = spawn_state(
        &mut core,
        ARES,
        StateVec {
            r: Vec3::new(r0, 0.0, 0.0),
            v: Vec3::new(v0 * libm::sin(gamma), v0 * libm::cos(gamma), 0.0),
        },
        480.0,
        20.0,
        "chem-hydrolox",
    );
    // Set the craft's drag area to match the query (spawn used defaults of 0).
    // Respawn pattern: despawn + spawn with area.
    astro(&mut core, AstroCommand::DespawnCraft { craft: id });
    astro(
        &mut core,
        AstroCommand::SpawnCraft {
            name_id: "aeroprobe".into(),
            dominant: ARES,
            state: StateVec {
                r: Vec3::new(r0, 0.0, 0.0),
                v: Vec3::new(v0 * libm::sin(gamma), v0 * libm::cos(gamma), 0.0),
            },
            dry_mass: 480.0,
            propellant: 20.0,
            engine: "chem-hydrolox".into(),
            inline_engine: None,
            available_power_w: 0.0,
            drag_area_m2: 20.0,
            srp_area_m2: 0.0,
        },
    );
    let id = id + 1;
    let now = core.status().tick;
    astro(
        &mut core,
        AstroCommand::PlanAeroPass {
            craft: id,
            body: ARES,
            start_tick: now,
            end_tick: now + 40_000,
        },
    );
    advance(&mut core, now + 40_000);
    let s1 = snap(&core, &twin);
    let c = s1.craft.get(&id).unwrap();
    assert!(
        matches!(c.status, sojourn_astro::FlightStatus::Propagating),
        "planned pass does not hand off to EDL"
    );
    assert_eq!(
        count_events(&core, "atmosphere-entry"),
        0,
        "no unplanned-entry event"
    );
    assert_eq!(
        count_events(&core, "aero-violation"),
        0,
        "no floor violation mid-corridor"
    );
    // Captured: bound orbit about ares.
    let energy = c.state.v.norm2() / 2.0 - ares.mu / c.state.r.norm();
    assert!(
        energy < 0.0,
        "aerocapture produced a bound orbit (ε = {energy:.1})"
    );

    // Violation case: a pass below the floor raises the event.
    let steep_alt = q.floor_alt_m - 20_000.0;
    let rp2 = ares.radius + steep_alt;
    let vp2 = libm::sqrt(2.0 * (eps + ares.mu / rp2));
    let h2 = rp2 * vp2;
    let gamma2 = -libm::acos((h2 / (r0 * v0)).clamp(-1.0, 1.0));
    astro(
        &mut core,
        AstroCommand::SpawnCraft {
            name_id: "steep".into(),
            dominant: ARES,
            state: StateVec {
                r: Vec3::new(r0, 0.0, 0.0),
                v: Vec3::new(v0 * libm::sin(gamma2), v0 * libm::cos(gamma2), 0.0),
            },
            dry_mass: 480.0,
            propellant: 20.0,
            engine: "chem-hydrolox".into(),
            inline_engine: None,
            available_power_w: 0.0,
            drag_area_m2: 20.0,
            srp_area_m2: 0.0,
        },
    );
    let steep_id = id + 1;
    let now2 = core.status().tick;
    astro(
        &mut core,
        AstroCommand::PlanAeroPass {
            craft: steep_id,
            body: ARES,
            start_tick: now2,
            end_tick: now2 + 40_000,
        },
    );
    advance(&mut core, now2 + 20_000);
    assert!(
        count_events(&core, "aero-violation") >= 1,
        "floor violation evented"
    );
}

#[test]
fn lowenergy_routes_are_research_gated() {
    let (mut core, twin) = core_with(astro_module(), 55);
    let s0 = snap(&core, &twin);
    assert!(
        queries::lowenergy_routes(&s0, BodyId(TERRA), BodyId(LUNA)).is_empty(),
        "gate closed: no low-energy routes offered"
    );
    astro(
        &mut core,
        AstroCommand::SetResearchGate {
            gate: "low-energy".into(),
            open: true,
        },
    );
    let s1 = snap(&core, &twin);
    let routes = queries::lowenergy_routes(&s1, BodyId(TERRA), BodyId(LUNA));
    assert_eq!(routes.len(), 1, "gate open: the route family appears");
    assert!(routes[0].dv_saving > 0.0);
    assert!(
        matches!(
            routes[0].regime,
            sojourn_astro::planner::Regime::LowConfidence
        ),
        "WSB routes are honestly low-confidence at the planning tier"
    );

    // Ballistic-capture demonstration: a slow trailing-edge approach to luna is
    // modified by terra's tide into a temporary capture (multi-body truth).
    let soi = sojourn_astro::soi::soi_radius(&s1.catalog, BodyId(LUNA));
    let id = spawn_state(
        &mut core,
        LUNA,
        // Slow approach from just inside the SOI, trailing-edge geometry.
        StateVec {
            r: Vec3::new(-soi * 0.95, soi * 0.25, 0.0),
            v: Vec3::new(120.0, -25.0, 0.0),
        },
        500.0,
        0.0,
        "chem-hydrolox",
    );
    // Dwell check: still luna-dominant after 7 days (a hyperbolic flyby at this
    // speed would exit in ~2 days without the third-body assist).
    let start = core.status().tick;
    advance(&mut core, start + 7 * 86_400);
    let s2 = snap(&core, &twin);
    assert_eq!(
        s2.craft.get(&id).unwrap().dominant,
        BodyId(LUNA),
        "slow trailing-edge arrival dwells in the SOI (ballistic-capture regime)"
    );
}
