//! The **engine-private** ground-truth store (R4, FR-WORLD-301/306). Truth is
//! seeded at world creation from sourced plausibility distributions via the
//! `world/seed` stream, then held here and read **only** by the module's own
//! resolution (`observe`) and a feature-gated privileged test accessor — never by
//! the query surface (`WorldSnapshot` carries no truth, structurally).

use crate::belief::{Property, SiteId};
use crate::sites::Sites;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sojourn_astro::BodyId;
use std::collections::BTreeMap;
use std::path::Path;

/// Where seeded astrobiology, if present, would live (a science object — never an
/// actor, Principle IX).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstrobioTier {
    /// Subsurface (e.g. Mars).
    Subsurface,
    /// Subglacial ocean (e.g. Europa, Enceladus).
    Ocean,
    /// Atmospheric (e.g. Venus clouds).
    Atmospheric,
    /// Brine (e.g. Ceres).
    Brine,
}

/// Seeded astrobiology ground truth for one candidate world.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AstrobioTruth {
    /// Whether life is present in this game (mostly false).
    pub present: bool,
    /// The tier it would occupy if present.
    pub tier: AstrobioTier,
}

/// An astrobiology candidate world (DATA, `data/world/astrobiology.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstrobioCandidate {
    /// Body id.
    pub body: BodyId,
    /// Sourced plausibility of presence (kept low — mostly negative per the design).
    pub presence_prob: f64,
    /// Tier if present.
    pub tier: AstrobioTier,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AstrobioFile {
    candidates: Vec<AstrobioCandidate>,
}

/// The loaded candidate set (DATA).
#[derive(Debug, Clone, Default)]
pub struct AstrobiologyCandidates {
    /// Candidates in body-id order.
    pub candidates: Vec<AstrobioCandidate>,
    /// Canonical source text (for the world content hash).
    pub canonical: String,
}

impl AstrobiologyCandidates {
    /// Load and validate `astrobiology.ron`.
    pub fn load_dir(dir: &Path) -> Result<AstrobiologyCandidates, String> {
        let path = dir.join("astrobiology.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        let f: AstrobioFile = ron::from_str(&text).map_err(|e| format!("astrobiology.ron: {e}"))?;
        let mut candidates = f.candidates;
        for c in &candidates {
            if c.source.trim().is_empty() {
                return Err("astrobiology.ron: a candidate has empty source".into());
            }
            if !(0.0..=1.0).contains(&c.presence_prob) {
                return Err("astrobiology.ron: presence_prob must be in [0,1]".into());
            }
        }
        candidates.sort_by_key(|c| c.body.0);
        Ok(AstrobiologyCandidates {
            candidates,
            canonical: text.replace("\r\n", "\n"),
        })
    }
}

/// The engine-private ground-truth store (lives in the world slice, serialized).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GroundTruth {
    values: BTreeMap<(SiteId, Property), f64>,
    astrobiology: BTreeMap<u32, AstrobioTruth>,
}

impl GroundTruth {
    /// Seed the store deterministically from the sourced distributions. Iteration
    /// order is fixed (sorted site ids, then candidates in body-id order) so the
    /// stream is consumed identically on every run and replay.
    pub fn seed<R: RngCore>(
        sites: &Sites,
        candidates: &AstrobiologyCandidates,
        rng: &mut R,
    ) -> GroundTruth {
        let mut values = BTreeMap::new();
        // Deterministic order: SiteId ascending, then the site's declared property
        // order.
        let mut ordered: Vec<(SiteId, usize)> = Vec::new();
        for (sid, _def) in sites.all() {
            ordered.push((sid, 0));
        }
        ordered.sort_by_key(|(s, _)| s.0);
        for (sid, _) in ordered {
            let def = sites.def(sid).expect("site");
            for pt in &def.properties {
                let v = pt.dist.sample(rng);
                values.insert((sid, pt.property), v);
            }
        }
        let mut astrobiology = BTreeMap::new();
        for c in &candidates.candidates {
            let u = crate::belief::uniform(rng);
            astrobiology.insert(
                c.body.0,
                AstrobioTruth {
                    present: u < c.presence_prob,
                    tier: c.tier,
                },
            );
        }
        GroundTruth {
            values,
            astrobiology,
        }
    }

    /// The seeded truth for `(site, property)` (engine-internal; crate-visible).
    pub(crate) fn value(&self, site: SiteId, p: Property) -> Option<f64> {
        self.values.get(&(site, p)).copied()
    }

    /// Seeded astrobiology truth for a body (engine-internal; crate-visible —
    /// the resolution path the mission slices will drive).
    #[allow(dead_code)]
    pub(crate) fn astrobiology(&self, body: BodyId) -> Option<AstrobioTruth> {
        self.astrobiology.get(&body.0).copied()
    }

    /// **Privileged** read of a site truth value (test/inspection only — gated so
    /// it is never part of the shipped public surface). The honesty audit
    /// (`tests/audit.rs`) runs *without* this feature.
    #[cfg(feature = "privileged")]
    pub fn privileged_value(&self, site: SiteId, p: Property) -> Option<f64> {
        self.values.get(&(site, p)).copied()
    }

    /// **Privileged** read of astrobiology truth (test/inspection only).
    #[cfg(feature = "privileged")]
    pub fn privileged_astrobiology(&self, body: BodyId) -> Option<AstrobioTruth> {
        self.astrobiology.get(&body.0).copied()
    }
}
