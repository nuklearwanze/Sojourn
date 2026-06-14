//! US1 — compose a base from modules; emergent properties derive from physics.

mod common;
use common::*;
use sojourn_base::ids::BaseId;

#[test]
fn power_margin_is_emergent_additive_and_traceable() {
    let mut e = BaseH::new();
    let bid = e.build(
        "luna-shackleton",
        "surface-base",
        &["pwr-solar", "hab-rigid", "eclss-physchem"],
    );
    let s = e.snap(inputs_for("luna-shackleton", benign_site()));
    let p = s.power(bid).unwrap();
    // 30 kW solar − (hab 2.5 kW + eclss 4 kW) = 23.5 kW.
    assert!(
        (p.generation_w - 30_000.0).abs() < 1.0,
        "gen {}",
        p.generation_w
    );
    assert!((p.margin_w - 23_500.0).abs() < 1.0, "margin {}", p.margin_w);

    // Trace resolves to sourced leaves.
    let tr = s.trace(bid, "power-margin").unwrap();
    assert!(tr.all_leaves_sourced(), "power trace leaves all sourced");

    // A second base with an extra power module has margin higher by that module's generation.
    let mut e2 = BaseH::new();
    let b2 = e2.build(
        "luna-shackleton",
        "surface-base",
        &["pwr-solar", "pwr-solar", "hab-rigid", "eclss-physchem"],
    );
    let m2 = e2
        .snap(inputs_for("luna-shackleton", benign_site()))
        .power(b2)
        .unwrap()
        .margin_w;
    assert!(
        (m2 - (23_500.0 + 30_000.0)).abs() < 1.0,
        "adding a power module raised margin by its generation ({m2})"
    );
    let _ = BaseId(0);
}

#[test]
fn shielding_closure_and_population_emerge_from_modules() {
    let mut e = BaseH::new();
    let bid = e.build(
        "luna-shackleton",
        "surface-base",
        &[
            "pwr-fission",
            "hab-rigid",
            "eclss-physchem",
            "shield-regolith",
        ],
    );
    let s = e.snap(inputs_for("luna-shackleton", benign_site()));

    // Shielding: exp(−1000/700) = exp(−1.4286) ≈ 0.2397; transmitted = 0.3 × factor < 0.5 limit.
    let sh = s.shielding(bid).unwrap();
    assert!((sh.attenuation_factor - libm::exp(-1000.0 / 700.0)).abs() < 1e-9);
    assert!(!sh.shortfall, "dose within limit");

    // ECLSS closure = best module (0.9); population = min(habitat 4, eclss crew_support 6) = 4.
    let ls = s.life_support(bid).unwrap();
    assert!((ls.closure_fraction - 0.9).abs() < 1e-9);
    assert_eq!(ls.population_capacity, 4);
}

#[test]
fn mixed_materials_shielding_composes_in_the_exponent() {
    let mut e = BaseH::new();
    let bid = e.build(
        "mars-jezero",
        "surface-base",
        &["pwr-fission", "shield-regolith", "shield-poly"],
    );
    let s = e.snap(inputs_for("mars-jezero", benign_site()));
    let sh = s.shielding(bid).unwrap();
    // exp(−(1000/700 + 100/300)) — the product of per-material attenuations.
    let expected = libm::exp(-(1000.0 / 700.0 + 100.0 / 300.0));
    assert!(
        (sh.attenuation_factor - expected).abs() < 1e-9,
        "mixed materials compose: {}",
        sh.attenuation_factor
    );
}
