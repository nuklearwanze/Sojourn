//! Per-game seeding (FR-RESP-301, R6, clarified Q2:A): dead ends within TRL bands
//! and breakthrough thresholds, drawn from the `research/seed` stream at world
//! creation. The dead-end seeding is **constructive** — for every capability
//! category it keeps one candidate path fully viable, so no seed can brick a
//! category. A CI sweep additionally verifies the invariant.

use crate::ids::{DomainId, TechId};
use crate::tree::ResearchData;
use rand_core::RngCore;
use std::collections::BTreeMap;

/// `(tech, target-TRL band) → is a dead end`.
pub type DeadEnds = BTreeMap<(TechId, u8), bool>;
/// Per-domain breakthrough insight threshold.
pub type Thresholds = BTreeMap<DomainId, f64>;

fn uniform<R: RngCore>(rng: &mut R) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

/// Seed dead ends (constructive) and breakthrough thresholds. Deterministic for a
/// fixed stream; iteration order is fixed (sorted category/tech/domain keys).
pub fn seed<R: RngCore>(data: &ResearchData, rng: &mut R) -> (DeadEnds, Thresholds) {
    let mut dead_ends: DeadEnds = BTreeMap::new();

    // Categories are a BTreeMap (already ordered). For each, keep one path fully
    // viable; the others may have seeded dead ends per TRL step.
    for cat in data.categories.values() {
        let mut paths = cat.paths.clone();
        paths.sort();
        if paths.is_empty() {
            continue;
        }
        let keep = (uniform(rng) * paths.len() as f64) as usize % paths.len();
        for (i, tech) in paths.iter().enumerate() {
            if i == keep {
                continue; // the guaranteed-viable path — never closed
            }
            let Some(node) = data.nodes.get(tech) else {
                continue;
            };
            for step in &node.trl_steps {
                if uniform(rng) < data.params.dead_end_rate {
                    dead_ends.insert((tech.clone(), step.trl), true);
                }
            }
        }
    }

    // Breakthrough thresholds per domain (insight must cross to trigger).
    let mut thresholds: Thresholds = BTreeMap::new();
    for dom in data.domains.keys() {
        let base = data.params.breakthrough_threshold;
        let jitter = 0.5 + uniform(rng); // 0.5..1.5×
        thresholds.insert(dom.clone(), base * jitter);
    }

    (dead_ends, thresholds)
}

/// Is `(tech, target_trl)` a seeded dead end?
pub fn is_dead_end(dead_ends: &DeadEnds, tech: &TechId, target_trl: u8) -> bool {
    dead_ends
        .get(&(tech.clone(), target_trl))
        .copied()
        .unwrap_or(false)
}

/// Verify the constructive guarantee for a given seed: every capability category
/// retains ≥1 fully-viable path (no dead end in any of its TRL steps). Used by the
/// reachability sweep (FR-RESP-901).
pub fn every_category_reachable(data: &ResearchData, dead_ends: &DeadEnds) -> bool {
    data.categories.values().all(|cat| {
        cat.paths.iter().any(|tech| {
            data.nodes.get(tech).is_some_and(|node| {
                node.trl_steps
                    .iter()
                    .all(|s| !is_dead_end(dead_ends, tech, s.trl))
            })
        })
    })
}
