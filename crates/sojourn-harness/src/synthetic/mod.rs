//! Synthetic load module (FR-CORE-704): a stand-in game system that exercises the
//! kernel the way content slices will — entity churn in an owned slice, seeded
//! event emission, command reactions, published views — and the carrier for the
//! mutation injections the determinism gate must catch (FR-CORE-705).

pub mod toy;

use crate::mutation::Mutation;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use slotmap::{DefaultKey, SlotMap};
use sojourn_core::{
    CoreError, EscalationDecl, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;

/// Ticks per (non-leap) simulated year at 1 s base ticks.
pub const TICKS_PER_YEAR: u64 = 365 * 86_400;

/// Scenario-supplied configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntheticConfig {
    /// Steady-state entity population.
    pub entities: u32,
    /// Expected emitted events per simulated year.
    pub events_per_year: u32,
    /// Base cadence in ticks.
    pub cadence_ticks: u64,
    /// Every Nth event is a `mission-milestone` instead of `kernel-diagnostic`.
    pub milestone_every: u32,
    /// Every Nth event is an `anomaly` (pause policy decides if it interrupts).
    pub anomaly_every: u32,
}

impl Default for SyntheticConfig {
    fn default() -> Self {
        SyntheticConfig {
            entities: 100,
            events_per_year: 40,
            cadence_ticks: 3600,
            milestone_every: 10,
            anomaly_every: 7,
        }
    }
}

/// The module's exclusively-owned state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticSlice {
    entities: SlotMap<DefaultKey, u64>,
    churn_total: u64,
    events_emitted: u64,
    last_value: i64,
}

/// The synthetic module.
pub struct SyntheticModule {
    /// Configuration.
    pub cfg: SyntheticConfig,
    /// Active nondeterminism injection (mutation builds only).
    pub mutation: Option<Mutation>,
}

impl SyntheticModule {
    /// A clean (gate-passing) module.
    pub fn new(cfg: SyntheticConfig) -> Self {
        SyntheticModule {
            cfg,
            mutation: None,
        }
    }
}

/// Per-step event-emission threshold over a u64 draw.
fn emission_threshold(cfg: &SyntheticConfig) -> u64 {
    let p = f64::from(cfg.events_per_year) * cfg.cadence_ticks as f64 / TICKS_PER_YEAR as f64;
    (p.min(1.0) * u64::MAX as f64) as u64
}

impl SimModule for SyntheticModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "synthetic".into(),
            owned_slice: "synthetic/slice".into(),
            publishes: vec!["synthetic/stats".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "kernel-diagnostic".into(),
                "anomaly".into(),
                "mission-milestone".into(),
            ],
            subscribes: vec!["module-command".into()],
            phase: 0,
            streams: vec!["synthetic/churn".into(), "synthetic/events".into()],
            cadence_ticks: self.cfg.cadence_ticks,
            escalations: Vec::<EscalationDecl>::new(),
        }
    }

    fn init(&self, ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        let mut entities = SlotMap::new();
        let rng = ctx.rng("synthetic/churn");
        for _ in 0..self.cfg.entities {
            let v = rng.next_u64();
            entities.insert(v);
        }
        Box::new(SyntheticSlice {
            entities,
            churn_total: 0,
            events_emitted: 0,
            last_value: 0,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        let slice = slice
            .as_any_mut()
            .downcast_mut::<SyntheticSlice>()
            .expect("synthetic slice type");

        // Entity churn: remove one (deterministically chosen), insert one.
        let churn = ctx.rng("synthetic/churn").next_u64();
        if !slice.entities.is_empty() {
            let idx = (churn % slice.entities.len() as u64) as usize;
            // slotmap iteration order is deterministic given identical history.
            let key = slice.entities.keys().nth(idx).expect("index in range");
            slice.entities.remove(key);
        }
        slice.entities.insert(churn);
        slice.churn_total += 1;

        // Mutation injection point (clean builds: no-op).
        if let Some(m) = &self.mutation {
            slice.last_value = slice.last_value.wrapping_add(crate::mutation::poison(*m));
        }

        // Seeded event emission.
        let draw = ctx.rng("synthetic/events").next_u64();
        if draw < emission_threshold(&self.cfg) {
            slice.events_emitted += 1;
            let n = slice.events_emitted;
            let payload = BTreeMap::from([("n".to_string(), ViewValue::U64(n))]);
            if self.cfg.anomaly_every > 0 && n % u64::from(self.cfg.anomaly_every) == 0 {
                ctx.emit("anomaly", payload);
            } else if self.cfg.milestone_every > 0 && n % u64::from(self.cfg.milestone_every) == 0 {
                ctx.emit("mission-milestone", payload);
            } else {
                ctx.emit("kernel-diagnostic", payload);
            }
        }
    }

    fn on_event(
        &self,
        slice: &mut dyn StateSlice,
        ev: &sojourn_core::EventRecord,
        ctx: &mut StepCtx<'_>,
    ) {
        let slice = slice
            .as_any_mut()
            .downcast_mut::<SyntheticSlice>()
            .expect("synthetic slice type");
        if ev.class == "module-command" {
            let key = match ev.payload.get("key") {
                Some(ViewValue::Str(k)) => k.as_str(),
                _ => return,
            };
            let value = match ev.payload.get("value") {
                Some(ViewValue::I64(v)) => *v,
                _ => 0,
            };
            match key {
                "set" => slice.last_value = value,
                "emit-milestone" => {
                    slice.events_emitted += 1;
                    ctx.emit(
                        "mission-milestone",
                        BTreeMap::from([("n".to_string(), ViewValue::I64(value))]),
                    );
                }
                _ => {}
            }
        }
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let slice = slice
            .as_any()
            .downcast_ref::<SyntheticSlice>()
            .expect("synthetic slice type");
        vec![ViewSnapshot {
            id: "synthetic/stats".into(),
            fields: BTreeMap::from([
                (
                    "entity_count".to_string(),
                    ViewValue::U64(slice.entities.len() as u64),
                ),
                ("churn_total".to_string(), ViewValue::U64(slice.churn_total)),
                (
                    "events_emitted".to_string(),
                    ViewValue::U64(slice.events_emitted),
                ),
                ("last_value".to_string(), ViewValue::I64(slice.last_value)),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let slice = slice
            .as_any()
            .downcast_ref::<SyntheticSlice>()
            .expect("synthetic slice type");
        postcard::to_allocvec(slice).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let slice: SyntheticSlice =
            postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
                reason: e.to_string(),
            })?;
        Ok(Box::new(slice))
    }
}
