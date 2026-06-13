//! The tradable-commodity taxonomy (data-model §2, FR-EC-107). FA-06 owns this
//! taxonomy; raw materials **reference** FA-03 resource ids while it extends the
//! set with processed/manufactured/consumable/strategic goods and services
//! (clarified Q3:A). Sourced; no combat commodity (Principle IX).

use crate::ids::CommodityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The unit a commodity is measured in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    /// Mass (kg).
    Kg,
    /// Discrete units.
    Each,
    /// Intangible service.
    Service,
}

/// What kind of tradable good this is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommodityKind {
    /// A raw in-situ material; references an FA-03 resource id.
    Raw {
        /// The FA-03 resource taxonomy id this raw material maps to.
        resource_ref: String,
    },
    /// A processed good derived from another commodity (e.g. LOX/LH₂ from ice).
    Processed {
        /// The commodity it is processed from.
        from: CommodityId,
    },
    /// A manufactured product (ZBLAN fibre, protein crystals, spares, feedstock).
    Manufactured,
    /// A life-support consumable (food, O₂, N₂ buffer).
    Consumable,
    /// A capped strategic material; references a `strategic.ron` supply cap.
    Strategic {
        /// The strategic-supply id this commodity draws from.
        cap_ref: String,
    },
    /// An intangible tradable service (launch, data, IP licence).
    Service,
}

/// A tradable commodity (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Commodity {
    /// Stable id.
    pub id: CommodityId,
    /// What kind of good.
    pub kind: CommodityKind,
    /// Unit of measure.
    pub unit: Unit,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommoditiesFile {
    commodities: Vec<Commodity>,
}

/// The loaded, validated commodity taxonomy.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Commodities {
    /// Commodities by id.
    pub by_id: BTreeMap<CommodityId, Commodity>,
    /// Canonical source text (for the econ content hash).
    pub canonical: String,
}

fn looks_like_weapon(id: &str) -> bool {
    let low = id.to_lowercase();
    ["weapon", "missile", "warhead", "gun", "bomb", "munition"]
        .iter()
        .any(|w| low.contains(w))
}

impl Commodities {
    /// Parse + validate `commodities.ron` content.
    pub fn from_source(text: &str) -> Result<Commodities, String> {
        let f: CommoditiesFile =
            ron::from_str(text).map_err(|e| format!("commodities.ron: {e}"))?;
        let mut by_id: BTreeMap<CommodityId, Commodity> = BTreeMap::new();
        for c in &f.commodities {
            if c.source.trim().is_empty() {
                return Err(format!("commodity '{}' has empty source", c.id.0));
            }
            if looks_like_weapon(&c.id.0) {
                return Err(format!(
                    "commodity '{}' looks like a weapon (Principle IX)",
                    c.id.0
                ));
            }
            if by_id.insert(c.id.clone(), c.clone()).is_some() {
                return Err(format!("duplicate commodity id '{}'", c.id.0));
            }
        }
        // Intra-file reference resolution (Processed.from / Strategic.cap_ref are
        // checked against the file; Raw.resource_ref resolves against the world
        // taxonomy in the harness `validate-data` pass).
        for c in &f.commodities {
            if let CommodityKind::Processed { from } = &c.kind
                && !by_id.contains_key(from)
            {
                return Err(format!(
                    "commodity '{}' processed from unknown '{}'",
                    c.id.0, from.0
                ));
            }
        }
        Ok(Commodities {
            by_id,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// A commodity by id.
    pub fn get(&self, id: &CommodityId) -> Option<&Commodity> {
        self.by_id.get(id)
    }

    /// Strategic-supply ids referenced by `Strategic` commodities.
    pub fn strategic_refs(&self) -> impl Iterator<Item = &str> {
        self.by_id.values().filter_map(|c| match &c.kind {
            CommodityKind::Strategic { cap_ref } => Some(cap_ref.as_str()),
            _ => None,
        })
    }

    /// Raw resource ids this taxonomy references (resolved against the world in CI).
    pub fn raw_resource_refs(&self) -> impl Iterator<Item = &str> {
        self.by_id.values().filter_map(|c| match &c.kind {
            CommodityKind::Raw { resource_ref } => Some(resource_ref.as_str()),
            _ => None,
        })
    }
}

/// A capped strategic-material supply (DATA, `strategic.ron`, FR-EC-106). The cap
/// is enforced by ledger conservation (a faction can't draw stock it lacks); the
/// `policy_gate` label is carried for FA-09, whose political cost is out of scope
/// here (consumed as an opaque input).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategicSupply {
    /// Stable id (referenced by `Strategic.cap_ref`).
    pub id: String,
    /// Annual world production cap (kg).
    pub annual_production_cap: f64,
    /// World stock currently available (kg).
    pub world_stock: f64,
    /// A policy-gate label (e.g. "civil-pu238", "heu", "leu") — FA-09 prices it.
    pub policy_gate: String,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct StrategicFile {
    supplies: Vec<StrategicSupply>,
}

/// The loaded, validated strategic-material supplies (module data).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StrategicDefs {
    /// Supplies by id.
    pub by_id: BTreeMap<String, StrategicSupply>,
    /// Canonical source text (for the econ content hash).
    pub canonical: String,
}

impl StrategicDefs {
    /// Parse + validate `strategic.ron`.
    pub fn from_source(text: &str) -> Result<StrategicDefs, String> {
        let f: StrategicFile = ron::from_str(text).map_err(|e| format!("strategic.ron: {e}"))?;
        let mut by_id = BTreeMap::new();
        for s in f.supplies {
            if s.source.trim().is_empty() {
                return Err(format!("strategic supply '{}' has empty source", s.id));
            }
            if s.annual_production_cap < 0.0 || s.world_stock < 0.0 {
                return Err(format!("strategic supply '{}': negative cap/stock", s.id));
            }
            if by_id.insert(s.id.clone(), s).is_some() {
                return Err("duplicate strategic supply".to_string());
            }
        }
        Ok(StrategicDefs {
            by_id,
            canonical: text.replace("\r\n", "\n"),
        })
    }
}
