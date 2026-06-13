//! US2 — the logistics network priced in delta-v.

mod common;
use common::*;
use sojourn_economy::EdgePrice;
use sojourn_economy::ledger::Currency;
use sojourn_economy::module::EconomyCommand;
use sojourn_economy::shipment::{ShipmentState, Vehicle, VehicleKind};

fn tug(dv: f64, payload: f64) -> Vehicle {
    Vehicle {
        propellant_kg: 5_000.0,
        dv_capacity_mps: dv,
        payload_kg: payload,
        reusable: true,
        kind: VehicleKind::Tug,
    }
}

#[test]
fn in_space_transfer_waits_for_window_then_delivers() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::DeltaV,
        amount: 10_000.0,
    });
    e.cmd(EconomyCommand::SetStock {
        commodity: "water".into(),
        location: "leo".into(),
        amount: 500.0,
    });
    // Dispatch LEO→EML-1 with a window 5 days out.
    e.cmd(EconomyCommand::DispatchShipment {
        faction: 0,
        edge: "leo-eml1".into(),
        cargo: vec![("water".into(), 500.0)],
        vehicle: tug(3_500.0, 1_000.0),
        edge_price: EdgePrice {
            dv_mps: 3_200.0,
            tof_s: (3 * DAY) as f64,
            next_window_tick: 5 * DAY,
            window_open: false,
        },
    });
    // Δv debited; cargo lifted from origin.
    assert_eq!(
        e.snap0().shipment(0).unwrap().state,
        ShipmentState::WaitingWindow
    );
    assert_eq!(e.snap0().stock("water", "leo"), 0.0);

    e.advance(2 * DAY);
    assert_eq!(
        e.snap0().shipment(0).unwrap().state,
        ShipmentState::WaitingWindow,
        "still waiting"
    );

    e.advance(4 * DAY); // now at 6 days → past the window → in transit
    assert_eq!(
        e.snap0().shipment(0).unwrap().state,
        ShipmentState::InTransit
    );

    e.advance(3 * DAY); // window (5d) + tof (3d) = 8d
    let s = e.snap0();
    assert_eq!(s.shipment(0).unwrap().state, ShipmentState::Delivered);
    assert_eq!(
        s.stock("water", "eml1"),
        500.0,
        "delivered to the destination"
    );
    assert!(
        s.balances(sojourn_economy::FactionId(0))[&Currency::DeltaV] < 10_000.0,
        "Δv debited"
    );
}

#[test]
fn launch_consumes_mass_to_orbit_while_inspace_consumes_dv() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::MassToOrbit,
        amount: 1_000.0,
    });
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::DeltaV,
        amount: 10_000.0,
    });
    // A launch edge consumes mass-to-orbit (not Δv).
    e.cmd(EconomyCommand::DispatchShipment {
        faction: 0,
        edge: "launch-leo".into(),
        cargo: vec![("spares".into(), 400.0)],
        vehicle: Vehicle {
            propellant_kg: 0.0,
            dv_capacity_mps: 0.0,
            payload_kg: 500.0,
            reusable: false,
            kind: VehicleKind::Launcher,
        },
        edge_price: EdgePrice {
            dv_mps: 0.0,
            tof_s: DAY as f64,
            next_window_tick: 0,
            window_open: true,
        },
    });
    let s = e.snap0();
    let bal = s.balances(sojourn_economy::FactionId(0));
    assert!(
        (bal[&Currency::MassToOrbit] - 600.0).abs() < 1e-9,
        "launch drew 400 kg mass-to-orbit"
    );
    assert_eq!(bal[&Currency::DeltaV], 10_000.0, "launch did not touch Δv");
}

#[test]
fn insufficient_dv_is_flagged_not_silently_completed() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::DeltaV,
        amount: 10_000.0,
    });
    // Vehicle Δv capacity below the edge requirement ⇒ rejected, no shipment created.
    e.cmd(EconomyCommand::DispatchShipment {
        faction: 0,
        edge: "leo-eml1".into(),
        cargo: vec![("water".into(), 100.0)],
        vehicle: tug(1_000.0, 1_000.0),
        edge_price: EdgePrice {
            dv_mps: 3_200.0,
            tof_s: (3 * DAY) as f64,
            next_window_tick: 0,
            window_open: true,
        },
    });
    assert!(
        e.snap0().shipment(0).is_none(),
        "under-fuelled dispatch created no shipment"
    );
}
