//! US7 — composite exposure: a crewed mission that outruns its consumables, beyond
//! abort reach, is a modelled loss-of-crew (the crewed-vs-robotic gap made real).

mod common;
use common::*;
use sojourn_crew::Sex;
use sojourn_crew::ids::{AssetId, FactionId};

#[test]
fn outrunning_consumables_beyond_abort_reach_loses_the_crew() {
    let mut e = CrewH::new();
    // Four crew, a single day of margin? No — a kilogram of stock against
    // ~25 kg/day of gross demand, on a Mars transit with no abort within reach.
    let crew: Vec<(&str, _)> = (0..4)
        .map(|i| (["c0", "c1", "c2", "c3"][i], facts(38.0, Sex::Male)))
        .collect();
    e.occupy(
        0,
        sizing(0.9, 0.5, false, true),
        env(0.6, "mars-transit", false),
        1.0,
        &crew,
    );

    // Pre-flight the roster reads out and the mission is already non-viable.
    let s = e.snap();
    assert_eq!(s.roster_state(FactionId(0)).len(), 4, "all four seated");
    assert!(s.member("c0").is_some());
    assert!(
        !s.viability(AssetId(0), 10.0).unwrap().consumables_ok,
        "1 kg cannot cover 10 days"
    );

    // One day of consumption (≈24.96 kg) exhausts the kilogram of stock ⇒
    // loss-of-crew for the whole complement.
    e.advance(DAY);
    let s = e.snap();
    assert!(
        s.loss_of_crew("c0"),
        "consumables exhaustion is a loss-of-crew"
    );
    assert!(s.loss_of_crew("c3"), "the whole complement is lost");
    assert_eq!(
        s.consumables(AssetId(0)).unwrap().stock_kg,
        0.0,
        "stock fully drawn down"
    );
    assert!(!s.viability(AssetId(0), 10.0).unwrap().consumables_ok);
}

#[test]
fn a_resupplied_mission_within_reach_survives_the_day() {
    let mut e = CrewH::new();
    let crew: Vec<(&str, _)> = (0..4)
        .map(|i| (["d0", "d1", "d2", "d3"][i], facts(38.0, Sex::Male)))
        .collect();
    // Ample stock, an abort within reach (LEO): the day passes without loss.
    e.occupy(
        0,
        sizing(0.9, 0.5, false, true),
        env(0.2, "leo", true),
        1.0e6,
        &crew,
    );
    e.advance(DAY);
    let s = e.snap();
    assert!(!s.loss_of_crew("d0"), "a well-provisioned crew survives");
    assert!(
        s.consumables(AssetId(0)).unwrap().stock_kg > 0.0,
        "stock remains"
    );
}
