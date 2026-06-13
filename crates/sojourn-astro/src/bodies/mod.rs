//! Body catalogue (contracts/body-catalog.md): the consumed interface FA-03
//! implements in production; loaded here from the sourced test-catalogue fixtures.

pub mod divert;
pub mod rails;

use crate::math::Elements;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Stable body identity within a catalogue version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BodyId(pub u32);

/// Exponential atmosphere model (drag, aero passes, entry-handoff boundary).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtmosphereDef {
    /// Entry-interface altitude (m above mean radius): crossing it outside a
    /// planned aero pass is the EDL handoff.
    pub interface_alt_m: f64,
    /// Drag-relevant ceiling (m): drag applies below this (FR-ASTRO-102).
    pub drag_alt_m: f64,
    /// Reference density at zero altitude (kg/m³).
    pub rho0: f64,
    /// Scale height (m).
    pub scale_height_m: f64,
    /// Provenance.
    pub source: String,
}

/// One catalogued body (immutable data; motion overrides live in the slice).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyDef {
    /// Identity.
    pub id: BodyId,
    /// Identifier-style name (display text is FA-10's concern).
    pub name_id: String,
    /// Gravitational parameter (m³/s²).
    pub mu: f64,
    /// Mean radius (m).
    pub radius: f64,
    /// Oblateness coefficient (dominant-body perturbation).
    pub j2: Option<f64>,
    /// Sidereal rotation period (s).
    pub rotation_period_s: Option<f64>,
    /// Atmosphere, if any.
    pub atmosphere: Option<AtmosphereDef>,
    /// Far-field gravity flag (clarified rule, FR-ASTRO-101).
    pub gravitating: bool,
    /// Small-body divert eligibility (FR-ASTRO-107).
    pub divertible: bool,
    /// Parent body (None = the system root/star).
    pub parent: Option<BodyId>,
    /// Rail definition about the parent (None only for the root).
    pub elements: Option<Elements>,
    /// Provenance (mandatory, Principle I).
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogFile {
    bodies: Vec<BodyDef>,
}

/// Dynamic motion state of a body (lives in the astro slice; FR-ASTRO-107).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodyMotion {
    /// Craft-grade numerical propagation after a diversion. State is relative
    /// to `dominant` (same convention as craft).
    Diverted {
        /// Dominant body (SOI owner) the state is expressed in.
        dominant: BodyId,
        /// Body-centred inertial state.
        state: crate::math::StateVec,
    },
    /// Back on a rail with post-diversion elements (about the original parent).
    ReRailed {
        /// New rail.
        elements: Elements,
    },
}

/// The loaded, validated catalogue.
#[derive(Debug, Clone)]
pub struct Catalog {
    bodies: BTreeMap<BodyId, BodyDef>,
    /// BLAKE3 of the canonical file content (slice pins it; saves verify it).
    pub content_hash: [u8; 32],
}

impl Catalog {
    /// Load and validate the catalogue from `dir`. FA-03 production form: if a
    /// `catalog/` subdirectory exists, every `*.ron` in it (in sorted filename
    /// order) is parsed as a `CatalogFile` and the bodies are merged — the real
    /// multi-file Solar-System catalogue. Otherwise the single FA-02
    /// `test-catalog.ron` fixture is loaded (unchanged behaviour, byte-identical
    /// content hash).
    pub fn load_dir(dir: &Path) -> Result<Catalog, String> {
        let catalog_dir = dir.join("catalog");
        if catalog_dir.is_dir() {
            let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&catalog_dir)
                .map_err(|e| format!("reading {}: {e}", catalog_dir.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "ron"))
                .collect();
            files.sort();
            if files.is_empty() {
                return Err(format!("no .ron catalogue files in {}", catalog_dir.display()));
            }
            let mut combined = String::new();
            let mut all_bodies = Vec::new();
            for f in &files {
                let text = std::fs::read_to_string(f)
                    .map_err(|e| format!("reading {}: {e}", f.display()))?;
                let cf: CatalogFile =
                    ron::from_str(&text).map_err(|e| format!("{}: {e}", f.display()))?;
                combined.push_str(&normalize(&text));
                combined.push('\n');
                all_bodies.extend(cf.bodies);
            }
            Self::from_bodies(all_bodies, &combined)
        } else {
            let path = dir.join("test-catalog.ron");
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("reading {}: {e}", path.display()))?;
            Self::from_source(&text)
        }
    }

    /// Build from a single file's content (the FA-02 fixture path).
    pub fn from_source(text: &str) -> Result<Catalog, String> {
        let file: CatalogFile = ron::from_str(text).map_err(|e| format!("catalogue: {e}"))?;
        Self::from_bodies(file.bodies, &normalize(text))
    }

    /// A clone of this catalogue extended with runtime-generated bodies
    /// (prospecting products, FR-WORLD-502). The content hash is **unchanged** —
    /// generated bodies are deterministic journalled runtime state pinned by the
    /// world slice, not part of the immutable data version. Used to build
    /// planning-query snapshots in which a generated body is a first-class target.
    pub fn with_generated(&self, extra: impl IntoIterator<Item = BodyDef>) -> Catalog {
        let mut bodies = self.bodies.clone();
        for b in extra {
            bodies.insert(b.id, b);
        }
        Catalog {
            bodies,
            content_hash: self.content_hash,
        }
    }

    /// Validate a body set and build the catalogue. `hash_text` is the canonical
    /// (newline-normalised) source text whose BLAKE3 the slice pins.
    fn from_bodies(bodies_vec: Vec<BodyDef>, hash_text: &str) -> Result<Catalog, String> {
        let mut bodies = BTreeMap::new();
        for b in bodies_vec {
            if b.source.trim().is_empty() {
                return Err(format!("body '{}' has an empty source field", b.name_id));
            }
            if b.mu <= 0.0 || b.radius <= 0.0 {
                return Err(format!(
                    "body '{}': mu and radius must be positive",
                    b.name_id
                ));
            }
            if let Some(el) = &b.elements {
                if !(0.0..1.0).contains(&el.ecc) {
                    return Err(format!(
                        "body '{}': rail eccentricity must be in [0,1) (elliptic rails)",
                        b.name_id
                    ));
                }
                if el.sma <= 0.0 {
                    return Err(format!("body '{}': rail sma must be positive", b.name_id));
                }
            }
            if b.parent.is_none() && b.elements.is_some() {
                return Err(format!("root body '{}' must not have elements", b.name_id));
            }
            if b.parent.is_some() && b.elements.is_none() {
                return Err(format!("body '{}' needs rail elements", b.name_id));
            }
            if bodies.insert(b.id, b).is_some() {
                return Err("duplicate body id".to_string());
            }
        }
        // Parent links resolve and are acyclic; divertible only on small bodies
        // (heuristic guard: must not gravitate far-field and must be < 100 km).
        for b in bodies.values() {
            if let Some(p) = b.parent {
                if !bodies.contains_key(&p) {
                    return Err(format!("body '{}' has unknown parent", b.name_id));
                }
                let mut seen = vec![b.id];
                let mut cur = p;
                loop {
                    if seen.contains(&cur) {
                        return Err(format!("parent cycle at body '{}'", b.name_id));
                    }
                    seen.push(cur);
                    match bodies[&cur].parent {
                        Some(next) => cur = next,
                        None => break,
                    }
                }
            }
            // Divertibility is flag-driven (body-catalogue contract): a divertible
            // body must not gravitate far-field. The FA-02 fixture-only radius
            // heuristic is dropped — real small bodies are validated by the data
            // build tool (FR-WORLD-106), not a magic radius here.
            if b.divertible && b.gravitating {
                return Err(format!(
                    "body '{}': a divertible body must not gravitate far-field",
                    b.name_id
                ));
            }
        }
        let content_hash = *blake3::hash(hash_text.as_bytes()).as_bytes();
        Ok(Catalog {
            bodies,
            content_hash,
        })
    }

    /// Body by id.
    pub fn body(&self, id: BodyId) -> Option<&BodyDef> {
        self.bodies.get(&id)
    }

    /// All bodies in id order.
    pub fn bodies(&self) -> impl Iterator<Item = &BodyDef> {
        self.bodies.values()
    }

    /// The system root (sole body without a parent).
    pub fn root(&self) -> BodyId {
        self.bodies
            .values()
            .find(|b| b.parent.is_none())
            .map(|b| b.id)
            .expect("validated catalogue has a root")
    }

    /// Children of a body, in id order.
    pub fn children(&self, id: BodyId) -> impl Iterator<Item = &BodyDef> {
        self.bodies.values().filter(move |b| b.parent == Some(id))
    }
}
