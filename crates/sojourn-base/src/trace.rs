//! Traceability tree (R13, FR-BC-106). Every emergent property is drillable to
//! **sourced leaves** — the honesty contract (Principle VIII). Reused shape from
//! FA-04/FA-06.

use serde::{Deserialize, Serialize};

/// A node in a traceability tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraceTree {
    /// A sourced input value.
    Leaf {
        /// What it is.
        name: String,
        /// Its value.
        value: f64,
        /// Provenance (sourced data).
        source: String,
    },
    /// A named operation over child inputs.
    Node {
        /// The operation.
        op: String,
        /// The computed value.
        value: f64,
        /// Inputs.
        inputs: Vec<TraceTree>,
    },
}

impl TraceTree {
    /// A sourced leaf.
    pub fn leaf(name: impl Into<String>, value: f64, source: impl Into<String>) -> TraceTree {
        TraceTree::Leaf {
            name: name.into(),
            value,
            source: source.into(),
        }
    }

    /// An operation node.
    pub fn node(op: impl Into<String>, value: f64, inputs: Vec<TraceTree>) -> TraceTree {
        TraceTree::Node {
            op: op.into(),
            value,
            inputs,
        }
    }

    /// The computed value at this node.
    pub fn value(&self) -> f64 {
        match self {
            TraceTree::Leaf { value, .. } | TraceTree::Node { value, .. } => *value,
        }
    }

    /// True iff every leaf carries a non-empty source (CI honesty gate).
    pub fn all_leaves_sourced(&self) -> bool {
        match self {
            TraceTree::Leaf { source, .. } => !source.trim().is_empty(),
            TraceTree::Node { inputs, .. } => inputs.iter().all(TraceTree::all_leaves_sourced),
        }
    }

    /// Count of leaves.
    pub fn leaf_count(&self) -> usize {
        match self {
            TraceTree::Leaf { .. } => 1,
            TraceTree::Node { inputs, .. } => inputs.iter().map(TraceTree::leaf_count).sum(),
        }
    }
}
