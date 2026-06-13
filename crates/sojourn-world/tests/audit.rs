//! US2 — the standing truth-leak audit (T023, SC-002). Runs over the **public**
//! query surface only (no `privileged` feature): the snapshot is structurally
//! truth-free, so every faction-facing query returns belief (the honest prior
//! before any survey), never the seeded ground truth.

mod common;
use common::*;
use sojourn_astro::BodyId;
use sojourn_world::WorldCommand;
use sojourn_world::belief::{FactionId, ObsClass, Property};

#[test]
fn unsurveyed_queries_return_the_prior_never_truth() {
    let (core, m) = core_with_world(7);
    let s = snap(&core, &m);
    let f = FactionId(0);
    let mut checked = 0;
    // Sweep every site on every body that carries sites, every property.
    for body in [BodyId(100), BodyId(4), BodyId(1002), BodyId(10)] {
        for sv in s.sites_on(body) {
            for p in &sv.properties {
                let got = s.believed_site(f, &sv.id, *p).expect("belief exists");
                let cert = s.certainty_site(f, &sv.id, *p).expect("certainty");
                // Unsurveyed ⇒ exactly the decoded prior, independent of seeded truth.
                let prior = m.priors.decode(*p, m.priors.prior_for(*p));
                assert!(
                    (got.value - prior.value).abs() < 1e-9
                        && (got.uncertainty - prior.uncertainty).abs() < 1e-9,
                    "{}.{:?} returns the prior, not truth",
                    sv.id,
                    p
                );
                assert!(cert.abs() < 1e-9, "unsurveyed ⇒ zero certainty");
                checked += 1;
            }
        }
    }
    assert!(
        checked >= 20,
        "audited the whole site/property surface ({checked})"
    );
}

#[test]
fn surveying_refines_only_the_acting_faction() {
    let (mut core, m) = core_with_world(7);
    advance(&mut core, 10); // so belief deltas land at a tick > 0
    for _ in 0..25 {
        world(
            &mut core,
            WorldCommand::Observe {
                faction: 0,
                site: "luna-shackleton".into(),
                property: Some(Property::Grade),
                class: ObsClass::SampleGrade,
                quality: 0.9,
            },
        );
    }
    let s = snap(&core, &m);
    let c0 = s
        .certainty_site(FactionId(0), "luna-shackleton", Property::Grade)
        .unwrap();
    let c1 = s
        .certainty_site(FactionId(1), "luna-shackleton", Property::Grade)
        .unwrap();
    assert!(c0 > 0.5, "faction 0 surveyed (certainty {c0:.3})");
    assert!(
        c1.abs() < 1e-9,
        "faction 1's belief untouched (per-faction)"
    );
    assert!(
        !s.belief_delta_since(FactionId(0), 0).is_empty(),
        "faction 0 has belief deltas"
    );
    assert!(
        s.belief_delta_since(FactionId(1), 0).is_empty(),
        "faction 1 has none"
    );
}

#[test]
fn truth_leak_audit_holds_after_partial_survey() {
    // Surveying one property must not leak any *other* unsurveyed property's truth.
    let (mut core, m) = core_with_world(3);
    for _ in 0..10 {
        world(
            &mut core,
            WorldCommand::Observe {
                faction: 0,
                site: "ares-jezero".into(),
                property: Some(Property::Grade),
                class: ObsClass::InSitu,
                quality: 0.7,
            },
        );
    }
    let s = snap(&core, &m);
    // Grade moved; Slope (unsurveyed) is still exactly the prior.
    assert!(
        s.certainty_site(FactionId(0), "ares-jezero", Property::Grade)
            .unwrap()
            > 0.0
    );
    let slope = s
        .believed_site(FactionId(0), "ares-jezero", Property::Slope)
        .unwrap();
    let prior = m
        .priors
        .decode(Property::Slope, m.priors.prior_for(Property::Slope));
    assert!(
        (slope.value - prior.value).abs() < 1e-9,
        "unsurveyed property stays at prior"
    );
}
