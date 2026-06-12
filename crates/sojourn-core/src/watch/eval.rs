//! Watch evaluation (FR-CORE-407): deterministic, evaluated at tick boundaries in
//! `WatchId` order, with the time-threshold extraction that lets the scheduler
//! jump quiet spans without ever jumping past a time-based watch's first truth.

use super::{CmpOp, WatchExpr, WatchSpec};
use crate::data::{DataSet, WatchTemplateDef};
use crate::error::CoreError;
use crate::state::{ViewId, ViewSnapshot, ViewValue};
use std::collections::BTreeMap;

/// Validate a player-supplied spec against the template catalogue and the run's
/// available templates (bindings resolved at run creation per the A1 remediation).
pub fn validate_spec(
    spec: &WatchSpec,
    data: &DataSet,
    available_views: &[ViewId],
) -> Result<(), CoreError> {
    validate_expr(&spec.expr, data, available_views)
}

fn validate_expr(
    expr: &WatchExpr,
    data: &DataSet,
    available_views: &[ViewId],
) -> Result<(), CoreError> {
    match expr {
        WatchExpr::Leaf { template, value } => {
            let t = data
                .watch_template(template)
                .ok_or_else(|| CoreError::CommandInvalid {
                    reason: format!("unknown watch template '{template}'"),
                })?;
            if !available_views.contains(&t.view) {
                return Err(CoreError::CommandInvalid {
                    reason: format!(
                        "watch template '{template}' unavailable: view '{}' is not published \
                         by any registered module in this run",
                        t.view
                    ),
                });
            }
            if !t.param.matches(value) {
                return Err(CoreError::CommandInvalid {
                    reason: format!(
                        "watch template '{template}' expects a {:?} parameter",
                        t.param
                    ),
                });
            }
            Ok(())
        }
        WatchExpr::And(children) | WatchExpr::Or(children) => {
            if children.is_empty() {
                return Err(CoreError::CommandInvalid {
                    reason: "empty AND/OR composition".into(),
                });
            }
            children
                .iter()
                .try_for_each(|c| validate_expr(c, data, available_views))
        }
    }
}

/// Evaluate an expression against the current published views. A leaf whose view
/// or field is absent evaluates `false` (the producing module has not published
/// yet) — deterministic by construction.
pub fn evaluate(expr: &WatchExpr, data: &DataSet, views: &BTreeMap<ViewId, ViewSnapshot>) -> bool {
    match expr {
        WatchExpr::Leaf { template, value } => {
            let Some(t) = data.watch_template(template) else {
                return false;
            };
            let Some(snapshot) = views.get(&t.view) else {
                return false;
            };
            let Some(field) = snapshot.fields.get(&t.field) else {
                return false;
            };
            compare(field, t.op, value)
        }
        WatchExpr::And(children) => children.iter().all(|c| evaluate(c, data, views)),
        WatchExpr::Or(children) => children.iter().any(|c| evaluate(c, data, views)),
    }
}

/// Strict same-variant comparison. A type mismatch (a module changed a field's
/// type — a defect) evaluates `false` rather than guessing across types.
pub(crate) fn compare(field: &ViewValue, op: CmpOp, param: &ViewValue) -> bool {
    let ord = match (field, param) {
        (ViewValue::I64(a), ViewValue::I64(b)) => a.partial_cmp(b),
        (ViewValue::U64(a), ViewValue::U64(b)) => a.partial_cmp(b),
        (ViewValue::F64(a), ViewValue::F64(b)) => a.partial_cmp(b), // NaN ⇒ None ⇒ false
        (ViewValue::Bool(a), ViewValue::Bool(b)) => a.partial_cmp(b),
        (ViewValue::Str(a), ViewValue::Str(b)) => a.partial_cmp(b),
        _ => {
            debug_assert!(
                false,
                "watch comparison type mismatch: {field:?} vs {param:?}"
            );
            None
        }
    };
    let Some(ord) = ord else { return false };
    match op {
        CmpOp::Lt => ord.is_lt(),
        CmpOp::Le => ord.is_le(),
        CmpOp::Gt => ord.is_gt(),
        CmpOp::Ge => ord.is_ge(),
        CmpOp::Eq => ord.is_eq(),
        CmpOp::Ne => ord.is_ne(),
    }
}

/// For quiet-span jumping: the earliest tick at which any **time-based** leaf of
/// this expression could first become true, given the current tick. Non-time
/// leaves only change when their producing module steps (already a jump bound),
/// so only `kernel/status.tick` / `kernel/status.sim_time_ns` leaves contribute.
pub fn earliest_time_threshold_tick(
    expr: &WatchExpr,
    data: &DataSet,
    base_tick_ns: u64,
    current_tick: u64,
) -> Option<u64> {
    match expr {
        WatchExpr::Leaf { template, value } => {
            let t = data.watch_template(template)?;
            if t.view != "kernel/status" {
                return None;
            }
            threshold_tick(t, value, base_tick_ns).filter(|&tk| tk > current_tick)
        }
        WatchExpr::And(children) | WatchExpr::Or(children) => children
            .iter()
            .filter_map(|c| earliest_time_threshold_tick(c, data, base_tick_ns, current_tick))
            .min(),
    }
}

/// Tick at which `field op param` first flips for monotonically increasing time
/// fields. Conservative: ops that flip true→false (Lt/Le/Ne against time) return
/// the same boundary — evaluating there is harmless and keeps the bound simple.
fn threshold_tick(t: &WatchTemplateDef, param: &ViewValue, base_tick_ns: u64) -> Option<u64> {
    match (t.field.as_str(), param) {
        ("tick", ViewValue::U64(v)) => match t.op {
            CmpOp::Ge | CmpOp::Eq => Some(*v),
            CmpOp::Gt | CmpOp::Ne => Some(v.saturating_add(1)),
            CmpOp::Lt | CmpOp::Le => Some(*v),
        },
        ("sim_time_ns", ViewValue::I64(v)) => {
            let v = (*v).max(0) as u64;
            let at_or_after = v.div_ceil(base_tick_ns);
            match t.op {
                CmpOp::Ge | CmpOp::Eq | CmpOp::Lt | CmpOp::Le => Some(at_or_after),
                CmpOp::Gt | CmpOp::Ne => Some(if v.is_multiple_of(base_tick_ns) {
                    at_or_after + 1
                } else {
                    at_or_after
                }),
            }
        }
        _ => None,
    }
}
