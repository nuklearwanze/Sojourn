//! US1 — the race for historic firsts (world-first > faction-first + prestige + tiebreak).

mod common;
use common::*;
use sojourn_polity::ids::{FactionId, MilestoneId};
use sojourn_polity::{Achievement, AchievementFacts, PolityCommand};

fn facts(reusable: bool, crewed: bool, body: Option<u32>, tags: &[&str]) -> AchievementFacts {
    AchievementFacts {
        body,
        crewed,
        reusable,
        propellant_sold_kg: 0.0,
        isru_origin: false,
        tags: tags.iter().map(|s| s.to_string()).collect(),
    }
}

fn achieve(e: &mut PolityH, faction: u32, milestone: u32, f: AchievementFacts) {
    e.cmd(PolityCommand::RecordAchievement(Achievement {
        faction: FactionId(faction),
        milestone: MilestoneId(milestone),
        facts: f,
    }));
}

#[test]
fn world_first_then_faction_first_with_prestige() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false), faction(1, false)], vec![], vec![]);

    // Milestone 1 = "first fully-reusable orbital flight" (Reusable + Tag("orbital")).
    achieve(&mut e, 0, 1, facts(true, false, None, &["orbital"]));
    let s = e.snap();
    let m = s.milestone(MilestoneId(1)).unwrap();
    assert_eq!(
        m.world_first.map(|(f, _)| f),
        Some(0),
        "faction 0 holds the world-first"
    );
    assert!(
        (s.prestige(FactionId(0)).unwrap() - m.weight).abs() < 1e-9,
        "full weight for a world-first"
    );

    // Faction 1 achieves the same later → faction-first only.
    e.advance(DAY);
    achieve(&mut e, 1, 1, facts(true, false, None, &["orbital"]));
    let s = e.snap();
    let m = s.milestone(MilestoneId(1)).unwrap();
    assert_eq!(
        m.world_first.map(|(f, _)| f),
        Some(0),
        "world-first stays with the first claimant"
    );
    assert!(
        m.faction_firsts.contains(&1),
        "faction 1 gets the faction-first"
    );
    let def_frac = 0.3; // milestones.ron id 1
    assert!(
        (s.prestige(FactionId(1)).unwrap() - m.weight * def_frac).abs() < 1e-6,
        "faction-first earns the lesser fraction"
    );
}

#[test]
fn unmet_conditions_award_nothing() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false)], vec![], vec![]);
    // Milestone 1 needs Reusable — claim it without reusability.
    achieve(&mut e, 0, 1, facts(false, false, None, &["orbital"]));
    assert!(
        e.snap()
            .milestone(MilestoneId(1))
            .unwrap()
            .world_first
            .is_none(),
        "unmet conditions ⇒ no award"
    );
}

#[test]
fn same_tick_tiebreak_goes_to_highest_prestige() {
    let mut e = PolityH::new();
    e.init(vec![faction(0, false), faction(1, false)], vec![], vec![]);
    // Give faction 1 a head start in prestige via milestone 2 (commercial LEO station).
    achieve(
        &mut e,
        1,
        2,
        facts(false, false, None, &["commercial-leo-station"]),
    );
    assert!(e.snap().prestige(FactionId(1)).unwrap() > 0.0);

    // Now both contest milestone 5 (cryo prop transfer) on the SAME tick (no advance).
    achieve(
        &mut e,
        0,
        5,
        facts(false, false, None, &["cryo-prop-transfer"]),
    );
    achieve(
        &mut e,
        1,
        5,
        facts(false, false, None, &["cryo-prop-transfer"]),
    );
    let s = e.snap();
    let m = s.milestone(MilestoneId(5)).unwrap();
    assert_eq!(
        m.world_first.map(|(f, _)| f),
        Some(1),
        "the higher-prestige faction wins the same-tick world-first"
    );
    assert!(
        m.faction_firsts.contains(&0),
        "the displaced claimant is recorded as faction-first"
    );
}
