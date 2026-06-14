//! SI-unit formatting (R6, FR-UI-1401). Pure, tested formatters — **SI only, no
//! imperial** — with consistent formatting. The single place the UI turns a raw
//! number into a player-facing string, so a unit audit is mechanical (SC-007).

/// A physical quantity kind, used to pick the SI unit + scaling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantity {
    /// Length / distance (m, km).
    Length,
    /// Velocity / Δv (m/s, km/s).
    Velocity,
    /// Mass (kg, t).
    Mass,
    /// Power (W, kW, MW).
    Power,
    /// Energy (J, kJ, MJ).
    Energy,
    /// Temperature (K).
    Temperature,
    /// Radiation dose (Sv, mSv).
    Dose,
    /// Pressure (Pa, kPa).
    Pressure,
    /// Currency (a plain magnitude with a $ prefix; SI-style grouping).
    Currency,
    /// A dimensionless count/ratio.
    Scalar,
    /// Duration (s, days, years).
    Duration,
}

fn round3(x: f64) -> f64 {
    libm::round(x * 1000.0) / 1000.0
}

/// Format a raw SI value (always supplied in base SI units) into a player-facing
/// string with an explicit unit. **No imperial units are ever produced.**
pub fn format(q: Quantity, si_value: f64) -> String {
    let v = si_value;
    let a = v.abs();
    match q {
        Quantity::Length => {
            if a >= 1000.0 {
                format!("{} km", round3(v / 1000.0))
            } else {
                format!("{} m", round3(v))
            }
        }
        Quantity::Velocity => {
            if a >= 1000.0 {
                format!("{} km/s", round3(v / 1000.0))
            } else {
                format!("{} m/s", round3(v))
            }
        }
        Quantity::Mass => {
            if a >= 1000.0 {
                format!("{} t", round3(v / 1000.0))
            } else {
                format!("{} kg", round3(v))
            }
        }
        Quantity::Power => scale_si(v, "W"),
        Quantity::Energy => scale_si(v, "J"),
        Quantity::Temperature => format!("{} K", round3(v)),
        Quantity::Dose => {
            if a < 1.0 {
                format!("{} mSv", round3(v * 1000.0))
            } else {
                format!("{} Sv", round3(v))
            }
        }
        Quantity::Pressure => scale_si(v, "Pa"),
        Quantity::Currency => format!("${}", group(round3(v))),
        Quantity::Scalar => format!("{}", round3(v)),
        Quantity::Duration => format_duration(v),
    }
}

/// Apply an SI magnitude prefix (k/M/G) to a base-unit value.
fn scale_si(v: f64, unit: &str) -> String {
    let a = v.abs();
    if a >= 1.0e9 {
        format!("{} G{unit}", round3(v / 1.0e9))
    } else if a >= 1.0e6 {
        format!("{} M{unit}", round3(v / 1.0e6))
    } else if a >= 1.0e3 {
        format!("{} k{unit}", round3(v / 1.0e3))
    } else {
        format!("{} {unit}", round3(v))
    }
}

/// Seconds → a readable SI-based duration (s / days / years).
fn format_duration(seconds: f64) -> String {
    let a = seconds.abs();
    if a >= 365.25 * 86400.0 {
        format!("{} yr", round3(seconds / (365.25 * 86400.0)))
    } else if a >= 86400.0 {
        format!("{} d", round3(seconds / 86400.0))
    } else {
        format!("{} s", round3(seconds))
    }
}

/// Group the integer part with thin separators (locale-neutral, SI-style).
fn group(v: f64) -> String {
    let neg = v < 0.0;
    let int = libm::trunc(v.abs()) as u64;
    let frac = v.abs() - int as f64;
    let s = int.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(*b as char);
    }
    if frac > 1e-9 {
        out.push_str(&format!(".{}", ((frac * 1000.0).round() as u64)));
    }
    if neg { format!("-{out}") } else { out }
}
