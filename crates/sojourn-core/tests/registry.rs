//! Module registry integration tests (T059, FR-CORE-501/503): ownership-overlap,
//! circular-dependency and unknown-view rejection, and deterministic ordering.

mod common;
use common::{TestModule, config, data};
use sojourn_core::{
    CoreError, EscalationDecl, InitCtx, ModuleManifest, SimCore, SimModule, StateSlice, StepCtx,
    ViewSnapshot,
};

/// Minimal module parameterised for registry tests.
struct Stub {
    id: &'static str,
    owned: &'static str,
    publishes: Vec<String>,
    reads: Vec<String>,
}

impl SimModule for Stub {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: self.id.into(),
            owned_slice: self.owned.into(),
            publishes: self.publishes.clone(),
            reads: self.reads.clone(),
            emits: vec![],
            subscribes: vec![],
            phase: 0,
            streams: vec![],
            cadence_ticks: 1000,
            escalations: Vec::<EscalationDecl>::new(),
        }
    }
    fn init(&self, _ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        Box::new(0u8)
    }
    fn step(&self, _slice: &mut dyn StateSlice, _ctx: &mut StepCtx<'_>) {}
    fn publish(&self, _slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        self.publishes
            .iter()
            .map(|id| ViewSnapshot {
                id: id.clone(),
                fields: Default::default(),
            })
            .collect()
    }
    fn save_slice(&self, _slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        Ok(vec![])
    }
    fn load_slice(&self, _bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        Ok(Box::new(0u8))
    }
}

fn stub(
    id: &'static str,
    owned: &'static str,
    publishes: &[&str],
    reads: &[&str],
) -> Box<dyn SimModule> {
    Box::new(Stub {
        id,
        owned,
        publishes: publishes.iter().map(|s| s.to_string()).collect(),
        reads: reads.iter().map(|s| s.to_string()).collect(),
    })
}

/// `SimCore` is intentionally not `Debug`; map the Ok side away before unwrapping.
fn expect_reg_error(r: Result<SimCore, CoreError>) {
    let err = r.map(|_| ()).unwrap_err();
    assert!(
        matches!(err, CoreError::RegistrationFailed { .. }),
        "got {err:?}"
    );
}

#[test]
fn duplicate_slice_ownership_rejected() {
    expect_reg_error(SimCore::create(
        config(1),
        data(),
        vec![
            stub("a", "shared", &["a/v"], &[]),
            stub("b", "shared", &["b/v"], &[]),
        ],
    ));
}

#[test]
fn unknown_view_read_rejected() {
    expect_reg_error(SimCore::create(
        config(1),
        data(),
        vec![stub("a", "a/s", &["a/v"], &["nope/v"])],
    ));
}

#[test]
fn circular_dependency_rejected() {
    // a reads b's view, b reads a's view → cycle.
    expect_reg_error(SimCore::create(
        config(1),
        data(),
        vec![
            stub("a", "a/s", &["a/v"], &["b/v"]),
            stub("b", "b/s", &["b/v"], &["a/v"]),
        ],
    ));
}

#[test]
fn duplicate_view_producer_rejected() {
    expect_reg_error(SimCore::create(
        config(1),
        data(),
        vec![
            stub("a", "a/s", &["shared/v"], &[]),
            stub("b", "b/s", &["shared/v"], &[]),
        ],
    ));
}

#[test]
fn registration_order_does_not_affect_outcome() {
    // Two independent modules registered in both orders → identical fingerprint
    // (deterministic derived ordering, FR-CORE-503).
    let build = |swap: bool| {
        let m1: Box<dyn SimModule> = Box::new(TestModule {
            cadence: 1000,
            ..Default::default()
        });
        let m2 = stub("zzz", "zzz/s", &["zzz/v"], &[]);
        let mods = if swap { vec![m2, m1] } else { vec![m1, m2] };
        let mut core = SimCore::create(config(3), data(), mods).unwrap();
        core.step(sojourn_core::StepRequest::Ticks(50_000)).unwrap();
        core.fingerprint().unwrap()
    };
    assert_eq!(build(false), build(true));
}
