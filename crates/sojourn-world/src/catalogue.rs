//! Indexes over the real catalogue (for the <50 ms filtered queries, FR-WORLD-802)
//! plus the sourced resource taxonomy and per-body metadata (composition class,
//! designation) that FA-03 carries alongside the astro `BodyDef` without changing
//! the body-catalogue contract.

use serde::{Deserialize, Serialize};
use sojourn_astro::{BodyDef, BodyId, Catalog};
use std::collections::BTreeMap;
use std::path::Path;

/// One resource kind in the sourced taxonomy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceKind {
    /// Identifier (e.g. `"water-ice"`).
    pub id: String,
    /// Human-readable name identifier (display text is FA-10's).
    pub name_id: String,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourcesFile {
    kinds: Vec<ResourceKind>,
    source: String,
}

/// The sourced resource taxonomy (DATA, `data/world/resources.ron`).
#[derive(Debug, Clone)]
pub struct Resources {
    /// Resource kinds.
    pub kinds: Vec<ResourceKind>,
    /// Provenance.
    pub source: String,
    /// Canonical source text (for the world content hash).
    pub canonical: String,
}

impl Resources {
    /// Load and validate `resources.ron`.
    pub fn load_dir(dir: &Path) -> Result<Resources, String> {
        let path = dir.join("resources.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        let f: ResourcesFile = ron::from_str(&text).map_err(|e| format!("resources.ron: {e}"))?;
        if f.source.trim().is_empty() {
            return Err("resources.ron: empty source".into());
        }
        for k in &f.kinds {
            if k.source.trim().is_empty() {
                return Err(format!("resources.ron: kind '{}' has empty source", k.id));
            }
        }
        Ok(Resources {
            kinds: f.kinds,
            source: f.source,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// Is `id` a known resource kind?
    pub fn has(&self, id: &str) -> bool {
        self.kinds.iter().any(|k| k.id == id)
    }
}

/// Per-body FA-03 metadata, keyed by body id (kept out of the astro `BodyDef`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyMeta {
    /// Body id this metadata describes.
    pub id: BodyId,
    /// Taxonomic / compositional class (e.g. `"C"`, `"S"`, `"M"`, `"ice"`).
    pub composition_class: String,
    /// IAU / MPC designation.
    pub designation: String,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyMetaFile {
    bodies: Vec<BodyMeta>,
}

/// Loaded body metadata (DATA, `data/world/body-meta.ron`).
#[derive(Debug, Clone, Default)]
pub struct BodyMetaTable {
    by_id: BTreeMap<BodyId, BodyMeta>,
    /// Canonical source text (for the world content hash).
    pub canonical: String,
}

impl BodyMetaTable {
    /// Load `body-meta.ron` if present (optional file).
    pub fn load_dir(dir: &Path) -> Result<BodyMetaTable, String> {
        let path = dir.join("body-meta.ron");
        if !path.exists() {
            return Ok(BodyMetaTable::default());
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        let f: BodyMetaFile = ron::from_str(&text).map_err(|e| format!("body-meta.ron: {e}"))?;
        let mut by_id = BTreeMap::new();
        for m in f.bodies {
            if m.source.trim().is_empty() {
                return Err(format!(
                    "body-meta.ron: '{}' has empty source",
                    m.designation
                ));
            }
            by_id.insert(m.id, m);
        }
        Ok(BodyMetaTable {
            by_id,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// Metadata for a body, if any.
    pub fn get(&self, id: BodyId) -> Option<&BodyMeta> {
        self.by_id.get(&id)
    }
}

/// Filters for catalogue queries (FR-WORLD-701; indexed, not full-scan).
#[derive(Debug, Clone, Copy, Default)]
pub struct BodyFilter {
    /// Only bodies whose parent is this id.
    pub parent: Option<BodyId>,
    /// Only gravitating (`Some(true)`) / non-gravitating (`Some(false)`) bodies.
    pub gravitating: Option<bool>,
    /// Only divertible bodies.
    pub divertible: Option<bool>,
}

/// Indexes built once at load for fast filtered access (FR-WORLD-802).
#[derive(Debug, Clone, Default)]
pub struct CatalogIndex {
    by_parent: BTreeMap<BodyId, Vec<BodyId>>,
    gravitating: Vec<BodyId>,
    divertible: Vec<BodyId>,
    all: Vec<BodyId>,
}

impl CatalogIndex {
    /// Build indexes over a catalogue (base ∪ generated already merged by caller).
    pub fn build(catalog: &Catalog) -> CatalogIndex {
        let mut idx = CatalogIndex::default();
        for b in catalog.bodies() {
            idx.all.push(b.id);
            if let Some(p) = b.parent {
                idx.by_parent.entry(p).or_default().push(b.id);
            }
            if b.gravitating {
                idx.gravitating.push(b.id);
            }
            if b.divertible {
                idx.divertible.push(b.id);
            }
        }
        idx
    }

    /// Body ids matching the filter (intersection of the relevant indexes),
    /// resolved against `catalog`. Ordered.
    pub fn query<'a>(&self, catalog: &'a Catalog, filter: &BodyFilter) -> Vec<&'a BodyDef> {
        let candidate: Vec<BodyId> = match filter.parent {
            Some(p) => self.by_parent.get(&p).cloned().unwrap_or_default(),
            None => self.all.clone(),
        };
        candidate
            .into_iter()
            .filter_map(|id| catalog.body(id))
            .filter(|b| filter.gravitating.is_none_or(|g| b.gravitating == g))
            .filter(|b| filter.divertible.is_none_or(|d| b.divertible == d))
            .collect()
    }

    /// Total bodies indexed.
    pub fn len(&self) -> usize {
        self.all.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.all.is_empty()
    }
}
