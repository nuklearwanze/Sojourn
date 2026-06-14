//! The read-only base-query surface (R13, `contracts/base-queries.md`). Pure
//! functions over a `BaseSnapshot` composing the base slice with the composed-value
//! inputs. No mutation, no hidden truth; emergent numbers trace to sourced leaves.

use crate::base::Base;
use crate::catalogue::Catalogue;
use crate::construction::{self, ConstructionProgress};
use crate::ids::BaseId;
use crate::inputs::{BaseInputs, SiteFacts};
use crate::lifesupport::{self, LifeSupport};
use crate::module::{BaseModule, BaseSlice};
use crate::power::{self, PowerBalance};
use crate::production::{self, LocalProduction};
use crate::shielding::{self, Shielding};
use crate::siting::{self, RedFlag};
use crate::sustainability::{self, EmbargoResult, SelfSufficiency};
use crate::trace::TraceTree;
use serde::{Deserialize, Serialize};
use sojourn_core::{CoreError, SimCore};
use std::collections::BTreeMap;

/// A base's full derived emergent state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergentProperties {
    /// Power balance.
    pub power: PowerBalance,
    /// Shielding / dose attenuation.
    pub shielding: Shielding,
    /// Static life support.
    pub life_support: LifeSupport,
    /// Limiting-factor self-sufficiency index.
    pub self_sufficiency: f64,
    /// Hazard exposure ∈ [0,1].
    pub hazard_exposure: f64,
}

/// Inputs consumed and outputs produced at a base's location (for the economy).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductionConsumption {
    /// Outputs produced (commodity → kg/day).
    pub produces: BTreeMap<String, f64>,
    /// Inputs consumed (commodity → kg/day).
    pub consumes: BTreeMap<String, f64>,
}

/// A truth-free, query-time snapshot of the bases.
pub struct BaseSnapshot {
    /// Tick.
    pub tick: u64,
    slice: BaseSlice,
    catalogue: Catalogue,
    inputs: BaseInputs,
}

impl BaseSnapshot {
    /// Extract from a running kernel, composing the host-supplied inputs.
    pub fn from_core(
        core: &SimCore,
        module: &BaseModule,
        inputs: BaseInputs,
    ) -> Result<BaseSnapshot, CoreError> {
        let tick = core.status().tick;
        core.with_slice("base", |slice| {
            let s = slice
                .as_any()
                .downcast_ref::<BaseSlice>()
                .expect("base slice type");
            BaseSnapshot {
                tick,
                slice: s.clone(),
                catalogue: module.catalogue.clone(),
                inputs: inputs.clone(),
            }
        })
    }

    /// Build directly from parts (tests).
    pub fn from_parts(module: &BaseModule, slice: BaseSlice, inputs: BaseInputs) -> BaseSnapshot {
        BaseSnapshot {
            tick: slice.tick,
            slice,
            catalogue: module.catalogue.clone(),
            inputs,
        }
    }

    /// A base by id.
    pub fn base(&self, id: BaseId) -> Option<&Base> {
        self.slice.bases.get(&id)
    }

    fn site_of(&self, base: &Base) -> Option<&SiteFacts> {
        self.inputs.site(&base.site)
    }

    // ---- emergent properties ----

    /// The power balance.
    pub fn power(&self, id: BaseId) -> Option<PowerBalance> {
        let b = self.base(id)?;
        Some(power::balance(b, &self.catalogue, self.site_of(b)))
    }

    /// The shielding state.
    pub fn shielding(&self, id: BaseId) -> Option<Shielding> {
        let b = self.base(id)?;
        Some(shielding::compute(b, &self.catalogue, self.site_of(b)))
    }

    /// Static life support.
    pub fn life_support(&self, id: BaseId) -> Option<LifeSupport> {
        let b = self.base(id)?;
        Some(lifesupport::compute(b, &self.catalogue))
    }

    /// On-site production.
    pub fn local_production(&self, id: BaseId) -> Option<LocalProduction> {
        let b = self.base(id)?;
        Some(production::compute(b, &self.catalogue, &self.inputs, id))
    }

    /// The full emergent state.
    pub fn emergent(&self, id: BaseId) -> Option<EmergentProperties> {
        let b = self.base(id)?;
        let site = self.site_of(b);
        let power = power::balance(b, &self.catalogue, site);
        let shielding = shielding::compute(b, &self.catalogue, site);
        let life_support = lifesupport::compute(b, &self.catalogue);
        let local = production::compute(b, &self.catalogue, &self.inputs, id);
        let loops = sustainability::loops(b, &self.catalogue, &life_support, &power, &local);
        let self_sufficiency = sustainability::self_sufficiency(&loops).index;
        let hazard_exposure = site.map(|s| s.hazard_level).unwrap_or(0.0);
        Some(EmergentProperties {
            power,
            shielding,
            life_support,
            self_sufficiency,
            hazard_exposure,
        })
    }

    // ---- sustainability ----

    /// The limiting-factor self-sufficiency index.
    pub fn self_sufficiency(&self, id: BaseId) -> Option<SelfSufficiency> {
        let loops = self.loops(id)?;
        Some(sustainability::self_sufficiency(&loops))
    }

    /// The embargo stress-test result over `years`.
    pub fn embargo(&self, id: BaseId, years: f64) -> Option<EmbargoResult> {
        let loops = self.loops(id)?;
        Some(sustainability::embargo(&loops, years))
    }

    fn loops(&self, id: BaseId) -> Option<Vec<sustainability::LoopState>> {
        let b = self.base(id)?;
        let site = self.site_of(b);
        let power = power::balance(b, &self.catalogue, site);
        let life_support = lifesupport::compute(b, &self.catalogue);
        let local = production::compute(b, &self.catalogue, &self.inputs, id);
        Some(sustainability::loops(
            b,
            &self.catalogue,
            &life_support,
            &power,
            &local,
        ))
    }

    // ---- construction ----

    /// Construction progress.
    pub fn construction(&self, id: BaseId) -> Option<ConstructionProgress> {
        let b = self.base(id)?;
        let pid = b.project?;
        let project = self.slice.projects.get(&pid)?;
        Some(construction::progress(b, project))
    }

    // ---- siting ----

    /// Siting red-flags.
    pub fn siting_flags(&self, id: BaseId) -> Option<Vec<RedFlag>> {
        let b = self.base(id)?;
        let site = self.site_of(b)?;
        let power = power::balance(b, &self.catalogue, Some(site));
        let shielding = shielding::compute(b, &self.catalogue, Some(site));
        Some(siting::flags(b, &self.catalogue, site, &power, &shielding))
    }

    // ---- exposure ----

    /// Production/consumption at the base's location (for the economy).
    pub fn production_consumption(&self, id: BaseId) -> Option<ProductionConsumption> {
        let b = self.base(id)?;
        let ls = lifesupport::compute(b, &self.catalogue);
        let local = production::compute(b, &self.catalogue, &self.inputs, id);
        let mut produces = local.manufacturing_rates.clone();
        if local.regolith_shield_rate_kg_day > 0.0 {
            produces.insert("shielding".into(), local.regolith_shield_rate_kg_day);
        }
        let mut consumes = BTreeMap::new();
        if ls.consumables_per_day_kg > 0.0 {
            consumes.insert("consumables".into(), ls.consumables_per_day_kg);
        }
        Some(ProductionConsumption { produces, consumes })
    }

    /// The base's settlement milestones.
    pub fn milestones(&self) -> Vec<String> {
        self.slice.milestones.iter().cloned().collect()
    }

    /// Traceability tree for an emergent property ("power-margin" | "shielding").
    pub fn trace(&self, id: BaseId, property: &str) -> Option<TraceTree> {
        let b = self.base(id)?;
        let site = self.site_of(b);
        match property {
            "power-margin" => Some(power::trace(b, &self.catalogue, site)),
            "shielding" => Some(shielding::trace(b, &self.catalogue)),
            _ => None,
        }
    }

    /// Compare two bases' emergent properties.
    pub fn compare(
        &self,
        a: BaseId,
        b: BaseId,
    ) -> Option<(EmergentProperties, EmergentProperties)> {
        Some((self.emergent(a)?, self.emergent(b)?))
    }
}
