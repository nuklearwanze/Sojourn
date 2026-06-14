//! The honest astrobiology model (R4, FR-PEA-301…308). The per-game ground truth is
//! drawn once at `InitWorld` (in `module.rs`) and is **never query-exposed**. Each
//! faction holds a **log-odds** posterior updated by staged evidence whose strength is
//! generated **conditioned on the ground truth** — so a negative world can emit only
//! capped pre-conclusive false hints (and a strong negative at SampleReturn), making
//! a conclusive-positive **structurally impossible** for negative truth. The community
//! consensus is a **prestige-weighted aggregate**; conclusive needs the band
//! (≥0.9 / ≤0.1) **and** a SampleReturn-tier item.

use crate::faction::Resolution;
use crate::inputs::EvidenceStage;
use crate::params::{AstroParams, StageLr};

/// The logistic (sigmoid): log-odds → probability.
pub fn logistic(x: f64) -> f64 {
    1.0 / (1.0 + libm::exp(-x))
}

/// The per-stage positive-evidence strength.
pub fn stage_strength(stage: EvidenceStage, lr: &StageLr) -> f64 {
    match stage {
        EvidenceStage::OrbitalHint => lr.orbital_hint,
        EvidenceStage::InSitu => lr.in_situ,
        EvidenceStage::Microscopy => lr.microscopy,
        EvidenceStage::SampleReturn => lr.sample_return,
    }
}

/// The log-odds delta from one evidence item, **conditioned on the ground truth**.
/// Positive worlds push the posterior up (strongly at SampleReturn); negative worlds
/// yield only a weak pre-conclusive false hint and a strong negative at SampleReturn.
pub fn evidence_delta(
    stage: EvidenceStage,
    quality: f64,
    ground_truth: bool,
    p: &AstroParams,
) -> f64 {
    let strength = stage_strength(stage, &p.stage_lr) * quality.clamp(0.0, 1.0);
    let conclusive_stage = stage == EvidenceStage::SampleReturn;
    if ground_truth {
        if conclusive_stage {
            strength // full conclusive-grade positive
        } else {
            strength * (1.0 - 0.5 * p.abiotic_competitor_strength) // abiotic-attenuated
        }
    } else if conclusive_stage {
        -1.5 * strength // strong exclusion
    } else {
        0.3 * strength * (1.0 - p.abiotic_competitor_strength) // weak false hint
    }
}

/// A faction's probability from its log-odds, applying the **false-hint cap** for a
/// negative-ground-truth world before any SampleReturn item (the honesty invariant).
pub fn faction_prob(
    log_odds: f64,
    ground_truth: bool,
    has_sample_return: bool,
    p: &AstroParams,
) -> f64 {
    let prob = logistic(log_odds);
    if !ground_truth && !has_sample_return {
        prob.min(p.false_hint_cap)
    } else {
        prob
    }
}

/// The prestige-weighted community consensus over `(prob, weight)` pairs.
pub fn weighted_consensus(items: &[(f64, f64)]) -> f64 {
    let total: f64 = items.iter().map(|(_, w)| w).sum();
    if total <= 0.0 {
        return 0.5;
    }
    items.iter().map(|(p, w)| p * w).sum::<f64>() / total
}

/// True iff the per-faction probabilities spread wide enough to count as a public
/// scientific disagreement.
pub fn disagreement(probs: &[f64], width: f64) -> bool {
    if probs.len() < 2 {
        return false;
    }
    let max = probs.iter().cloned().fold(f64::MIN, f64::max);
    let min = probs.iter().cloned().fold(f64::MAX, f64::min);
    max - min > width
}

/// The conclusive resolution, if any: the consensus must cross the band **and** a
/// SampleReturn-tier item must exist.
pub fn is_conclusive(
    consensus: f64,
    has_sample_return: bool,
    p: &AstroParams,
) -> Option<Resolution> {
    if p.sample_return_required && !has_sample_return {
        return None;
    }
    if consensus >= p.band_positive {
        Some(Resolution::Positive)
    } else if consensus <= p.band_negative {
        Some(Resolution::Negative)
    } else {
        None
    }
}
