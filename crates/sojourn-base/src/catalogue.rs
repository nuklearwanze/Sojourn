//! The module catalogue + tuning params + class templates (DATA, immutable). Each
//! module is typed by its class-specific `params` and references the FA-05 `tech`
//! that researches it. All sourced; no per-base/per-module magic numbers (II).

use crate::ids::ModuleTypeId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

fn normalize(t: &str) -> String {
    t.replace("\r\n", "\n")
}

/// Class-specific module parameters (the class is the variant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleParams {
    /// Crew habitat/accommodation.
    Habitat {
        /// Crew it accommodates.
        crew_accommodation: u32,
        /// Pressurised volume (m³).
        pressurized_volume_m3: f64,
    },
    /// Power generation.
    Power {
        /// Generation at the reference distance (W).
        gen_w: f64,
        /// PV (scales with solar distance) vs constant (RTG/fission).
        solar: bool,
    },
    /// Physico-chemical / closed-loop ECLSS.
    Eclss {
        /// Best closure fraction ∈ [0,1].
        closure_fraction: f64,
        /// Open-loop consumables per crew-day (kg).
        per_crew_day_kg: f64,
        /// Crew this ECLSS can support.
        crew_support: u32,
    },
    /// Bioregenerative food production (the food self-sufficiency loop, G1).
    Greenhouse {
        /// Food produced (kg/day).
        food_kg_per_day: f64,
        /// Crew this can feed.
        crew_fed: u32,
        /// Power demand (W) — also carried in the top-level `power_demand_w`.
        power_demand_w: f64,
    },
    /// Hosts an FA-06 ISRU process (output composed in).
    IsruHost {
        /// The FA-06 process this module hosts.
        process_ref: String,
    },
    /// Science payload.
    Science {
        /// Data rate (arbitrary units).
        data_rate: f64,
    },
    /// Buffer storage for a commodity.
    Storage {
        /// Commodity stored.
        commodity: String,
        /// Buffer capacity (kg).
        buffer_capacity_kg: f64,
    },
    /// On-site manufacturing.
    Manufacturing {
        /// Output commodity.
        output_commodity: String,
        /// Output rate (kg/day).
        rate_kg_per_day: f64,
    },
    /// Passive radiation shielding.
    Shielding {
        /// Material (resolves to a `params.ron` attenuation length).
        material: String,
        /// Areal density (kg/m²).
        areal_density_kg_m2: f64,
    },
}

/// A catalogue module type (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleType {
    /// Stable id.
    pub id: ModuleTypeId,
    /// FA-05 technology that researches it.
    pub tech: String,
    /// Dry mass (kg) — the construction delivered-mass demand.
    pub dry_mass_kg: f64,
    /// Operating power demand (W).
    pub power_demand_w: f64,
    /// Class-specific params.
    pub params: ModuleParams,
    /// Provenance.
    pub source: String,
}

/// A shielding material's mass-attenuation length (kg/m²) for `exp(−ρx/λ)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShieldMaterial {
    /// Material id.
    pub id: String,
    /// Mass-attenuation length λ (kg/m²).
    pub attenuation_length_kg_m2: f64,
    /// Provenance.
    pub source: String,
}

/// Per-crew loop demands (self-sufficiency / embargo) — sourced.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopDemands {
    /// Food demand per crew-day (kg).
    pub food_kg_per_crew_day: f64,
    /// Materials demand per day (kg, base-level).
    pub materials_kg_per_day: f64,
    /// Spares demand per day (kg, base-level).
    pub spares_kg_per_day: f64,
}

/// Tuning params (DATA, `params.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Params {
    /// PV reference distance (m) for solar scaling.
    pub solar_ref_m: f64,
    /// Crew annual dose limit (Sv/yr) — the shielding-shortfall threshold.
    pub crew_dose_limit_sv_yr: f64,
    /// Shielding materials (λ per material).
    pub shield_materials: Vec<ShieldMaterial>,
    /// Construction labour: crew-hours per kg delivered.
    pub crew_hr_per_kg: f64,
    /// Regolith → shielding conversion ratio (kg shield per kg regolith feedstock).
    pub regolith_to_shield_ratio: f64,
    /// Per-crew/base loop demands.
    pub loop_demands: LoopDemands,
    /// Provenance.
    pub source: String,
}

impl Params {
    /// The mass-attenuation length for a material (None if unknown).
    pub fn attenuation_length(&self, material: &str) -> Option<f64> {
        self.shield_materials
            .iter()
            .find(|m| m.id == material)
            .map(|m| m.attenuation_length_kg_m2)
    }
}

/// A base-class template (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassTemplate {
    /// Stable id.
    pub id: String,
    /// Orbital station (vs surface base)?
    pub orbital: bool,
    /// Crewed?
    pub crewed: bool,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModulesFile {
    modules: Vec<ModuleType>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassesFile {
    classes: Vec<ClassTemplate>,
}

/// The loaded, validated catalogue.
#[derive(Debug, Clone)]
pub struct Catalogue {
    /// Module types by id.
    pub modules: BTreeMap<ModuleTypeId, ModuleType>,
    /// Tuning params.
    pub params: Params,
    /// Class templates by id.
    pub classes: BTreeMap<String, ClassTemplate>,
    /// Content hash (saves pin it).
    pub content_hash: [u8; 32],
}

fn looks_like_weapon(id: &str) -> bool {
    let low = id.to_lowercase();
    ["weapon", "missile", "warhead", "gun", "turret", "bomb"]
        .iter()
        .any(|w| low.contains(w))
}

impl Catalogue {
    /// Load and validate from a directory (`data/base`).
    pub fn load_dir(dir: &Path) -> Result<Catalogue, String> {
        let read =
            |n: &str| std::fs::read_to_string(dir.join(n)).map_err(|e| format!("reading {n}: {e}"));
        let mod_txt = read("modules.ron")?;
        let params_txt = read("params.ron")?;
        let classes_txt = read("classes.ron")?;

        let mf: ModulesFile = ron::from_str(&mod_txt).map_err(|e| format!("modules.ron: {e}"))?;
        let params: Params = ron::from_str(&params_txt).map_err(|e| format!("params.ron: {e}"))?;
        let clf: ClassesFile =
            ron::from_str(&classes_txt).map_err(|e| format!("classes.ron: {e}"))?;

        if params.source.trim().is_empty() {
            return Err("params.ron: empty source".into());
        }
        for m in &params.shield_materials {
            if m.source.trim().is_empty() {
                return Err(format!("shield material '{}' has empty source", m.id));
            }
            if m.attenuation_length_kg_m2 <= 0.0 {
                return Err(format!("shield material '{}': λ must be > 0", m.id));
            }
        }
        let mut modules = BTreeMap::new();
        for m in mf.modules {
            if m.source.trim().is_empty() {
                return Err(format!("module '{}' has empty source", m.id.0));
            }
            if m.dry_mass_kg < 0.0 || m.power_demand_w < 0.0 {
                return Err(format!("module '{}': negative mass/power", m.id.0));
            }
            if looks_like_weapon(&m.id.0) {
                return Err(format!(
                    "module '{}' looks like a weapon (Principle IX)",
                    m.id.0
                ));
            }
            match &m.params {
                ModuleParams::Eclss {
                    closure_fraction, ..
                } if !(0.0..=1.0).contains(closure_fraction) => {
                    return Err(format!("module '{}': closure ∉ [0,1]", m.id.0));
                }
                ModuleParams::Shielding { material, .. }
                    if params.attenuation_length(material).is_none() =>
                {
                    return Err(format!(
                        "module '{}': shielding material '{material}' has no attenuation length",
                        m.id.0
                    ));
                }
                _ => {}
            }
            modules.insert(m.id.clone(), m);
        }
        let mut classes = BTreeMap::new();
        for c in clf.classes {
            if c.source.trim().is_empty() {
                return Err(format!("class '{}' has empty source", c.id));
            }
            classes.insert(c.id.clone(), c);
        }

        let canonical = format!(
            "{}\0{}\0{}",
            normalize(&mod_txt),
            normalize(&params_txt),
            normalize(&classes_txt)
        );
        let content_hash = *blake3::hash(canonical.as_bytes()).as_bytes();
        Ok(Catalogue {
            modules,
            params,
            classes,
            content_hash,
        })
    }

    /// A module type by id.
    pub fn module(&self, id: &ModuleTypeId) -> Option<&ModuleType> {
        self.modules.get(id)
    }
}
