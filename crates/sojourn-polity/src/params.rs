//! The sourced parameter set (DATA, immutable). Nine RON files under `data/polity/`
//! plus their semantic validation and a blake3 `content_hash` saves pin (the
//! FA-02…08 pattern). No magic numbers live in code (Principle II/V); the
//! astrobiology **priors** stay owned by FA-03 — only the *mechanics* params are here.

use serde::{Deserialize, Serialize};
use std::path::Path;

fn normalize(t: &str) -> String {
    t.replace("\r\n", "\n")
}

// ----- US1: milestone catalogue -----------------------------------------------

/// A machine-checkable award condition over composed `AchievementFacts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    /// The vehicle is fully reusable.
    Reusable,
    /// The mission is crewed.
    Crewed,
    /// The achievement involves a specific body (catalogue id).
    Body(u32),
    /// At least this mass of propellant was sold.
    PropellantSold(f64),
    /// The product is off-Earth (ISRU) produced.
    IsruOrigin,
    /// A free-form capability tag is present.
    Tag(String),
}

/// A catalogued historic first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MilestoneDef {
    /// Stable id.
    pub id: u32,
    /// Era (foothold/cislunar/frontier/endgame).
    pub era: String,
    /// Human-readable description.
    pub description: String,
    /// Prestige/score significance weight.
    pub weight: f64,
    /// Fraction of the weight a faction-first earns (vs a world-first).
    pub faction_first_fraction: f64,
    /// All must hold for the milestone to be awarded.
    pub conditions: Vec<Condition>,
    /// Provenance.
    pub source: String,
}

/// The milestone catalogue (a representative sourced subset at this slice).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MilestoneCatalogue {
    /// The firsts.
    pub milestones: Vec<MilestoneDef>,
    /// Provenance.
    pub source: String,
}

// ----- US2: mood --------------------------------------------------------------

/// Public/political mood coefficients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoodParams {
    /// Mood lift from a world-first.
    pub world_first_delta: f64,
    /// Mood lift from a routine success.
    pub success_delta: f64,
    /// Mood drop from a routine (robotic) failure (negative).
    pub routine_failure_delta: f64,
    /// Mood drop from a loss-of-crew (negative; deeper than a routine failure).
    pub loss_of_crew_delta: f64,
    /// Daily exponential decay rate toward the baseline.
    pub decay_per_day: f64,
    /// Mood lower bound.
    pub mood_min: f64,
    /// Mood upper bound.
    pub mood_max: f64,
    /// Crewed-flight approval freeze window after a loss-of-crew (days).
    pub loc_recovery_days: f64,
    /// Appropriation modifier slope: `factor = 1 + slope × mood`.
    pub appropriation_slope: f64,
    /// Valuation modifier slope: `factor = 1 + slope × mood`.
    pub valuation_slope: f64,
    /// Approval latency at baseline (days).
    pub approval_base_days: f64,
    /// Approval latency sensitivity to mood (days; latency = base − mood × this).
    pub approval_mood_days: f64,
    /// Provenance.
    pub source: String,
}

// ----- US4: events ------------------------------------------------------------

/// An event class / source definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDef {
    /// Class id (matches `data/kernel/event-classes.ron`).
    pub class: String,
    /// Base daily probability (before state factors).
    pub base_rate: f64,
    /// Interrupt (pause) vs log-only.
    pub interrupt: bool,
    /// Provenance.
    pub source: String,
}

/// The event catalogue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventCatalogue {
    /// Event classes.
    pub classes: Vec<EventDef>,
    /// Provenance.
    pub source: String,
}

// ----- US5: policy ------------------------------------------------------------

/// A policy/treaty lever.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLever {
    /// Lever key.
    pub id: String,
    /// Minimum level.
    pub min: f64,
    /// Maximum level.
    pub max: f64,
    /// Starting level.
    pub default: f64,
    /// Seeded drift magnitude per period.
    pub drift_per_period: f64,
    /// Lobby nudge step.
    pub lobby_step: f64,
    /// Level at/above which a required approval is granted.
    pub grant_threshold: f64,
    /// Provenance.
    pub source: String,
}

/// Policy params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyParams {
    /// Levers.
    pub levers: Vec<PolicyLever>,
    /// Provenance.
    pub source: String,
}

// ----- US6: planetary protection ---------------------------------------------

/// A body's COSPAR protection facts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectionBody {
    /// Body id.
    pub body: u32,
    /// COSPAR category I–V.
    pub category: u8,
    /// Special Region?
    pub special_region: bool,
    /// Bioburden limit.
    pub bioburden_limit: f64,
    /// Provenance.
    pub source: String,
}

/// Contamination grading params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminationParams {
    /// Degradation exponent on the bioburden overage ratio.
    pub overage_exponent: f64,
    /// Degradation scale.
    pub overage_scale: f64,
    /// Multiplier for a crash (uncontrolled impact).
    pub crash_factor: f64,
    /// Multiplier for a soft landing.
    pub soft_factor: f64,
    /// Back-contamination penalty (science/reputation) when no chain is in place.
    pub backcontam_penalty: f64,
}

/// Planetary-protection params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectionParams {
    /// Per-body COSPAR facts (defaults bridged from FA-03 at `InitWorld`).
    pub bodies: Vec<ProtectionBody>,
    /// Contamination grading.
    pub contamination: ContaminationParams,
    /// Provenance.
    pub source: String,
}

// ----- US3: astrobiology ------------------------------------------------------

/// How the community consensus aggregates per-faction posteriors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusWeight {
    /// Equal-weight mean.
    Equal,
    /// Prestige/science-output weighted (the clarified rule).
    PrestigeWeighted,
}

/// Per-stage positive-evidence log-odds magnitudes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageLr {
    /// Orbital biosignature hint (weakest).
    pub orbital_hint: f64,
    /// In-situ chemistry.
    pub in_situ: f64,
    /// Microscopy / metabolism.
    pub microscopy: f64,
    /// Sample return (strongest; conclusive-gating).
    pub sample_return: f64,
}

/// Astrobiology evidence/consensus params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstroParams {
    /// Per-stage log-odds magnitudes.
    pub stage_lr: StageLr,
    /// Abiotic-competitor attenuation of pre-conclusive evidence ∈ [0,1].
    pub abiotic_competitor_strength: f64,
    /// Consensus aggregation rule.
    pub consensus_weight: ConsensusWeight,
    /// Positive confidence band (consensus ≥ this ⇒ conclusive-positive).
    pub band_positive: f64,
    /// Negative confidence band (consensus ≤ this ⇒ conclusive-negative).
    pub band_negative: f64,
    /// A SampleReturn-tier item is required for a conclusive resolution.
    pub sample_return_required: bool,
    /// A negative-ground-truth world's consensus cannot exceed this pre-conclusive.
    pub false_hint_cap: f64,
    /// Posterior spread (across factions) above which they "publicly disagree".
    pub disagreement_width: f64,
    /// Provenance.
    pub source: String,
}

// ----- US7: AI world ----------------------------------------------------------

/// AI-world behaviour params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiParams {
    /// Base competence ∈ [0,1].
    pub base_competence: f64,
    /// Plausibility-envelope capability cap.
    pub envelope_cap: f64,
    /// Science-tide advance per AI faction per day.
    pub tide_advance_per_day: f64,
    /// Capability needed before an AI faction pursues/claims a milestone.
    pub claim_threshold: f64,
    /// Provenance.
    pub source: String,
}

// ----- US8: Grand Goals & scoring --------------------------------------------

/// Grand-Goal thresholds + scoring weights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalParams {
    /// Pathfinder: exploration firsts threshold.
    pub pathfinder_firsts: u32,
    /// Homestead: embargo-survival index threshold.
    pub homestead_index: f64,
    /// Prospector: profitable off-Earth tonnage threshold (t).
    pub prospector_tonnage: f64,
    /// Seeker: conclusively-resolved worlds threshold (= 3).
    pub seeker_worlds: u32,
    /// Penalty (fraction) applied to the composite score on a goal change.
    pub change_penalty: f64,
    /// Composite-score weight on prestige.
    pub prestige_weight: f64,
    /// Composite-score weight on milestone total.
    pub milestone_weight: f64,
    /// Composite-score weight on Grand-Goal progress.
    pub goal_weight: f64,
    /// Provenance.
    pub source: String,
}

// ----- cross-cutting ----------------------------------------------------------

/// Cross-cutting config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolityCfg {
    /// Default scoring horizon (years).
    pub horizon_years: u16,
    /// Composite-score normalisation divisor.
    pub score_normalisation: f64,
    /// Provenance.
    pub source: String,
}

/// The full loaded, validated parameter set.
#[derive(Debug, Clone)]
pub struct Params {
    /// Milestone catalogue.
    pub milestones: MilestoneCatalogue,
    /// Mood.
    pub mood: MoodParams,
    /// Events.
    pub events: EventCatalogue,
    /// Policy.
    pub policy: PolicyParams,
    /// Planetary protection.
    pub protection: ProtectionParams,
    /// Astrobiology.
    pub astro: AstroParams,
    /// AI world.
    pub ai: AiParams,
    /// Grand Goals & scoring.
    pub goals: GoalParams,
    /// Cross-cutting config.
    pub cfg: PolityCfg,
    /// Content hash (saves pin it).
    pub content_hash: [u8; 32],
}

impl Params {
    /// Load and validate the parameter set from a directory (`data/polity`).
    pub fn load_dir(dir: &Path) -> Result<Params, String> {
        let read =
            |n: &str| std::fs::read_to_string(dir.join(n)).map_err(|e| format!("reading {n}: {e}"));
        let texts = [
            ("milestones.ron", read("milestones.ron")?),
            ("mood.ron", read("mood.ron")?),
            ("events.ron", read("events.ron")?),
            ("policy.ron", read("policy.ron")?),
            ("protection.ron", read("protection.ron")?),
            ("astrobiology.ron", read("astrobiology.ron")?),
            ("ai.ron", read("ai.ron")?),
            ("goals.ron", read("goals.ron")?),
            ("params.ron", read("params.ron")?),
        ];
        let milestones: MilestoneCatalogue =
            ron::from_str(&texts[0].1).map_err(|e| format!("milestones.ron: {e}"))?;
        let mood: MoodParams = ron::from_str(&texts[1].1).map_err(|e| format!("mood.ron: {e}"))?;
        let events: EventCatalogue =
            ron::from_str(&texts[2].1).map_err(|e| format!("events.ron: {e}"))?;
        let policy: PolicyParams =
            ron::from_str(&texts[3].1).map_err(|e| format!("policy.ron: {e}"))?;
        let protection: ProtectionParams =
            ron::from_str(&texts[4].1).map_err(|e| format!("protection.ron: {e}"))?;
        let astro: AstroParams =
            ron::from_str(&texts[5].1).map_err(|e| format!("astrobiology.ron: {e}"))?;
        let ai: AiParams = ron::from_str(&texts[6].1).map_err(|e| format!("ai.ron: {e}"))?;
        let goals: GoalParams =
            ron::from_str(&texts[7].1).map_err(|e| format!("goals.ron: {e}"))?;
        let cfg: PolityCfg = ron::from_str(&texts[8].1).map_err(|e| format!("params.ron: {e}"))?;

        validate(
            &milestones,
            &mood,
            &events,
            &policy,
            &protection,
            &astro,
            &ai,
            &goals,
            &cfg,
        )?;

        let canonical = texts
            .iter()
            .map(|(_, t)| normalize(t))
            .collect::<Vec<_>>()
            .join("\0");
        let content_hash = *blake3::hash(canonical.as_bytes()).as_bytes();
        Ok(Params {
            milestones,
            mood,
            events,
            policy,
            protection,
            astro,
            ai,
            goals,
            cfg,
            content_hash,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn validate(
    milestones: &MilestoneCatalogue,
    mood: &MoodParams,
    events: &EventCatalogue,
    policy: &PolicyParams,
    protection: &ProtectionParams,
    astro: &AstroParams,
    ai: &AiParams,
    goals: &GoalParams,
    cfg: &PolityCfg,
) -> Result<(), String> {
    // Source presence (every sub-system + every catalogue entry).
    let mut sources: Vec<(&str, &str)> = vec![
        ("milestones", &milestones.source),
        ("mood", &mood.source),
        ("events", &events.source),
        ("policy", &policy.source),
        ("protection", &protection.source),
        ("astrobiology", &astro.source),
        ("ai", &ai.source),
        ("goals", &goals.source),
        ("params", &cfg.source),
    ];
    for m in &milestones.milestones {
        sources.push(("milestone", &m.source));
    }
    for e in &events.classes {
        sources.push(("event", &e.source));
    }
    for l in &policy.levers {
        sources.push(("policy-lever", &l.source));
    }
    for b in &protection.bodies {
        sources.push(("protection-body", &b.source));
    }
    for (n, s) in sources {
        if s.trim().is_empty() {
            return Err(format!("{n}: empty source"));
        }
    }

    // Milestones: unique ids, positive weight, faction-first fraction in (0,1).
    let mut seen = std::collections::BTreeSet::new();
    for m in &milestones.milestones {
        if !seen.insert(m.id) {
            return Err(format!("milestone id {} duplicated", m.id));
        }
        if m.weight <= 0.0 {
            return Err(format!("milestone {} weight must be > 0", m.id));
        }
        if !(m.faction_first_fraction > 0.0 && m.faction_first_fraction < 1.0) {
            return Err(format!(
                "milestone {} faction_first_fraction must be in (0,1)",
                m.id
            ));
        }
    }

    // Mood: loss-of-crew strictly deeper than routine failure; recovery ≥ 2 yr floor.
    if mood.mood_min >= mood.mood_max {
        return Err("mood bounds must satisfy min < max".into());
    }
    if mood.loss_of_crew_delta.abs() <= mood.routine_failure_delta.abs() {
        return Err("loss-of-crew mood delta must be deeper than a routine failure".into());
    }
    if mood.loc_recovery_days < 730.0 {
        return Err(
            "loc_recovery_days must be ≥ the 2-year floor (NASA post-accident pauses)".into(),
        );
    }

    // Events: base rates in [0,1].
    for e in &events.classes {
        if !(0.0..=1.0).contains(&e.base_rate) {
            return Err(format!("event '{}' base_rate must be in [0,1]", e.class));
        }
    }

    // Policy: min ≤ default ≤ max; pp-stringency lever present.
    for l in &policy.levers {
        if !(l.min <= l.default && l.default <= l.max) {
            return Err(format!(
                "policy lever '{}' must satisfy min ≤ default ≤ max",
                l.id
            ));
        }
    }
    if !policy.levers.iter().any(|l| l.id == "pp-stringency") {
        return Err("policy must include the 'pp-stringency' lever (single source for US6)".into());
    }

    // Protection: category 1..=5, positive limit, crash ≥ soft ≥ 1, monotone curve.
    for b in &protection.bodies {
        if !(1..=5).contains(&b.category) {
            return Err(format!("protection body {} category must be I..V", b.body));
        }
        if b.bioburden_limit <= 0.0 {
            return Err(format!(
                "protection body {} bioburden_limit must be > 0",
                b.body
            ));
        }
    }
    let c = &protection.contamination;
    if !(c.crash_factor >= c.soft_factor && c.soft_factor >= 1.0) {
        return Err("contamination: crash_factor ≥ soft_factor ≥ 1 required".into());
    }
    if c.overage_exponent <= 0.0 || c.overage_scale <= 0.0 {
        return Err("contamination: overage curve must be monotone (exponent>0, scale>0)".into());
    }

    // Astrobiology: band ordering, LR ordering, false-hint cap below the positive band.
    if !(0.0 < astro.band_negative
        && astro.band_negative < astro.band_positive
        && astro.band_positive < 1.0)
    {
        return Err("astrobiology: require 0 < band_negative < band_positive < 1".into());
    }
    let lr = &astro.stage_lr;
    if !(lr.orbital_hint < lr.in_situ
        && lr.in_situ < lr.microscopy
        && lr.microscopy < lr.sample_return)
    {
        return Err(
            "astrobiology: stage LRs must increase OrbitalHint<InSitu<Microscopy<SampleReturn"
                .into(),
        );
    }
    if astro.false_hint_cap >= astro.band_positive {
        return Err(
            "astrobiology: false_hint_cap must be < band_positive (honesty invariant)".into(),
        );
    }

    // AI: envelope cap present/positive.
    if ai.envelope_cap <= 0.0 {
        return Err("ai: envelope_cap must be > 0".into());
    }

    // Goals: thresholds positive; Seeker = 3.
    if goals.seeker_worlds != 3 {
        return Err("goals: seeker_worlds must be 3 (OVERVIEW §9)".into());
    }
    if goals.pathfinder_firsts == 0
        || goals.prospector_tonnage <= 0.0
        || goals.homestead_index <= 0.0
    {
        return Err("goals: thresholds must be positive".into());
    }
    Ok(())
}
