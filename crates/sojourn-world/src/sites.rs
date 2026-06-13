//! Surveyable sites (FR-WORLD-401/402/403). A site is anchored to a body **by id**
//! (so it follows a diverted body — edge case), carries a COSPAR
//! planetary-protection category (catalogue-level knowledge) and a set of
//! surveyable properties, each with a sourced **plausibility distribution** the
//! ground truth is seeded from at world creation.

use crate::belief::{Property, SiteId};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sojourn_astro::BodyId;
use std::collections::BTreeMap;
use std::path::Path;

/// COSPAR planetary-protection category (catalogue-level, cheaply knowable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PpCategory {
    /// I — no protection.
    I,
    /// II — record-keeping.
    II,
    /// III — controlled flyby/orbit.
    III,
    /// IV — controlled landing.
    IV,
    /// V — sample return / restricted Earth return.
    V,
}

/// Where a site sits on (or around) its body.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Placement {
    /// Surface coordinates (degrees).
    Surface {
        /// Latitude.
        lat: f64,
        /// Longitude.
        lon: f64,
    },
    /// An orbital site at this altitude (m above mean radius).
    Orbital {
        /// Altitude (m).
        alt_m: f64,
    },
}

/// A sourced plausibility distribution for a ground-truth value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Dist {
    /// Reality fixes this value (no per-game variation).
    Fixed(f64),
    /// Uniform on `[lo, hi]`.
    Uniform {
        /// Lower bound.
        lo: f64,
        /// Upper bound.
        hi: f64,
    },
    /// Log-normal (for positive quantities like grade).
    LogNormal {
        /// Mean of the underlying normal (natural-log space).
        mean_log: f64,
        /// Standard deviation of the underlying normal.
        sd_log: f64,
    },
}

impl Dist {
    /// Draw a value from a kernel stream (deterministic).
    pub fn sample<R: RngCore>(&self, rng: &mut R) -> f64 {
        match self {
            Dist::Fixed(v) => *v,
            Dist::Uniform { lo, hi } => lo + (hi - lo) * crate::belief::uniform(rng),
            Dist::LogNormal { mean_log, sd_log } => {
                libm::exp(mean_log + sd_log * crate::belief::standard_normal(rng))
            }
        }
    }
}

/// A surveyable property's truth distribution at a site.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyTruth {
    /// Which property.
    pub property: Property,
    /// The sourced plausibility distribution its ground truth is drawn from.
    pub dist: Dist,
}

/// A site definition (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteDef {
    /// Stable data key.
    pub id: String,
    /// Anchor body id.
    pub body: BodyId,
    /// Placement on/around the body.
    pub placement: Placement,
    /// Planetary-protection category.
    pub pp_category: PpCategory,
    /// Surveyable properties + their truth distributions.
    pub properties: Vec<PropertyTruth>,
    /// Resource kind this site's grade refers to (taxonomy id).
    pub resource: String,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SitesFile {
    sites: Vec<SiteDef>,
}

/// The loaded site set (DATA, `data/world/sites.ron`).
#[derive(Debug, Clone, Default)]
pub struct Sites {
    defs: Vec<SiteDef>,
    id_of: BTreeMap<String, SiteId>,
    index_of: BTreeMap<SiteId, usize>,
    by_body: BTreeMap<BodyId, Vec<SiteId>>,
    /// Canonical source text (for the world content hash).
    pub canonical: String,
}

impl Sites {
    /// Load and validate `sites.ron`. `SiteId`s are assigned from the sorted data
    /// keys (stable identity across data reorderings).
    pub fn load_dir(dir: &Path) -> Result<Sites, String> {
        let path = dir.join("sites.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        Self::from_source(&text)
    }

    /// Build from file content.
    pub fn from_source(text: &str) -> Result<Sites, String> {
        let f: SitesFile = ron::from_str(text).map_err(|e| format!("sites.ron: {e}"))?;
        let mut sorted: Vec<String> = f.sites.iter().map(|s| s.id.clone()).collect();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != f.sites.len() {
            return Err("sites.ron: duplicate site id".into());
        }
        let id_of: BTreeMap<String, SiteId> = sorted
            .iter()
            .enumerate()
            .map(|(i, k)| (k.clone(), SiteId(i as u32)))
            .collect();
        let mut defs = Vec::new();
        let mut index_of = BTreeMap::new();
        let mut by_body: BTreeMap<BodyId, Vec<SiteId>> = BTreeMap::new();
        for d in f.sites {
            if d.source.trim().is_empty() {
                return Err(format!("sites.ron: site '{}' has empty source", d.id));
            }
            let sid = id_of[&d.id];
            index_of.insert(sid, defs.len());
            by_body.entry(d.body).or_default().push(sid);
            defs.push(d);
        }
        for v in by_body.values_mut() {
            v.sort();
        }
        Ok(Sites {
            defs,
            id_of,
            index_of,
            by_body,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// Resolve a data key to a `SiteId`.
    pub fn resolve(&self, key: &str) -> Option<SiteId> {
        self.id_of.get(key).copied()
    }

    /// The definition for a `SiteId`.
    pub fn def(&self, id: SiteId) -> Option<&SiteDef> {
        self.index_of.get(&id).map(|&i| &self.defs[i])
    }

    /// All site ids on a body (ordered).
    pub fn on_body(&self, body: BodyId) -> &[SiteId] {
        self.by_body.get(&body).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// All sites, in load order.
    pub fn all(&self) -> impl Iterator<Item = (SiteId, &SiteDef)> {
        self.defs
            .iter()
            .map(move |d| (self.id_of[&d.id], d))
    }

    /// The truth distribution for `(site, property)`, if the site has it.
    pub fn property_dist(&self, id: SiteId, p: Property) -> Option<Dist> {
        self.def(id)?
            .properties
            .iter()
            .find(|pt| pt.property == p)
            .map(|pt| pt.dist)
    }
}
