//! Per-faction belief + the Gaussian precision-addition refinement model
//! (R3, contracts/belief-model.md). Also the home of the small key types the
//! truth/belief/observation machinery shares ([`FactionId`], [`SiteId`],
//! [`Target`], [`Property`], [`ObsClass`]).
//!
//! Scalar properties carry a Gaussian `(mean, var)` in a transformed space
//! (log-space for positive quantities). The update is precision addition with a
//! per-class floor clamp — which is what gives the spec's guarantees
//! structurally: variance is monotonically non-increasing (information never
//! decreases), the estimate moves toward truth, and repeated observations
//! converge to the class floor, never to truth.

use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Opaque faction identity (FA-09 binds real factions later; callers supply ids).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u32);

/// Stable site identity within a world-data version (assigned at load from the
/// sorted data keys).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SiteId(pub u32);

/// What a belief or truth attaches to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Target {
    /// A catalogued or generated body.
    Body(sojourn_astro::BodyId),
    /// A surveyable site.
    Site(SiteId),
}

/// A surveyable scalar property. Ordinal hazard is carried as a severity scalar
/// (a documented simplification; categorical confusion-matrix refinement is a
/// later slice). Astrobiology presence is handled separately (truth-only here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Property {
    /// Resource grade (positive; log-space).
    Grade,
    /// Illumination fraction.
    Illumination,
    /// Surface slope / roughness.
    Slope,
    /// Thermal environment.
    Thermal,
    /// Hazard severity (ordinal carried as a scalar).
    HazardLevel,
}

impl Property {
    /// All surveyable properties, in a fixed order.
    pub const ALL: [Property; 5] = [
        Property::Grade,
        Property::Illumination,
        Property::Slope,
        Property::Thermal,
        Property::HazardLevel,
    ];
}

/// Observation class. Floors order remote ≥ in-situ ≥ sample-grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ObsClass {
    /// Remote sensing (loosest floor).
    RemoteSensing,
    /// In-situ measurement.
    InSitu,
    /// Returned-sample grade (tightest floor).
    SampleGrade,
}

/// A belief: Gaussian `(mean, var)` in the property's transformed space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    /// Mean in transformed space (natural log for log-space properties).
    pub mean: f64,
    /// Variance — monotonically non-increasing under observation; floored.
    pub var: f64,
    /// Tick of the last observation (0 = prior only).
    pub last_obs_tick: u64,
}

/// A logged belief revision (per-faction, tick-stamped) powering
/// `belief_delta_since` (FR-WORLD-701).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BeliefChange {
    /// When the belief changed.
    pub tick: u64,
    /// Whose belief changed.
    pub faction: FactionId,
    /// What it is about.
    pub target: Target,
    /// Which property.
    pub property: Property,
}

/// A decoded estimate in natural units (what queries return).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PropertyEstimate {
    /// Best estimate in natural units.
    pub value: f64,
    /// 1-sigma uncertainty in natural units.
    pub uncertainty: f64,
}

/// Per-class measurement-noise parameters (DATA).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassModel {
    /// Best achievable measurement sigma — the class floor.
    pub floor: f64,
    /// Measurement sigma at quality 0.
    pub sigma0: f64,
}

/// Per-property prior + refinement parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyModel {
    /// Default prior mean (transformed space).
    pub prior_mean: f64,
    /// Default prior variance (MUST exceed every class floor variance).
    pub prior_var: f64,
    /// Positive quantity carried in log-space?
    pub log_space: bool,
    /// Sigma scale multiplier for this property.
    pub scale: f64,
    /// Observation classes that can sense this property.
    pub classes: Vec<ObsClass>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertyEntry {
    property: Property,
    prior_mean: f64,
    prior_var: f64,
    log_space: bool,
    scale: f64,
    classes: Vec<ObsClass>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PriorsFile {
    remote: ClassModel,
    insitu: ClassModel,
    samplegrade: ClassModel,
    properties: Vec<PropertyEntry>,
    source: String,
}

/// Loaded refinement parameters (DATA, `data/world/priors.ron`).
#[derive(Debug, Clone)]
pub struct Priors {
    classes: BTreeMap<ObsClass, ClassModel>,
    properties: BTreeMap<Property, PropertyModel>,
    /// Provenance.
    pub source: String,
    /// Canonical (newline-normalised) source text, for the world content hash.
    pub canonical: String,
}

impl Priors {
    /// Load and validate `priors.ron` from a directory.
    pub fn load_dir(dir: &Path) -> Result<Priors, String> {
        let path = dir.join("priors.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        Self::from_source(&text)
    }

    /// Build from file content.
    pub fn from_source(text: &str) -> Result<Priors, String> {
        let f: PriorsFile = ron::from_str(text).map_err(|e| format!("priors.ron: {e}"))?;
        if f.source.trim().is_empty() {
            return Err("priors.ron: empty source".into());
        }
        let classes = BTreeMap::from([
            (ObsClass::RemoteSensing, f.remote),
            (ObsClass::InSitu, f.insitu),
            (ObsClass::SampleGrade, f.samplegrade),
        ]);
        for (c, m) in &classes {
            if !(m.floor > 0.0 && m.sigma0 >= m.floor) {
                return Err(format!("priors.ron: class {c:?} needs 0 < floor ≤ sigma0"));
            }
        }
        let mut properties = BTreeMap::new();
        for e in f.properties {
            if e.prior_var <= 0.0 || e.scale <= 0.0 {
                return Err(format!(
                    "priors.ron: property {:?} needs positive prior_var and scale",
                    e.property
                ));
            }
            // Prior must be wider than the tightest reachable class floor, so the
            // monotone-information guarantee holds (R3).
            let tightest = classes
                .values()
                .map(|c| (e.scale * c.floor).powi(2))
                .fold(f64::INFINITY, f64::min);
            if e.prior_var <= tightest {
                return Err(format!(
                    "priors.ron: property {:?} prior_var {} must exceed its tightest floor var {}",
                    e.property, e.prior_var, tightest
                ));
            }
            properties.insert(
                e.property,
                PropertyModel {
                    prior_mean: e.prior_mean,
                    prior_var: e.prior_var,
                    log_space: e.log_space,
                    scale: e.scale,
                    classes: e.classes,
                },
            );
        }
        Ok(Priors {
            classes,
            properties,
            source: f.source,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// The model for a property.
    pub fn model(&self, p: Property) -> Option<&PropertyModel> {
        self.properties.get(&p)
    }

    /// The prior estimate for a property (the honest default when no observation
    /// has happened — guarantees a belief always exists, R3 edge case).
    pub fn prior_for(&self, p: Property) -> Estimate {
        let m = self.properties.get(&p).expect("known property");
        Estimate {
            mean: m.prior_mean,
            var: m.prior_var,
            last_obs_tick: 0,
        }
    }

    /// Does `class` sense `property`?
    pub fn senses(&self, p: Property, c: ObsClass) -> bool {
        self.properties
            .get(&p)
            .is_some_and(|m| m.classes.contains(&c))
    }

    /// Whether the property is carried in log-space.
    pub fn log_space(&self, p: Property) -> bool {
        self.properties.get(&p).is_some_and(|m| m.log_space)
    }

    /// Measurement sigma for `(class, quality)` on `property` (None if the class
    /// cannot sense it). Linear between `sigma0` (quality 0) and `floor`
    /// (quality 1), scaled per property.
    pub fn measurement_sigma(&self, p: Property, c: ObsClass, quality: f64) -> Option<f64> {
        let m = self.properties.get(&p)?;
        if !m.classes.contains(&c) {
            return None;
        }
        let cm = self.classes.get(&c)?;
        let q = quality.clamp(0.0, 1.0);
        Some(m.scale * (cm.sigma0 + (cm.floor - cm.sigma0) * q))
    }

    /// The floor variance for `class` on `property` (the tightest reachable).
    pub fn floor_var(&self, p: Property, c: ObsClass) -> f64 {
        let m = self.properties.get(&p).expect("known property");
        let cm = self.classes.get(&c).expect("known class");
        (m.scale * cm.floor).powi(2)
    }

    /// Decode an estimate into natural units for a query result.
    pub fn decode(&self, p: Property, e: Estimate) -> PropertyEstimate {
        if self.log_space(p) {
            let value = libm::exp(e.mean);
            // Delta-method 1-sigma in natural units for a log-space belief.
            PropertyEstimate {
                value,
                uncertainty: value * libm::sqrt(e.var),
            }
        } else {
            PropertyEstimate {
                value: e.mean,
                uncertainty: libm::sqrt(e.var),
            }
        }
    }

    /// Encode a natural-units truth into the property's estimator space.
    pub fn encode_truth(&self, p: Property, truth: f64) -> f64 {
        if self.log_space(p) {
            libm::log(truth.max(1e-300))
        } else {
            truth
        }
    }
}

/// The Gaussian precision-addition update with a floor clamp (R3). Returns the
/// posterior estimate. Guarantees: `post.var ≤ prior.var` (information never
/// decreases) and `post.var ≥ floor_v` (converges to the class floor, not zero).
pub fn refine(prior: Estimate, measurement: f64, sigma: f64, floor_v: f64, tick: u64) -> Estimate {
    let sig2 = (sigma * sigma).max(1e-300);
    let post_var = 1.0 / (1.0 / prior.var + 1.0 / sig2);
    let post_mean = post_var * (prior.mean / prior.var + measurement / sig2);
    Estimate {
        mean: post_mean,
        var: post_var.max(floor_v),
        last_obs_tick: tick,
    }
}

/// A standard-normal draw from a kernel stream (Box–Muller, libm-only).
pub fn standard_normal<R: RngCore>(rng: &mut R) -> f64 {
    let u1 = uniform(rng).max(1e-300);
    let u2 = uniform(rng);
    libm::sqrt(-2.0 * libm::log(u1)) * libm::cos(2.0 * core::f64::consts::PI * u2)
}

/// A uniform draw in [0, 1) from a kernel stream (same construction as astro).
pub fn uniform<R: RngCore>(rng: &mut R) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn est(mean: f64, var: f64) -> Estimate {
        Estimate {
            mean,
            var,
            last_obs_tick: 0,
        }
    }

    #[test]
    fn variance_never_increases_information_never_decreases() {
        // A worse (larger-sigma) observation still only adds precision (R3).
        let prior = est(0.0, 1.0);
        let tight = refine(prior, 0.5, 0.2, 1e-6, 1);
        assert!(tight.var < prior.var);
        // A later, much worse observation must not widen the belief.
        let after_bad = refine(tight, -3.0, 10.0, 1e-6, 2);
        assert!(after_bad.var <= tight.var + 1e-12, "information never decreases");
    }

    #[test]
    fn converges_to_floor_not_to_zero() {
        // Repeated identical-class observations approach the floor, never 0.
        let floor_v = 0.04; // sigma floor 0.2
        let mut e = est(0.0, 1.0);
        for k in 0..1000 {
            e = refine(e, 1.0, 0.2, floor_v, k);
        }
        assert!(e.var >= floor_v - 1e-12, "never below the floor");
        assert!(e.var <= floor_v + 1e-9, "converges to the floor");
        // Mean is pulled toward the (noise-free) measurement.
        assert!((e.mean - 1.0).abs() < 0.05);
    }

    #[test]
    fn stable_at_convergence_no_oscillation() {
        let floor_v = 0.04;
        let mut e = est(1.0, floor_v);
        for k in 0..100 {
            let before = e.var;
            e = refine(e, 1.0, 0.2, floor_v, k);
            assert!((e.var - before).abs() < 1e-12, "stable at the floor");
        }
    }

    #[test]
    fn poor_class_cannot_reach_a_tight_floor() {
        // A loose class (large floor) cannot beat a tighter class's floor.
        let mut remote = est(0.0, 1.0);
        for k in 0..1000 {
            remote = refine(remote, 1.0, 0.6, 0.36, k); // floor sigma 0.6
        }
        assert!(remote.var >= 0.36 - 1e-9, "remote sensing stuck at its floor");
    }
}
