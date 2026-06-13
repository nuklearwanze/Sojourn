//! Prospecting fields and deterministic body generation (FR-WORLD-501/502). A
//! field carries a sourced population model; a prospecting command samples new
//! small bodies from it via the `world/prospect` stream, giving each a
//! collision-free permanent id ([`crate::ids`]). Generated bodies are full FA-02
//! citizens (railed, targetable, divertible) — per-world facts whose *knowledge*
//! stays per-faction.

use crate::ids::GeneratedIds;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sojourn_astro::{BodyDef, BodyId, Elements};
use std::collections::BTreeMap;
use std::path::Path;

const AU_M: f64 = 1.495_978_707e11;
const G: f64 = 6.674_30e-11;

/// A uniform `[lo, hi]` range parameter.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Range {
    /// Lower bound.
    pub lo: f64,
    /// Upper bound.
    pub hi: f64,
}

impl Range {
    fn sample<R: RngCore>(&self, rng: &mut R) -> f64 {
        self.lo + (self.hi - self.lo) * crate::belief::uniform(rng)
    }
}

/// A prospecting field (DATA, `data/world/prospecting-fields.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProspectField {
    /// Stable data key.
    pub id: String,
    /// Expected new bodies per unit effort.
    pub count_per_effort: f64,
    /// Semi-major axis distribution (AU).
    pub sma_au: Range,
    /// Eccentricity distribution.
    pub ecc: Range,
    /// Inclination distribution (degrees).
    pub inc_deg: Range,
    /// Diameter distribution (m).
    pub diameter_m: Range,
    /// Bulk density (kg/m³) for the mass/μ estimate.
    pub density_kg_m3: f64,
    /// Taxonomic type weights (id, weight).
    pub type_mix: Vec<(String, f64)>,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldsFile {
    fields: Vec<ProspectField>,
}

/// The loaded prospecting fields (DATA).
#[derive(Debug, Clone, Default)]
pub struct ProspectFields {
    fields: BTreeMap<String, ProspectField>,
    /// Canonical source text (for the world content hash).
    pub canonical: String,
}

impl ProspectFields {
    /// Load and validate `prospecting-fields.ron`.
    pub fn load_dir(dir: &Path) -> Result<ProspectFields, String> {
        let path = dir.join("prospecting-fields.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        let f: FieldsFile = ron::from_str(&text).map_err(|e| format!("prospecting-fields.ron: {e}"))?;
        let mut fields = BTreeMap::new();
        for fd in f.fields {
            if fd.source.trim().is_empty() {
                return Err(format!("prospecting-fields.ron: '{}' empty source", fd.id));
            }
            if fd.density_kg_m3 <= 0.0 || fd.count_per_effort < 0.0 {
                return Err(format!("prospecting-fields.ron: '{}' bad params", fd.id));
            }
            let wsum: f64 = fd.type_mix.iter().map(|(_, w)| *w).sum();
            if wsum <= 0.0 {
                return Err(format!("prospecting-fields.ron: '{}' type_mix sums to 0", fd.id));
            }
            fields.insert(fd.id.clone(), fd);
        }
        Ok(ProspectFields {
            fields,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// A field by id.
    pub fn get(&self, id: &str) -> Option<&ProspectField> {
        self.fields.get(id)
    }

    /// Field ids, ordered.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.fields.keys().map(String::as_str)
    }
}

/// Generate zero or more new bodies from a field for the given effort, drawing
/// from `rng` and allocating ids from `ids`. Bodies orbit `root` (the Sun) at
/// `epoch_ns`. Deterministic for a fixed stream and id state.
pub fn generate<R: RngCore>(
    field: &ProspectField,
    effort: f64,
    rng: &mut R,
    ids: &mut GeneratedIds,
    root: BodyId,
    epoch_ns: i64,
) -> Vec<BodyDef> {
    // Count = floor(expected) + Bernoulli(frac).
    let expected = (field.count_per_effort * effort).max(0.0);
    let base = libm::floor(expected);
    let frac = expected - base;
    let mut n = base as u64;
    if crate::belief::uniform(rng) < frac {
        n += 1;
    }

    let wsum: f64 = field.type_mix.iter().map(|(_, w)| *w).sum();
    let mut out = Vec::new();
    for _ in 0..n {
        let sma = field.sma_au.sample(rng) * AU_M;
        let ecc = field.ecc.sample(rng).clamp(0.0, 0.95);
        let inc = field.inc_deg.sample(rng).to_radians();
        let raan = crate::belief::uniform(rng) * 2.0 * core::f64::consts::PI;
        let argp = crate::belief::uniform(rng) * 2.0 * core::f64::consts::PI;
        let m0 = crate::belief::uniform(rng) * 2.0 * core::f64::consts::PI;
        let diameter = field.diameter_m.sample(rng).max(1.0);
        let radius = 0.5 * diameter;
        let mass = field.density_kg_m3 * (4.0 / 3.0) * core::f64::consts::PI * radius.powi(3);
        let mu = (G * mass).max(1.0);

        // Taxonomic type (cumulative draw).
        let pick = crate::belief::uniform(rng) * wsum;
        let mut acc = 0.0;
        let mut ty = field
            .type_mix
            .first()
            .map(|(t, _)| t.clone())
            .unwrap_or_else(|| "X".into());
        for (t, w) in &field.type_mix {
            acc += *w;
            if pick <= acc {
                ty = t.clone();
                break;
            }
        }

        let id = ids.alloc();
        out.push(BodyDef {
            id,
            name_id: format!("gen-{ty}-{}", id.0),
            mu,
            radius,
            j2: None,
            rotation_period_s: None,
            atmosphere: None,
            gravitating: false,
            divertible: true,
            parent: Some(root),
            elements: Some(Elements {
                sma,
                ecc,
                inc,
                raan,
                argp,
                mean_anomaly_at_epoch: m0,
                epoch_ns,
            }),
            source: format!("generated: prospecting field '{}' ({})", field.id, field.source),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha12Rng;
    use rand_core::SeedableRng;

    fn field() -> ProspectField {
        ProspectField {
            id: "belt".into(),
            count_per_effort: 3.0,
            sma_au: Range { lo: 2.1, hi: 3.3 },
            ecc: Range { lo: 0.0, hi: 0.3 },
            inc_deg: Range { lo: 0.0, hi: 20.0 },
            diameter_m: Range { lo: 200.0, hi: 5000.0 },
            density_kg_m3: 2000.0,
            type_mix: vec![("C".into(), 0.75), ("S".into(), 0.17), ("M".into(), 0.08)],
            source: "test".into(),
        }
    }

    #[test]
    fn generation_is_deterministic_per_seed() {
        let f = field();
        let run_once = |seed: u64| {
            let mut rng = ChaCha12Rng::seed_from_u64(seed);
            let mut ids = GeneratedIds::default();
            generate(&f, 2.0, &mut rng, &mut ids, BodyId(0), 0)
        };
        assert_eq!(run_once(99), run_once(99), "identical seed ⇒ identical bodies (ids/orbits)");
    }

    #[test]
    fn ids_are_collision_free_and_in_the_generated_range() {
        let f = field();
        let mut rng = ChaCha12Rng::seed_from_u64(3);
        let mut ids = GeneratedIds::default();
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..50 {
            for b in generate(&f, 5.0, &mut rng, &mut ids, BodyId(0), 0) {
                assert!(GeneratedIds::is_generated(b.id), "id in generated range");
                assert!(seen.insert(b.id.0), "ids never collide");
                assert!(b.divertible && !b.gravitating, "generated bodies are divertible small bodies");
                assert_eq!(b.parent, Some(BodyId(0)));
                assert!(b.elements.is_some());
            }
        }
    }

    #[test]
    fn aggregate_matches_field_distribution() {
        let f = field();
        let mut rng = ChaCha12Rng::seed_from_u64(7);
        let mut ids = GeneratedIds::default();
        let mut smas = Vec::new();
        for s in 0..400 {
            // Vary effort deterministically; accumulate many draws.
            let _ = s;
            for b in generate(&f, 4.0, &mut rng, &mut ids, BodyId(0), 0) {
                smas.push(b.elements.unwrap().sma / AU_M);
            }
        }
        assert!(smas.len() > 1000, "enough samples ({})", smas.len());
        let mean: f64 = smas.iter().sum::<f64>() / smas.len() as f64;
        // Uniform[2.1, 3.3] ⇒ mean 2.7.
        assert!((mean - 2.7).abs() < 0.05, "sma mean {mean} ≈ 2.7 within tolerance");
    }
}
