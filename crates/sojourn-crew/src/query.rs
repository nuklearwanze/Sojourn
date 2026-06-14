//! The read-only crew-state query surface (R13, `contracts/crew-queries.md`). Pure
//! functions over a `CrewSnapshot` of the stored dynamic slice + the captured
//! composed values. REID, capability and viability are derived here; no mutation,
//! no hidden truth.

use crate::asset::{CrewMember, CrewStatus, CrewedAsset, Deconditioning};
use crate::hazard;
use crate::ids::{AssetId, AstronautId, FactionId};
use crate::inputs::EdlSuitability;
use crate::module::{CrewModule, CrewSlice};
use crate::params::Params;
use crate::{consumables, eclss, edl, physiology, psychology, radiation};
use serde::{Deserialize, Serialize};
use sojourn_core::{CoreError, SimCore};

/// A crew member's derived view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewMemberView {
    /// Identity.
    pub id: String,
    /// Career dose (Sv).
    pub career_dose_sv: f64,
    /// REID (percent).
    pub reid_pct: f64,
    /// Deconditioning indices.
    pub decon: Deconditioning,
    /// Psychological load.
    pub psych_load: f64,
    /// Composite capability ∈ [0,1].
    pub capability: f64,
    /// Status.
    pub status: CrewStatus,
}

/// Consumables view.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Consumables {
    /// Stock (kg).
    pub stock_kg: f64,
    /// Gross daily consumption (kg).
    pub gross_per_day: f64,
    /// Resupply make-up rate (kg/day).
    pub makeup_rate_kg_day: f64,
}

/// ECLSS risk view.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EclssRisk {
    /// Daily failure probability.
    pub failure_prob: f64,
    /// Critical (failed beyond abort reach)?
    pub critical: bool,
}

/// Composite mission viability.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viability {
    /// Consumables cover the duration?
    pub consumables_ok: bool,
    /// Dose/REID within limits?
    pub dose_ok: bool,
    /// ECLSS not critically failed?
    pub eclss_ok: bool,
    /// Crew capability above the floor?
    pub capability_ok: bool,
    /// Overall viable?
    pub viable: bool,
}

/// A truth-free, query-time snapshot of the crew slice.
pub struct CrewSnapshot {
    /// Tick.
    pub tick: u64,
    slice: CrewSlice,
    params: Params,
}

impl CrewSnapshot {
    /// Extract from a running kernel (the slice holds the captured composed values).
    pub fn from_core(core: &SimCore, module: &CrewModule) -> Result<CrewSnapshot, CoreError> {
        let tick = core.status().tick;
        core.with_slice("crew", |slice| {
            let s = slice
                .as_any()
                .downcast_ref::<CrewSlice>()
                .expect("crew slice type");
            CrewSnapshot {
                tick,
                slice: s.clone(),
                params: module.params.clone(),
            }
        })
    }

    /// Build directly from parts (tests).
    pub fn from_parts(module: &CrewModule, slice: CrewSlice) -> CrewSnapshot {
        CrewSnapshot {
            tick: slice.last_tick,
            slice,
            params: module.params.clone(),
        }
    }

    fn member_ref(&self, id: &str) -> Option<&CrewMember> {
        self.slice.members.get(&AstronautId(id.to_string()))
    }

    fn asset_ref(&self, id: AssetId) -> Option<&CrewedAsset> {
        self.slice.assets.get(&id)
    }

    /// A crew member's REID (percent).
    pub fn reid(&self, astronaut: &str) -> Option<f64> {
        let m = self.member_ref(astronaut)?;
        Some(radiation::reid_pct(
            m.career_dose_sv,
            &m.facts,
            &self.params.radiation,
        ))
    }

    /// A crew member's composite capability ∈ [0,1] (decon × psych × health).
    pub fn capability(&self, astronaut: &str) -> Option<f64> {
        let m = self.member_ref(astronaut)?;
        let reid = radiation::reid_pct(m.career_dose_sv, &m.facts, &self.params.radiation);
        Some(hazard::capability(&[
            physiology::capability_factor(&m.decon),
            psychology::capability_factor(m.psych_load),
            (1.0 - reid / 100.0).clamp(0.0, 1.0),
        ]))
    }

    /// A crew member's full derived view.
    pub fn member(&self, astronaut: &str) -> Option<CrewMemberView> {
        let m = self.member_ref(astronaut)?;
        Some(CrewMemberView {
            id: m.id.0.clone(),
            career_dose_sv: m.career_dose_sv,
            reid_pct: self.reid(astronaut)?,
            decon: m.decon,
            psych_load: m.psych_load,
            capability: self.capability(astronaut)?,
            status: m.status,
        })
    }

    /// True iff the crew member has been physically lost.
    pub fn loss_of_crew(&self, astronaut: &str) -> bool {
        matches!(
            self.member_ref(astronaut).map(|m| m.status),
            Some(CrewStatus::Lost)
        )
    }

    /// All crew + their views for a faction.
    pub fn roster_state(&self, faction: FactionId) -> Vec<CrewMemberView> {
        self.slice
            .members
            .values()
            .filter(|m| {
                self.slice
                    .assets
                    .get(&m.asset)
                    .map(|a| a.faction == faction)
                    .unwrap_or(false)
            })
            .filter_map(|m| self.member(&m.id.0))
            .collect()
    }

    /// Consumables view for an asset.
    pub fn consumables(&self, asset: AssetId) -> Option<Consumables> {
        let a = self.asset_ref(asset)?;
        Some(Consumables {
            stock_kg: a.consumables_kg,
            gross_per_day: consumables::daily_consumption(a, &self.params.consumables),
            makeup_rate_kg_day: consumables::makeup_rate(a, &self.params.consumables),
        })
    }

    /// ECLSS risk for an asset.
    pub fn eclss_risk(&self, asset: AssetId) -> Option<EclssRisk> {
        let a = self.asset_ref(asset)?;
        Some(EclssRisk {
            failure_prob: eclss::failure_prob(a, &self.params.eclss),
            critical: eclss::critical(a),
        })
    }

    /// EDL crew-risk for an asset with the given vehicle suitability.
    pub fn edl_risk(&self, asset: AssetId, suitability: &EdlSuitability) -> Option<f64> {
        let a = self.asset_ref(asset)?;
        let cap = self.crew_min_capability(a);
        Some(edl::crew_loss_prob(
            suitability,
            &a.env.body,
            cap,
            &self.params.edl,
        ))
    }

    fn crew_min_capability(&self, a: &CrewedAsset) -> f64 {
        a.crew
            .iter()
            .filter_map(|id| self.capability(&id.0))
            .fold(1.0, f64::min)
    }

    /// Composite mission viability for an asset over `mission_days`.
    pub fn viability(&self, asset: AssetId, mission_days: f64) -> Option<Viability> {
        let a = self.asset_ref(asset)?;
        let h = &self.params.hazard;
        let consumables_ok = !a.sizing.crewed
            || consumables::coverage_days(a, &self.params.consumables) >= mission_days;
        let dose_ok = a.crew.iter().all(|id| {
            self.member_ref(&id.0)
                .map(|m| {
                    radiation::reid_pct(m.career_dose_sv, &m.facts, &self.params.radiation)
                        < h.viability_reid_pct
                })
                .unwrap_or(true)
        });
        let eclss_ok = !eclss::critical(a);
        let capability_ok = self.crew_min_capability(a) >= h.viability_capability_min;
        Some(Viability {
            consumables_ok,
            dose_ok,
            eclss_ok,
            capability_ok,
            viable: consumables_ok && dose_ok && eclss_ok && capability_ok,
        })
    }
}
