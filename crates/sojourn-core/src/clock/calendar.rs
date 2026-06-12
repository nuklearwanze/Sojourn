//! Deterministic proleptic-Gregorian civil calendar (FR-CORE-103, research R6).
//!
//! Pure functions only — no timezone, no locale, no external date crate. Epoch is
//! 2026-01-01T00:00:00 game-UTC. Day arithmetic uses Howard Hinnant's published
//! `days_from_civil` algorithm, exhaustively unit-tested through 2126 and beyond.

use super::{NS_PER_SEC, SimTimeNs};
use serde::{Deserialize, Serialize};

/// Civil date-time (game UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateTime {
    /// Calendar year (e.g. 2026).
    pub year: i32,
    /// Month 1–12.
    pub month: u8,
    /// Day of month 1–31.
    pub day: u8,
    /// Hour 0–23.
    pub hour: u8,
    /// Minute 0–59.
    pub minute: u8,
    /// Second 0–59.
    pub second: u8,
}

/// Days from civil epoch 1970-01-01 (Hinnant's algorithm, valid far beyond our span).
fn days_from_civil(y: i32, m: u8, d: u8) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = i64::from((m + 9) % 12); // Mar=0 .. Feb=11
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// Inverse of [`days_from_civil`].
fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8; // [1, 12]
    ((y + i64::from(m <= 2)) as i32, m, d)
}

/// Days from the game epoch (2026-01-01) to 1970-01-01.
fn epoch_day_offset() -> i64 {
    days_from_civil(2026, 1, 1)
}

/// Is `year` a Gregorian leap year?
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Nanoseconds from the game epoch to midnight, 1 January of `year`.
pub fn ns_at_year_start(year: u16) -> i64 {
    let days = days_from_civil(i32::from(year), 1, 1) - epoch_day_offset();
    days * 86_400 * NS_PER_SEC
}

/// Civil date-time at an exact simulated instant.
pub fn datetime_at(t: SimTimeNs) -> DateTime {
    let total_secs = t.0.div_euclid(NS_PER_SEC);
    let days = total_secs.div_euclid(86_400);
    let secs_of_day = total_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days + epoch_day_offset());
    DateTime {
        year,
        month,
        day,
        hour: (secs_of_day / 3600) as u8,
        minute: ((secs_of_day % 3600) / 60) as u8,
        second: (secs_of_day % 60) as u8,
    }
}

/// Nanoseconds from the game epoch to midnight on the given civil date.
pub fn ns_at_date(year: i32, month: u8, day: u8) -> i64 {
    let days = days_from_civil(year, month, day) - epoch_day_offset();
    days * 86_400 * NS_PER_SEC
}

/// Calendar year containing the instant (fiscal year == calendar year in v1 kernel;
/// faction-specific fiscal offsets are content-slice data).
pub fn year_of(t: SimTimeNs) -> i32 {
    datetime_at(t).year
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_jan_1_2026() {
        let d = datetime_at(SimTimeNs(0));
        assert_eq!(
            (d.year, d.month, d.day, d.hour, d.minute, d.second),
            (2026, 1, 1, 0, 0, 0)
        );
    }

    #[test]
    fn leap_years_through_span() {
        // 2026 span includes century rule at 2100 (NOT leap) and 2096/2104 (leap).
        assert!(!is_leap_year(2026));
        assert!(is_leap_year(2028));
        assert!(is_leap_year(2096));
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(2104));
        assert!(is_leap_year(2048));
        assert!(!is_leap_year(2126));
    }

    #[test]
    fn feb29_exists_only_in_leap_years() {
        // 2028-02-29 valid; the day after 2100-02-28 is 2100-03-01.
        let ns = ns_at_date(2028, 2, 29);
        let d = datetime_at(SimTimeNs(ns));
        assert_eq!((d.year, d.month, d.day), (2028, 2, 29));
        let after_feb28_2100 = ns_at_date(2100, 2, 28) + 86_400 * NS_PER_SEC;
        let d = datetime_at(SimTimeNs(after_feb28_2100));
        assert_eq!((d.year, d.month, d.day), (2100, 3, 1));
    }

    #[test]
    fn year_boundaries_exact() {
        for year in [2027u16, 2050, 2076, 2100, 2101, 2126] {
            let ns = ns_at_year_start(year);
            let d = datetime_at(SimTimeNs(ns));
            assert_eq!(
                (d.year, d.month, d.day, d.hour),
                (i32::from(year), 1, 1, 0),
                "year {year}"
            );
            // One nanosecond earlier is 23:59:59 of 31 December of the prior year.
            let before = datetime_at(SimTimeNs(ns - NS_PER_SEC));
            assert_eq!(
                (
                    before.year,
                    before.month,
                    before.day,
                    before.hour,
                    before.minute,
                    before.second
                ),
                (i32::from(year) - 1, 12, 31, 23, 59, 59)
            );
        }
    }

    #[test]
    fn round_trip_every_day_of_century() {
        // Walk every single day 2026-01-01 .. 2126-12-31 and round-trip it.
        let mut day_count = 0i64;
        for year in 2026..=2126 {
            for month in 1u8..=12 {
                let mdays = match month {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                    4 | 6 | 9 | 11 => 30,
                    2 if is_leap_year(year) => 29,
                    _ => 28,
                };
                for day in 1..=mdays {
                    let ns = ns_at_date(year, month, day);
                    let d = datetime_at(SimTimeNs(ns));
                    assert_eq!((d.year, d.month, d.day), (year, month, day));
                    day_count += 1;
                }
            }
        }
        // 101 years with 25 leap days (2028..=2124 step 4, minus 2100): 101*365 + 24
        assert_eq!(day_count, 101 * 365 + 24);
    }

    #[test]
    fn century_of_ticks_has_zero_drift() {
        // SC-011: tick × base never drifts. 100 years at 1 s ticks.
        let base: i64 = 1_000_000_000;
        let horizon = ns_at_year_start(2126);
        assert_eq!(horizon % base, 0, "century is an exact number of 1 s ticks");
        let ticks = horizon / base;
        assert_eq!(ticks * base, horizon, "exact integer reconstruction");
        let d = datetime_at(SimTimeNs(ticks * base));
        assert_eq!((d.year, d.month, d.day), (2126, 1, 1));
    }
}
