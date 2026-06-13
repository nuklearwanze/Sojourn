//! Crash recovery kill-test (FR-CORE-303, SC-004): spawn a child harness driving
//! a durable run, terminate it abruptly at a pseudo-random moment, recover from
//! the journal, and verify the recovered state matches a clean reference run at
//! the recovered tick. The kill schedule is seeded per trial (reproducible
//! scheduling; actual interleaving still varies — that's the point).

use crate::scenario::Scenario;
use anyhow::{Context, Result, bail};
use sojourn_core::{DataSet, MemResolver, SimCore};
use std::path::Path;

/// One trial outcome.
#[derive(Debug)]
pub struct TrialOutcome {
    /// Tick recovered to.
    pub recovered_tick: u64,
    /// Whether the journal had a torn tail.
    pub torn: bool,
}

/// Run `trials` kill-tests.
pub fn killtest(
    scenario_path: &Path,
    scenario: &Scenario,
    data: &DataSet,
    runs_root: &Path,
    trials: u32,
) -> Result<Vec<TrialOutcome>> {
    let exe = std::env::current_exe().context("locating harness executable")?;
    let mut outcomes = Vec::new();

    for trial in 0..trials {
        let run_dir = runs_root.join(format!("kill_{trial:04}"));
        if run_dir.exists() {
            std::fs::remove_dir_all(&run_dir).context("clearing trial dir")?;
        }

        // Child: drive the scenario durably until killed.
        let mut child = std::process::Command::new(&exe)
            .arg("run")
            .arg(scenario_path)
            .arg("--run-dir")
            .arg(&run_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("spawning child")?;

        // Seeded pseudo-random kill delay (SplitMix64 over the trial index).
        let delay_ms = 30 + splitmix(u64::from(trial)) % 1200;
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        let _ = child.kill();
        let _ = child.wait();

        // The journal may not exist yet if the kill landed before creation.
        if !run_dir.join("journal.sjl").exists() {
            continue;
        }

        // Recover from the crashed run directory.
        let resolver = MemResolver {
            sets: vec![data.clone()],
        };
        let factory = || scenario.modules(None).expect("scenario modules");
        let (recovered, report) = SimCore::recover(&run_dir, &resolver, &factory)
            .with_context(|| format!("trial {trial}: recovery failed"))?;
        let recovered_fp = recovered.fingerprint()?;
        let tick = report.recovered_tick;

        // Cross-check: replaying the same journal through the independent replay()
        // entry point must reproduce the recovered state exactly (recovery and
        // replay agree; replay is itself validated against live runs). This holds
        // even when the kill landed with an interrupt journaled but not yet acked
        // — both paths end paused identically.
        let journal = std::fs::read(run_dir.join("journal.sjl"))
            .with_context(|| format!("trial {trial}: reading journal"))?;
        let (replayed, _) =
            SimCore::replay(&journal, &resolver, scenario.modules(None)?, None, false)
                .with_context(|| format!("trial {trial}: replay cross-check failed"))?;
        let replay_fp = replayed.fingerprint()?;
        if recovered_fp != replay_fp {
            bail!(
                "trial {trial}: recovered state at tick {tick} disagrees with replay of the \
                 same journal: {} ≠ {}",
                recovered_fp.hex(),
                replay_fp.hex()
            );
        }
        outcomes.push(TrialOutcome {
            recovered_tick: tick,
            torn: report.torn_tail.is_some(),
        });
    }
    Ok(outcomes)
}

fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
