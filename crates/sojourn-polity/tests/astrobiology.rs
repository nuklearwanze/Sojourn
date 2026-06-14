//! US3 — the astrobiology question, answered honestly (seeded ground truth, staged
//! evidence, per-faction beliefs → prestige-weighted consensus, the honesty invariant).

mod common;
use common::*;
use sojourn_polity::faction::Resolution;
use sojourn_polity::ids::{CandidateId, FactionId};
use sojourn_polity::{EvidenceInput, EvidenceStage, PolityCommand};

fn collect(e: &mut PolityH, faction: u32, cand: CandidateId, stage: EvidenceStage) {
    e.cmd(PolityCommand::CollectEvidence(EvidenceInput {
        faction: FactionId(faction),
        candidate: cand,
        stage,
        quality: 1.0,
    }));
}

fn full_chain(e: &mut PolityH, faction: u32, cand: CandidateId) {
    for stage in [
        EvidenceStage::OrbitalHint,
        EvidenceStage::InSitu,
        EvidenceStage::Microscopy,
        EvidenceStage::SampleReturn,
    ] {
        collect(e, faction, cand, stage);
    }
}

#[test]
fn positive_world_resolves_conclusively_positive() {
    let mut e = PolityH::new();
    // prior 1.0 ⇒ ground truth is positive this game (deterministic forcing).
    e.init(
        vec![faction(0, false), faction(1, false)],
        vec![candidate(121, 1.0)],
        vec![],
    );
    let c = cid(121);

    // Pre-conclusive evidence raises belief but cannot conclude (no SampleReturn yet).
    for f in [0, 1] {
        collect(&mut e, f, c, EvidenceStage::OrbitalHint);
        collect(&mut e, f, c, EvidenceStage::Microscopy);
    }
    e.advance(DAY);
    assert!(
        e.snap().posterior(FactionId(0), c).unwrap() > 0.5,
        "evidence raises the posterior"
    );
    assert!(
        !e.snap().is_conclusive(c),
        "no conclusion without sample-return-tier evidence"
    );

    // Sample return ⇒ the consensus crosses the positive band ⇒ conclusive positive.
    for f in [0, 1] {
        collect(&mut e, f, c, EvidenceStage::SampleReturn);
    }
    e.advance(DAY);
    let s = e.snap();
    assert!(
        s.consensus(c).unwrap() >= 0.9,
        "consensus crosses the positive band ({:?})",
        s.consensus(c)
    );
    assert_eq!(s.conclusive(c), Some(Resolution::Positive));
}

#[test]
fn negative_world_is_never_conclusive_positive() {
    let mut e = PolityH::new();
    // prior 0.0 ⇒ ground truth is negative this game.
    e.init(
        vec![faction(0, false), faction(1, false)],
        vec![candidate(111, 0.0)],
        vec![],
    );
    let c = cid(111);

    // Even a pile of pre-conclusive hints stays capped below the positive band.
    for f in [0, 1] {
        collect(&mut e, f, c, EvidenceStage::OrbitalHint);
        collect(&mut e, f, c, EvidenceStage::Microscopy);
    }
    e.advance(DAY);
    let s = e.snap();
    assert!(
        s.posterior(FactionId(0), c).unwrap() <= 0.7 + 1e-9,
        "a negative world's pre-conclusive belief is capped (false-hint cap)"
    );
    assert!(
        s.conclusive(c).is_none(),
        "a false hint is never a conclusion"
    );

    // Sample return drives the consensus down to a conclusive NEGATIVE — never positive.
    for f in [0, 1] {
        full_chain(&mut e, f, c);
    }
    e.advance(DAY);
    let s = e.snap();
    assert_eq!(
        s.conclusive(c),
        Some(Resolution::Negative),
        "the honest outcome is a conclusive negative"
    );
    assert_ne!(
        s.conclusive(c),
        Some(Resolution::Positive),
        "NEVER a false 'life confirmed'"
    );
    assert!(
        s.consensus(c).unwrap() <= 0.1,
        "consensus crosses the negative band"
    );
}

#[test]
fn ground_truth_is_faithful_to_the_prior_across_seeds() {
    let n = 160u64;
    let mut positives = 0u32;
    for seed in 0..n {
        let mut e = PolityH::with_seed(seed);
        e.init(vec![faction(0, false)], vec![candidate(121, 0.5)], vec![]);
        let c = cid(121);
        full_chain(&mut e, 0, c);
        e.advance(DAY);
        if e.snap().conclusive(c) == Some(Resolution::Positive) {
            positives += 1;
        }
    }
    let frac = positives as f64 / n as f64;
    assert!(
        (frac - 0.5).abs() < 0.13,
        "ground-truth positive fraction ≈ the 0.5 prior (got {frac})"
    );
}

#[test]
fn factions_can_publicly_disagree() {
    let mut e = PolityH::new();
    e.init(
        vec![faction(0, false), faction(1, false)],
        vec![candidate(121, 1.0)],
        vec![],
    );
    let c = cid(121);
    // Only faction 0 has done the science.
    full_chain(&mut e, 0, c);
    e.advance(DAY);
    let s = e.snap();
    assert!(
        s.posterior(FactionId(0), c).unwrap() > 0.9,
        "the faction with the data believes"
    );
    assert!(
        (s.posterior(FactionId(1), c).unwrap() - 0.5).abs() < 1e-9,
        "the faction without data is at 'unknown'"
    );
    assert!(s.disagreement(c), "the community publicly disagrees");
}
