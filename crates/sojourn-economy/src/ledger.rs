//! The six-currency ledger + location-addressed stocks with conserved, recorded
//! transactions (data-model §3, FR-EC-101…105, R3). The spine every other
//! mechanic debits and credits. Atomic apply; rejects any transaction that would
//! drive a balance/stock negative (conservation, FR-EC-104).

use crate::EconomyError;
use crate::ids::{CommodityId, FactionId, LocationId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The six currencies (design/00-OVERVIEW §1, 03-ECONOMY §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Currency {
    /// Money.
    Funds,
    /// Δv / propellant budget (m/s-equivalent accounting).
    DeltaV,
    /// Mass-to-orbit launch capacity (kg).
    MassToOrbit,
    /// Trained astronaut-hours.
    CrewTime,
    /// Controller + DSN/relay capacity.
    OpsCapacity,
    /// Political / reputation capital.
    PoliticalCapital,
}

impl Currency {
    /// A short display label.
    pub fn label(self) -> &'static str {
        match self {
            Currency::Funds => "funds",
            Currency::DeltaV => "delta-v",
            Currency::MassToOrbit => "mass-to-orbit",
            Currency::CrewTime => "crew-time",
            Currency::OpsCapacity => "ops-capacity",
            Currency::PoliticalCapital => "political-capital",
        }
    }
}

/// A (commodity, location) stock coordinate — the location-addressed good.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StockKey {
    /// The commodity.
    pub commodity: CommodityId,
    /// The location it sits at.
    pub location: LocationId,
}

impl StockKey {
    /// Construct from string refs.
    pub fn new(commodity: &str, location: &str) -> StockKey {
        StockKey {
            commodity: CommodityId(commodity.to_string()),
            location: LocationId(location.to_string()),
        }
    }
}

/// Why a transaction occurred (the audit cause).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cause {
    /// Launch to orbit (mass-to-orbit consumed).
    Launch,
    /// In-space transfer (propellant/Δv consumed).
    Transfer,
    /// ISRU conversion.
    IsruConvert,
    /// Consumption (life support, ops).
    Consume,
    /// Production (manufacturing).
    Produce,
    /// A market trade.
    Trade,
    /// Funding appropriation / revenue.
    Appropriation,
    /// Financing (equity/debt/owner injection).
    Financing,
    /// Contract reward.
    ContractReward,
    /// Penalty (failed contract, fines).
    Penalty,
    /// Facility operating cost.
    FacilityOpex,
    /// An explicit loss to a sink (boil-off, process inefficiency).
    Loss,
}

/// A signed change to one account/stock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Leg {
    /// A currency delta for a faction.
    Currency(FactionId, Currency, f64),
    /// A stock delta at a location.
    Stock(StockKey, f64),
}

/// A recorded, conserving credit/debit applied atomically.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// When (tick).
    pub tick: u64,
    /// Owner the transaction is attributed to.
    pub faction: FactionId,
    /// The balanced set of legs.
    pub legs: Vec<Leg>,
    /// Why.
    pub cause: Cause,
}

/// A faction's six-currency balances.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// Balances by currency.
    pub balances: BTreeMap<Currency, f64>,
}

impl Account {
    /// Current balance of a currency (0 if unset).
    pub fn get(&self, c: Currency) -> f64 {
        self.balances.get(&c).copied().unwrap_or(0.0)
    }
}

/// The ledger: per-faction accounts + location-addressed stocks + journal.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    /// Per-faction currency accounts.
    pub accounts: BTreeMap<FactionId, Account>,
    /// Location-addressed stocks.
    pub stocks: BTreeMap<StockKey, f64>,
    /// The audit/replay journal (most recent transactions; full history is in the event log).
    pub journal: Vec<Transaction>,
}

fn milli(x: f64) -> i64 {
    (x * 1000.0) as i64
}

impl Ledger {
    /// Current balance of a faction's currency.
    pub fn balance(&self, f: FactionId, c: Currency) -> f64 {
        self.accounts.get(&f).map(|a| a.get(c)).unwrap_or(0.0)
    }

    /// Stock of a commodity at a location.
    pub fn stock(&self, key: &StockKey) -> f64 {
        self.stocks.get(key).copied().unwrap_or(0.0)
    }

    /// All non-zero stocks (sorted).
    pub fn inventory(&self) -> Vec<(StockKey, f64)> {
        self.stocks
            .iter()
            .filter(|&(_, &q)| q != 0.0)
            .map(|(k, &q)| (k.clone(), q))
            .collect()
    }

    /// Set an opening balance (setup only; not journalled).
    pub fn set_balance(&mut self, f: FactionId, c: Currency, v: f64) {
        self.accounts.entry(f).or_default().balances.insert(c, v);
    }

    /// Set an opening stock (setup only; not journalled).
    pub fn set_stock(&mut self, key: StockKey, v: f64) {
        self.stocks.insert(key, v);
    }

    /// Apply a transaction atomically. Rejects (leaving state untouched) if any
    /// debit would drive a balance/stock negative (conservation, FR-EC-104).
    pub fn apply(&mut self, tx: Transaction) -> Result<(), EconomyError> {
        // Phase 1: feasibility — compute the resulting value of every touched
        // account/stock and reject before mutating.
        for leg in &tx.legs {
            match leg {
                Leg::Currency(f, c, d) => {
                    let cur = self.balance(*f, *c);
                    if cur + d < -1e-9 {
                        return Err(EconomyError::InsufficientBalance {
                            faction: f.0,
                            currency: c.label().to_string(),
                            need: milli(-d),
                            have: milli(cur),
                        });
                    }
                }
                Leg::Stock(key, d) => {
                    let cur = self.stock(key);
                    if cur + d < -1e-9 {
                        return Err(EconomyError::InsufficientStock {
                            commodity: key.commodity.0.clone(),
                            location: key.location.0.clone(),
                            need: milli(-d),
                            have: milli(cur),
                        });
                    }
                }
            }
        }
        // Phase 2: apply.
        for leg in &tx.legs {
            match leg {
                Leg::Currency(f, c, d) => {
                    let acct = self.accounts.entry(*f).or_default();
                    let e = acct.balances.entry(*c).or_insert(0.0);
                    *e += d;
                }
                Leg::Stock(key, d) => {
                    let e = self.stocks.entry(key.clone()).or_insert(0.0);
                    *e += d;
                }
            }
        }
        self.journal.push(tx);
        Ok(())
    }

    /// Transactions since a tick (audit/replay window).
    pub fn journal_since(&self, since_tick: u64) -> Vec<&Transaction> {
        self.journal
            .iter()
            .filter(|t| t.tick >= since_tick)
            .collect()
    }
}
