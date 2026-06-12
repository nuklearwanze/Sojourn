//! Integer tick clock (FR-CORE-101/102/103). Time is exact by construction:
//! `sim_time = tick × base_tick_ns`, both integers — a century accumulates zero
//! representation drift (SC-011). There is no float anywhere in timekeeping and no
//! wall-clock anywhere in the kernel.

pub mod calendar;

use serde::{Deserialize, Serialize};

/// Monotonic tick index — the only clock the simulation has.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Tick(pub u64);

/// Exact simulated time in nanoseconds since the epoch 2026-01-01T00:00:00 (game UTC).
///
/// `i64` holds ~292 years of nanoseconds — comfortably past the 2126 horizon and any
/// sandbox continuation we permit (sandbox play is capped at +150 years past epoch).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct SimTimeNs(pub i64);

/// Seconds in a non-leap civil year (used only for horizon arithmetic via the calendar).
pub const NS_PER_SEC: i64 = 1_000_000_000;

/// The simulation clock: tick index plus the exact tick duration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimClock {
    /// Current tick.
    pub tick: Tick,
    /// Exact duration of one tick in nanoseconds (config data, default 1 s).
    pub base_tick_ns: u64,
    /// Tick at which the configured horizon is reached (end-of-run signal).
    pub horizon_tick: Tick,
}

impl SimClock {
    /// Build a clock at tick 0 for a run with the given horizon in calendar years.
    ///
    /// The horizon tick lands exactly at `epoch + horizon_years` civil years
    /// (leap days included), i.e. midnight on 1 January of the horizon year.
    pub fn new(base_tick_ns: u64, horizon_years: u16) -> Self {
        let horizon_ns = calendar::ns_at_year_start(2026 + horizon_years);
        // Round up: the horizon fires at the first tick at-or-after the instant.
        // (unsigned div_ceil: the horizon is always after the epoch)
        let horizon_tick = (horizon_ns as u64).div_ceil(base_tick_ns);
        SimClock {
            tick: Tick(0),
            base_tick_ns,
            horizon_tick: Tick(horizon_tick),
        }
    }

    /// Exact simulated time at the current tick.
    pub fn now(&self) -> SimTimeNs {
        self.time_at(self.tick)
    }

    /// Exact simulated time at an arbitrary tick.
    pub fn time_at(&self, tick: Tick) -> SimTimeNs {
        // Checked: century × 1e9 ns/tick fits i64; overflow here is a config bug.
        SimTimeNs(
            (tick.0 as i64)
                .checked_mul(self.base_tick_ns as i64)
                .expect("sim time overflow"),
        )
    }

    /// First tick whose time is at-or-after the given instant.
    pub fn tick_at_or_after(&self, t: SimTimeNs) -> Tick {
        if t.0 <= 0 {
            return Tick(0);
        }
        Tick((t.0 as u64).div_ceil(self.base_tick_ns))
    }

    /// Civil calendar date-time at the current tick.
    pub fn date(&self) -> calendar::DateTime {
        calendar::datetime_at(self.now())
    }
}
