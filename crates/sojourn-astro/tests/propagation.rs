//! Perturbation behaviour (T022, US1): drag decay, SRP displacement + shadow,
//! low-thrust work-energy, mass-flow coupling.

mod common;
use common::*;
use sojourn_astro::{AstroCommand, StateVec, Vec3};

#[test]
fn drag_decays_low_orbits_monotonically_and_only_below_interface() {
    let module = astro_module_with(|c| {
        c.enable_third_body = false;
        c.enable_j2 = false;
        c.enable_srp = false;
    });
    let (mut core, twin) = core_with(module, 12);
    // Low orbit in the drag region (200 km, above the 120 km entry interface);
    // control orbit above the 600 km drag ceiling.
    let low = spawn_with_area(&mut core, 6.378137e6 + 2.0e5, 50.0);
    let high = spawn_with_area(&mut core, 6.378137e6 + 7.0e5, 50.0);

    let s0 = snap(&core, &twin);
    let sma0_low = sma_of(&s0, low);
    let sma0_high = sma_of(&s0, high);
    // Track decay across three checkpoints.
    let mut last = sma0_low;
    for k in 1..=3u64 {
        advance(&mut core, k * 3000);
        let s = snap(&core, &twin);
        let now = sma_of(&s, low);
        assert!(
            now < last,
            "drag decays the low orbit monotonically (step {k})"
        );
        last = now;
    }
    let s_end = snap(&core, &twin);
    let sma_high_end = sma_of(&s_end, high);
    assert!(
        (sma_high_end - sma0_high).abs() / sma0_high < 1e-9,
        "no drag above the interface"
    );
}

#[test]
fn srp_displaces_high_area_craft() {
    let module = astro_module_with(|c| {
        c.enable_third_body = false;
        c.enable_j2 = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 13);
    // Two identical high orbits; one with a huge SRP area (solar-sail-like).
    let r = 3.0e8;
    let plain = spawn_state(
        &mut core,
        TERRA,
        circular_state(r),
        100.0,
        0.0,
        "chem-hydrolox",
    );
    let sail = {
        astro(
            &mut core,
            AstroCommand::SpawnCraft {
                name_id: "sail".into(),
                dominant: TERRA,
                state: circular_state(r),
                dry_mass: 100.0,
                propellant: 0.0,
                engine: "chem-hydrolox".into(),
                available_power_w: 0.0,
                drag_area_m2: 0.0,
                srp_area_m2: 400.0,
            },
        );
        plain + 1
    };
    advance(&mut core, 7 * 86_400);
    let s = snap(&core, &twin);
    let (_, sp) = craft_state(&s, plain);
    let (_, ss) = craft_state(&s, sail);
    let separation = (sp.r - ss.r).norm();
    assert!(
        separation > 1.0e4,
        "SRP measurably displaces the high-area craft (separation {separation:.1} m)"
    );
}

#[test]
fn low_thrust_energy_equals_work_and_mass_flows() {
    let module = astro_module_with(|c| {
        c.enable_third_body = false;
        c.enable_j2 = false;
        c.enable_srp = false;
        c.enable_drag = false;
    });
    let (mut core, twin) = core_with(module, 14);
    let r = 7.0e6;
    let id = spawn_circular(&mut core, TERRA, MU_TERRA, r, 50.0, 10.0, "ep-ion");
    astro(
        &mut core,
        AstroCommand::SetGuidanceArc {
            craft: id,
            law: sojourn_astro::craft::GuidanceLaw::Tangential { sign: 1 },
            end: sojourn_astro::craft::ArcEnd::AtTick(86_400),
        },
    );
    let s0 = snap(&core, &twin);
    let (_, st0) = craft_state(&s0, id);
    let m0 = s0.craft.get(&id).unwrap().mass;
    let e0 = st0.v.norm2() / 2.0 - MU_TERRA / st0.r.norm();

    advance(&mut core, 86_400);
    let s1 = snap(&core, &twin);
    let (_, st1) = craft_state(&s1, id);
    let m1 = s1.craft.get(&id).unwrap().mass;
    let e1 = st1.v.norm2() / 2.0 - MU_TERRA / st1.r.norm();

    // Mass flowed: Δm = T·t/ve exactly (constant thrust all day).
    let def_ve = 3100.0 * 9.80665;
    let expected_dm = 0.092 * 86_400.0 / def_ve;
    let dm = m0.propellant - m1.propellant;
    assert!(
        (dm - expected_dm).abs() / expected_dm < 1e-3,
        "mass flow matches T·t/ve: {dm} vs {expected_dm}"
    );
    // Energy increased by roughly the thrust work (per unit mass): W ≈ ∫ a·v dt.
    // Tangential thrust on a near-circular orbit: ΔE ≈ a·v·t.
    let a = 0.092 / m0.total();
    let v = libm::sqrt(MU_TERRA / r);
    let expected_de = a * v * 86_400.0;
    let de = e1 - e0;
    assert!(
        (de - expected_de).abs() / expected_de < 0.05,
        "energy change ≈ thrust work: {de:.2} vs {expected_de:.2}"
    );
    assert!(de > 0.0);
}

// --- helpers ---

fn circular_state(r: f64) -> StateVec {
    let v = libm::sqrt(MU_TERRA / r);
    StateVec {
        r: Vec3::new(r, 0.0, 0.0),
        v: Vec3::new(0.0, v, 0.0),
    }
}

fn spawn_with_area(core: &mut sojourn_core::SimCore, r: f64, area: f64) -> u64 {
    let before = match core.view("astro/status").unwrap().fields.get("craft_count") {
        Some(sojourn_core::ViewValue::U64(n)) => *n,
        _ => 0,
    };
    astro(
        core,
        AstroCommand::SpawnCraft {
            name_id: format!("drag-{before}"),
            dominant: TERRA,
            state: circular_state(r),
            dry_mass: 500.0,
            propellant: 0.0,
            engine: "chem-hydrolox".into(),
            available_power_w: 0.0,
            drag_area_m2: area,
            srp_area_m2: 0.0,
        },
    );
    before
}

fn sma_of(s: &sojourn_astro::AstroSnapshot, id: u64) -> f64 {
    let (_, st) = craft_state(s, id);
    let el = sojourn_astro::math::state_to_elements(MU_TERRA, &st, 0);
    el.sma
}
