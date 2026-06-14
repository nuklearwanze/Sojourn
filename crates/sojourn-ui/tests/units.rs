//! US14 — SI-unit formatting (no imperial; consistent) (FR-UI-1401, SC-007).

use sojourn_ui::units::{Quantity, format};

#[test]
fn si_formatters_scale_and_label_correctly() {
    assert_eq!(format(Quantity::Velocity, 6200.0), "6.2 km/s");
    assert_eq!(format(Quantity::Velocity, 250.0), "250 m/s");
    assert_eq!(format(Quantity::Mass, 1500.0), "1.5 t");
    assert_eq!(format(Quantity::Mass, 42.0), "42 kg");
    assert_eq!(format(Quantity::Power, 40000.0), "40 kW");
    assert_eq!(format(Quantity::Power, 2.5e6), "2.5 MW");
    assert_eq!(format(Quantity::Temperature, 288.0), "288 K");
    assert_eq!(format(Quantity::Dose, 0.003), "3 mSv");
    assert_eq!(format(Quantity::Dose, 1.5), "1.5 Sv");
    assert_eq!(format(Quantity::Duration, 86400.0 * 30.0), "30 d");
}

#[test]
fn no_imperial_units_are_produced() {
    // A representative sweep — none of the SI formatters emit an imperial token.
    let banned = ["ft", "mph", "lb", "mile", "psi", "°F", "inch", "gallon"];
    let samples = [
        format(Quantity::Velocity, 6200.0),
        format(Quantity::Mass, 1500.0),
        format(Quantity::Power, 40000.0),
        format(Quantity::Energy, 5.0e6),
        format(Quantity::Temperature, 288.0),
        format(Quantity::Dose, 0.5),
        format(Quantity::Pressure, 101325.0),
        format(Quantity::Currency, 25_000_000_000.0),
    ];
    for s in &samples {
        for b in &banned {
            assert!(!s.contains(b), "imperial token '{b}' in '{s}'");
        }
    }
}
