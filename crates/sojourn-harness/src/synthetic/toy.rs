//! The reference "toy" module (SC-010): implemented strictly from
//! `specs/002-sim-core/contracts/module-contract.md`, using nothing but the
//! documented surface. It owns a counter slice, steps daily, occasionally emits a
//! diagnostic event, and publishes one view. If this file needs kernel-internal
//! knowledge to work, the contract documentation has failed.

use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sojourn_core::{
    CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx, ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;

/// One simulated day at 1 s ticks.
const DAY_TICKS: u64 = 86_400;

/// Owned state: a counter and a seeded accumulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToySlice {
    days: u64,
    accumulator: u64,
}

/// The reference module.
pub struct ToyModule;

impl SimModule for ToyModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "toy".into(),
            owned_slice: "toy/slice".into(),
            publishes: vec!["toy/state".into()],
            reads: vec!["kernel/status".into()],
            emits: vec!["kernel-diagnostic".into()],
            subscribes: vec![],
            phase: 10,
            streams: vec!["toy/walk".into()],
            cadence_ticks: DAY_TICKS,
            escalations: vec![],
        }
    }

    fn init(&self, ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        let seed_step = ctx.rng("toy/walk").next_u64();
        Box::new(ToySlice {
            days: 0,
            accumulator: seed_step,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        let slice = slice
            .as_any_mut()
            .downcast_mut::<ToySlice>()
            .expect("toy slice");
        slice.days += 1;
        slice.accumulator = slice
            .accumulator
            .wrapping_add(ctx.rng("toy/walk").next_u64());
        // Roughly monthly: a diagnostic heartbeat.
        if slice.days % 30 == 0 {
            ctx.emit(
                "kernel-diagnostic",
                BTreeMap::from([("days".to_string(), ViewValue::U64(slice.days))]),
            );
        }
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let slice = slice
            .as_any()
            .downcast_ref::<ToySlice>()
            .expect("toy slice");
        vec![ViewSnapshot {
            id: "toy/state".into(),
            fields: BTreeMap::from([
                ("days".to_string(), ViewValue::U64(slice.days)),
                ("accumulator".to_string(), ViewValue::U64(slice.accumulator)),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let slice = slice
            .as_any()
            .downcast_ref::<ToySlice>()
            .expect("toy slice");
        postcard::to_allocvec(slice).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let slice: ToySlice =
            postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
                reason: e.to_string(),
            })?;
        Ok(Box::new(slice))
    }
}
