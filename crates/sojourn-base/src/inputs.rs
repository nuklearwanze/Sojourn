//! Composed-value integration seams (R2, `contracts/integration-seams.md`). Every
//! cross-slice value enters as a plain, serializable input the host assembles from
//! the upstream query surfaces — the FA-04/FA-06 decoupling. The base never imports
//! the world/research/economy crates; their ids are plain strings here.

use crate::ids::{BaseId, FactionId, ModuleId, SiteId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Planetary-protection category (mirrors the FA-03 world value as a plain enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PpCategory {
    /// Category I–II (no/low restriction).
    Unrestricted,
    /// Category III–IV (controlled).
    Controlled,
    /// Category V / Special Region (strict forward-contamination rules).
    SpecialRegion,
}

/// The surveyed facts about a Site, from FA-03's belief-state (never ground truth).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteFacts {
    /// Planetary-protection category.
    pub pp_category: PpCategory,
    /// Mean illumination fraction ∈ [0,1] (0 = permanent shadow).
    pub illumination: f64,
    /// Surface/thermal temperature (K).
    pub thermal_k: f64,
    /// Local slope (degrees).
    pub slope_deg: f64,
    /// Comms visibility to a relay/DSN.
    pub comms_visible: bool,
    /// Hazard level ∈ [0,1].
    pub hazard_level: f64,
    /// Ambient radiation environment (Sv/yr, unshielded).
    pub radiation_env_sv_yr: f64,
    /// Surveyed resource grade ∈ [0,1] (for on-site ISRU feedstock).
    pub resource_grade: f64,
    /// Heliocentric distance (m) for PV scaling.
    pub solar_distance_m: f64,
}

/// A technology's maturity, from FA-05 (gates module composition).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TechMaturity {
    /// Technology readiness level.
    pub trl: u8,
    /// Domain understanding ∈ [0,1].
    pub understanding: f64,
    /// Flyable (TRL ≥ 6)?
    pub flyable: bool,
}

/// Construction-delivery status for a module, from FA-06's delivery accounting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DeliveryStatus {
    /// Mass delivered to the base for this module (kg).
    pub delivered_mass_kg: f64,
    /// Crew-time applied (hours).
    pub crew_time_hr: f64,
}

/// On-site ISRU output a base hosts (from FA-06 ISRU plants), per commodity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IsruOutput {
    /// Production rate (kg/day).
    pub rate_kg_per_day: f64,
}

/// The bundle of composed values the host assembles. Tests pass stubs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BaseInputs {
    /// Site facts keyed by site id.
    pub sites: BTreeMap<SiteId, SiteFacts>,
    /// Tech maturities keyed by (faction, tech id).
    pub maturities: BTreeMap<(FactionId, String), TechMaturity>,
    /// Delivery status keyed by (base, module).
    pub delivery: BTreeMap<(BaseId, ModuleId), DeliveryStatus>,
    /// ISRU output keyed by (base, commodity).
    pub isru: BTreeMap<(BaseId, String), IsruOutput>,
}

impl BaseInputs {
    /// Site facts for a site (defaulting to a 1-AU, unshielded, unrestricted site).
    pub fn site(&self, id: &SiteId) -> Option<&SiteFacts> {
        self.sites.get(id)
    }

    /// A faction's maturity for a technology.
    pub fn maturity(&self, faction: FactionId, tech: &str) -> Option<&TechMaturity> {
        self.maturities.get(&(faction, tech.to_string()))
    }

    /// ISRU output for a base + commodity.
    pub fn isru_rate(&self, base: BaseId, commodity: &str) -> f64 {
        self.isru
            .get(&(base, commodity.to_string()))
            .map(|o| o.rate_kg_per_day)
            .unwrap_or(0.0)
    }
}
