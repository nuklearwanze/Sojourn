//! Porkchop solving (FR-ASTRO-403): departure × arrival grids over the railed
//! ephemerides — windows emerge from geometry, never from scripted data.

use super::lambert;
use crate::bodies::{BodyId, Catalog, rails::BodyStates};
use crate::math::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Grid query (ticks are kernel ticks; base tick = 1 s).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PorkchopQuery {
    /// Departure body.
    pub from: BodyId,
    /// Arrival body.
    pub to: BodyId,
    /// Departure span (ticks, inclusive).
    pub depart_span: (u64, u64),
    /// Time-of-flight span (s).
    pub tof_span: (f64, f64),
    /// Grid resolution (departure cells, tof cells); each ≤ 64.
    pub grid: (u16, u16),
    /// Revolutions to attempt per cell (0 = direct only).
    pub max_revs: u8,
}

/// One grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PorkchopCell {
    /// Departure tick.
    pub depart_tick: u64,
    /// Time of flight (s).
    pub tof_s: f64,
    /// Total Δv (departure v∞ + arrival v∞ magnitudes; m/s). NaN-free: only
    /// meaningful when `solvable`.
    pub dv_total: f64,
    /// Departure characteristic energy C3 (km²/s² convention in m²/s²).
    pub c3: f64,
    /// Solvable with the requested geometry?
    pub solvable: bool,
}

/// A solved grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PorkchopGrid {
    /// Cells in row-major (departure-major) order.
    pub cells: Vec<PorkchopCell>,
    /// The query that produced it.
    pub query: PorkchopQuery,
}

impl PorkchopGrid {
    /// The minimum-Δv solvable cell, if any.
    pub fn best(&self) -> Option<&PorkchopCell> {
        self.cells.iter().filter(|c| c.solvable).min_by(|a, b| {
            a.dv_total
                .partial_cmp(&b.dv_total)
                .expect("no NaN in solvable")
        })
    }
}

/// Solve a porkchop grid between two catalogue bodies (heliocentric legs).
pub fn solve(
    catalog: &Catalog,
    motions: &BTreeMap<BodyId, crate::bodies::BodyMotion>,
    q: &PorkchopQuery,
    max_iter: u32,
    tol: f64,
) -> PorkchopGrid {
    let root = catalog.root();
    let mu = catalog.body(root).expect("root").mu;
    let (nd, nt) = (q.grid.0.clamp(1, 64), q.grid.1.clamp(1, 64));
    let mut cells = Vec::with_capacity(usize::from(nd) * usize::from(nt));

    for i in 0..nd {
        let depart_tick = lerp_u64(q.depart_span.0, q.depart_span.1, i, nd);
        let t1_ns = depart_tick as i64 * 1_000_000_000;
        let mut s1ctx = BodyStates::new(catalog, motions, t1_ns);
        let from = s1ctx.absolute(q.from);
        for j in 0..nt {
            let tof = lerp_f64(q.tof_span.0, q.tof_span.1, j, nt);
            let t2_ns = t1_ns + (tof * 1e9) as i64;
            let mut s2ctx = BodyStates::new(catalog, motions, t2_ns);
            let to = s2ctx.absolute(q.to);

            let mut solved = None;
            for revs in 0..=q.max_revs {
                if let Some(sol) = lambert::solve(mu, from.r, to.r, tof, revs, max_iter, tol) {
                    let vinf_dep = (sol.v1 - from.v).norm();
                    let vinf_arr = (sol.v2 - to.v).norm();
                    let dv = vinf_dep + vinf_arr;
                    if solved.map(|(best, _)| dv < best).unwrap_or(true) {
                        solved = Some((dv, vinf_dep));
                    }
                }
            }
            cells.push(match solved {
                Some((dv, vinf_dep)) => PorkchopCell {
                    depart_tick,
                    tof_s: tof,
                    dv_total: dv,
                    c3: vinf_dep * vinf_dep,
                    solvable: true,
                },
                None => PorkchopCell {
                    depart_tick,
                    tof_s: tof,
                    dv_total: 0.0,
                    c3: 0.0,
                    solvable: false,
                },
            });
        }
    }
    PorkchopGrid {
        cells,
        query: q.clone(),
    }
}

/// Convert a solvable cell into a heliocentric departure-burn plan for a craft
/// state: the Δv (world frame) to enter the transfer at the cell's departure
/// time, assuming the craft is at the departure body's state (escape spirals
/// and parking-orbit ejection refine this in later slices).
pub fn cell_departure_dv(
    catalog: &Catalog,
    motions: &BTreeMap<BodyId, crate::bodies::BodyMotion>,
    q: &PorkchopQuery,
    cell: &PorkchopCell,
    max_iter: u32,
    tol: f64,
) -> Option<Vec3> {
    if !cell.solvable {
        return None;
    }
    let root = catalog.root();
    let mu = catalog.body(root).expect("root").mu;
    let t1_ns = cell.depart_tick as i64 * 1_000_000_000;
    let mut s1 = BodyStates::new(catalog, motions, t1_ns);
    let from = s1.absolute(q.from);
    let t2_ns = t1_ns + (cell.tof_s * 1e9) as i64;
    let mut s2 = BodyStates::new(catalog, motions, t2_ns);
    let to = s2.absolute(q.to);
    let sol = lambert::solve(mu, from.r, to.r, cell.tof_s, 0, max_iter, tol)?;
    Some(sol.v1 - from.v)
}

fn lerp_u64(lo: u64, hi: u64, i: u16, n: u16) -> u64 {
    if n <= 1 {
        return lo;
    }
    lo + (hi - lo) * u64::from(i) / u64::from(n - 1)
}

fn lerp_f64(lo: f64, hi: f64, i: u16, n: u16) -> f64 {
    if n <= 1 {
        return lo;
    }
    lo + (hi - lo) * f64::from(i) / f64::from(n - 1)
}
