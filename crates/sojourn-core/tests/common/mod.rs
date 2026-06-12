//! Shared fixtures for the kernel integration tests: the repo's kernel data set
//! and a deterministic test module exercising the whole contract surface.
//!
//! Each test binary includes this module independently, so not every helper is
//! used by every binary — dead-code analysis is per-binary.
#![allow(dead_code)]

use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sojourn_core::{
    CoreError, DataSet, InitCtx, ModuleManifest, RunConfig, RunMode, SimModule, StateSlice,
    StepCtx, ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Load the repo's kernel data files.
pub fn data() -> DataSet {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/kernel");
    DataSet::load_dir(&dir).expect("kernel data loads")
}

/// Standard run configuration.
pub fn config(seed: u64) -> RunConfig {
    config_mode(seed, RunMode::SaveAnywhere)
}

/// Run configuration with a mode.
pub fn config_mode(seed: u64, run_mode: RunMode) -> RunConfig {
    RunConfig {
        master_seed: seed,
        horizon_years: 100,
        run_mode,
        difficulty_inputs: BTreeMap::new(),
    }
}

/// A unique temp dir under the cargo test tmpdir.
pub fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}_{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear temp dir");
    }
    dir
}

/// The test module's owned slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSlice {
    pub steps: u64,
    pub walk: u64,
    pub last_value: i64,
}

/// Deterministic test module. On its first step it schedules `anomaly_at` events
/// (exact-tick occurrences); `double_at` ticks raise two anomalies at once
/// (simultaneity); `module-command` events mutate state or emit milestones.
pub struct TestModule {
    pub cadence: u64,
    pub anomaly_at: Vec<u64>,
    pub double_at: Vec<u64>,
}

impl Default for TestModule {
    fn default() -> Self {
        TestModule {
            cadence: 3600,
            anomaly_at: vec![],
            double_at: vec![],
        }
    }
}

impl SimModule for TestModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "test".into(),
            owned_slice: "test/slice".into(),
            publishes: vec!["test/state".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "anomaly".into(),
                "kernel-diagnostic".into(),
                "mission-milestone".into(),
            ],
            subscribes: vec!["module-command".into()],
            phase: 0,
            streams: vec!["test/walk".into()],
            cadence_ticks: self.cadence,
            escalations: vec![],
        }
    }

    fn init(&self, ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        let walk = ctx.rng("test/walk").next_u64();
        Box::new(TestSlice {
            steps: 0,
            walk,
            last_value: 0,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        let s = slice.as_any_mut().downcast_mut::<TestSlice>().unwrap();
        s.steps += 1;
        s.walk = s.walk.wrapping_add(ctx.rng("test/walk").next_u64());
        if s.steps == 1 {
            let now = ctx.tick().0;
            for &t in &self.anomaly_at {
                if t > now {
                    ctx.schedule(
                        sojourn_core::Tick(t),
                        "anomaly",
                        BTreeMap::from([("at".to_string(), ViewValue::U64(t))]),
                    );
                }
            }
            for &t in &self.double_at {
                if t > now {
                    for n in 0..2u64 {
                        ctx.schedule(
                            sojourn_core::Tick(t),
                            "anomaly",
                            BTreeMap::from([("n".to_string(), ViewValue::U64(n))]),
                        );
                    }
                }
            }
        }
    }

    fn on_event(
        &self,
        slice: &mut dyn StateSlice,
        ev: &sojourn_core::EventRecord,
        ctx: &mut StepCtx<'_>,
    ) {
        let s = slice.as_any_mut().downcast_mut::<TestSlice>().unwrap();
        if ev.class == "module-command" {
            match (ev.payload.get("key"), ev.payload.get("value")) {
                (Some(ViewValue::Str(k)), Some(ViewValue::I64(v))) if k == "set" => {
                    s.last_value = *v;
                }
                (Some(ViewValue::Str(k)), Some(ViewValue::I64(v))) if k == "boom" => {
                    ctx.emit(
                        "mission-milestone",
                        BTreeMap::from([("v".to_string(), ViewValue::I64(*v))]),
                    );
                }
                _ => {}
            }
        }
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice.as_any().downcast_ref::<TestSlice>().unwrap();
        vec![ViewSnapshot {
            id: "test/state".into(),
            fields: BTreeMap::from([
                ("steps".to_string(), ViewValue::U64(s.steps)),
                ("walk".to_string(), ViewValue::U64(s.walk)),
                ("last_value".to_string(), ViewValue::I64(s.last_value)),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice.as_any().downcast_ref::<TestSlice>().unwrap();
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: TestSlice = postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })?;
        Ok(Box::new(s))
    }
}
