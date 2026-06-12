//! The double-run determinism check (FR-CORE-703, SC-001): the same scenario run
//! twice with deliberately different stepping patterns must produce identical
//! checkpoint fingerprints AND an identical full event log. On divergence, the
//! first divergent checkpoint and a structural diff hint are reported.

use crate::mutation::Mutation;
use crate::scenario::{Scenario, StepPattern, drive, full_event_log};
use anyhow::{Result, bail};
use sojourn_core::DataSet;

/// Outcome of a verify run.
#[derive(Debug)]
pub struct VerifyOutcome {
    /// Checkpoints compared.
    pub checkpoints: usize,
    /// Events compared.
    pub events: usize,
}

/// Run the scenario twice (max-stride vs 7919-tick chunks) and compare.
pub fn verify(
    scenario: &Scenario,
    data: &DataSet,
    mutation: Option<Mutation>,
) -> Result<VerifyOutcome> {
    let mut a = scenario.create(data, None, mutation)?;
    let out_a = drive(&mut a, scenario, None, StepPattern::MaxStride)?;
    let log_a = full_event_log(&a)?;

    let mut b = scenario.create(data, None, mutation)?;
    let out_b = drive(&mut b, scenario, None, StepPattern::Chunk(7919))?;
    let log_b = full_event_log(&b)?;

    if out_a.final_tick != out_b.final_tick {
        bail!(
            "final tick differs between stepping patterns: {} vs {}",
            out_a.final_tick,
            out_b.final_tick
        );
    }
    for ((ta, fa), (tb, fb)) in out_a.checkpoints.iter().zip(&out_b.checkpoints) {
        if ta != tb || fa != fb {
            // Structural diff for the divergent checkpoint.
            let save_b = b.save()?;
            let diff = a.fingerprint_diff(&save_b)?;
            let sections: Vec<String> = diff
                .divergent_sections
                .iter()
                .map(|s| s.section.clone())
                .collect();
            bail!(
                "DIVERGENCE at checkpoint tick {ta}: {} ≠ {} — divergent state sections \
                 (at final tick): [{}]",
                fa.hex(),
                fb.hex(),
                sections.join(", ")
            );
        }
    }
    if out_a.checkpoints.len() != out_b.checkpoints.len() {
        bail!(
            "checkpoint count differs: {} vs {}",
            out_a.checkpoints.len(),
            out_b.checkpoints.len()
        );
    }
    if log_a != log_b {
        let first = log_a
            .iter()
            .zip(&log_b)
            .position(|(x, y)| x != y)
            .map(|i| format!("first divergent event index {i} (tick {})", log_a[i].tick))
            .unwrap_or_else(|| format!("event counts differ: {} vs {}", log_a.len(), log_b.len()));
        bail!("DIVERGENCE in event log: {first}");
    }
    let fa = a.fingerprint()?;
    let fb = b.fingerprint()?;
    if fa != fb {
        bail!("final fingerprints differ: {} ≠ {}", fa.hex(), fb.hex());
    }
    Ok(VerifyOutcome {
        checkpoints: out_a.checkpoints.len(),
        events: log_a.len(),
    })
}
