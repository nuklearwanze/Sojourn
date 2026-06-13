//! The component catalogue + propulsion family models + tuning params + class
//! templates (FR-VD-101/301, immutable module data). Every component is typed by
//! its class-specific `params` and references the FA-05 `tech` that researches it.
//! All sourced; no per-design magic numbers (Principle II).

use crate::ids::ComponentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

fn normalize(t: &str) -> String {
    t.replace("\r\n", "\n")
}

/// Propulsion family (design/02-TECH-TREE.md §B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropFamily {
    /// Chemical.
    Chemical,
    /// Electric (ion/Hall/MPD/VASIMR/electrospray) — power-limited.
    Electric,
    /// Nuclear-thermal.
    NuclearThermal,
    /// Nuclear-electric — reactor + radiators as first-class mass.
    NuclearElectric,
    /// Gated frontier.
    Frontier,
}

/// Class-specific component parameters (the class is the variant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentParams {
    /// Primary structure rated to a thrust load.
    Structure {
        /// Maximum thrust the structure can take (N).
        thrust_limit_n: f64,
    },
    /// Propellant tank.
    Tank {
        /// Propellant capacity (kg).
        capacity_kg: f64,
        /// Cryo boil-off fraction per day.
        boiloff_frac_per_day: f64,
    },
    /// Power generation.
    Power {
        /// Generation at the reference distance (W).
        gen_w: f64,
        /// PV (scales with solar distance) vs constant (RTG/fission).
        solar: bool,
    },
    /// Thermal radiator.
    Thermal {
        /// Heat rejection per kg of radiator (W/kg).
        reject_w_per_kg: f64,
    },
    /// Avionics/GNC.
    Avionics {
        /// Power demand (W).
        power_demand_w: f64,
    },
    /// Comms.
    Comms {
        /// Power demand (W).
        power_demand_w: f64,
    },
    /// Life support / crew accommodation.
    LifeSupport {
        /// ECLSS closure fraction (0 open-loop … 1 fully closed).
        closure_fraction: f64,
        /// Open-loop consumables per crew per day (kg).
        per_crew_day_kg: f64,
        /// Crew capacity.
        crew: u32,
        /// Radiation-shield mass (kg) carried.
        shield_kg: f64,
        /// Power demand (W).
        power_demand_w: f64,
    },
    /// Science/cargo payload.
    Payload {
        /// Power demand (W).
        power_demand_w: f64,
        /// Waste heat (W).
        waste_heat_w: f64,
    },
    /// EDL kit (heat shield / parachutes / retroprop).
    EdlKit {
        /// Peak heat flux the shield survives (W/m²).
        heat_shield_w_m2: f64,
        /// Reference area (m²) for the ballistic coefficient.
        area_m2: f64,
    },
    /// Landing gear.
    Landing,
    /// Reaction control.
    Rcs,
    /// Docking/berthing.
    Docking,
    /// A propulsion engine (the FA-02 endpoint source).
    Engine {
        /// Family.
        family: PropFamily,
        /// Specific impulse (s).
        isp_s: f64,
        /// Maximum thrust (N).
        max_thrust_n: f64,
        /// Throttle limits.
        throttle: (f64, f64),
        /// Power-limited (Electric/NuclearElectric)?
        power_limited: bool,
        /// Rated electrical power (W).
        rated_power_w: f64,
        /// Reactor + shielding mass (kg) for nuclear families (first-class dry mass).
        reactor_mass_kg: f64,
        /// Waste heat to reject (W) → radiator mass.
        waste_heat_w: f64,
        /// Cryo boil-off fraction per day.
        boiloff_frac_per_day: f64,
    },
}

/// One catalogued component.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    /// Identity.
    pub id: ComponentId,
    /// Researching technology (FA-05 key; availability/maturity via the provider).
    pub tech: String,
    /// Structural/component dry mass (kg) — excludes derived reactor/radiator mass.
    pub dry_mass_kg: f64,
    /// Class-specific parameters.
    pub params: ComponentParams,
    /// Provenance (mandatory, Principle I).
    pub source: String,
}

/// Tuning parameters (DATA, `params.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Params {
    /// Solar flux reference distance (m) for PV scaling (1 AU).
    pub solar_ref_m: f64,
    /// Unit cost per kg of dry mass (abstract).
    pub cost_per_kg: f64,
    /// Maturity cost multiplier at TRL 6 (immature is pricier).
    pub immaturity_cost_mult: f64,
    /// Learning-curve exponent (unit cost ∝ count^(-b)).
    pub learning_exponent: f64,
    /// Build days per kg of dry mass.
    pub build_days_per_kg: f64,
    /// Provenance.
    pub source: String,
}

/// A vehicle-class template (required component classes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassTemplate {
    /// Class id (e.g. `"lander"`).
    pub id: String,
    /// Whether a lander check (T/W vs local g) applies.
    pub lander: bool,
    /// Whether crewed sizing applies.
    pub crewed: bool,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentsFile {
    components: Vec<Component>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassesFile {
    classes: Vec<ClassTemplate>,
}

/// The loaded, validated catalogue.
#[derive(Debug, Clone)]
pub struct Catalogue {
    /// Components by id.
    pub components: BTreeMap<ComponentId, Component>,
    /// Tuning params.
    pub params: Params,
    /// Class templates by id.
    pub classes: BTreeMap<String, ClassTemplate>,
    /// Content hash (saves pin it).
    pub content_hash: [u8; 32],
}

impl Catalogue {
    /// Load and validate from a directory (`data/vehicle`).
    pub fn load_dir(dir: &Path) -> Result<Catalogue, String> {
        let read =
            |n: &str| std::fs::read_to_string(dir.join(n)).map_err(|e| format!("reading {n}: {e}"));
        let comp_txt = read("components.ron")?;
        let params_txt = read("params.ron")?;
        let classes_txt = read("classes.ron")?;

        let cf: ComponentsFile =
            ron::from_str(&comp_txt).map_err(|e| format!("components.ron: {e}"))?;
        let params: Params = ron::from_str(&params_txt).map_err(|e| format!("params.ron: {e}"))?;
        let clf: ClassesFile =
            ron::from_str(&classes_txt).map_err(|e| format!("classes.ron: {e}"))?;

        let mut components = BTreeMap::new();
        for c in cf.components {
            if c.source.trim().is_empty() {
                return Err(format!("component '{}' has empty source", c.id.0));
            }
            if c.dry_mass_kg < 0.0 {
                return Err(format!("component '{}': negative dry mass", c.id.0));
            }
            // Principle IX: no combat/weapons components.
            let low = c.id.0.to_lowercase();
            if low.contains("weapon") || low.contains("missile") || low.contains("gun") {
                return Err(format!(
                    "component '{}' looks like a weapon (Principle IX)",
                    c.id.0
                ));
            }
            if let ComponentParams::Engine {
                isp_s,
                max_thrust_n,
                ..
            } = &c.params
                && (*isp_s <= 0.0 || *max_thrust_n <= 0.0)
            {
                return Err(format!(
                    "engine '{}': Isp and thrust must be positive",
                    c.id.0
                ));
            }
            components.insert(c.id.clone(), c);
        }
        if params.source.trim().is_empty() {
            return Err("params.ron: empty source".into());
        }
        let mut classes = BTreeMap::new();
        for t in clf.classes {
            if t.source.trim().is_empty() {
                return Err(format!("class '{}' has empty source", t.id));
            }
            classes.insert(t.id.clone(), t);
        }

        let canonical = format!(
            "{}\0{}\0{}",
            normalize(&comp_txt),
            normalize(&params_txt),
            normalize(&classes_txt)
        );
        let content_hash = *blake3::hash(canonical.as_bytes()).as_bytes();
        Ok(Catalogue {
            components,
            params,
            classes,
            content_hash,
        })
    }

    /// A component by id.
    pub fn component(&self, id: &ComponentId) -> Option<&Component> {
        self.components.get(id)
    }
}
