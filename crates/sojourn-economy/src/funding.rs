//! Funding models (data-model §5, R8, FR-EC-301…305). Agency appropriations
//! (directed funds, carry-over, fiscal cliff, gutting) and private cash-runway
//! (financing, bankruptcy) — faction-configurable from data, asymmetry without
//! different physics. Money truth lives in the ledger's Funds currency; funding
//! state holds the metadata + soft-fail flags.

use crate::ids::FactionId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Agency carry-over rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CarryOver {
    /// Unspent appropriation is lost at period end (NASA-style).
    UseItOrLoseIt,
    /// Unspent appropriation carries forward (ESA-style).
    Multiyear,
}

/// An earmarked budget line directed by politics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectedLine {
    /// Amount (Funds).
    pub amount: f64,
    /// What it is earmarked for.
    pub label: String,
}

/// A private-financing instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinancingKind {
    /// Equity raise.
    Equity,
    /// Debt.
    Debt,
    /// Owner injection (finite).
    OwnerInjection,
}

/// A per-faction funding profile (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FundingProfile {
    /// Appropriation-funded agency.
    Agency {
        /// Periodic baseline appropriation (Funds).
        baseline_funds: f64,
        /// Fractional volatility of the baseline.
        volatility: f64,
        /// Directed/earmarked lines.
        directed: Vec<DirectedLine>,
        /// Carry-over rule.
        carry_over: CarryOver,
        /// Caretaker floor below which the agency is gutted.
        caretaker_floor: f64,
        /// Fiscal period length (days).
        period_days: u64,
    },
    /// Revenue-funded private company.
    Private {
        /// Starting cash (Funds).
        starting_cash: f64,
        /// Baseline daily burn (Funds/day).
        daily_burn: f64,
        /// Available financing instruments.
        financing: Vec<FinancingKind>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileEntry {
    faction: u32,
    profile: FundingProfile,
    source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FundingFile {
    profiles: Vec<ProfileEntry>,
}

/// The loaded, validated funding profiles (module data).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FundingDefs {
    /// Profiles by faction.
    pub by_faction: BTreeMap<FactionId, FundingProfile>,
    /// Canonical source text (for the econ content hash).
    pub canonical: String,
}

impl FundingDefs {
    /// Parse + validate `funding.ron`.
    pub fn from_source(text: &str) -> Result<FundingDefs, String> {
        let f: FundingFile = ron::from_str(text).map_err(|e| format!("funding.ron: {e}"))?;
        let mut by_faction = BTreeMap::new();
        for p in f.profiles {
            if p.source.trim().is_empty() {
                return Err(format!("funding profile {} has empty source", p.faction));
            }
            match &p.profile {
                FundingProfile::Agency {
                    baseline_funds,
                    caretaker_floor,
                    period_days,
                    ..
                } => {
                    if *baseline_funds < 0.0 || *caretaker_floor < 0.0 || *period_days == 0 {
                        return Err(format!("funding profile {}: bad agency params", p.faction));
                    }
                }
                FundingProfile::Private {
                    starting_cash,
                    daily_burn,
                    ..
                } => {
                    if *starting_cash < 0.0 || *daily_burn < 0.0 {
                        return Err(format!("funding profile {}: bad private params", p.faction));
                    }
                }
            }
            if by_faction.insert(FactionId(p.faction), p.profile).is_some() {
                return Err(format!("duplicate funding profile for {}", p.faction));
            }
        }
        Ok(FundingDefs {
            by_faction,
            canonical: text.replace("\r\n", "\n"),
        })
    }
}

/// Per-faction funding state (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FundingState {
    /// Agency state.
    Agency {
        /// Tick at which the current fiscal period ends.
        period_end_tick: u64,
        /// Whether the agency has been gutted (soft-fail).
        gutted: bool,
    },
    /// Private state.
    Private {
        /// Daily burn (Funds/day).
        burn_rate: f64,
        /// Whether the company is bankrupt (game-over).
        bankrupt: bool,
    },
}
