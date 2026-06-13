//! Track B — Engineering Programs (FR-RESP-201/203, R4): a Technology advanced
//! through TRL via test campaigns, with cost/schedule P50/P80, a minimum-duration
//! floor (schedule compression → overrun risk, never sub-floor time), facility + UL
//! gating, and S-curve progress.

use crate::campaigns;
use crate::ids::{FactionId, PersonId, ProgramId, TechId};
use crate::rp_de::Allocation;
use crate::tree::{Params, TechNode};
use serde::{Deserialize, Serialize};

/// An engineering program (slice state).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    /// Identity.
    pub id: ProgramId,
    /// Owner.
    pub faction: FactionId,
    /// Target technology.
    pub tech: TechId,
    /// Current TRL.
    pub trl: u8,
    /// S-curve progress in the current step (0..1).
    pub step_progress: f64,
    /// Elapsed days on the current step (the min-duration floor).
    pub days_on_step: f64,
    /// P50 schedule estimate to target (days).
    pub est_p50_days: f64,
    /// P80 schedule estimate to target (days).
    pub est_p80_days: f64,
    /// Realised elapsed days.
    pub actual_days: f64,
    /// Assigned lead (trait effects).
    pub lead: Option<PersonId>,
    /// Rising on stalled progress (the dead-end hint).
    pub risk_index: f64,
    /// Consecutive test failures (dead-end signal when high without UL growth).
    pub fail_streak: u32,
    /// Groups parallel approaches to one capability.
    pub parallel_tag: Option<String>,
}

/// What an advance produced this step (the module emits events + injects UL).
#[derive(Debug, Default)]
pub struct AdvanceOut {
    /// Reached a new TRL this step.
    pub trl_advanced: Option<u8>,
    /// A test campaign failed (spectacular = TRL-7 flight demo).
    pub test_failed: Option<bool>,
    /// The approach was confirmed a dead end.
    pub dead_end_confirmed: bool,
    /// UL to inject into the tech's first floor domain (failure-that-teaches).
    pub ul_inject: f64,
    /// Blocked this step (facility/UL gate) — reason for diagnostics.
    pub blocked: Option<String>,
}

impl Program {
    /// Estimate P50/P80 schedule to the next-after target (sum of remaining step
    /// floors); P80 carries the overrun margin.
    pub fn estimate(&mut self, node: &TechNode, params: &Params) {
        let remaining: f64 = node
            .trl_steps
            .iter()
            .filter(|s| s.trl > self.trl)
            .map(|s| s.min_duration_days)
            .sum();
        self.est_p50_days = remaining;
        self.est_p80_days = remaining * (1.0 + params.overrun_sigma * 2.0);
    }
}

/// Advance a program one step. `de` is the Design Effort applied; `uniform` draws
/// from the deterministic stream. Mutates the program; returns what to emit.
#[allow(clippy::too_many_arguments)]
pub fn advance(
    prog: &mut Program,
    node: &TechNode,
    de: f64,
    dt_days: f64,
    _ul_margin: f64,
    is_dead_end: bool,
    alloc: &Allocation,
    params: &Params,
    low_mult: f64,
    qual_mult: f64,
    overrun_mult: f64,
    uniform: &mut impl FnMut() -> f64,
) -> AdvanceOut {
    let mut out = AdvanceOut::default();
    if prog.trl >= 9 || de <= 0.0 {
        return out;
    }
    let target = prog.trl + 1;
    let Some(step) = node.trl_steps.iter().find(|s| s.trl == target) else {
        return out;
    };
    // Facility gate (caller-asserted availability, R13).
    if let Some(cap) = &step.facility
        && !alloc.has_facility(cap)
    {
        out.blocked = Some(format!("facility '{cap}' unavailable"));
        prog.risk_index += 0.001 * dt_days;
        return out;
    }

    prog.days_on_step += dt_days;
    prog.actual_days += dt_days;
    // S-curve progress: DE advances toward 1.0 scaled by step cost.
    let mult = if target <= 5 { low_mult } else { qual_mult };
    prog.step_progress += (de * mult / step.cost.max(1.0))
        * (1.0 + step.scurve_k * prog.step_progress * (1.0 - prog.step_progress));
    prog.step_progress = prog.step_progress.min(1.0);

    // A step may only COMPLETE after its minimum duration (the floor) and full
    // progress. Over-funding cannot buy sub-floor time — it just idles.
    if prog.step_progress < 1.0 || prog.days_on_step < step.min_duration_days {
        return out;
    }

    // Test campaign at the step boundary.
    let fail_prob =
        campaigns::fail_probability(params, target, prog.risk_index, is_dead_end, overrun_mult);
    if uniform() < fail_prob {
        // Failure-that-teaches.
        out.test_failed = Some(target == 7); // a TRL-7 flight demo is spectacular
        out.ul_inject = params.failure_ul_inject;
        prog.fail_streak += 1;
        prog.risk_index += 0.1 * (1.0 + if is_dead_end { 1.0 } else { 0.0 });
        prog.step_progress = 0.6; // re-work
        prog.days_on_step = 0.0;
        // Repeated failure on a seeded dead end → confirmation.
        if is_dead_end && prog.fail_streak >= 3 {
            out.dead_end_confirmed = true;
        }
    } else {
        // Pass.
        prog.trl = target;
        prog.step_progress = 0.0;
        prog.days_on_step = 0.0;
        prog.fail_streak = 0;
        prog.risk_index = (prog.risk_index - 0.05).max(0.0);
        out.trl_advanced = Some(target);
    }
    out
}
