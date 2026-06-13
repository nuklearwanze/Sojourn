//! Traceability (FR-VD-202, Principle VIII): every derived number expands to a tree
//! whose leaves are **sourced data values** and whose internal nodes are named
//! operations. Built as data so it is machine-checkable and FA-10-renderable.

use serde::{Deserialize, Serialize};

/// A node in a derivation's traceability tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraceTree {
    /// A sourced data leaf: a value that came directly from a data file.
    Leaf {
        /// What it is (e.g. `"engine.isp_s"`).
        name: String,
        /// Its value.
        value: f64,
        /// Provenance (the data entry's `source`).
        source: String,
    },
    /// A derived value: a named operation over inputs.
    Node {
        /// The operation (e.g. `"rocket-equation"`).
        op: String,
        /// The resulting value.
        value: f64,
        /// The inputs.
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

    /// A derived node.
    pub fn node(op: impl Into<String>, value: f64, inputs: Vec<TraceTree>) -> TraceTree {
        TraceTree::Node {
            op: op.into(),
            value,
            inputs,
        }
    }

    /// The value at this node.
    pub fn value(&self) -> f64 {
        match self {
            TraceTree::Leaf { value, .. } | TraceTree::Node { value, .. } => *value,
        }
    }

    /// Does every leaf carry a non-empty source? (Principle II — no magic numbers
    /// leak into a derivation.)
    pub fn all_leaves_sourced(&self) -> bool {
        match self {
            TraceTree::Leaf { source, .. } => !source.trim().is_empty(),
            TraceTree::Node { inputs, .. } => inputs.iter().all(TraceTree::all_leaves_sourced),
        }
    }

    /// Count the leaves.
    pub fn leaf_count(&self) -> usize {
        match self {
            TraceTree::Leaf { .. } => 1,
            TraceTree::Node { inputs, .. } => inputs.iter().map(TraceTree::leaf_count).sum(),
        }
    }
}
