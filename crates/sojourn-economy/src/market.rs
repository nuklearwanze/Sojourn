//! Markets (data-model §8, R12, FR-EC-601/604/605). A launch market whose $/kg by
//! orbit class moves with a world-capacity index (parametric + seeded), plus
//! tourism/in-space-manufacturing markets with finite sizes and price ceilings.
//! Faction-agnostic; seeded where stochastic.

use crate::ids::OrbitClass;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Launch-market parameters (DATA, `launch_market.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchMarketDefs {
    /// Base $/kg by orbit class (at the reference capacity).
    pub base_price_per_kg: BTreeMap<OrbitClass, f64>,
    /// Reference world-capacity index the base prices are quoted at.
    pub reference_capacity: f64,
    /// Price elasticity to capacity (price ∝ (reference/index)^elasticity).
    pub elasticity: f64,
    /// Fractional seeded fluctuation applied each market tick.
    pub fluctuation: f64,
    /// Provenance.
    pub source: String,
}

impl LaunchMarketDefs {
    /// Parse + validate `launch_market.ron`.
    pub fn from_source(text: &str) -> Result<LaunchMarketDefs, String> {
        let d: LaunchMarketDefs =
            ron::from_str(text).map_err(|e| format!("launch_market.ron: {e}"))?;
        if d.source.trim().is_empty() {
            return Err("launch_market.ron: empty source".into());
        }
        if d.reference_capacity <= 0.0 {
            return Err("launch_market.ron: reference_capacity must be > 0".into());
        }
        for (oc, p) in &d.base_price_per_kg {
            if *p < 0.0 {
                return Err(format!("launch_market.ron: negative price for '{}'", oc.0));
            }
        }
        Ok(d)
    }

    /// Canonical text (caller normalizes for the econ content hash).
    pub fn price_at(&self, orbit: &OrbitClass, world_capacity_index: f64) -> Option<f64> {
        let base = *self.base_price_per_kg.get(orbit)?;
        let idx = world_capacity_index.max(1e-9);
        // Higher capacity ⇒ lower $/kg (elasticity > 0).
        Some(base * libm::pow(self.reference_capacity / idx, self.elasticity))
    }
}

/// A niche market with a finite size and a price ceiling (DATA, in `markets.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NicheMarket {
    /// Id (e.g. "tourism-orbital", "ism-zblan").
    pub id: String,
    /// Annual market size cap (Funds or units).
    pub size_cap: f64,
    /// Maximum unit price.
    pub price_ceiling: f64,
}

/// RFP-generation + partnership parameters (DATA, in `markets.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketParams {
    /// RFPs are generated every this many days (on the seeded `contracts` stream).
    pub rfp_period_days: u64,
    /// Trust lost when a partner reneges.
    pub trust_decay_on_renege: f64,
    /// A market move beyond this fraction emits `market-shock`.
    pub shock_threshold: f64,
    /// Niche markets.
    pub niche: Vec<NicheMarket>,
    /// Provenance.
    pub source: String,
}

impl MarketParams {
    /// Parse + validate `markets.ron`.
    pub fn from_source(text: &str) -> Result<MarketParams, String> {
        let m: MarketParams = ron::from_str(text).map_err(|e| format!("markets.ron: {e}"))?;
        if m.source.trim().is_empty() {
            return Err("markets.ron: empty source".into());
        }
        for n in &m.niche {
            if n.size_cap < 0.0 || n.price_ceiling < 0.0 {
                return Err(format!("markets.ron: niche '{}' has negative param", n.id));
            }
        }
        Ok(m)
    }

    /// A niche market by id.
    pub fn niche(&self, id: &str) -> Option<&NicheMarket> {
        self.niche.iter().find(|n| n.id == id)
    }
}

/// Launch-market dynamic state (SLICE).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LaunchMarketState {
    /// The world launch-capacity index (rises as the world fields more reuse).
    pub world_capacity_index: f64,
}
