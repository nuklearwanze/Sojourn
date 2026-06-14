//! US2 — traceability rendering (sourced leaves, missing-source flag, no-tree
//! fallback) (FR-UI-201/202).

use sojourn_ui::trace::{TraceRender, flatten};

fn dv_tree() -> TraceRender {
    TraceRender::node(
        "rocket equation",
        6200.0,
        vec![
            TraceRender::leaf("Isp", 380.0, "RL10 hot-fire data (data/vehicle)"),
            TraceRender::leaf("mass ratio", 2.7, "stack mass budget"),
        ],
    )
}

#[test]
fn a_tree_flattens_to_sourced_rows() {
    let rows = flatten(6200.0, Some(&dv_tree()));
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].depth, 0); // the operation node
    assert!(rows[0].source.is_none());
    assert_eq!(rows[1].depth, 1); // a sourced leaf
    assert!(rows[1].source.as_deref().unwrap().contains("RL10"));
    assert!(dv_tree().all_leaves_sourced());
}

#[test]
fn a_missing_source_is_flagged_not_hidden() {
    let t = TraceRender::node("x", 1.0, vec![TraceRender::leaf("bad", 1.0, "  ")]);
    assert!(!t.all_leaves_sourced());
    let rows = flatten(1.0, Some(&t));
    let leaf = rows.iter().find(|r| r.label == "bad").unwrap();
    assert!(leaf.missing_source, "a missing source must be flagged");
}

#[test]
fn no_tree_renders_derivation_unavailable() {
    let rows = flatten(42.0, None);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, "derivation unavailable");
    assert_eq!(rows[0].value, 42.0);
}
