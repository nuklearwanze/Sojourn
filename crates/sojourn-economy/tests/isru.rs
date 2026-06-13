//! US5 — ISRU break-even economics & scale-up.

mod common;
use common::*;
use sojourn_economy::GradeBelief;
use sojourn_economy::isru;
use sojourn_economy::module::EconomyCommand;

#[test]
fn break_even_is_negative_below_scale_and_positive_above() {
    let m = module();
    let proc = m.isru.get("lunar-ice-electrolysis").unwrap();
    // LEO launch price ~$2,700/kg saved and to deliver the plant.
    let below = isru::break_even(proc, 100.0, 2_700.0, 2_700.0, 10.0);
    let above = isru::break_even(proc, 100.0, 2_700.0, 2_700.0, 3_650.0);
    assert!(
        below.net < 0.0,
        "below break-even is net-negative ({})",
        below.net
    );
    assert!(
        above.net > 0.0,
        "above break-even is net-positive ({})",
        above.net
    );
    assert!(!below.positive && above.positive);
}

#[test]
fn plant_produces_to_the_local_stock_and_ramps_to_production() {
    let mut e = Econ::new();
    e.advance(DAY); // operate at a real (non-zero) tick
    e.cmd(EconomyCommand::SiteIsruPlant {
        faction: 0,
        node: "shackleton".into(),
        process: "lunar-ice-electrolysis".into(),
    });
    // Operate long enough to ramp pilot → production (90 days/level, 3 levels).
    e.cmd(EconomyCommand::OperateIsru {
        faction: 0,
        plant: 0,
        days: 200,
        grade: GradeBelief {
            resource_id: "water-ice".into(),
            believed_grade: 1.0,
            certainty: 1.0,
        },
        power_factor: 1.0,
    });
    let s = e.snap0();
    // Output credited to the local LOX stock at the plant's node.
    assert!(
        s.stock("lox", "shackleton") > 1_000.0,
        "ISRU credited the local stock"
    );
    let plant = s.plant(0).unwrap();
    assert_eq!(plant.scale_level, 2, "ramped to production scale");
    assert!(plant.online_tick > 0, "reached production (isru-online)");
    assert!(plant.cumulative_output > 0.0);
}

#[test]
fn first_of_a_kind_yield_is_lower_than_production_yield() {
    let m = module();
    let proc = m.isru.get("lunar-ice-electrolysis").unwrap();
    let pilot = isru::daily_yield(proc, 0, 1.0, 1.0);
    let production = isru::daily_yield(proc, 2, 1.0, 1.0);
    assert!(
        pilot < production,
        "pilot under-performs production ({pilot} < {production})"
    );
}
