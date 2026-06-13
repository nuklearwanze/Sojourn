//! The read-only design-query surface (FR-VD-801): pure functions over a
//! `DesignSnapshot` composing the design (vehicle slice), a **maturity map** (the
//! C1 decoupling — FA-05 in production, a stub in tests) and a supplied solar
//! distance/gravity. Derived outputs are computed here, never stored (R3).

use crate::catalogue::{Catalogue, ComponentParams};
use crate::cost::{self, CostEstimate};
use crate::deltav;
use crate::design::VehicleDesign;
use crate::edl::{self, EdlSuitability};
use crate::guards::{self, GuardInputs, RedFlag};
use crate::ids::{ComponentId, DesignId, FactionId};
use crate::lifesupport::{self, LifeSupportSizing};
use crate::mass;
use crate::module::VehicleSlice;
use crate::power::{self, PowerBalance};
use crate::propulsion;
use crate::reliability::{self, ReliabilityResult};
use crate::thermal::{self, ThermalBalance};
use crate::trace::TraceTree;
use serde::{Deserialize, Serialize};
use sojourn_astro::propulsion::EngineDef;
use sojourn_core::{CoreError, SimCore};
use std::collections::BTreeMap;

/// A component technology's maturity (the FA-05 `maturity()` shape, decoupled).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ComponentMaturity {
    /// Per-use reliability ∈ [0,1].
    pub reliability: f64,
    /// Current TRL.
    pub trl: u8,
    /// Flyable (TRL ≥ 6)?
    pub flyable: bool,
}

impl ComponentMaturity {
    /// An unresearched/unavailable technology.
    pub fn unavailable() -> ComponentMaturity {
        ComponentMaturity {
            reliability: 0.0,
            trl: 0,
            flyable: false,
        }
    }
}

/// The full derived performance of a design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedOutputs {
    /// Dry mass (kg).
    pub dry_mass: f64,
    /// Wet mass (kg).
    pub wet_mass: f64,
    /// Propellant (kg).
    pub propellant: f64,
    /// Per-stage Δv (m/s).
    pub stage_dvs: Vec<f64>,
    /// Total Δv (m/s).
    pub total_dv: f64,
    /// First-stage thrust (N).
    pub max_thrust: f64,
    /// T/W at the stated target body (if any).
    pub tw_at_target: Option<f64>,
    /// Power balance.
    pub power: PowerBalance,
    /// Thermal balance.
    pub thermal: ThermalBalance,
    /// Composed reliability.
    pub reliability: f64,
    /// Static life-support sizing.
    pub life_support: LifeSupportSizing,
    /// EDL/landing suitability (if a target body is stated).
    pub edl: Option<EdlSuitability>,
    /// Physical cost/build estimate.
    pub cost: CostEstimate,
}

/// A truth-free, faction-scoped snapshot for design queries.
pub struct DesignSnapshot {
    /// Tick.
    pub tick: u64,
    designs: BTreeMap<DesignId, VehicleDesign>,
    catalogue: Catalogue,
    maturity: BTreeMap<String, ComponentMaturity>,
}

impl DesignSnapshot {
    /// Extract from a running kernel; `maturity` is the per-tech maturity map the
    /// host composes from FA-05 (or a stub in tests).
    pub fn from_core(
        core: &SimCore,
        module: &crate::module::VehicleModule,
        maturity: BTreeMap<String, ComponentMaturity>,
    ) -> Result<DesignSnapshot, CoreError> {
        let tick = core.status().tick;
        core.with_slice("vehicle", |slice| {
            let s = slice
                .as_any()
                .downcast_ref::<VehicleSlice>()
                .expect("vehicle slice type");
            DesignSnapshot {
                tick,
                designs: s.designs.clone(),
                catalogue: module.catalogue.clone(),
                maturity: maturity.clone(),
            }
        })
    }

    /// Build directly from parts (tests).
    pub fn new(
        designs: BTreeMap<DesignId, VehicleDesign>,
        catalogue: Catalogue,
        maturity: BTreeMap<String, ComponentMaturity>,
    ) -> DesignSnapshot {
        DesignSnapshot {
            tick: 0,
            designs,
            catalogue,
            maturity,
        }
    }

    fn design(&self, id: DesignId) -> Option<&VehicleDesign> {
        self.designs.get(&id)
    }

    fn maturity_of(&self, tech: &str) -> ComponentMaturity {
        self.maturity
            .get(tech)
            .copied()
            .unwrap_or_else(ComponentMaturity::unavailable)
    }

    /// Availability of a component for a faction (query-time gating, R4).
    pub fn availability(
        &self,
        component: &ComponentId,
    ) -> Option<(bool, String, ComponentMaturity)> {
        let c = self.catalogue.component(component)?;
        let m = self.maturity_of(&c.tech);
        Some((m.flyable, c.tech.clone(), m))
    }

    /// The full derivation for a design at a solar distance (m).
    pub fn derive(&self, _f: FactionId, id: DesignId, solar_dist_m: f64) -> Option<DerivedOutputs> {
        let d = self.design(id)?;
        let cat = &self.catalogue;
        let power = power::balance(d, cat, solar_dist_m);
        let avail = power.generation_w.max(0.0);
        let thermal = thermal::balance(d, cat);
        let dry = mass::dry_mass(d, cat);
        let wet = mass::wet_mass(d, cat);
        let stage_dvs = deltav::stage_dvs(d, cat, avail);
        let total_dv = stage_dvs.iter().sum();
        let max_thrust = deltav::first_stage_thrust(d, cat, avail);
        let tw_at_target = d
            .mission
            .target_body_g
            .map(|g| deltav::thrust_to_weight(d, cat, avail, g));
        let rel = reliability::compose(d, cat, &|t| self.maturity_of(t)).composed;
        let life_support = lifesupport::sizing(d, cat, d.mission.mission_days.unwrap_or(0.0));
        let edl = d
            .mission
            .target_body_g
            .map(|g| edl::suitability(d, cat, g, avail));
        let avg_trl = {
            let trls: Vec<f64> = d
                .all_components()
                .filter_map(|cid| cat.component(cid))
                .map(|c| f64::from(self.maturity_of(&c.tech).trl))
                .collect();
            if trls.is_empty() {
                0.0
            } else {
                trls.iter().sum::<f64>() / trls.len() as f64
            }
        };
        let cost = cost::estimate(dry, &cat.params, avg_trl, d.production_count);
        Some(DerivedOutputs {
            dry_mass: dry,
            wet_mass: wet,
            propellant: mass::propellant(d),
            stage_dvs,
            total_dv,
            max_thrust,
            tw_at_target,
            power,
            thermal,
            reliability: rel,
            life_support,
            edl,
            cost,
        })
    }

    /// Traceability tree for dry mass.
    pub fn trace_dry_mass(&self, id: DesignId) -> Option<TraceTree> {
        let d = self.design(id)?;
        Some(mass::trace_dry_mass(d, &self.catalogue))
    }

    /// Traceability tree for a stage's Δv.
    pub fn trace_stage_dv(&self, id: DesignId, stage: usize) -> Option<TraceTree> {
        let d = self.design(id)?;
        Some(deltav::trace_stage_dv(d, &self.catalogue, stage))
    }

    /// Composed reliability + breakdown.
    pub fn reliability(&self, id: DesignId) -> Option<ReliabilityResult> {
        let d = self.design(id)?;
        Some(reliability::compose(d, &self.catalogue, &|t| {
            self.maturity_of(t)
        }))
    }

    /// The FA-02 endpoint params per engine in the design (carried inline at spawn).
    pub fn engine_defs(&self, id: DesignId) -> Vec<EngineDef> {
        let Some(d) = self.design(id) else {
            return Vec::new();
        };
        d.stages
            .iter()
            .filter_map(|s| s.engine.as_ref())
            .filter_map(|eid| self.catalogue.component(eid))
            .filter_map(propulsion::engine_def)
            .collect()
    }

    /// Realism red-flags.
    pub fn red_flags(&self, id: DesignId, solar_dist_m: f64) -> Option<Vec<RedFlag>> {
        let d = self.design(id)?;
        let cat = &self.catalogue;
        let out = self.derive(FactionId(0), id, solar_dist_m)?;
        let structure_limit_n = d
            .all_components()
            .filter_map(|cid| cat.component(cid))
            .filter_map(|c| match &c.params {
                ComponentParams::Structure { thrust_limit_n } => Some(*thrust_limit_n),
                _ => None,
            })
            .fold(0.0, f64::max);
        let sub = reliability::compose(d, cat, &|t| self.maturity_of(t))
            .sub_trl6
            .len();
        let g = GuardInputs {
            class: cat.classes.get(&d.class),
            power_margin_w: out.power.margin_w,
            thermal_margin_w: out.thermal.margin_w,
            total_dv: out.total_dv,
            lander_tw: out.tw_at_target.unwrap_or(f64::INFINITY),
            structure_limit_n,
            max_thrust_n: out.max_thrust,
            crew_capacity: out.life_support.crew_capacity,
            sub_trl6_count: sub,
            mission: d.mission,
        };
        Some(guards::check(&g))
    }

    /// Physical cost/build estimate.
    pub fn cost(&self, id: DesignId, solar_dist_m: f64) -> Option<CostEstimate> {
        self.derive(FactionId(0), id, solar_dist_m).map(|o| o.cost)
    }

    /// Field-by-field comparison of two designs' derived outputs.
    pub fn compare(
        &self,
        a: DesignId,
        b: DesignId,
        solar_dist_m: f64,
    ) -> Option<(DerivedOutputs, DerivedOutputs)> {
        Some((
            self.derive(FactionId(0), a, solar_dist_m)?,
            self.derive(FactionId(0), b, solar_dist_m)?,
        ))
    }
}
