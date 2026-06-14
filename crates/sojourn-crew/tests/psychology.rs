//! US4 — psychology under isolation, confinement & comms-lag.

mod common;
use common::*;
use sojourn_crew::{Sex, psychology};

#[test]
fn psych_load_accrues_over_a_mission() {
    let mut e = CrewH::new();
    e.occupy(
        0,
        sizing(0.9, 1.0, false, true),
        env(0.6, "default", true),
        1.0e7,
        &[("p", facts(40.0, Sex::Male))],
    );
    e.advance(100 * DAY);
    assert!(
        e.snap().member("p").unwrap().psych_load > 0.0,
        "psychological load accrues over time"
    );
}

#[test]
fn higher_psych_load_raises_the_anomaly_probability() {
    let p = module().params;
    let low = psychology::anomaly_prob(0.1, 0.0, &p.psychology);
    let high = psychology::anomaly_prob(0.6, 0.0, &p.psychology);
    assert!(
        high > low,
        "anomaly probability rises with psych load ({high} > {low})"
    );
    // Ops oversubscription raises it further.
    let oversub = psychology::anomaly_prob(0.6, 2.0, &p.psychology);
    assert!(
        oversub > high,
        "ops oversubscription raises the anomaly probability"
    );
}
