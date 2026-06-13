//! US2 — the astro seam: a route Δv/TOF computed from the **real FA-02 planner
//! math** (Earth-class μ from the astro catalogue) flows into the economy as an
//! `EdgePrice` and a shipment delivers. Uses the test-only `sojourn-astro`
//! dev-dependency; production code stays core-only (R1).

mod common;
use common::*;
use sojourn_economy::EdgePrice;
use sojourn_economy::ledger::Currency;
use sojourn_economy::module::EconomyCommand;
use sojourn_economy::shipment::{ShipmentState, Vehicle, VehicleKind};

/// A Hohmann transfer Δv + TOF between two circular orbits of a body, using the
/// astrodynamics catalogue's real gravitational parameter (FA-02's two-body math).
fn hohmann_edge_price() -> EdgePrice {
    let catalog = sojourn_astro::AstroModule::load(std::path::Path::new(
        common::repo("data/astro").to_str().unwrap(),
    ))
    .expect("astro data")
    .catalog;
    // The most massive body (a central body) gives a well-conditioned transfer.
    let central = catalog
        .bodies()
        .max_by(|a, b| a.mu.partial_cmp(&b.mu).unwrap())
        .expect("a body");
    let mu = central.mu;
    let r1 = central.radius + 4.0e5; // ~400 km parking orbit
    let r2 = central.radius + 3.5786e7; // ~GEO-class
    let a = 0.5 * (r1 + r2);
    let v1 = libm::sqrt(mu / r1);
    let vt1 = libm::sqrt(mu * (2.0 / r1 - 1.0 / a));
    let v2 = libm::sqrt(mu / r2);
    let vt2 = libm::sqrt(mu * (2.0 / r2 - 1.0 / a));
    let dv = (vt1 - v1) + (v2 - vt2);
    let tof = core::f64::consts::PI * libm::sqrt(a * a * a / mu);
    assert!(dv.is_finite() && dv > 0.0, "real Hohmann Δv computed: {dv}");
    EdgePrice {
        dv_mps: dv,
        tof_s: tof,
        next_window_tick: 0,
        window_open: true,
    }
}

#[test]
fn fa02_priced_route_flows_through_the_economy() {
    let price = hohmann_edge_price();
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::DeltaV,
        amount: 20_000.0,
    });
    e.cmd(EconomyCommand::SetStock {
        commodity: "water".into(),
        location: "leo".into(),
        amount: 250.0,
    });
    e.cmd(EconomyCommand::DispatchShipment {
        faction: 0,
        edge: "leo-eml1".into(),
        cargo: vec![("water".into(), 250.0)],
        vehicle: Vehicle {
            propellant_kg: 5_000.0,
            dv_capacity_mps: price.dv_mps + 500.0,
            payload_kg: 1_000.0,
            reusable: true,
            kind: VehicleKind::Tug,
        },
        edge_price: price,
    });
    // The TOF is sub-day; one daily step delivers.
    e.advance(2 * DAY);
    let s = e.snap0();
    assert_eq!(s.shipment(0).unwrap().state, ShipmentState::Delivered);
    assert_eq!(
        s.stock("water", "eml1"),
        250.0,
        "the astro-priced shipment delivered"
    );
}
