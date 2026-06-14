//! US7 — the AI world (claims firsts + advances the tide, within the plausibility envelope).

mod common;
use common::*;
use sojourn_polity::Difficulty;
use sojourn_polity::ids::FactionId;

#[test]
fn ai_factions_claim_firsts_within_the_plausibility_envelope() {
    let mut e = PolityH::new();
    // Faction 1 is AI; give it a mature tide so it pursues milestones.
    e.init(vec![faction(0, false), faction(1, true)], vec![], vec![]);
    e.set_tide(&[(0, 0.5), (1, 1.0)], 0.5, Difficulty::default());
    e.advance(5_000 * DAY);
    let s = e.snap();

    let ai_world_firsts = s
        .ledger()
        .iter()
        .filter(|m| m.world_first.map(|(f, _)| f == 1).unwrap_or(false))
        .count();
    assert!(
        ai_world_firsts > 0,
        "the AI world claims world-firsts (the race)"
    );
    assert!(
        s.ai_capability(FactionId(1)) <= module().params.ai.envelope_cap + 1e-9,
        "AI capability never exceeds the plausibility envelope"
    );
    assert!(
        s.science_tide() > 0.5,
        "the AI world advances the global science tide"
    );
}

#[test]
fn difficulty_tunes_competence_but_never_the_envelope() {
    let p = module().params;
    use sojourn_polity::ai::capability;
    let normal = capability(1.0, 1.0, &p.ai);
    let cranked = capability(1.0, 100.0, &p.ai); // a huge difficulty competence multiplier
    assert!(cranked >= normal, "difficulty raises competence");
    assert!(
        cranked <= p.ai.envelope_cap + 1e-9,
        "but never past the plausibility envelope ({cranked})"
    );
}
