//! US7 — capital facilities & ground segment / ops pool.

mod common;
use common::*;
use sojourn_economy::FactionId;
use sojourn_economy::facility::FacilityKind;
use sojourn_economy::ledger::Currency;
use sojourn_economy::module::EconomyCommand;

#[test]
fn ground_segment_sizes_the_ops_pool_and_oversubscription_degrades() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::Funds,
        amount: 5_000_000_000.0,
    });
    // Commission a DSN facility (capacity 30) → sizes the ops/comms pool.
    e.cmd(EconomyCommand::CommissionFacility {
        faction: 0,
        template: "dsn".into(),
    });
    assert_eq!(e.snap0().ops_utilisation(FactionId(0)).capacity, 30.0);

    // Load 40 active craft on a 30-capacity pool → oversubscribed, degraded.
    e.cmd(EconomyCommand::SetOpsLoad {
        faction: 0,
        used: 40.0,
    });
    let pool = e.snap0().ops_utilisation(FactionId(0));
    assert!(pool.oversubscribed(), "pool oversubscribed");
    assert!(pool.data_return_factor() < 1.0, "data return degraded");
    assert!(pool.anomaly_multiplier() > 1.0, "anomaly risk raised");

    // Upgrade the facility → pool grows, relieving the load.
    e.cmd(EconomyCommand::UpgradeFacility {
        faction: 0,
        facility: 0,
    });
    let bigger = e.snap0().ops_utilisation(FactionId(0));
    assert_eq!(bigger.capacity, 60.0, "upgrade doubled capacity");
    assert!(!bigger.oversubscribed(), "load now within capacity");
}

#[test]
fn production_line_capacity_gates_build_throughput() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::Funds,
        amount: 5_000_000_000.0,
    });
    e.cmd(EconomyCommand::CommissionFacility {
        faction: 0,
        template: "prod-line".into(),
    });
    assert_eq!(
        e.snap0()
            .facility_capacity(FactionId(0), FacilityKind::ProductionLine),
        10.0,
        "production-line throughput capacity is exposed"
    );
}
