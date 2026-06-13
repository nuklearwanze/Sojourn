//! Propulsion family models → the FA-02 endpoint (FR-VD-301/302/303, R5). An
//! Engine component maps to a [`sojourn_astro::EngineDef`] carried inline into the
//! propagator; electric propulsion is power-limited; nuclear reactor/radiator mass
//! is first-class dry mass (mass model).

use crate::catalogue::{Component, ComponentParams};
use sojourn_astro::propulsion::EngineDef;

/// Build the FA-02 `EngineDef` (the inline endpoint params) for an engine component.
pub fn engine_def(c: &Component) -> Option<EngineDef> {
    match &c.params {
        ComponentParams::Engine {
            isp_s,
            max_thrust_n,
            throttle,
            power_limited,
            rated_power_w,
            ..
        } => Some(EngineDef {
            id: c.id.0.clone(),
            isp_s: *isp_s,
            max_thrust_n: *max_thrust_n,
            throttle_min: throttle.0,
            throttle_max: throttle.1,
            power_limited: *power_limited,
            rated_power_w: *rated_power_w,
            source: c.source.clone(),
        }),
        _ => None,
    }
}

/// Max delivered thrust (N) at full throttle — power-limited for EP given the
/// vehicle's available electrical power (free thrust is structurally impossible).
pub fn delivered_max_thrust(c: &Component, available_power_w: f64) -> f64 {
    match &c.params {
        ComponentParams::Engine {
            max_thrust_n,
            power_limited,
            rated_power_w,
            ..
        } => {
            let mut t = *max_thrust_n;
            if *power_limited {
                t *= (available_power_w / rated_power_w.max(1e-9)).clamp(0.0, 1.0);
            }
            t
        }
        _ => 0.0,
    }
}

/// The engine's rated power draw (W) — its demand on the power balance when active.
pub fn engine_power_demand(c: &Component) -> f64 {
    match &c.params {
        ComponentParams::Engine {
            power_limited,
            rated_power_w,
            ..
        } if *power_limited => *rated_power_w,
        _ => 0.0,
    }
}

/// The engine's waste heat (W) — its load on the thermal balance.
pub fn engine_waste_heat(c: &Component) -> f64 {
    match &c.params {
        ComponentParams::Engine { waste_heat_w, .. } => *waste_heat_w,
        _ => 0.0,
    }
}
