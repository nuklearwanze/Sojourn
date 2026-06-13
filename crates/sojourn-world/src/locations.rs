//! Dynamical locations (FR-WORLD-201/202): first-class, queryable, time-resolvable
//! nodes that are not bodies — orbit bands, collinear Lagrange points, staging
//! orbits and surface anchors. Each resolves to a point or region via FA-02's
//! railed body positions (and a first-order collinear L-point solution computed
//! in-crate). Identities are string-keyed in data, stable across saves and
//! catalogue versions.

use serde::{Deserialize, Serialize};
use sojourn_astro::bodies::rails::BodyStates;
use sojourn_astro::{BodyId, BodyMotion, Catalog, Vec3};
use std::collections::BTreeMap;
use std::path::Path;

/// Collinear Lagrange point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LPoint {
    /// L1 (between primary and secondary).
    L1,
    /// L2 (beyond the secondary).
    L2,
}

/// Staging-orbit family (characterised as a region in v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StagingKind {
    /// Halo orbit.
    Halo,
    /// Near-rectilinear halo orbit.
    Nrho,
    /// Distant retrograde orbit.
    Dro,
}

/// What kind of location this is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LocationKind {
    /// An orbit band around a body (radial shell region).
    OrbitBand {
        /// Body.
        body: BodyId,
        /// Lower altitude (m).
        alt_lo_m: f64,
        /// Upper altitude (m).
        alt_hi_m: f64,
        /// Inclination class label.
        inclination_class: String,
    },
    /// A collinear Lagrange point of a (primary, secondary) pair.
    LagrangePoint {
        /// Primary body.
        primary: BodyId,
        /// Secondary body.
        secondary: BodyId,
        /// Which collinear point.
        point: LPoint,
    },
    /// A named staging orbit (characterised region in v1).
    StagingOrbit {
        /// Primary body.
        primary: BodyId,
        /// Family.
        kind: StagingKind,
        /// Characteristic radius of the region (m).
        char_radius_m: f64,
    },
    /// A surface anchor at a body-fixed location.
    SurfaceAnchor {
        /// Body.
        body: BodyId,
        /// Latitude (degrees).
        lat: f64,
        /// Longitude (degrees).
        lon: f64,
    },
}

/// A resolved location at a time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Resolved {
    /// A point (root-centred inertial position, m).
    Point(Vec3),
    /// A region with a representative centre and radius (m).
    Region {
        /// Representative centre (root-centred inertial, m).
        center: Vec3,
        /// Characteristic radius (m).
        radius_m: f64,
    },
}

/// A location definition (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationDef {
    /// Stable data key.
    pub id: String,
    /// What it is.
    pub kind: LocationKind,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocationsFile {
    locations: Vec<LocationDef>,
}

/// The loaded location catalogue (DATA, `data/world/locations.ron`).
#[derive(Debug, Clone, Default)]
pub struct Locations {
    defs: Vec<LocationDef>,
    index_of: BTreeMap<String, usize>,
    /// Canonical source text (for the world content hash).
    pub canonical: String,
}

impl Locations {
    /// Load and validate `locations.ron`.
    pub fn load_dir(dir: &Path) -> Result<Locations, String> {
        let path = dir.join("locations.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        Self::from_source(&text)
    }

    /// Build from file content.
    pub fn from_source(text: &str) -> Result<Locations, String> {
        let f: LocationsFile = ron::from_str(text).map_err(|e| format!("locations.ron: {e}"))?;
        let mut defs = Vec::new();
        let mut index_of = BTreeMap::new();
        for d in f.locations {
            if d.source.trim().is_empty() {
                return Err(format!("locations.ron: '{}' has empty source", d.id));
            }
            if index_of.insert(d.id.clone(), defs.len()).is_some() {
                return Err(format!("locations.ron: duplicate id '{}'", d.id));
            }
            defs.push(d);
        }
        Ok(Locations {
            defs,
            index_of,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// All location ids, in load order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.defs.iter().map(|d| d.id.as_str())
    }

    /// The definition for an id.
    pub fn def(&self, id: &str) -> Option<&LocationDef> {
        self.index_of.get(id).map(|&i| &self.defs[i])
    }

    /// Validate that every referenced body exists in `catalog`.
    pub fn validate_refs(&self, catalog: &Catalog) -> Result<(), String> {
        for d in &self.defs {
            let bodies: Vec<BodyId> = match &d.kind {
                LocationKind::OrbitBand { body, .. } | LocationKind::SurfaceAnchor { body, .. } => {
                    vec![*body]
                }
                LocationKind::LagrangePoint {
                    primary, secondary, ..
                } => vec![*primary, *secondary],
                LocationKind::StagingOrbit { primary, .. } => vec![*primary],
            };
            for b in bodies {
                if catalog.body(b).is_none() {
                    return Err(format!(
                        "location '{}' references unknown body {}",
                        d.id, b.0
                    ));
                }
            }
            if let LocationKind::OrbitBand {
                alt_lo_m, alt_hi_m, ..
            } = &d.kind
                && alt_lo_m >= alt_hi_m
            {
                return Err(format!("location '{}': alt_lo must be < alt_hi", d.id));
            }
        }
        Ok(())
    }

    /// Resolve a location at time `t_ns` against railed body positions.
    pub fn resolve(
        &self,
        id: &str,
        catalog: &Catalog,
        motions: &BTreeMap<BodyId, BodyMotion>,
        t_ns: i64,
    ) -> Option<Resolved> {
        let def = self.def(id)?;
        let mut states = BodyStates::new(catalog, motions, t_ns);
        Some(match &def.kind {
            LocationKind::OrbitBand {
                body,
                alt_lo_m,
                alt_hi_m,
                ..
            } => {
                let b = catalog.body(*body)?;
                let center = states.absolute(*body).r;
                Resolved::Region {
                    center,
                    radius_m: b.radius + 0.5 * (alt_lo_m + alt_hi_m),
                }
            }
            LocationKind::LagrangePoint {
                primary,
                secondary,
                point,
            } => {
                let mu1 = catalog.body(*primary)?.mu;
                let mu2 = catalog.body(*secondary)?.mu;
                let p = states.absolute(*primary).r;
                let s = states.absolute(*secondary).r;
                let sep = s - p;
                let r = sep.norm();
                // First-order collinear distance from the secondary.
                let alpha = libm::cbrt(mu2 / (3.0 * (mu1 + mu2)));
                let u = sep * (1.0 / r.max(1e-9));
                let pos = match point {
                    LPoint::L1 => s - u * (alpha * r),
                    LPoint::L2 => s + u * (alpha * r),
                };
                Resolved::Point(pos)
            }
            LocationKind::StagingOrbit {
                primary,
                char_radius_m,
                ..
            } => Resolved::Region {
                center: states.absolute(*primary).r,
                radius_m: *char_radius_m,
            },
            LocationKind::SurfaceAnchor { body, lat, lon } => {
                let b = catalog.body(*body)?;
                let center = states.absolute(*body).r;
                // Body-fixed unit vector from lat/lon, spun about global Z by the
                // rotation phase (FA-02's global-Z spin idealisation).
                let latr = lat.to_radians();
                let lonr = lon.to_radians();
                let phase = b
                    .rotation_period_s
                    .map(|t| 2.0 * core::f64::consts::PI * (t_ns as f64 / 1e9) / t)
                    .unwrap_or(0.0);
                let a = lonr + phase;
                let dir = Vec3 {
                    x: libm::cos(latr) * libm::cos(a),
                    y: libm::cos(latr) * libm::sin(a),
                    z: libm::sin(latr),
                };
                Resolved::Point(center + dir * b.radius)
            }
        })
    }
}
