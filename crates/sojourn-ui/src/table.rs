//! Virtualised-table derive math (R8, FR-UI-1405). Pure filter/sort/group + a
//! `visible_range` so the renderer paints **only the visible rows** — thousands-row
//! catalogues/fleets/ledgers stay within the responsiveness budget (SC-006). All
//! derivation is deterministic and headlessly tested; the renderer holds no logic.

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    /// Ascending.
    Asc,
    /// Descending.
    Desc,
}

/// A table over `Row` with pure filter/sort/group + virtualisation.
pub struct TableModel<'a, Row> {
    rows: &'a [Row],
    order: Vec<usize>,
}

impl<'a, Row> TableModel<'a, Row> {
    /// Build over the backing rows (identity order).
    pub fn new(rows: &'a [Row]) -> TableModel<'a, Row> {
        TableModel {
            rows,
            order: (0..rows.len()).collect(),
        }
    }

    /// Keep only rows matching the predicate (filter).
    pub fn filter(mut self, pred: impl Fn(&Row) -> bool) -> Self {
        self.order.retain(|&i| pred(&self.rows[i]));
        self
    }

    /// Sort by a key extractor in the given direction (stable).
    pub fn sort_by<K: Ord>(mut self, key: impl Fn(&Row) -> K, dir: SortDir) -> Self {
        self.order.sort_by(|&a, &b| {
            let o = key(&self.rows[a]).cmp(&key(&self.rows[b]));
            match dir {
                SortDir::Asc => o,
                SortDir::Desc => o.reverse(),
            }
        });
        self
    }

    /// The number of rows after filtering.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// True iff no rows remain.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// The full derived order (row indices) — for grouping/iteration.
    pub fn order(&self) -> &[usize] {
        &self.order
    }

    /// The **visible** row indices for a viewport `[first, first+count)`, clamped —
    /// the virtualisation contract (the renderer paints only these).
    pub fn visible_range(&self, first: usize, count: usize) -> &[usize] {
        let n = self.order.len();
        let start = first.min(n);
        let end = (first + count).min(n);
        &self.order[start..end]
    }
}
