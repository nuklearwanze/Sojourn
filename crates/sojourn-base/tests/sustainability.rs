//! US5 — self-sufficiency index & embargo stress test.

mod common;
use common::*;

// A well-supplied base where the binding loop is air-water (so the index == ECLSS closure).
const FULL: &[&str] = &[
    "pwr-fission",
    "pwr-fission",
    "hab-rigid",
    "greenhouse",
    "metal-fab",
    "fab-shop",
    "store-consumables",
];

#[test]
fn self_sufficiency_index_is_the_limiting_factor_and_rises_with_the_binding_loop() {
    // Base A: 90%-closure ECLSS ⇒ air-water ratio 0.9 binds.
    let mut a = BaseH::new();
    let mut mods_a = vec!["eclss-physchem"];
    mods_a.extend_from_slice(FULL);
    let ba = a.build("luna", "settlement", &mods_a);
    let sa = a
        .snap(inputs_for("luna", benign_site()))
        .self_sufficiency(ba)
        .unwrap();
    assert_eq!(sa.binding_loop, "air-water");
    assert!(
        (sa.index - 0.9).abs() < 1e-9,
        "index = closure (0.9): {}",
        sa.index
    );

    // Base B: 98%-closure ECLSS ⇒ improving the binding loop raises the index.
    let mut b = BaseH::new();
    let mut mods_b = vec!["eclss-highclosure"];
    mods_b.extend_from_slice(FULL);
    let bb = b.build("luna", "settlement", &mods_b);
    let sb = b
        .snap(inputs_for("luna", benign_site()))
        .self_sufficiency(bb)
        .unwrap();
    assert!(
        sb.index > sa.index,
        "improving the binding loop raised the index ({} > {})",
        sb.index,
        sa.index
    );
}

#[test]
fn embargo_survives_above_threshold_and_fails_below() {
    // Survivor: closed loops + consumables buffer bridge the small water deficit.
    let mut surv = BaseH::new();
    let mut mods = vec!["eclss-highclosure"];
    mods.extend_from_slice(FULL);
    let bs = surv.build("luna", "settlement", &mods);
    let r = surv
        .snap(inputs_for("luna", benign_site()))
        .embargo(bs, 5.0)
        .unwrap();
    assert!(
        r.survives,
        "well-closed base survives the 5-year embargo: {:?}",
        r.failing_loops
    );

    // Failer: no food / no manufacturing ⇒ food, materials, spares loops have no supply or buffer.
    let mut fail = BaseH::new();
    let bf = fail.build(
        "luna",
        "settlement",
        &["pwr-fission", "hab-rigid", "eclss-physchem"],
    );
    let rf = fail
        .snap(inputs_for("luna", benign_site()))
        .embargo(bf, 5.0)
        .unwrap();
    assert!(!rf.survives, "an under-closed base fails the embargo");
    assert!(
        rf.failing_loops.contains(&"food".to_string()),
        "food loop fails: {:?}",
        rf.failing_loops
    );
}
