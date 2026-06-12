//! Player watch conditions (FR-CORE-406, clarified 2026-06-12): instances of a
//! curated, data-driven template catalogue, composable with AND/OR. No free-form
//! expression language.

pub mod eval;

use crate::state::ViewValue;
use serde::{Deserialize, Serialize};

/// Comparison operator a template applies between the bound view field and the
/// player-supplied parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpOp {
    /// Less than.
    Lt,
    /// Less than or equal.
    Le,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Ge,
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
}

/// Parameter type a template requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamKind {
    /// Signed integer (also `SimTimeNs`).
    I64,
    /// Unsigned integer.
    U64,
    /// Float.
    F64,
    /// Boolean.
    Bool,
    /// Identifier string.
    Str,
}

impl ParamKind {
    /// Does a value have this kind?
    pub fn matches(self, v: &ViewValue) -> bool {
        matches!(
            (self, v),
            (ParamKind::I64, ViewValue::I64(_))
                | (ParamKind::U64, ViewValue::U64(_))
                | (ParamKind::F64, ViewValue::F64(_))
                | (ParamKind::Bool, ViewValue::Bool(_))
                | (ParamKind::Str, ViewValue::Str(_))
        )
    }
}

/// Composition tree: template leaves combined with AND/OR (the v1.0 expressiveness).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WatchExpr {
    /// One template instance: `template(field) op value`.
    Leaf {
        /// Template id from the data catalogue.
        template: String,
        /// Player-supplied parameter (type per the template's `param`).
        value: ViewValue,
    },
    /// All sub-expressions true.
    And(Vec<WatchExpr>),
    /// Any sub-expression true.
    Or(Vec<WatchExpr>),
}

/// A player-supplied watch condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchSpec {
    /// The condition tree.
    pub expr: WatchExpr,
}

/// A registered watch and its edge state.
///
/// Edge semantics (documented, deterministic): a watch **fires at the tick its
/// expression is first true** — including the registration tick itself — then
/// disarms; it **re-arms when the expression evaluates false** at a later tick
/// boundary, after which it can fire again on the next false→true edge. An
/// oscillating condition therefore fires once per true-period, never more.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchState {
    /// Watch id.
    pub id: u64,
    /// The condition.
    pub spec: WatchSpec,
    /// Armed: will fire on the next evaluation that is true.
    pub armed: bool,
}
