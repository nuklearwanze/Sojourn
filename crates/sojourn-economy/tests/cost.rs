//! US4 — cost model: P50/P80 uncertainty & learning curves.

mod common;
use common::module;
use sojourn_economy::cost;
use sojourn_economy::inputs::{TechMaturity, VehicleCost};

fn basis() -> VehicleCost {
    VehicleCost {
        unit_cost_basis: 1_000_000.0,
        build_days_basis: 100.0,
        payload_kg: 5_000.0,
        propellant_kg: 18_000.0,
        dv_mps: 6_000.0,
    }
}

fn mature() -> TechMaturity {
    TechMaturity {
        trl: 9,
        understanding: 0.9,
        flyable: true,
    }
}

#[test]
fn estimate_has_p50_below_p80_and_overrun_is_reproducible() {
    let p = module().cost;
    let est = cost::estimate(&basis(), &mature(), 0, &p);
    assert!(est.p50 < est.p80, "P50 < P80 ({} < {})", est.p50, est.p80);

    // Realised cost can exceed P50 and reproduces for the same draw.
    let r_mid = cost::realise(&est, &p, 0.5);
    assert!(r_mid >= est.p50, "realisation is an overrun (≥ P50)");
    assert_eq!(
        r_mid,
        cost::realise(&est, &p, 0.5),
        "same draw ⇒ same realisation"
    );
    let r_hi = cost::realise(&est, &p, 1.0);
    assert!(
        r_hi > est.p50,
        "a high draw overruns P50 ({r_hi} > {})",
        est.p50
    );
}

#[test]
fn learning_curve_lowers_unit_cost_with_production() {
    let p = module().cost;
    let first = cost::estimate(&basis(), &mature(), 0, &p);
    let tenth = cost::estimate(&basis(), &mature(), 10, &p);
    let hundredth = cost::estimate(&basis(), &mature(), 100, &p);
    assert!(tenth.p50 < first.p50, "10th unit cheaper than 1st");
    assert!(
        hundredth.p50 < tenth.p50,
        "100th cheaper than 10th (monotonic)"
    );
    assert!(hundredth.learning_factor < 1.0);
}
