//! Traceability rendering (R5, FR-UI-201/202). The slices each publish a structurally
//! identical `TraceTree` (sourced `Leaf` / operation `Node`). This module mirrors it
//! as a UI-side `TraceRender` and flattens it into an inspector-friendly, indented
//! list — rendering **operation nodes and sourced leaves exactly as the core
//! provides**, **flagging a missing source**, and showing "derivation unavailable"
//! when a value has no tree. The UI never reconstructs a derivation.

/// A UI-side mirror of a slice traceability node.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceRender {
    /// A sourced input value.
    Leaf {
        /// What it is.
        name: String,
        /// Its value.
        value: f64,
        /// Provenance (empty ⇒ flagged as a data defect, not hidden).
        source: String,
    },
    /// A named operation over child inputs.
    Node {
        /// The operation.
        op: String,
        /// The computed value.
        value: f64,
        /// Inputs.
        inputs: Vec<TraceRender>,
    },
}

impl TraceRender {
    /// A sourced leaf.
    pub fn leaf(name: impl Into<String>, value: f64, source: impl Into<String>) -> TraceRender {
        TraceRender::Leaf {
            name: name.into(),
            value,
            source: source.into(),
        }
    }

    /// An operation node.
    pub fn node(op: impl Into<String>, value: f64, inputs: Vec<TraceRender>) -> TraceRender {
        TraceRender::Node {
            op: op.into(),
            value,
            inputs,
        }
    }

    /// True iff every leaf carries a non-empty source.
    pub fn all_leaves_sourced(&self) -> bool {
        match self {
            TraceRender::Leaf { source, .. } => !source.trim().is_empty(),
            TraceRender::Node { inputs, .. } => inputs.iter().all(TraceRender::all_leaves_sourced),
        }
    }
}

/// One rendered row of an expanded derivation (for an indented inspector list).
#[derive(Debug, Clone, PartialEq)]
pub struct TraceRow {
    /// Indent depth.
    pub depth: usize,
    /// Operation or leaf label.
    pub label: String,
    /// Formatted value.
    pub value: f64,
    /// `Some(source)` for a leaf; `None` for an operation node.
    pub source: Option<String>,
    /// True iff this is a leaf whose source is missing (visibly flagged).
    pub missing_source: bool,
}

/// Flatten a derivation (or its absence) into inspector rows. `None` ⇒ a single
/// "derivation unavailable" row carrying the value (FR-UI-201).
pub fn flatten(value: f64, tree: Option<&TraceRender>) -> Vec<TraceRow> {
    match tree {
        None => vec![TraceRow {
            depth: 0,
            label: "derivation unavailable".into(),
            value,
            source: None,
            missing_source: false,
        }],
        Some(t) => {
            let mut out = Vec::new();
            walk(t, 0, &mut out);
            out
        }
    }
}

fn walk(t: &TraceRender, depth: usize, out: &mut Vec<TraceRow>) {
    match t {
        TraceRender::Leaf {
            name,
            value,
            source,
        } => {
            let missing = source.trim().is_empty();
            out.push(TraceRow {
                depth,
                label: name.clone(),
                value: *value,
                source: Some(if missing {
                    "⚠ missing source".into()
                } else {
                    source.clone()
                }),
                missing_source: missing,
            });
        }
        TraceRender::Node { op, value, inputs } => {
            out.push(TraceRow {
                depth,
                label: op.clone(),
                value: *value,
                source: None,
                missing_source: false,
            });
            for child in inputs {
                walk(child, depth + 1, out);
            }
        }
    }
}
