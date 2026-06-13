//! Immutable research data (FR-RESP-701/702): the A1–A17 knowledge domains, the
//! engineering tech-tree node subset, the capability-category → candidate-path map,
//! the tuning params and the personnel-trait definitions — all schema-validated and
//! sourced. Loaded once at module init, hashed and pinned in saves.

use crate::ids::{DomainId, TechId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
}

// ----------------------------------------------------------------- domains

/// A knowledge domain (Track A).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainDef {
    /// Id (`"A1"`).
    pub id: DomainId,
    /// Name identifier (display is FA-10's).
    pub name_id: String,
    /// Coupled domains + synergy weights.
    pub synergy: Vec<(DomainId, f64)>,
    /// UL above which growth cost rises sharply (diminishing returns).
    pub dr_knee: f64,
    /// Steepness of the diminishing-returns falloff.
    pub dr_steepness: f64,
    /// Basic vs applied science (insight-pressure weighting).
    pub basic_science: bool,
    /// Whether missions inject UL here (A8–A16).
    pub mission_injectable: bool,
    /// Provenance.
    pub source: String,
}

// ----------------------------------------------------------------- tech tree

/// How a tech prerequisite may be satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrereqKind {
    /// Requires the prerequisite technology to have been built (no leapfrog).
    Product,
    /// May be satisfied by domain-UL investment (a leapfrog seam).
    UlSatisfiable,
}

/// A technology prerequisite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TechPrereq {
    /// The prerequisite technology.
    pub tech: TechId,
    /// How it may be satisfied.
    pub kind: PrereqKind,
}

/// One TRL step's cost/time/facility gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrlStep {
    /// The TRL this step reaches (2..=9).
    pub trl: u8,
    /// Sourced cost (abstract Design-Effort-days).
    pub cost: f64,
    /// Minimum duration floor (days) — funding cannot compress below this.
    pub min_duration_days: f64,
    /// Required facility capability (caller asserts availability).
    pub facility: Option<String>,
    /// S-curve steepness for in-step progress.
    pub scurve_k: f64,
}

/// Reliability-curve parameters for a technology (R10).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReliabilityCurve {
    /// Asymptotic reliability ceiling.
    pub ceiling: f64,
    /// Reliability at TRL 6 with no heritage (the floor).
    pub trl6_floor: f64,
    /// Weight of domain-UL margin in the curve.
    pub ul_weight: f64,
    /// Weight of accumulated flight-units (heritage) in the curve.
    pub heritage_weight: f64,
}

/// A technology node (Track B).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TechNode {
    /// Id (`"B1.5"`).
    pub id: TechId,
    /// Capability category (the reachability unit).
    pub capability_category: String,
    /// Real 2026 starting maturity.
    pub start_trl: u8,
    /// Domain-UL availability floors.
    pub ul_floors: Vec<(DomainId, f64)>,
    /// Tech prerequisites (incl. cross-branch + leapfrog seams).
    pub tech_prereqs: Vec<TechPrereq>,
    /// Per-step TRL ladder.
    pub trl_steps: Vec<TrlStep>,
    /// Reliability curve.
    pub reliability: ReliabilityCurve,
    /// Heritage-discount source (starts partway up the ladder).
    pub derivative_of: Option<TechId>,
    /// Provenance.
    pub source: String,
}

/// A capability category → candidate technology paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityCategory {
    /// Category id.
    pub id: String,
    /// Candidate technology paths (≥2 — the reachability invariant's domain).
    pub paths: Vec<TechId>,
    /// Provenance.
    pub source: String,
}

// ----------------------------------------------------------------- params + traits

/// Global tuning parameters (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Params {
    /// RP generated per scientist-skill-point per day.
    pub rp_per_scientist_day: f64,
    /// DE generated per engineer-skill-point per day.
    pub de_per_engineer_day: f64,
    /// UL gained per RP at the base of the diminishing-returns curve.
    pub ul_per_rp: f64,
    /// Overrun sigma (fractional cost/schedule variance).
    pub overrun_sigma: f64,
    /// Test-failure base probability at the bottom of a TRL step.
    pub test_fail_base: f64,
    /// UL injected into the domain by a failure-that-teaches.
    pub failure_ul_inject: f64,
    /// Insight gained per basic-science RP (breakthrough pressure).
    pub insight_per_basic_rp: f64,
    /// World-tide exogenous baseline UL growth per day.
    pub tide_baseline_per_day: f64,
    /// World-tide weight on aggregate faction UL.
    pub tide_aggregate_weight: f64,
    /// Catch-up discount per UL level below World UL.
    pub catchup_per_level: f64,
    /// Career dose limit (astronaut leaves the ready pool above this).
    pub career_dose_limit: f64,
    /// Training duration (days) to make an astronaut Ready.
    pub astronaut_train_days: f64,
    /// Effective-UL penalty when a domain loses its last specialist (tacit knowledge).
    pub tacit_per_head: f64,
    /// Per-step dead-end probability for non-guaranteed candidate paths.
    pub dead_end_rate: f64,
    /// Base insight pressure required to trigger a breakthrough (seeded ± variation).
    pub breakthrough_threshold: f64,
    /// Provenance.
    pub source: String,
}

/// A personnel trait's modifiers (multiplicative unless noted).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitDef {
    /// Trait id.
    pub id: String,
    /// Multiplier on low-TRL (1–5) program progress.
    pub low_trl_mult: f64,
    /// Multiplier on qualification (6–9) program progress.
    pub qual_mult: f64,
    /// Multiplier on breakthrough insight accrual.
    pub breakthrough_mult: f64,
    /// Multiplier on overrun variance.
    pub overrun_mult: f64,
    /// Additive reliability bonus.
    pub reliability_bonus: f64,
    /// Provenance.
    pub source: String,
}

// ----------------------------------------------------------------- loaded data

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DomainsFile {
    domains: Vec<DomainDef>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TechFile {
    nodes: Vec<TechNode>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CategoriesFile {
    categories: Vec<CapabilityCategory>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraitsFile {
    traits: Vec<TraitDef>,
}

/// The full, validated research data set.
#[derive(Debug, Clone)]
pub struct ResearchData {
    /// Domains by id.
    pub domains: BTreeMap<DomainId, DomainDef>,
    /// Tech nodes by id.
    pub nodes: BTreeMap<TechId, TechNode>,
    /// Capability categories by id.
    pub categories: BTreeMap<String, CapabilityCategory>,
    /// Tuning params.
    pub params: Params,
    /// Trait definitions by id.
    pub traits: BTreeMap<String, TraitDef>,
    /// BLAKE3 over the canonical concatenation of all data files (saves pin it).
    pub content_hash: [u8; 32],
}

impl ResearchData {
    /// Load and validate from `research_dir` (`data/research`) and `tech_dir`
    /// (`data/tech`).
    pub fn load(research_dir: &Path, tech_dir: &Path) -> Result<ResearchData, String> {
        let read = |p: std::path::PathBuf| {
            std::fs::read_to_string(&p).map_err(|e| format!("reading {}: {e}", p.display()))
        };
        let domains_txt = read(research_dir.join("domains.ron"))?;
        let params_txt = read(research_dir.join("params.ron"))?;
        let traits_txt = read(research_dir.join("traits.ron"))?;
        let tech_txt = read(tech_dir.join("tech-tree.ron"))?;
        let cats_txt = read(tech_dir.join("capability-categories.ron"))?;

        let domains_f: DomainsFile =
            ron::from_str(&domains_txt).map_err(|e| format!("domains.ron: {e}"))?;
        let tech_f: TechFile = ron::from_str(&tech_txt).map_err(|e| format!("tech-tree.ron: {e}"))?;
        let cats_f: CategoriesFile =
            ron::from_str(&cats_txt).map_err(|e| format!("capability-categories.ron: {e}"))?;
        let params: Params = ron::from_str(&params_txt).map_err(|e| format!("params.ron: {e}"))?;
        let traits_f: TraitsFile =
            ron::from_str(&traits_txt).map_err(|e| format!("traits.ron: {e}"))?;

        let mut domains = BTreeMap::new();
        for d in domains_f.domains {
            if d.source.trim().is_empty() {
                return Err(format!("domain '{}' has empty source", d.id.0));
            }
            domains.insert(d.id.clone(), d);
        }
        let mut nodes = BTreeMap::new();
        for n in tech_f.nodes {
            if n.source.trim().is_empty() {
                return Err(format!("tech node '{}' has empty source", n.id.0));
            }
            if !(1..=9).contains(&n.start_trl) {
                return Err(format!("tech '{}': start_trl must be 1..=9", n.id.0));
            }
            nodes.insert(n.id.clone(), n);
        }
        let mut categories = BTreeMap::new();
        for c in cats_f.categories {
            if c.source.trim().is_empty() {
                return Err(format!("category '{}' has empty source", c.id));
            }
            if c.paths.len() < 2 {
                return Err(format!(
                    "capability category '{}' needs ≥2 candidate paths (design L244)",
                    c.id
                ));
            }
            categories.insert(c.id.clone(), c);
        }
        if params.source.trim().is_empty() {
            return Err("params.ron: empty source".into());
        }
        let mut traits = BTreeMap::new();
        for t in traits_f.traits {
            if t.source.trim().is_empty() {
                return Err(format!("trait '{}' has empty source", t.id));
            }
            traits.insert(t.id.clone(), t);
        }

        // --- Referential integrity (FR-RESP-901) ----------------------------
        for d in domains.values() {
            for (s, _) in &d.synergy {
                if !domains.contains_key(s) {
                    return Err(format!("domain '{}' synergy → unknown domain '{}'", d.id.0, s.0));
                }
            }
        }
        for n in nodes.values() {
            for (dom, _) in &n.ul_floors {
                if !domains.contains_key(dom) {
                    return Err(format!("tech '{}' UL floor → unknown domain '{}'", n.id.0, dom.0));
                }
            }
            for p in &n.tech_prereqs {
                if !nodes.contains_key(&p.tech) {
                    return Err(format!("tech '{}' prereq → unknown tech '{}'", n.id.0, p.tech.0));
                }
            }
            if let Some(d) = &n.derivative_of
                && !nodes.contains_key(d)
            {
                return Err(format!("tech '{}' derivative_of unknown tech '{}'", n.id.0, d.0));
            }
            if !categories.contains_key(&n.capability_category) {
                return Err(format!(
                    "tech '{}' capability_category '{}' not in the category map",
                    n.id.0, n.capability_category
                ));
            }
            // Principle IX: no combat/weapons capability categories.
            let cat = n.capability_category.to_lowercase();
            if cat.contains("weapon") || cat.contains("combat") || cat.contains("orion") {
                return Err(format!(
                    "tech '{}' has a forbidden combat/weapons category '{}' (Principle IX)",
                    n.id.0, n.capability_category
                ));
            }
        }
        for c in categories.values() {
            for t in &c.paths {
                if !nodes.contains_key(t) {
                    return Err(format!("category '{}' path → unknown tech '{}'", c.id, t.0));
                }
            }
        }

        let canonical = format!(
            "{}\0{}\0{}\0{}\0{}",
            normalize(&domains_txt),
            normalize(&tech_txt),
            normalize(&cats_txt),
            normalize(&params_txt),
            normalize(&traits_txt)
        );
        let content_hash = *blake3::hash(canonical.as_bytes()).as_bytes();

        Ok(ResearchData {
            domains,
            nodes,
            categories,
            params,
            traits,
            content_hash,
        })
    }
}
