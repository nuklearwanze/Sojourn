//! US14 — virtualised-table derive math (filter/sort/visible-range) (FR-UI-1405, SC-006).

use sojourn_ui::table::{SortDir, TableModel};

#[derive(Clone)]
struct Row {
    name: String,
    grade: u32,
}

fn rows(n: u32) -> Vec<Row> {
    (0..n)
        .map(|i| Row {
            name: format!("r{:05}", n - 1 - i),
            grade: i % 5,
        })
        .collect()
}

#[test]
fn filter_sort_and_visible_range_are_correct_on_a_large_set() {
    let data = rows(10_000);
    let model = TableModel::new(&data)
        .filter(|r| r.grade >= 3)
        .sort_by(|r| r.name.clone(), SortDir::Asc);
    assert_eq!(model.len(), 4000, "filter keeps grades 3,4 (2 of every 5)");

    // Virtualisation: only the viewport slice is returned.
    let window = model.visible_range(0, 25);
    assert_eq!(window.len(), 25);
    // Sorted ascending by name ⇒ the first visible row is the smallest name kept.
    // name = "r{05}" of (n-1-i), grade = i%5; i=9999 (name r00000) has grade 4 (kept).
    let first = &data[window[0]];
    assert!(first.grade >= 3);
    assert_eq!(first.name, "r00000");
}

#[test]
fn visible_range_clamps_at_the_end() {
    let data = rows(100);
    let model = TableModel::new(&data);
    let window = model.visible_range(90, 50); // asks past the end
    assert_eq!(window.len(), 10, "clamped to the remaining rows");
}
