//! US7 — cost & build-time estimate with a learning curve.

mod common;
use common::*;
use sojourn_vehicle::DesignId;
use sojourn_vehicle::design::MissionReqs;

#[test]
fn learning_curve_lowers_unit_cost_with_production() {
    let m = vehicle_module();
    let cost_at = |count: u32| {
        let mut d = design(
            "tug",
            vec![stage(
                &["struct-allium", "tank-cryo", "eng-hydrolox"],
                1000.0,
                Some("eng-hydrolox"),
            )],
            MissionReqs::default(),
            vec![],
        );
        d.production_count = count;
        let s = snapshot(m.catalogue.clone(), d, maturity_for(&m, 0.9, 9));
        s.cost(DesignId(0), AU).unwrap()
    };
    let first = cost_at(0);
    let hundredth = cost_at(100);
    assert!(first.unit_cost > 0.0 && first.build_days > 0.0);
    assert!(
        hundredth.unit_cost < first.unit_cost,
        "learning curve lowers unit cost ({} < {})",
        hundredth.unit_cost,
        first.unit_cost
    );
    assert!(hundredth.learning_factor < 1.0);
}
