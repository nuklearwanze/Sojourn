//! Save→load→continue identity (FR-CORE-601, SC-002): a run that saves and
//! reloads at arbitrary ticks must evolve identically to one that never saved —
//! including the loaded state itself being bit-identical to the saved state.

use crate::scenario::{Scenario, StepPattern, drive, drive_resumed};
use anyhow::{Result, bail};
use sojourn_core::{DataSet, MemResolver, SimCore};

/// Run the round-trip check with saves at the given ticks.
pub fn roundtrip(scenario: &Scenario, data: &DataSet, save_at: &[u64]) -> Result<()> {
    // Reference: unbroken run to the end.
    let mut reference = scenario.create(data, None, None)?;
    let ref_out = drive(&mut reference, scenario, None, StepPattern::MaxStride)?;
    let ref_fp = reference.fingerprint()?;
    let resolver = MemResolver {
        sets: vec![data.clone()],
    };

    for &save_tick in save_at {
        if save_tick >= scenario.until_tick {
            bail!(
                "save tick {save_tick} is past until_tick {}",
                scenario.until_tick
            );
        }
        // Fresh run to the save point.
        let mut run = scenario.create(data, None, None)?;
        drive(
            &mut run,
            scenario,
            Some(save_tick),
            StepPattern::Chunk(4099),
        )?;
        let save_fp = run.fingerprint()?;
        let bytes = run.save()?;

        // Load must restore bit-identically.
        let mut loaded = SimCore::load(&bytes, &resolver, scenario.modules(None)?)?;
        let loaded_fp = loaded.fingerprint()?;
        if loaded_fp != save_fp {
            bail!(
                "load(save@{save_tick}) is not identical: {} ≠ {}",
                loaded_fp.hex(),
                save_fp.hex()
            );
        }

        // Continue to the end and compare with the unbroken run. Commands already
        // applied through the save tick must not be re-submitted.
        let out = drive_resumed(
            &mut loaded,
            scenario,
            None,
            StepPattern::MaxStride,
            Some(save_tick),
        )?;
        let fp = loaded.fingerprint()?;
        if fp != ref_fp || out.final_tick != ref_out.final_tick {
            let diff = reference.fingerprint_diff(&loaded.save()?)?;
            let sections: Vec<String> = diff
                .divergent_sections
                .iter()
                .map(|s| s.section.clone())
                .collect();
            bail!(
                "save@{save_tick}: reloaded run diverged from unbroken run \
                 ({} ≠ {}) — divergent sections: [{}]",
                fp.hex(),
                ref_fp.hex(),
                sections.join(", ")
            );
        }
    }
    Ok(())
}
