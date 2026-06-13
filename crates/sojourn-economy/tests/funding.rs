//! US3 — funding models: appropriations & cash-runway.

mod common;
use common::*;
use sojourn_economy::FactionId;
use sojourn_economy::funding::FundingState;
use sojourn_economy::ledger::Currency;
use sojourn_economy::module::EconomyCommand;

#[test]
fn appropriation_funds_then_collapse_guts_the_agency() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::RegisterFunding {
        faction: 0,
        period_end_tick: 365 * DAY,
    });
    // A healthy appropriation credits Funds and leaves the agency solvent.
    e.cmd(EconomyCommand::ApplyAppropriation {
        faction: 0,
        amount: 25_000_000_000.0,
        directed: 2_000_000_000.0,
        political_modifier: 0.0,
    });
    let s = e.snap0();
    assert!(
        s.balances(FactionId(0))[&Currency::Funds] > 25_000_000_000.0,
        "appropriation credited"
    );
    assert!(matches!(
        s.funding_state(FactionId(0)),
        Some(FundingState::Agency { gutted: false, .. })
    ));

    // A collapse below the caretaker floor guts the agency.
    e.cmd(EconomyCommand::ApplyAppropriation {
        faction: 0,
        amount: 1_000_000_000.0, // < 5 B floor
        directed: 0.0,
        political_modifier: 0.0,
    });
    assert!(
        matches!(
            e.snap0().funding_state(FactionId(0)),
            Some(FundingState::Agency { gutted: true, .. })
        ),
        "appropriation below the caretaker floor gutted the agency"
    );
}

#[test]
fn private_company_goes_bankrupt_when_cash_is_exhausted() {
    let mut e = Econ::new();
    e.cmd(EconomyCommand::RegisterFunding {
        faction: 1,
        period_end_tick: 0,
    });
    // Drop cash low so the first monthly burn (3M/day × 30) exhausts it.
    e.cmd(EconomyCommand::SetBalance {
        faction: 1,
        currency: Currency::Funds,
        amount: 1_000_000.0,
    });
    e.advance(30 * DAY); // first market sub-tick applies the burn
    let s = e.snap0();
    assert!(
        matches!(
            s.funding_state(FactionId(1)),
            Some(FundingState::Private { bankrupt: true, .. })
        ),
        "burn exceeding cash drove the private faction bankrupt"
    );
    assert!(
        s.balances(FactionId(1))[&Currency::Funds] <= 0.0 + 1e-6,
        "cash floored at zero"
    );
}
