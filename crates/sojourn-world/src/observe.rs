//! Observation handling (FR-WORLD-304): applies an `Observe` command to a
//! faction's belief. Validation is **trust-the-caller** — structural only (target
//! exists, class/quality valid, property sensible); entitlement is a later
//! mission-slice concern. Each sensed property draws seeded measurement noise and
//! applies the Gaussian precision-add refinement, then logs the change and
//! reports any survey milestone crossed.

use crate::belief::{
    BeliefChange, Estimate, FactionId, ObsClass, Priors, Property, Target, refine, standard_normal,
};
use crate::sites::Sites;
use crate::truth::GroundTruth;
use rand_core::RngCore;
use sojourn_core::CommandOutcome;
use std::collections::BTreeMap;

/// The belief map type (per `(faction, target, property)`).
pub type BeliefMap = BTreeMap<(FactionId, Target, Property), Estimate>;

/// Result of applying an observation: the command outcome and the properties
/// whose belief crossed a survey milestone (near the class floor).
pub struct ObserveResult {
    /// Journalled outcome.
    pub outcome: CommandOutcome,
    /// Properties that crossed a milestone (for `survey-milestone` events).
    pub milestones: Vec<Property>,
}

/// Apply an observation. `site_key` is the site's data id (resolved here).
#[allow(clippy::too_many_arguments)]
pub fn apply_observe<R: RngCore>(
    beliefs: &mut BeliefMap,
    truth: &GroundTruth,
    change_log: &mut Vec<BeliefChange>,
    priors: &Priors,
    sites: &Sites,
    rng: &mut R,
    tick: u64,
    faction: FactionId,
    site_key: &str,
    property: Option<Property>,
    class: ObsClass,
    quality: f64,
) -> ObserveResult {
    let reject = |r: String| ObserveResult {
        outcome: CommandOutcome::Rejected(r),
        milestones: Vec::new(),
    };

    // --- Structural validation (trust-the-caller) ----------------------------
    if !(0.0..=1.0).contains(&quality) {
        return reject(format!("observation quality {quality} must be in [0,1]"));
    }
    let Some(sid) = sites.resolve(site_key) else {
        return reject(format!("unknown site '{site_key}'"));
    };
    let def = sites.def(sid).expect("resolved site");

    // Which properties to observe.
    let props: Vec<Property> = match property {
        Some(p) => {
            if !def.properties.iter().any(|pt| pt.property == p) {
                return reject(format!("site '{site_key}' has no property {p:?}"));
            }
            if !priors.senses(p, class) {
                return reject(format!("class {class:?} cannot sense {p:?}"));
            }
            vec![p]
        }
        None => def
            .properties
            .iter()
            .map(|pt| pt.property)
            .filter(|p| priors.senses(*p, class))
            .collect(),
    };
    if props.is_empty() {
        return reject(format!("class {class:?} senses no property of '{site_key}'"));
    }

    // --- Refinement ----------------------------------------------------------
    let target = Target::Site(sid);
    let mut milestones = Vec::new();
    for p in props {
        let Some(truth_val) = truth.value(sid, p) else {
            continue; // site declares the property but it was not seeded — skip
        };
        let sigma = priors.measurement_sigma(p, class, quality).expect("senses");
        let floor_v = priors.floor_var(p, class);
        let t = priors.encode_truth(p, truth_val);
        let m = t + sigma * standard_normal(rng);

        let prior_est = beliefs
            .get(&(faction, target, p))
            .copied()
            .unwrap_or_else(|| priors.prior_for(p));
        let post = refine(prior_est, m, sigma, floor_v, tick);

        // Milestone: belief crossed into "near the floor" this observation.
        let near = 2.0 * floor_v;
        if prior_est.var > near && post.var <= near {
            milestones.push(p);
        }

        beliefs.insert((faction, target, p), post);
        change_log.push(BeliefChange {
            tick,
            faction,
            target,
            property: p,
        });
    }

    ObserveResult {
        outcome: CommandOutcome::Applied,
        milestones,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::belief::Priors;
    use crate::truth::{AstrobiologyCandidates, GroundTruth};
    use rand_chacha::ChaCha12Rng;
    use rand_core::SeedableRng;

    const PRIORS: &str = r#"(
        remote: (floor: 0.30, sigma0: 0.90),
        insitu: (floor: 0.12, sigma0: 0.50),
        samplegrade: (floor: 0.03, sigma0: 0.20),
        properties: [
            (property: Grade, prior_mean: -2.3, prior_var: 1.0, log_space: true, scale: 1.0, classes: [RemoteSensing, InSitu, SampleGrade]),
        ],
        source: "test",
    )"#;
    const SITES: &str = r#"(
        sites: [
            (id: "s1", body: 0, placement: Surface(lat: 0.0, lon: 0.0), pp_category: II,
             properties: [ (property: Grade, dist: Fixed(0.1)) ], resource: "water-ice", source: "test"),
        ],
    )"#;

    fn setup() -> (Priors, Sites, GroundTruth) {
        let priors = Priors::from_source(PRIORS).unwrap();
        let sites = Sites::from_source(SITES).unwrap();
        let mut seed = ChaCha12Rng::seed_from_u64(1);
        let truth = GroundTruth::seed(&sites, &AstrobiologyCandidates::default(), &mut seed);
        (priors, sites, truth)
    }

    fn run(seed: u64, class: ObsClass, quality: f64, n: u64) -> (BeliefMap, Vec<BeliefChange>) {
        let (priors, sites, truth) = setup();
        let mut beliefs = BeliefMap::new();
        let mut log = Vec::new();
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        for k in 0..n {
            let res = apply_observe(
                &mut beliefs, &truth, &mut log, &priors, &sites, &mut rng, k,
                FactionId(0), "s1", Some(Property::Grade), class, quality,
            );
            assert!(matches!(res.outcome, CommandOutcome::Applied));
        }
        (beliefs, log)
    }

    #[test]
    fn refinement_is_monotonic_and_converges_to_truth() {
        let (priors, sites, truth) = setup();
        let sid = sites.resolve("s1").unwrap();
        let key = (FactionId(0), Target::Site(sid), Property::Grade);
        let mut beliefs = BeliefMap::new();
        let mut log = Vec::new();
        let mut rng = ChaCha12Rng::seed_from_u64(42);
        let mut last = f64::INFINITY;
        for k in 0..80 {
            apply_observe(
                &mut beliefs, &truth, &mut log, &priors, &sites, &mut rng, k,
                FactionId(0), "s1", Some(Property::Grade), ObsClass::SampleGrade, 0.9,
            );
            let v = beliefs[&key].var;
            assert!(v <= last + 1e-12, "variance monotonically non-increasing");
            last = v;
        }
        let dec = priors.decode(Property::Grade, beliefs[&key]);
        assert!((dec.value - 0.1).abs() < 0.03, "estimate converges toward truth 0.1 (got {})", dec.value);
        assert!(last >= 0.03_f64.powi(2) - 1e-9, "never below the sample-grade floor");
    }

    #[test]
    fn poor_class_cannot_reach_sample_grade_certainty() {
        let sid;
        let remote_var = {
            let (priors, sites, _t) = setup();
            sid = sites.resolve("s1").unwrap();
            let _ = priors;
            run(7, ObsClass::RemoteSensing, 1.0, 200).0[&(FactionId(0), Target::Site(sid), Property::Grade)].var
        };
        // Remote floor 0.30 ⇒ floor var 0.09; sample-grade floor var 9e-4.
        assert!(remote_var >= 0.30_f64.powi(2) - 1e-9, "remote sensing stuck at its floor");
        assert!(remote_var > 0.03_f64.powi(2), "cannot reach sample-grade certainty");
    }

    #[test]
    fn deterministic_per_seed() {
        let (a, la) = run(123, ObsClass::InSitu, 0.5, 30);
        let (b, lb) = run(123, ObsClass::InSitu, 0.5, 30);
        assert_eq!(a, b, "identical seed ⇒ identical belief");
        assert_eq!(la, lb, "identical change log");
    }

    #[test]
    fn invalid_commands_rejected_deterministically() {
        let (priors, sites, truth) = setup();
        let mut beliefs = BeliefMap::new();
        let mut log = Vec::new();
        let mut rng = ChaCha12Rng::seed_from_u64(1);
        // Unknown site.
        let r1 = apply_observe(&mut beliefs, &truth, &mut log, &priors, &sites, &mut rng, 0,
            FactionId(0), "nope", Some(Property::Grade), ObsClass::InSitu, 0.5);
        assert!(matches!(r1.outcome, CommandOutcome::Rejected(_)));
        // Out-of-range quality.
        let r2 = apply_observe(&mut beliefs, &truth, &mut log, &priors, &sites, &mut rng, 0,
            FactionId(0), "s1", Some(Property::Grade), ObsClass::InSitu, 2.0);
        assert!(matches!(r2.outcome, CommandOutcome::Rejected(_)));
        // Property the site does not expose.
        let r3 = apply_observe(&mut beliefs, &truth, &mut log, &priors, &sites, &mut rng, 0,
            FactionId(0), "s1", Some(Property::Slope), ObsClass::InSitu, 0.5);
        assert!(matches!(r3.outcome, CommandOutcome::Rejected(_)));
        assert!(beliefs.is_empty(), "no state change on rejection");
    }

    #[test]
    fn two_factions_hold_independent_beliefs() {
        let (priors, sites, truth) = setup();
        let sid = sites.resolve("s1").unwrap();
        let mut beliefs = BeliefMap::new();
        let mut log = Vec::new();
        let mut rng = ChaCha12Rng::seed_from_u64(5);
        for k in 0..10 {
            apply_observe(&mut beliefs, &truth, &mut log, &priors, &sites, &mut rng, k,
                FactionId(0), "s1", Some(Property::Grade), ObsClass::SampleGrade, 0.9);
        }
        // Faction 0 surveyed; faction 1 never did.
        assert!(beliefs.contains_key(&(FactionId(0), Target::Site(sid), Property::Grade)));
        assert!(!beliefs.contains_key(&(FactionId(1), Target::Site(sid), Property::Grade)));
    }
}
