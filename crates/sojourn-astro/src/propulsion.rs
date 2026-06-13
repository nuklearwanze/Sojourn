//! The propulsion interface (contracts/propulsion-interface.md): defined and
//! consumed here, implemented by FA-04 in production. Fixture engines (sourced
//! real-engine-class parameters) ship for tests and scenarios.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Standard gravity (m/s²) for Isp ↔ exhaust-velocity conversion (CODATA g0).
pub const G0: f64 = 9.80665;

/// The per-craft propulsion contract FA-04 implements (FR-ASTRO-501).
pub trait PropulsionEndpoint {
    /// Total mass (kg).
    fn total_mass(&self) -> f64;
    /// Dry mass (kg).
    fn dry_mass(&self) -> f64;
    /// Remaining propellant (kg).
    fn propellant(&self) -> f64;
    /// Effective exhaust velocity (m/s).
    fn exhaust_velocity(&self) -> f64;
    /// Maximum thrust at full throttle and full power (N).
    fn max_thrust(&self) -> f64;
    /// Throttle limits (min, max), each in [0, 1].
    fn throttle_range(&self) -> (f64, f64);
    /// Available electrical power (W).
    fn available_power(&self) -> f64;
    /// Rated electrical power (W) at which `max_thrust` is delivered.
    /// Defaults to `available_power` (no limiting effect) for endpoints that
    /// are not power-limited.
    fn rated_power(&self) -> f64 {
        self.available_power()
    }
    /// Does thrust scale with available power?
    fn power_limited(&self) -> bool;
    /// Drag reference area (m², cannonball).
    fn drag_area(&self) -> f64;
    /// SRP reference area (m², cannonball).
    fn srp_area(&self) -> f64;
    /// Debit propellant (called only by the burn executor).
    fn consume(&mut self, propellant_kg: f64);
}

/// A fixture engine definition (DATA, `data/astro/engines.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineDef {
    /// Engine id referenced by craft.
    pub id: String,
    /// Specific impulse (s).
    pub isp_s: f64,
    /// Maximum thrust (N).
    pub max_thrust_n: f64,
    /// Throttle floor.
    pub throttle_min: f64,
    /// Throttle ceiling.
    pub throttle_max: f64,
    /// Thrust scales with available power when true.
    pub power_limited: bool,
    /// Rated electrical power (W; meaningful when power-limited).
    pub rated_power_w: f64,
    /// Provenance (mandatory).
    pub source: String,
}

impl EngineDef {
    /// Effective exhaust velocity (m/s).
    pub fn exhaust_velocity(&self) -> f64 {
        self.isp_s * G0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnginesFile {
    engines: Vec<EngineDef>,
}

/// Loaded fixture engine catalogue.
#[derive(Debug, Clone)]
pub struct Engines {
    map: BTreeMap<String, EngineDef>,
}

impl Engines {
    /// Load `engines.ron` from a directory.
    pub fn load_dir(dir: &Path) -> Result<Engines, String> {
        let path = dir.join("engines.ron");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;
        let file: EnginesFile = ron::from_str(&text).map_err(|e| format!("engines.ron: {e}"))?;
        let mut map = BTreeMap::new();
        for e in file.engines {
            if e.source.trim().is_empty() {
                return Err(format!("engine '{}' has an empty source field", e.id));
            }
            if e.isp_s <= 0.0 || e.max_thrust_n <= 0.0 {
                return Err(format!(
                    "engine '{}': Isp and thrust must be positive",
                    e.id
                ));
            }
            if !(0.0..=1.0).contains(&e.throttle_min)
                || !(0.0..=1.0).contains(&e.throttle_max)
                || e.throttle_min > e.throttle_max
            {
                return Err(format!("engine '{}': invalid throttle range", e.id));
            }
            if map.insert(e.id.clone(), e).is_some() {
                return Err("duplicate engine id".to_string());
            }
        }
        Ok(Engines { map })
    }

    /// Look up an engine.
    pub fn get(&self, id: &str) -> Option<&EngineDef> {
        self.map.get(id)
    }
}

/// A live binding of an engine definition to a craft's mutable mass state —
/// the fixture implementation of [`PropulsionEndpoint`] (FR-ASTRO-502).
pub struct EngineBinding<'a> {
    /// Engine parameters.
    pub def: &'a EngineDef,
    /// The craft's mass state (mutated by `consume`).
    pub mass: &'a mut crate::craft::MassState,
    /// Commanded throttle (clamped to range by the executor).
    pub throttle: f64,
    /// Available electrical power (W).
    pub available_power_w: f64,
    /// Cannonball areas (m²).
    pub drag_area_m2: f64,
    /// SRP area (m²).
    pub srp_area_m2: f64,
}

impl PropulsionEndpoint for EngineBinding<'_> {
    fn total_mass(&self) -> f64 {
        self.mass.dry + self.mass.propellant
    }
    fn dry_mass(&self) -> f64 {
        self.mass.dry
    }
    fn propellant(&self) -> f64 {
        self.mass.propellant
    }
    fn exhaust_velocity(&self) -> f64 {
        self.def.exhaust_velocity()
    }
    fn max_thrust(&self) -> f64 {
        self.def.max_thrust_n
    }
    fn throttle_range(&self) -> (f64, f64) {
        (self.def.throttle_min, self.def.throttle_max)
    }
    fn available_power(&self) -> f64 {
        self.available_power_w
    }
    fn rated_power(&self) -> f64 {
        self.def.rated_power_w.max(1e-9)
    }
    fn power_limited(&self) -> bool {
        self.def.power_limited
    }
    fn drag_area(&self) -> f64 {
        self.drag_area_m2
    }
    fn srp_area(&self) -> f64 {
        self.srp_area_m2
    }
    fn consume(&mut self, propellant_kg: f64) {
        debug_assert!(propellant_kg >= 0.0);
        self.mass.propellant = (self.mass.propellant - propellant_kg).max(0.0);
    }
}

/// Delivered thrust (N) for an endpoint at its commanded throttle, respecting
/// throttle clamps and power limiting — free thrust is structurally impossible.
pub fn delivered_thrust(ep: &dyn PropulsionEndpoint, commanded_throttle: f64) -> f64 {
    if ep.propellant() <= 0.0 {
        return 0.0;
    }
    let (lo, hi) = ep.throttle_range();
    let throttle = if commanded_throttle <= 0.0 {
        0.0
    } else {
        commanded_throttle.clamp(lo, hi)
    };
    let mut t = ep.max_thrust() * throttle;
    if ep.power_limited() {
        t *= (ep.available_power() / ep.rated_power()).clamp(0.0, 1.0);
    }
    t
}
