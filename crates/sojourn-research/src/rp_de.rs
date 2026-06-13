//! RP/DE portfolio allocation (FR-RESP-103). Generation is from personnel
//! (`personnel::faction_rp`/`faction_de`); this module holds the per-faction split
//! across domains and programs plus the caller-asserted available facilities, and
//! the efficiency-weight helpers.

use crate::ids::{DomainId, FactionId, ProgramId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A faction's current research portfolio.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Allocation {
    /// Science split: domain → relative weight.
    pub domain_splits: Vec<(DomainId, f64)>,
    /// Engineering split: program → relative weight.
    pub program_splits: Vec<(ProgramId, f64)>,
    /// Caller-asserted available facility capabilities (FA-06 binds real ones later).
    pub facilities: Vec<String>,
}

/// Per-faction allocations.
pub type AllocMap = BTreeMap<FactionId, Allocation>;

impl Allocation {
    /// Fraction of the budget directed at a domain (normalised weights).
    pub fn domain_fraction(&self, dom: &DomainId) -> f64 {
        fraction(&self.domain_splits, |k| k == dom)
    }

    /// Fraction of the budget directed at a program.
    pub fn program_fraction(&self, prog: ProgramId) -> f64 {
        fraction(&self.program_splits, |k| *k == prog)
    }

    /// Is a facility capability available to this faction?
    pub fn has_facility(&self, cap: &str) -> bool {
        self.facilities.iter().any(|c| c == cap)
    }
}

fn fraction<K>(splits: &[(K, f64)], matches: impl Fn(&K) -> bool) -> f64 {
    let total: f64 = splits.iter().map(|(_, w)| w.max(0.0)).sum();
    if total <= 0.0 {
        return 0.0;
    }
    splits
        .iter()
        .filter(|(k, _)| matches(k))
        .map(|(_, w)| w.max(0.0))
        .sum::<f64>()
        / total
}
