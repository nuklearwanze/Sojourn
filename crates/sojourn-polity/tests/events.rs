//! US4 — the seeded, state-driven event system (a riskier faction earns more events).

mod common;
use common::*;
use sojourn_polity::Difficulty;

/// Count anomalies for a single faction at a given maturity (science-tide) level over
/// a fixed horizon, same seed — so only the maturity differs.
fn anomalies_at_maturity(tide: f64) -> usize {
    let mut e = PolityH::with_seed(7);
    e.init(vec![faction(0, false)], vec![], vec![]);
    e.set_tide(&[(0, tide)], tide, Difficulty::default());
    e.advance(10_000 * DAY);
    e.snap().event_count_of("anomaly")
}

#[test]
fn a_less_mature_faction_earns_more_anomalies() {
    let mature = anomalies_at_maturity(1.0); // risk factor 1.0
    let immature = anomalies_at_maturity(0.0); // risk factor 2.0
    assert!(
        immature > mature,
        "lower maturity ⇒ a strictly higher realised anomaly rate ({immature} > {mature})"
    );
    assert!(mature > 0, "events do fire over a long horizon");
}
