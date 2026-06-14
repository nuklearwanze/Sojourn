//! The view-model (R2, `contracts/viewmodel.md`): pure functions mapping core
//! snapshot data to display structs. **No physics, no authoritative mutation** — only
//! presentation derivation (format, sort/filter/group, visible-range, trace
//! flattening, gating). Each `build` is pure in its inputs and **headlessly testable**
//! over stub inputs (FR-UI-1506). The host assembles the inputs from the slices'
//! read-only surfaces; tests stub them directly.

use crate::host::{EventView, Status};
use crate::table::{SortDir, TableModel};
use crate::units::{self, Quantity};
use crate::widgets_data::{
    BaseSchematic, DvLadder, EvidenceMeter, PorkchopField, ResourceLedger, TrlLadder,
    UnderstandingBars,
};

/// The level of detail the player has chosen (progressive disclosure, FR-UI-203).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disclosure {
    /// Newcomer summary.
    Summary,
    /// Full Aurora-grade detail.
    Full,
}

// ---- Shell (US1) ------------------------------------------------------------

/// The persistent shell view-model.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellVM {
    /// The formatted date (year-month-day).
    pub date: String,
    /// The tick.
    pub tick: u64,
    /// Pending interrupts (surfaced regardless of screen, FR-UI-1003).
    pub pending_interrupts: u64,
    /// The newest event classes for the bottom ticker.
    pub ticker: Vec<String>,
}

/// Build the shell from the run status + recent events.
pub fn shell(status: &Status, events: &[EventView]) -> ShellVM {
    let (y, m, d) = status.date;
    ShellVM {
        date: format!("{y:04}-{m:02}-{d:02}"),
        tick: status.tick,
        pending_interrupts: status.pending_interrupts,
        ticker: events
            .iter()
            .rev()
            .take(8)
            .map(|e| e.class.clone())
            .collect(),
    }
}

// ---- System Map (US1) -------------------------------------------------------

/// A body as the host read it from the astro/world surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyInput {
    /// Catalogue id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// Heliocentric position (m).
    pub pos: (f64, f64),
    /// Apparent radius at the current zoom (pixels) — for LOD culling.
    pub apparent_px: f64,
}

/// A rendered body (post-LOD).
#[derive(Debug, Clone, PartialEq)]
pub struct BodyView {
    /// Id.
    pub id: u32,
    /// Label.
    pub name: String,
    /// Screen-space position (m, the renderer projects).
    pub pos: (f64, f64),
}

/// The System Map view-model.
#[derive(Debug, Clone, PartialEq)]
pub struct MapVM {
    /// Bodies that survive level-of-detail culling.
    pub bodies: Vec<BodyView>,
    /// Count culled (surfaced honestly, not hidden).
    pub culled: usize,
}

/// Build the map with level-of-detail: bodies sub-pixel at the current zoom are culled
/// (FR-UI-1405, SC-006). The UI computes no positions — it renders what it is given.
pub fn map(bodies: &[BodyInput], min_apparent_px: f64) -> MapVM {
    let mut shown = Vec::new();
    let mut culled = 0;
    for b in bodies {
        if b.apparent_px >= min_apparent_px {
            shown.push(BodyView {
                id: b.id,
                name: b.name.clone(),
                pos: b.pos,
            });
        } else {
            culled += 1;
        }
    }
    MapVM {
        bodies: shown,
        culled,
    }
}

// ---- Operations / Fleet (US7) ----------------------------------------------

/// A craft/asset row as the host read it.
#[derive(Debug, Clone, PartialEq)]
pub struct CraftRow {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// Location label.
    pub location: String,
    /// Fuel fraction ∈ [0,1].
    pub fuel: f64,
    /// Health fraction ∈ [0,1].
    pub health: f64,
    /// Current task.
    pub task: String,
}

/// The operations table view-model (virtualised).
#[derive(Debug, Clone, PartialEq)]
pub struct OperationsVM {
    /// The visible rows (already filtered/sorted, virtualised to the viewport).
    pub visible: Vec<CraftRow>,
    /// Total rows after filtering (for the scrollbar).
    pub total: usize,
}

/// Build the operations table: filter to crewed-or-active, sort by name, virtualise to
/// the viewport `[first, first+count)` (FR-UI-701, SC-006).
pub fn operations(rows: &[CraftRow], first: usize, count: usize) -> OperationsVM {
    let model = TableModel::new(rows).sort_by(|r| r.name.clone(), SortDir::Asc);
    let total = model.len();
    let visible = model
        .visible_range(first, count)
        .iter()
        .map(|&i| rows[i].clone())
        .collect();
    OperationsVM { visible, total }
}

// ---- Trajectory Planner (US4) ----------------------------------------------

/// The planner view-model.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannerVM {
    /// The porkchop field (Δv/TOF from the core).
    pub porkchop: PorkchopField,
    /// The Δv ladder vs the vehicle's available Δv.
    pub ladder: DvLadder,
    /// Required Δv (formatted SI).
    pub required: String,
    /// Feasible against the selected vehicle?
    pub feasible: bool,
}

/// Build the planner: surface the porkchop + the required-vs-available Δv check
/// (FR-UI-401…403).
pub fn planner(porkchop: PorkchopField, ladder: DvLadder) -> PlannerVM {
    PlannerVM {
        required: units::format(Quantity::Velocity, ladder.required()),
        feasible: ladder.feasible(),
        porkchop,
        ladder,
    }
}

// ---- Economy (US8) ----------------------------------------------------------

/// The economy view-model.
#[derive(Debug, Clone, PartialEq)]
pub struct EconomyVM {
    /// The resource-by-location ledger.
    pub ledger: ResourceLedger,
    /// Per-location subtotals (location, formatted mass) — grouped.
    pub subtotals: Vec<(String, String)>,
}

/// Build the economy ledger grouped by Δv-location (FR-UI-802).
pub fn economy(ledger: ResourceLedger) -> EconomyVM {
    use std::collections::BTreeMap;
    let mut by_loc: BTreeMap<String, f64> = BTreeMap::new();
    for (loc, _commodity, mass) in &ledger.rows {
        *by_loc.entry(loc.clone()).or_insert(0.0) += mass;
    }
    let subtotals = by_loc
        .into_iter()
        .map(|(loc, m)| (loc, units::format(Quantity::Mass, m)))
        .collect();
    EconomyVM { ledger, subtotals }
}

// ---- Bases (US9) ------------------------------------------------------------

/// The bases view-model.
#[derive(Debug, Clone, PartialEq)]
pub struct BasesVM {
    /// The base schematic + gauges.
    pub schematic: BaseSchematic,
    /// Per-gauge `(name, formatted, ok)`.
    pub gauges: Vec<(String, String, bool)>,
}

/// Build the base schematic gauges (FR-UI-902).
pub fn bases(schematic: BaseSchematic) -> BasesVM {
    let gauges = schematic
        .gauges
        .iter()
        .map(|(name, value, threshold)| {
            (
                name.clone(),
                units::format(Quantity::Scalar, *value),
                value >= threshold,
            )
        })
        .collect();
    BasesVM { schematic, gauges }
}

// ---- R&D (US5) --------------------------------------------------------------

/// The R&D view-model.
#[derive(Debug, Clone, PartialEq)]
pub struct ResearchVM {
    /// Domain Understanding bars.
    pub bars: UnderstandingBars,
    /// TRL ladders for the active engineering programs.
    pub ladders: Vec<TrlLadder>,
    /// Dead-end warnings present?
    pub has_dead_end: bool,
}

/// Build the R&D view-model (FR-UI-501/502).
pub fn research(bars: UnderstandingBars, ladders: Vec<TrlLadder>) -> ResearchVM {
    let has_dead_end = ladders.iter().any(|l| l.dead_end);
    ResearchVM {
        bars,
        ladders,
        has_dead_end,
    }
}

// ---- Science Returns & Astrobiology (US12) ---------------------------------

/// What the host read for a candidate from FA-09 (NO ground truth — none is exposed).
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateInput {
    /// The candidate label.
    pub name: String,
    /// The prestige-weighted consensus ∈ [0,1].
    pub consensus: f64,
    /// Per-faction posteriors.
    pub posteriors: Vec<f64>,
    /// The core's conclusive flag: `Some(true)` positive, `Some(false)` negative,
    /// `None` open. **The only source of a "conclusive" verdict.**
    pub conclusive: Option<bool>,
    /// Factions publicly disagree?
    pub disagreement: bool,
}

/// The astrobiology view-model.
#[derive(Debug, Clone, PartialEq)]
pub struct AstrobiologyVM {
    /// The evidence meter.
    pub meter: EvidenceMeter,
}

/// Build the evidence meter with the **honesty guard** (FR-UI-1202, SC-009): the UI
/// reports `conclusive` **only** as the core set it — it has no ground-truth input and
/// can never manufacture a conclusive-positive.
pub fn astrobiology(c: &CandidateInput) -> AstrobiologyVM {
    AstrobiologyVM {
        meter: EvidenceMeter {
            candidate: c.name.clone(),
            consensus: c.consensus.clamp(0.0, 1.0),
            posteriors: c.posteriors.clone(),
            // pass-through ONLY: never derive a positive the core did not set.
            conclusive: c.conclusive,
            disagreement: c.disagreement,
        },
    }
}
