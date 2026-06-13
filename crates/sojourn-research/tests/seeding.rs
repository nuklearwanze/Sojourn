//! US3 — the tree is alive: constructive capability-reachability (FR-RESP-301, SC-003)
//! and that dead ends are actually seeded.

mod common;
use common::*;
use rand_chacha::ChaCha12Rng;
use rand_core::SeedableRng;
use sojourn_research::seeding;

#[test]
fn every_capability_category_reachable_across_seeds() {
    let m = research_module();
    for seed in 0..300u64 {
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        let (dead_ends, _thresholds) = seeding::seed(&m.data, &mut rng);
        assert!(
            seeding::every_category_reachable(&m.data, &dead_ends),
            "seed {seed} bricked a capability category (constructive guarantee violated)"
        );
    }
}

#[test]
fn dead_ends_are_actually_seeded() {
    // The mechanic is real — some seeds close some (non-last) paths.
    let m = research_module();
    let mut any_dead = false;
    for seed in 0..50u64 {
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        let (dead_ends, _) = seeding::seed(&m.data, &mut rng);
        if dead_ends.values().any(|&v| v) {
            any_dead = true;
        }
    }
    assert!(any_dead, "dead ends are seeded across some games");
}

#[test]
fn seeding_is_deterministic() {
    let m = research_module();
    let run = |seed: u64| {
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        seeding::seed(&m.data, &mut rng)
    };
    let (a_de, a_th) = run(42);
    let (b_de, b_th) = run(42);
    assert_eq!(a_de, b_de);
    assert_eq!(a_th, b_th);
}
