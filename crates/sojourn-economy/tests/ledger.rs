//! US1 — the six-currency ledger & location-addressed resources.

mod common;
use common::*;
use sojourn_economy::FactionId;
use sojourn_economy::ledger::{Cause, Currency, Leg, StockKey};
use sojourn_economy::module::EconomyCommand;

#[test]
fn currencies_and_location_addressed_stocks() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 0,
        currency: Currency::Funds,
        amount: 1_000.0,
    });
    e.cmd(EconomyCommand::SetStock {
        commodity: "water".into(),
        location: "leo".into(),
        amount: 1_000.0,
    });
    e.cmd(EconomyCommand::SetStock {
        commodity: "water".into(),
        location: "eml1".into(),
        amount: 1_000.0,
    });

    let s = e.snap0();
    assert_eq!(s.balances(FactionId(0))[&Currency::Funds], 1_000.0);
    // The same commodity at two locations is two distinct goods.
    assert_eq!(s.stock("water", "leo"), 1_000.0);
    assert_eq!(s.stock("water", "eml1"), 1_000.0);
    assert_eq!(s.stock("water", "mars"), 0.0);
    assert_eq!(s.inventory().len(), 2);
}

#[test]
fn overdraw_is_rejected_and_conservation_holds() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetStock {
        commodity: "water".into(),
        location: "leo".into(),
        amount: 100.0,
    });

    // Over-draw: attempt to debit 150 from a 100 stock ⇒ rejected, stock unchanged.
    e.cmd(EconomyCommand::Transact {
        faction: 0,
        legs: vec![Leg::Stock(StockKey::new("water", "leo"), -150.0)],
        cause: Cause::Consume,
    });
    assert_eq!(
        e.snap0().stock("water", "leo"),
        100.0,
        "over-draw left stock unchanged"
    );

    // A conserving transfer: 40 kg water LEO → EML-1 (mass in = mass out).
    e.cmd(EconomyCommand::Transact {
        faction: 0,
        legs: vec![
            Leg::Stock(StockKey::new("water", "leo"), -40.0),
            Leg::Stock(StockKey::new("water", "eml1"), 40.0),
        ],
        cause: Cause::Transfer,
    });
    let s = e.snap0();
    assert_eq!(s.stock("water", "leo"), 60.0);
    assert_eq!(s.stock("water", "eml1"), 40.0);
    // Conservation: total water across locations is preserved.
    let total: f64 = s.inventory().iter().map(|(_, q)| q).sum();
    assert!(
        (total - 100.0).abs() < 1e-9,
        "water conserved across the transfer ({total})"
    );
}
