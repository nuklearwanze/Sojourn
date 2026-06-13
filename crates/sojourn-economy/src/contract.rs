//! Contracts, partnerships & IP licensing (data-model §8, R13, FR-EC-602/603/604).
//! The CLPS/COTS service-contract model (post RFP → bid → award → fulfil/fail),
//! consortium trust state, and technology/data licensing. Faction-agnostic; the
//! same mechanisms FA-09's AI will drive.

use crate::ids::{ContractId, FactionId};
use serde::{Deserialize, Serialize};

/// What a contract requires delivered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deliverable {
    /// Destination node id.
    pub node: String,
    /// Commodity id to deliver.
    pub commodity: String,
    /// Mass/units required.
    pub amount: f64,
}

/// Contract lifecycle state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractState {
    /// Open for bids.
    Posted,
    /// Bids received (faction ids).
    Bid(Vec<FactionId>),
    /// Awarded to a faction.
    Awarded(FactionId),
    /// In progress (winner working).
    InProgress(FactionId),
    /// Completed successfully.
    Fulfilled(FactionId),
    /// Failed (penalty applied).
    Failed(FactionId),
}

/// A service contract (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    /// Identity.
    pub id: ContractId,
    /// Who posted it.
    pub issuer: FactionId,
    /// What it requires.
    pub deliverable: Deliverable,
    /// Reward on success (Funds).
    pub reward_funds: f64,
    /// Heritage awarded on success (a maturity/prestige signal for the host).
    pub heritage: f64,
    /// Penalty on failure (Funds).
    pub penalty: f64,
    /// Lifecycle state.
    pub state: ContractState,
}

impl Contract {
    /// Record a bid (no-op if already past bidding).
    pub fn bid(&mut self, bidder: FactionId) {
        match &mut self.state {
            ContractState::Posted => self.state = ContractState::Bid(vec![bidder]),
            ContractState::Bid(v) if !v.contains(&bidder) => v.push(bidder),
            _ => {}
        }
    }

    /// Award to a faction that has bid (or directly).
    pub fn award(&mut self, winner: FactionId) -> bool {
        match &self.state {
            ContractState::Posted | ContractState::Bid(_) => {
                self.state = ContractState::Awarded(winner);
                true
            }
            _ => false,
        }
    }

    /// Report fulfilment/failure; returns the (reward, penalty) effect to apply.
    pub fn report(&mut self, success: bool) -> Option<(f64, f64)> {
        let winner = match &self.state {
            ContractState::Awarded(f) | ContractState::InProgress(f) => *f,
            _ => return None,
        };
        if success {
            self.state = ContractState::Fulfilled(winner);
            Some((self.reward_funds, 0.0))
        } else {
            self.state = ContractState::Failed(winner);
            Some((0.0, self.penalty))
        }
    }
}

/// A consortium partnership with trust state (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Partnership {
    /// The faction pair (ordered low, high).
    pub pair: (FactionId, FactionId),
    /// Trust ∈ [0,1].
    pub trust: f64,
}

impl Partnership {
    /// Order a pair canonically.
    pub fn order(a: FactionId, b: FactionId) -> (FactionId, FactionId) {
        if a.0 <= b.0 { (a, b) } else { (b, a) }
    }
}

/// A technology/data licence earning royalties (SLICE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct License {
    /// The technology/data id licensed.
    pub tech_ref: String,
    /// Licensor.
    pub licensor: FactionId,
    /// Licensee.
    pub licensee: FactionId,
    /// Royalty rate per use (Funds).
    pub royalty_rate: f64,
}
