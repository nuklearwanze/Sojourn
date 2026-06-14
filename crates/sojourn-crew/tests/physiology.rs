//! US3 — physiological deconditioning & artificial gravity.

mod common;
use common::*;
use sojourn_crew::Sex;

#[test]
fn artificial_gravity_materially_reduces_deconditioning() {
    let mut micro = CrewH::new();
    micro.occupy(
        0,
        sizing(0.9, 1.0, false, true),
        env(0.6, "default", true),
        1.0e7,
        &[("m", facts(40.0, Sex::Male))],
    );
    micro.advance(200 * DAY);
    let micro_s = micro.snap();
    let micro_decon = micro_s.member("m").unwrap().decon.mean();

    let mut spin = CrewH::new();
    spin.occupy(
        0,
        sizing(0.9, 1.0, true, true),
        env(0.6, "default", true),
        1.0e7,
        &[("s", facts(40.0, Sex::Male))],
    );
    spin.advance(200 * DAY);
    let spin_s = spin.snap();
    let spin_decon = spin_s.member("s").unwrap().decon.mean();

    assert!(micro_decon > 0.0, "micro-g deconditioning accrues");
    assert!(
        spin_decon * 3.0 < micro_decon,
        "artificial gravity slows it materially ({spin_decon} ≪ {micro_decon})"
    );
    assert!(
        spin_s.capability("s").unwrap() > micro_s.capability("m").unwrap(),
        "less deconditioning ⇒ higher capability"
    );
}
