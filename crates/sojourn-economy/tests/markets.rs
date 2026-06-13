//! US6 — markets, contracts, partnerships & IP.

mod common;
use common::*;
use sojourn_economy::FactionId;
use sojourn_economy::contract::ContractState;
use sojourn_economy::ledger::Currency;
use sojourn_economy::module::EconomyCommand;

#[test]
fn contract_lifecycle_pays_reward_on_fulfilment() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::PostRfp {
        issuer: 0,
        node: "shackleton".into(),
        commodity: "water".into(),
        amount: 1_000.0,
        reward_funds: 50_000_000.0,
        heritage: 1.0,
        penalty: 10_000_000.0,
    });
    e.cmd(EconomyCommand::BidContract {
        faction: 1,
        contract: 0,
    });
    e.cmd(EconomyCommand::AwardContract {
        contract: 0,
        winner: 1,
    });
    e.cmd(EconomyCommand::ReportContract {
        contract: 0,
        success: true,
    });

    let s = e.snap0();
    assert_eq!(
        s.contract(0).unwrap().state,
        ContractState::Fulfilled(FactionId(1))
    );
    assert!(
        (s.balances(FactionId(1))[&Currency::Funds] - 50_000_000.0).abs() < 1.0,
        "winner earned the contract reward"
    );
}

#[test]
fn failed_contract_applies_a_penalty() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::SetBalance {
        faction: 1,
        currency: Currency::Funds,
        amount: 100_000_000.0,
    });
    e.cmd(EconomyCommand::PostRfp {
        issuer: 0,
        node: "llo".into(),
        commodity: "spares".into(),
        amount: 10.0,
        reward_funds: 5_000_000.0,
        heritage: 0.5,
        penalty: 8_000_000.0,
    });
    e.cmd(EconomyCommand::AwardContract {
        contract: 0,
        winner: 1,
    });
    e.cmd(EconomyCommand::ReportContract {
        contract: 0,
        success: false,
    });
    let s = e.snap0();
    assert_eq!(
        s.contract(0).unwrap().state,
        ContractState::Failed(FactionId(1))
    );
    assert!(
        s.balances(FactionId(1))[&Currency::Funds] < 100_000_000.0,
        "penalty applied"
    );
}

#[test]
fn launch_price_falls_with_world_capacity_and_trust_decays_on_renege() {
    let mut e = Econ::new();
    let base = e.snap0().market_price("leo").unwrap();
    e.cmd(EconomyCommand::SetMarketCapacity { index: 2.0 });
    let cheaper = e.snap0().market_price("leo").unwrap();
    assert!(
        cheaper < base,
        "doubling world capacity lowers $/kg ({cheaper} < {base})"
    );

    e.cmd(EconomyCommand::FormPartnership {
        a: 0,
        b: 1,
        trust: 0.8,
    });
    assert!(
        (e.snap0()
            .partnership(FactionId(0), FactionId(1))
            .unwrap()
            .trust
            - 0.8)
            .abs()
            < 1e-9
    );
    e.cmd(EconomyCommand::RenegePartnership { a: 0, b: 1 });
    assert!(
        e.snap0()
            .partnership(FactionId(0), FactionId(1))
            .unwrap()
            .trust
            < 0.8,
        "reneging decayed trust with lasting effect"
    );
}
