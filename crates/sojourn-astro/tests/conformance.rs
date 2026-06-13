//! Kernel conformance for the astro module (T019, FR-ASTRO-106): the FA-01
//! contract gate every module crate must pass.

mod common;
use common::*;

#[test]
fn astro_passes_the_kernel_conformance_suite() {
    let factory = || -> Box<dyn sojourn_core::SimModule> { Box::new(astro_module()) };
    // 30 simulated days exercises cadence + boundary machinery.
    let report =
        sojourn_core::run_suite(&factory, &kernel_data(), 4242, 30 * 86_400).expect("suite runs");
    assert!(report.ok(), "conformance failures: {:?}", report.failed);
}

#[test]
fn spawned_world_double_runs_identically_with_events() {
    // A fuller double-run than the suite's: spawn craft, commit a small plan,
    // cross an SOI — twice, with different stepping — identical fingerprints.
    use sojourn_astro::{AstroCommand, Vec3};
    let run = |chunk: u64| {
        let (mut core, _twin) = core_with(astro_module(), 99);
        let id = spawn_circular(
            &mut core,
            TERRA,
            MU_TERRA,
            7.0e6,
            500.0,
            1500.0,
            "chem-hydrolox",
        );
        astro(
            &mut core,
            AstroCommand::CreateNode {
                craft: id,
                epoch_tick: 4000,
                dv_prn: Vec3::new(100.0, 0.0, 0.0),
            },
        );
        astro(
            &mut core,
            AstroCommand::CommitPlan {
                craft: id,
                nodes: vec![0],
                aim: None,
            },
        );
        let mut t = core.status().tick;
        while t < 20_000 {
            t = (t + chunk).min(20_000);
            advance(&mut core, t);
        }
        core.fingerprint().unwrap()
    };
    assert_eq!(
        run(20_000),
        run(777),
        "stepping pattern cannot affect history"
    );
}
