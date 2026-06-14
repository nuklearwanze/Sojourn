//! The `SimModule` implementation: the crew slice, the command surface via
//! `ModulePayload`, and the **daily seeded step** that evolves the stored dynamic
//! state (consumption, dose, deconditioning, psych accrual + SPE/ECLSS/anomaly
//! rolls + viability/loss-of-crew checks). Depends only on `sojourn-core`; composed
//! values are captured into the slice at command time (R3/R4).

use crate::asset::{CrewMember, CrewStatus, CrewedAsset, Deconditioning, EclssState};
use crate::ids::{AssetId, AstronautId, FactionId, MissionId};
use crate::inputs::{AssetSizing, AstronautFacts, EdlSuitability, EnvFacts, TechMaturity};
use crate::params::Params;
use crate::{consumables, eclss, edl, physiology, psychology, radiation};
use serde::{Deserialize, Serialize};
use sojourn_core::{
    Command, CommandOutcome, CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const TICKS_PER_DAY: u64 = 86_400;

/// A journalled crew command (routed via `Command::ModulePayload`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrewCommand {
    /// Occupy an asset (captures the composed sizing/env/maturity).
    OccupyAsset {
        /// Faction.
        faction: u32,
        /// Asset id (caller-assigned, unique).
        asset: u32,
        /// Captured static sizing.
        sizing: AssetSizing,
        /// Captured environment.
        env: EnvFacts,
        /// Captured ECLSS-tech maturity.
        eclss_maturity: TechMaturity,
        /// Ops oversubscription (FA-06).
        ops_oversub: f64,
        /// Opening consumables stock (kg).
        consumables_kg: f64,
    },
    /// Seat a crew member (captures roster facts incl. age/sex).
    AssignCrew {
        /// Asset id.
        asset: u32,
        /// Astronaut id.
        astronaut: String,
        /// Captured roster facts.
        facts: AstronautFacts,
    },
    /// Refresh the asset's environment + ops (e.g. when the craft moves).
    UpdateEnv {
        /// Asset id.
        asset: u32,
        /// New environment.
        env: EnvFacts,
        /// New ops oversubscription.
        ops_oversub: f64,
    },
    /// Apply ECLSS maintenance for the day.
    Maintain {
        /// Asset id.
        asset: u32,
        /// Crew-hours applied.
        crew_hr: f64,
        /// Spares applied (kg).
        spares_kg: f64,
    },
    /// Set whether the crew shelters against an SPE.
    Shelter {
        /// Asset id.
        asset: u32,
        /// Sheltering?
        sheltering: bool,
    },
    /// Replenish consumables.
    Resupply {
        /// Asset id.
        asset: u32,
        /// Mass added (kg).
        kg: f64,
    },
    /// Evaluate an EDL attempt (rolls the crew-risk on `crew/edl-risk`).
    EvaluateEdl {
        /// Asset id.
        asset: u32,
        /// Vehicle EDL suitability.
        suitability: EdlSuitability,
    },
    /// Vacate (retire) an asset.
    VacateAsset {
        /// Asset id.
        asset: u32,
    },
}

/// Encode a [`CrewCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn crew_payload(cmd: &CrewCommand) -> Command {
    let kind = match cmd {
        CrewCommand::OccupyAsset { .. } => "occupy",
        CrewCommand::AssignCrew { .. } => "assign",
        CrewCommand::UpdateEnv { .. } => "update-env",
        CrewCommand::Maintain { .. } => "maintain",
        CrewCommand::Shelter { .. } => "shelter",
        CrewCommand::Resupply { .. } => "resupply",
        CrewCommand::EvaluateEdl { .. } => "evaluate-edl",
        CrewCommand::VacateAsset { .. } => "vacate",
    };
    Command::ModulePayload {
        module: "crew".into(),
        kind: kind.into(),
        payload: postcard::to_allocvec(cmd).expect("crew command encodes"),
    }
}

/// The crew slice — all owned dynamic state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewSlice {
    /// Crewed assets.
    pub assets: BTreeMap<AssetId, CrewedAsset>,
    /// Per-astronaut dynamic health records.
    pub members: BTreeMap<AstronautId, CrewMember>,
    /// Astronauts physically lost.
    pub lost: BTreeSet<AstronautId>,
    /// Last stepped tick.
    pub last_tick: u64,
    /// Crew-data content hash (pin).
    pub data_hash: [u8; 32],
}

/// The crew module (immutable loaded parameters).
pub struct CrewModule {
    /// The sourced parameter set.
    pub params: Params,
}

impl CrewModule {
    /// Load and validate the parameter set from a data directory (`data/crew`).
    pub fn load(dir: &Path) -> Result<CrewModule, String> {
        Ok(CrewModule {
            params: Params::load_dir(dir)?,
        })
    }

    fn slice_mut<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut CrewSlice {
        slice
            .as_any_mut()
            .downcast_mut::<CrewSlice>()
            .expect("crew slice type")
    }
}

/// A uniform `f64 ∈ [0,1)` from a kernel rng.
fn uniform(rng: &mut dyn rand_core::RngCore) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

impl SimModule for CrewModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "crew".into(),
            owned_slice: "crew/slice".into(),
            publishes: vec!["crew/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "spe-storm".into(),
                "eclss-failure".into(),
                "crew-anomaly".into(),
                "astronaut-grounded".into(),
                "loss-of-crew".into(),
            ],
            subscribes: vec![],
            phase: 1,
            streams: vec![
                "crew/spe-storm".into(),
                "crew/eclss-failure".into(),
                "crew/edl-risk".into(),
                "crew/anomaly".into(),
            ],
            cadence_ticks: TICKS_PER_DAY,
            escalations: vec![],
        }
    }

    fn init(&self, _ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        Box::new(CrewSlice {
            assets: BTreeMap::new(),
            members: BTreeMap::new(),
            lost: BTreeSet::new(),
            last_tick: 0,
            data_hash: self.params.content_hash,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        let now = ctx.tick().0;
        let s = self.slice_mut(slice);
        if now <= s.last_tick {
            return;
        }
        let dt_days = (now - s.last_tick) as f64 / TICKS_PER_DAY as f64;
        s.last_tick = now;
        let asset_ids: Vec<AssetId> = s.assets.keys().copied().collect();

        // Pass 1: draw all seeded daily rolls (per stream, per asset, in id order).
        let spe: BTreeMap<AssetId, f64> = {
            let rng = ctx.rng("crew/spe-storm");
            asset_ids.iter().map(|&a| (a, uniform(rng))).collect()
        };
        let eclss_roll: BTreeMap<AssetId, f64> = {
            let rng = ctx.rng("crew/eclss-failure");
            asset_ids.iter().map(|&a| (a, uniform(rng))).collect()
        };
        let anomaly_roll: BTreeMap<AssetId, f64> = {
            let rng = ctx.rng("crew/anomaly");
            asset_ids.iter().map(|&a| (a, uniform(rng))).collect()
        };

        // Pass 2: apply continuous accrual + the drawn discrete outcomes.
        let mut events: Vec<(String, BTreeMap<String, ViewValue>)> = Vec::new();
        for aid in asset_ids {
            self.step_asset(
                s,
                aid,
                now,
                dt_days,
                spe[&aid],
                eclss_roll[&aid],
                anomaly_roll[&aid],
                &mut events,
            );
        }
        for (class, payload) in events {
            ctx.emit(&class, payload);
        }
    }

    fn on_command(
        &self,
        slice: &mut dyn StateSlice,
        kind: &str,
        payload: &[u8],
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let cmd: CrewCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed crew payload (kind '{kind}'): {e}"
                ));
            }
        };
        let now = ctx.tick().0;
        self.apply_command(slice, cmd, now, ctx)
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<CrewSlice>()
            .expect("crew slice type");
        vec![ViewSnapshot {
            id: "crew/status".into(),
            fields: BTreeMap::from([
                ("assets".to_string(), ViewValue::U64(s.assets.len() as u64)),
                ("crew".to_string(), ViewValue::U64(s.members.len() as u64)),
                ("lost".to_string(), ViewValue::U64(s.lost.len() as u64)),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<CrewSlice>()
            .expect("crew slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: CrewSlice = postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })?;
        if s.data_hash != self.params.content_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.data_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's crew data differs from the loaded data/crew content; install \
                       the matching version"
                    .into(),
            });
        }
        Ok(Box::new(s))
    }
}

impl CrewModule {
    #[allow(clippy::too_many_arguments)]
    fn step_asset(
        &self,
        s: &mut CrewSlice,
        aid: AssetId,
        now: u64,
        dt_days: f64,
        spe_roll: f64,
        eclss_roll: f64,
        anomaly_roll: f64,
        events: &mut Vec<(String, BTreeMap<String, ViewValue>)>,
    ) {
        let p = &self.params;
        let Some(asset) = s.assets.get(&aid) else {
            return;
        };
        if !asset.sizing.crewed {
            return;
        }
        let crew_ids: Vec<AstronautId> = asset.crew.iter().cloned().collect();
        let elapsed_days = (now - asset.occupied_since_tick) as f64 / TICKS_PER_DAY as f64;
        let gcr = radiation::daily_gcr(asset, &p.radiation) * dt_days;
        let spe_hit = spe_roll < (p.radiation.spe_daily_prob * dt_days).min(1.0);
        let spe = if spe_hit {
            radiation::spe_dose(asset, &p.radiation)
        } else {
            0.0
        };
        let cm_eff = physiology::countermeasure_eff(asset, &p.physiology);
        let psych_inc = psychology::daily_load(asset, elapsed_days, &p.psychology) * dt_days;
        let anomaly_load = crew_ids
            .iter()
            .filter_map(|id| s.members.get(id).map(|m| m.psych_load))
            .fold(0.0, f64::max);
        let anomaly_p = psychology::anomaly_prob(anomaly_load, asset.ops_oversub, &p.psychology);
        let eclss_p = eclss::failure_prob(asset, &p.eclss);

        // --- consumables depletion ---
        let consumption = consumables::daily_consumption(asset, &p.consumables) * dt_days;
        let asset = s.assets.get_mut(&aid).unwrap();
        asset.consumables_kg -= consumption;
        let exhausted = asset.consumables_kg <= 0.0;
        if exhausted {
            asset.consumables_kg = 0.0;
        }

        // --- ECLSS degrade + maintenance + failure roll ---
        let (mc, ms) = (asset.maint_crew_hr, asset.maint_spares_kg);
        eclss::accrue(&mut asset.eclss, mc, ms, &p.eclss);
        asset.maint_crew_hr = 0.0;
        asset.maint_spares_kg = 0.0;
        let mut eclss_failed_now = false;
        if !asset.eclss.failed && eclss_roll < (eclss_p * dt_days).min(1.0) {
            asset.eclss.failed = true;
            eclss_failed_now = true;
        }
        let critical = eclss::critical(asset);

        if spe_hit {
            events.push((
                "spe-storm".into(),
                BTreeMap::from([("asset".to_string(), ViewValue::U64(u64::from(aid.0)))]),
            ));
        }
        if eclss_failed_now {
            events.push((
                "eclss-failure".into(),
                BTreeMap::from([
                    ("asset".to_string(), ViewValue::U64(u64::from(aid.0))),
                    ("critical".to_string(), ViewValue::Bool(critical)),
                ]),
            ));
        }
        if anomaly_roll < (anomaly_p * dt_days).min(1.0) {
            events.push((
                "crew-anomaly".into(),
                BTreeMap::from([("asset".to_string(), ViewValue::U64(u64::from(aid.0)))]),
            ));
        }

        // --- per-member accrual + grounding ---
        for id in &crew_ids {
            let Some(m) = s.members.get_mut(id) else {
                continue;
            };
            if m.status == CrewStatus::Lost {
                continue;
            }
            m.career_dose_sv += gcr + spe;
            physiology::accrue(&mut m.decon, cm_eff, &p.physiology);
            m.psych_load += psych_inc;
            if m.status == CrewStatus::Active
                && radiation::grounded(m.career_dose_sv, &m.facts, &p.radiation)
            {
                m.status = CrewStatus::Grounded;
                events.push((
                    "astronaut-grounded".into(),
                    BTreeMap::from([("astronaut".to_string(), ViewValue::Str(id.0.clone()))]),
                ));
            }
        }

        // --- loss-of-crew: consumables exhaustion (U1) or critical ECLSS (FR-LSC-503) ---
        if exhausted || critical {
            let cause = if exhausted { "consumables" } else { "eclss" };
            self.lose_crew(s, aid, cause, events);
        }
    }

    /// Mark every (non-lost) crew member on an asset lost + emit `loss-of-crew`.
    fn lose_crew(
        &self,
        s: &mut CrewSlice,
        aid: AssetId,
        cause: &str,
        events: &mut Vec<(String, BTreeMap<String, ViewValue>)>,
    ) {
        let crew: Vec<AstronautId> = s
            .assets
            .get(&aid)
            .map(|a| a.crew.iter().cloned().collect())
            .unwrap_or_default();
        for id in crew {
            if let Some(m) = s.members.get_mut(&id)
                && m.status != CrewStatus::Lost
            {
                m.status = CrewStatus::Lost;
                s.lost.insert(id.clone());
                events.push((
                    "loss-of-crew".into(),
                    BTreeMap::from([
                        ("astronaut".to_string(), ViewValue::Str(id.0.clone())),
                        ("asset".to_string(), ViewValue::U64(u64::from(aid.0))),
                        ("cause".to_string(), ViewValue::Str(cause.into())),
                    ]),
                ));
            }
        }
    }

    fn apply_command(
        &self,
        slice: &mut dyn StateSlice,
        cmd: CrewCommand,
        now: u64,
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let s = self.slice_mut(slice);
        match cmd {
            CrewCommand::OccupyAsset {
                faction,
                asset,
                sizing,
                env,
                eclss_maturity,
                ops_oversub,
                consumables_kg,
            } => {
                let reliability =
                    eclss::reliability_from_maturity(&eclss_maturity, &self.params.eclss);
                s.assets.insert(
                    AssetId(asset),
                    CrewedAsset {
                        id: AssetId(asset),
                        faction: FactionId(faction),
                        mission: Some(MissionId(asset)),
                        sizing,
                        env,
                        eclss_maturity,
                        ops_oversub,
                        consumables_kg,
                        eclss: EclssState {
                            reliability,
                            degradation: 0.0,
                            maintenance_deficit: 0.0,
                            failed: false,
                        },
                        sheltering: false,
                        maint_crew_hr: 0.0,
                        maint_spares_kg: 0.0,
                        occupied_since_tick: now,
                        crew: BTreeSet::new(),
                    },
                );
                CommandOutcome::Applied
            }
            CrewCommand::AssignCrew {
                asset,
                astronaut,
                facts,
            } => {
                if !s.assets.contains_key(&AssetId(asset)) {
                    return CommandOutcome::Rejected(format!("unknown asset {asset}"));
                }
                let id = AstronautId(astronaut);
                s.assets
                    .get_mut(&AssetId(asset))
                    .unwrap()
                    .crew
                    .insert(id.clone());
                match s.members.get_mut(&id) {
                    // Re-assignment: career dose + deconditioning persist across
                    // missions (FR-LSC-203); psych load resets for the new mission.
                    Some(m) => {
                        m.asset = AssetId(asset);
                        m.facts = facts;
                        m.psych_load = 0.0;
                    }
                    None => {
                        s.members.insert(
                            id.clone(),
                            CrewMember {
                                id,
                                asset: AssetId(asset),
                                facts,
                                career_dose_sv: 0.0,
                                decon: Deconditioning::default(),
                                psych_load: 0.0,
                                status: CrewStatus::Active,
                            },
                        );
                    }
                }
                CommandOutcome::Applied
            }
            CrewCommand::UpdateEnv {
                asset,
                env,
                ops_oversub,
            } => match s.assets.get_mut(&AssetId(asset)) {
                Some(a) => {
                    a.env = env;
                    a.ops_oversub = ops_oversub;
                    CommandOutcome::Applied
                }
                None => CommandOutcome::Rejected(format!("unknown asset {asset}")),
            },
            CrewCommand::Maintain {
                asset,
                crew_hr,
                spares_kg,
            } => match s.assets.get_mut(&AssetId(asset)) {
                Some(a) => {
                    a.maint_crew_hr += crew_hr.max(0.0);
                    a.maint_spares_kg += spares_kg.max(0.0);
                    CommandOutcome::Applied
                }
                None => CommandOutcome::Rejected(format!("unknown asset {asset}")),
            },
            CrewCommand::Shelter { asset, sheltering } => match s.assets.get_mut(&AssetId(asset)) {
                Some(a) => {
                    a.sheltering = sheltering;
                    CommandOutcome::Applied
                }
                None => CommandOutcome::Rejected(format!("unknown asset {asset}")),
            },
            CrewCommand::Resupply { asset, kg } => match s.assets.get_mut(&AssetId(asset)) {
                Some(a) => {
                    a.consumables_kg += kg.max(0.0);
                    CommandOutcome::Applied
                }
                None => CommandOutcome::Rejected(format!("unknown asset {asset}")),
            },
            CrewCommand::EvaluateEdl { asset, suitability } => {
                let Some(a) = s.assets.get(&AssetId(asset)).cloned() else {
                    return CommandOutcome::Rejected(format!("unknown asset {asset}"));
                };
                let cap = a
                    .crew
                    .iter()
                    .filter_map(|id| self.member_capability(s, id))
                    .fold(1.0, f64::min);
                let prob = edl::crew_loss_prob(&suitability, &a.env.body, cap, &self.params.edl);
                let roll = {
                    let rng = ctx.rng("crew/edl-risk");
                    uniform(rng)
                };
                let mut events = Vec::new();
                if roll < prob {
                    self.lose_crew(s, AssetId(asset), "edl", &mut events);
                }
                for (class, payload) in events {
                    ctx.emit(&class, payload);
                }
                CommandOutcome::Applied
            }
            CrewCommand::VacateAsset { asset } => {
                // Retire the asset; the crew's **career records persist** (career
                // dose follows the astronaut across missions, FR-LSC-203). Members
                // simply become unassigned until their next AssignCrew.
                if s.assets.remove(&AssetId(asset)).is_some() {
                    CommandOutcome::Applied
                } else {
                    CommandOutcome::Rejected(format!("unknown asset {asset}"))
                }
            }
        }
    }

    fn member_capability(&self, s: &CrewSlice, id: &AstronautId) -> Option<f64> {
        let m = s.members.get(id)?;
        let reid = radiation::reid_pct(m.career_dose_sv, &m.facts, &self.params.radiation);
        Some(crate::hazard::capability(&[
            physiology::capability_factor(&m.decon),
            psychology::capability_factor(m.psych_load),
            (1.0 - reid / 100.0).clamp(0.0, 1.0),
        ]))
    }
}
