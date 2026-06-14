//! US3 — siting respects planetary protection & physical suitability.

mod common;
use common::*;
use sojourn_base::ids::BaseId;
use sojourn_base::siting::Severity;
use sojourn_base::{PpCategory, SiteFacts};

fn site(pp: PpCategory, illumination: f64, slope_deg: f64, radiation_env_sv_yr: f64) -> SiteFacts {
    SiteFacts {
        pp_category: pp,
        illumination,
        thermal_k: 250.0,
        slope_deg,
        comms_visible: true,
        hazard_level: 0.3,
        radiation_env_sv_yr,
        resource_grade: 0.5,
        solar_distance_m: AU,
    }
}

fn has(flags: &[sojourn_base::siting::RedFlag], constraint: &str, sev: Severity) -> bool {
    flags
        .iter()
        .any(|f| f.constraint == constraint && f.severity == sev)
}

#[test]
fn special_region_without_containment_is_hard_flagged() {
    let mut e = BaseH::new();
    // No shielding module ⇒ no containment proxy.
    let bid = e.build(
        "europa-special",
        "surface-base",
        &["pwr-fission", "hab-rigid"],
    );
    let s = e.snap(inputs_for(
        "europa-special",
        site(PpCategory::SpecialRegion, 0.5, 3.0, 0.2),
    ));
    let flags = s.siting_flags(bid).unwrap();
    assert!(
        has(&flags, "planetary-protection", Severity::Hard),
        "PP violation flagged: {flags:?}"
    );
}

#[test]
fn solar_only_base_in_permanent_shadow_is_flagged() {
    let mut e = BaseH::new();
    let bid = e.build("psr-crater", "surface-base", &["pwr-solar", "hab-rigid"]);
    let s = e.snap(inputs_for(
        "psr-crater",
        site(PpCategory::Unrestricted, 0.0, 3.0, 0.2),
    ));
    let flags = s.siting_flags(bid).unwrap();
    assert!(
        has(&flags, "illumination", Severity::Hard),
        "shadow flagged: {flags:?}"
    );
}

#[test]
fn high_radiation_without_shielding_is_flagged() {
    let mut e = BaseH::new();
    let bid = e.build("high-rad", "surface-base", &["pwr-fission", "hab-rigid"]);
    // 1.0 Sv/yr unshielded (attenuation 1.0) > 0.5 limit.
    let s = e.snap(inputs_for(
        "high-rad",
        site(PpCategory::Unrestricted, 0.9, 3.0, 1.0),
    ));
    let flags = s.siting_flags(bid).unwrap();
    assert!(
        has(&flags, "shielding", Severity::Hard),
        "shielding shortfall flagged: {flags:?}"
    );
    let _ = BaseId(0);
}

#[test]
fn unbuildable_slope_is_flagged() {
    let mut e = BaseH::new();
    let bid = e.build(
        "steep",
        "surface-base",
        &["pwr-fission", "hab-rigid", "shield-regolith"],
    );
    let s = e.snap(inputs_for(
        "steep",
        site(PpCategory::Unrestricted, 0.9, 40.0, 0.2),
    ));
    let flags = s.siting_flags(bid).unwrap();
    assert!(
        has(&flags, "slope", Severity::Hard),
        "slope flagged: {flags:?}"
    );
}
