//! The data shapes for the eight bespoke widgets (R11, FR §4). Each is a renderer-
//! agnostic struct the view-model builds from core reads; the egui painter in
//! `sojourn-ui-desktop` draws it. Keeping the shape here keeps the widget data
//! **testable** while the paint stays thin.

/// An interactive Δv/TOF/C3 contour field for a transfer window (porkchop plot).
#[derive(Debug, Clone, PartialEq)]
pub struct PorkchopField {
    /// Departure-date samples (days from the window start).
    pub departures: Vec<f64>,
    /// Arrival-date samples (days from the window start).
    pub arrivals: Vec<f64>,
    /// Δv (m/s) per (departure, arrival) cell, row-major over `departures × arrivals`.
    pub dv: Vec<f64>,
    /// The minimum-Δv cell `(departure_idx, arrival_idx)`.
    pub optimum: Option<(usize, usize)>,
}

/// A Δv ladder: stage/segment Δv vs the vehicle's available Δv (live).
#[derive(Debug, Clone, PartialEq)]
pub struct DvLadder {
    /// Named Δv segments required (m/s).
    pub segments: Vec<(String, f64)>,
    /// The vehicle's available Δv (m/s).
    pub available: f64,
}

impl DvLadder {
    /// Total required Δv.
    pub fn required(&self) -> f64 {
        self.segments.iter().map(|(_, v)| v).sum()
    }
    /// True iff the plan fits the vehicle's available Δv (FR-UI-403).
    pub fn feasible(&self) -> bool {
        self.required() <= self.available
    }
}

/// A TRL ladder with the current rung + test/risk overlay.
#[derive(Debug, Clone, PartialEq)]
pub struct TrlLadder {
    /// Current TRL (1–9).
    pub current: u8,
    /// Test-campaign progress ∈ [0,1].
    pub test_progress: f64,
    /// Risk index ∈ [0,1].
    pub risk: f64,
    /// Dead-end warning present?
    pub dead_end: bool,
}

/// Domain Understanding-Level bars with a world-tide ghost + insight pressure.
#[derive(Debug, Clone, PartialEq)]
pub struct UnderstandingBars {
    /// Per-domain `(name, UL ∈ [0,1], world-tide ghost ∈ [0,1], insight-pressure ∈ [0,1])`.
    pub domains: Vec<(String, f64, f64, f64)>,
}

/// Inventory grouped by Δv-addressed location (the resource-by-location ledger).
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceLedger {
    /// `(location, commodity, mass_kg)` rows.
    pub rows: Vec<(String, String, f64)>,
}

/// The logistics graph: nodes (dynamical locations) + edges priced in Δv/TOF.
#[derive(Debug, Clone, PartialEq)]
pub struct LogisticsGraph {
    /// Node labels.
    pub nodes: Vec<String>,
    /// `(from_idx, to_idx, dv_m_s, tof_days)`.
    pub edges: Vec<(usize, usize, f64, f64)>,
}

/// A base schematic with live emergent-property gauges.
#[derive(Debug, Clone, PartialEq)]
pub struct BaseSchematic {
    /// Module labels in the layout.
    pub modules: Vec<String>,
    /// `(gauge_name, value, ok_threshold)` — power margin, ECLSS closure, population
    /// cap, sustainability index.
    pub gauges: Vec<(String, f64, f64)>,
}

/// The astrobiology evidence meter (probabilistic, multi-stage, per candidate world).
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMeter {
    /// The candidate world label.
    pub candidate: String,
    /// The prestige-weighted community consensus ∈ [0,1].
    pub consensus: f64,
    /// Per-faction posteriors ∈ [0,1] (may publicly disagree).
    pub posteriors: Vec<f64>,
    /// Conclusive status: `Some(true)` positive, `Some(false)` negative, `None` open.
    pub conclusive: Option<bool>,
    /// Factions publicly disagree?
    pub disagreement: bool,
}
