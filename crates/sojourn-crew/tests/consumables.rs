//! US1 — consumables versus ECLSS closure (A1: air/water recycled, food open-loop).

mod common;
use common::*;
use sojourn_crew::Sex;
use sojourn_crew::ids::AssetId;

#[test]
fn makeup_splits_air_water_and_food_and_robotic_is_exempt() {
    let mut e = CrewH::new();
    let crew: Vec<(&str, _)> = (0..4)
        .map(|i| (["a0", "a1", "a2", "a3"][i], facts(40.0, Sex::Male)))
        .collect();
    e.occupy(
        0,
        sizing(0.9, 0.5, false, true),
        env(0.6, "default", true),
        100_000.0,
        &crew,
    );
    // air/water = (0.84+3.5+0.1)×4 = 17.76; food = 1.8×4 = 7.2; gross = 24.96.
    let c = e.snap().consumables(AssetId(0)).unwrap();
    assert!(
        (c.gross_per_day - 24.96).abs() < 1e-9,
        "gross {}",
        c.gross_per_day
    );
    // make-up = 17.76 × (1 − 0.9) + 7.2 = 1.776 + 7.2 = 8.976.
    assert!(
        (c.makeup_rate_kg_day - 8.976).abs() < 1e-9,
        "makeup {}",
        c.makeup_rate_kg_day
    );

    // Higher closure lowers the air/water make-up (not the food term).
    let bcrew: Vec<(&str, _)> = (0..4)
        .map(|i| (["b0", "b1", "b2", "b3"][i], facts(40.0, Sex::Male)))
        .collect();
    e.occupy(
        1,
        sizing(0.98, 0.5, false, true),
        env(0.6, "default", true),
        100_000.0,
        &bcrew,
    );
    let c2 = e.snap().consumables(AssetId(1)).unwrap();
    assert!(
        c2.makeup_rate_kg_day < c.makeup_rate_kg_day,
        "closure lowers make-up"
    );
    assert!((c2.makeup_rate_kg_day - (17.76 * 0.02 + 7.2)).abs() < 1e-9);

    // A robotic asset consumes nothing.
    e.occupy(
        2,
        sizing(0.9, 0.5, false, false),
        env(0.6, "default", true),
        100_000.0,
        &[],
    );
    let rc = e.snap().consumables(AssetId(2)).unwrap();
    assert_eq!(rc.gross_per_day, 0.0);
    assert_eq!(rc.makeup_rate_kg_day, 0.0);
}

#[test]
fn a_mission_that_cannot_cover_its_duration_is_non_viable() {
    let mut e = CrewH::new();
    let crew: Vec<(&str, _)> = (0..4)
        .map(|i| (["a0", "a1", "a2", "a3"][i], facts(40.0, Sex::Male)))
        .collect();
    // 1000 kg / 24.96 kg/day ≈ 40 days of coverage.
    e.occupy(
        0,
        sizing(0.9, 0.5, false, true),
        env(0.6, "default", true),
        1_000.0,
        &crew,
    );
    assert!(
        e.snap().viability(AssetId(0), 30.0).unwrap().viable,
        "30-day mission covered"
    );
    assert!(
        !e.snap()
            .viability(AssetId(0), 100.0)
            .unwrap()
            .consumables_ok,
        "100-day mission not covered"
    );

    e.cmd(sojourn_crew::CrewCommand::Resupply {
        asset: 0,
        kg: 5_000.0,
    });
    assert!(
        e.snap()
            .viability(AssetId(0), 100.0)
            .unwrap()
            .consumables_ok,
        "resupply restores coverage"
    );
}
