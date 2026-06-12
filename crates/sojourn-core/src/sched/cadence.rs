//! Cadence & quiet-span arithmetic. Pure functions over simulation state —
//! never wall-clock, never warp rate (FR-CORE-204: warp is pure playback speed).

use crate::data::DataSet;
use crate::module::{EscalationDecl, ModuleManifest};
use crate::state::{KernelState, ViewSnapshot};
use crate::watch::eval as watch_eval;
use std::collections::BTreeMap;

/// Is this module escalated to fine (every-tick) stepping right now? Evaluated
/// over the module's own published views at the last completed boundary.
pub fn escalated(manifest: &ModuleManifest, views: &BTreeMap<String, ViewSnapshot>) -> bool {
    manifest
        .escalations
        .iter()
        .any(|e| escalation_true(e, views))
}

fn escalation_true(e: &EscalationDecl, views: &BTreeMap<String, ViewSnapshot>) -> bool {
    let Some(view) = views.get(&e.view) else {
        return false;
    };
    let Some(field) = view.fields.get(&e.field) else {
        return false;
    };
    // Same strict same-variant comparison the watch system uses.
    watch_eval::compare(field, e.op, &e.value)
}

/// Next tick strictly after `current` at which this module is due.
pub fn next_due_tick(manifest: &ModuleManifest, current: u64, is_escalated: bool) -> u64 {
    if is_escalated {
        current + 1
    } else {
        let c = manifest.cadence_ticks.max(1);
        ((current / c) + 1) * c
    }
}

/// The next tick at which *anything* can happen — the quiet-span jump target
/// (FR-CORE-402). Considers, and takes the minimum of:
/// module due ticks (cadence or escalation), the earliest scheduled event,
/// time-based watch thresholds, the horizon, and the caller's target.
pub fn next_active_tick(
    kernel: &KernelState,
    manifests: &[ModuleManifest],
    data: &DataSet,
    target: u64,
) -> u64 {
    let current = kernel.clock.tick.0;
    let mut next = target;

    for m in manifests {
        let esc = escalated(m, &kernel.views);
        next = next.min(next_due_tick(m, current, esc));
    }
    if let Some((&t, _)) = kernel.scheduled.range((current + 1)..).next() {
        next = next.min(t);
    }
    for w in kernel.watches.values() {
        if w.armed
            && let Some(t) = watch_eval::earliest_time_threshold_tick(
                &w.spec.expr,
                data,
                kernel.clock.base_tick_ns,
                current,
            )
        {
            next = next.min(t);
        }
    }
    if !kernel.horizon_signalled {
        next = next.min(kernel.clock.horizon_tick.0.max(current + 1));
    }
    next.max(current + 1)
}
