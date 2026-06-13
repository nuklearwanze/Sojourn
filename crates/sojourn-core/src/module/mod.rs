//! The module contract (FR-CORE-501…503, contracts/module-contract.md): manifest
//! declarations, registration validation, and the derived deterministic order.

pub mod conformance;
pub mod ctx;

use crate::data::DataSet;
use crate::error::CoreError;
use crate::event::EventRecord;
use crate::state::{StateSlice, ViewId, ViewSnapshot, ViewValue};
use crate::watch::CmpOp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// View published by the kernel itself (always available).
pub const KERNEL_STATUS_VIEW: &str = "kernel/status";

/// A state condition that escalates a module to fine (every-tick) stepping
/// (FR-CORE-104). Evaluated over the module's **own** published view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationDecl {
    /// View to read (must be one this module publishes).
    pub view: ViewId,
    /// Field within the view.
    pub field: String,
    /// Comparison.
    pub op: CmpOp,
    /// Threshold value.
    pub value: ViewValue,
}

/// Everything the kernel needs to order, isolate, seed and schedule a module —
/// the six declarations of FR-CORE-501.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Module id (`"kernel"` is reserved). Stream paths must start with `"{id}/"`.
    pub id: String,
    /// (a) Exclusively-owned state slice type tag (unique across modules).
    pub owned_slice: String,
    /// Views this module publishes (unique across modules).
    pub publishes: Vec<ViewId>,
    /// (b) Read-only views consumed from others (or the kernel).
    pub reads: Vec<ViewId>,
    /// (c) Event classes this module emits.
    pub emits: Vec<String>,
    /// (c) Event classes this module subscribes to.
    pub subscribes: Vec<String>,
    /// (d) Update-phase hint; final order is the documented derivation below.
    pub phase: i32,
    /// (e) Randomness sub-stream paths (each `"{id}/..."`).
    pub streams: Vec<String>,
    /// (f) Base update cadence in ticks (≥ 1).
    pub cadence_ticks: u64,
    /// (f) Conditions forcing fine (every-tick) stepping.
    pub escalations: Vec<EscalationDecl>,
}

/// What every game-system slice implements (contracts/module-contract.md).
pub trait SimModule {
    /// Static declaration.
    fn manifest(&self) -> ModuleManifest;

    /// Create the owned slice for a new run (randomness via declared streams only).
    fn init(&self, ctx: &mut ctx::InitCtx<'_>) -> Box<dyn StateSlice>;

    /// Advance the owned slice for one scheduled step.
    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut ctx::StepCtx<'_>);

    /// React to a subscribed event (delivered in the documented intra-tick order).
    fn on_event(&self, slice: &mut dyn StateSlice, ev: &EventRecord, ctx: &mut ctx::StepCtx<'_>) {
        let _ = (slice, ev, ctx);
    }

    /// Apply a typed module command (`Command::ModulePayload` routed here at
    /// command-application time, step 1 of the tick order). The outcome is
    /// journaled like any command; a malformed payload MUST be a deterministic
    /// `Rejected`, never a panic. Modules that accept no typed commands keep
    /// this default.
    fn on_command(
        &self,
        slice: &mut dyn StateSlice,
        kind: &str,
        payload: &[u8],
        ctx: &mut ctx::StepCtx<'_>,
    ) -> crate::command::CommandOutcome {
        let _ = (slice, payload, ctx);
        crate::command::CommandOutcome::Rejected(format!(
            "module accepts no typed commands (kind '{kind}')"
        ))
    }

    /// Publish the read-only view(s) of the slice at a tick boundary. Must emit
    /// exactly the views declared in `publishes`.
    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot>;

    /// Serialize the slice (canonical encoding; covered by saves and fingerprints).
    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError>;

    /// Restore the slice.
    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError>;
}

/// Validated registration: modules in derived deterministic order.
pub struct Registry {
    /// Modules in execution order.
    pub modules: Vec<Box<dyn SimModule>>,
    /// Manifests, parallel to `modules`.
    pub manifests: Vec<ModuleManifest>,
    /// View → producing module index (kernel views excluded).
    pub view_producer: BTreeMap<ViewId, usize>,
}

impl Registry {
    /// Validate manifests and derive the deterministic execution order
    /// (FR-CORE-501/503). Order derivation, documented: Kahn's topological sort
    /// over produces→consumes edges, selecting among ready modules by
    /// `(phase, id)` — total, stable, and independent of registration order;
    /// adding a module can never silently reorder unrelated existing modules.
    pub fn build(modules: Vec<Box<dyn SimModule>>, data: &DataSet) -> Result<Registry, CoreError> {
        let manifests: Vec<ModuleManifest> = modules.iter().map(|m| m.manifest()).collect();

        // --- Uniqueness & well-formedness -----------------------------------
        let mut ids = BTreeSet::new();
        let mut slices = BTreeSet::new();
        let mut producers: BTreeMap<ViewId, usize> = BTreeMap::new();
        let mut streams = BTreeSet::new();
        for (i, m) in manifests.iter().enumerate() {
            if m.id == "kernel" || m.id.is_empty() {
                return Err(reg(format!("module id '{}' is reserved or empty", m.id)));
            }
            if !ids.insert(m.id.clone()) {
                return Err(reg(format!("duplicate module id '{}'", m.id)));
            }
            if !slices.insert(m.owned_slice.clone()) {
                return Err(reg(format!(
                    "state slice '{}' claimed by more than one module (single-writer, FR-CORE-502)",
                    m.owned_slice
                )));
            }
            if m.cadence_ticks == 0 {
                return Err(reg(format!("module '{}': cadence_ticks must be ≥ 1", m.id)));
            }
            for v in &m.publishes {
                if v == KERNEL_STATUS_VIEW {
                    return Err(reg(format!(
                        "module '{}' may not publish the kernel view",
                        m.id
                    )));
                }
                if producers.insert(v.clone(), i).is_some() {
                    return Err(reg(format!("view '{v}' published by more than one module")));
                }
            }
            for s in &m.streams {
                if !s.starts_with(&format!("{}/", m.id)) {
                    return Err(reg(format!(
                        "module '{}': stream '{s}' must be namespaced '{}/…'",
                        m.id, m.id
                    )));
                }
                if !streams.insert(s.clone()) {
                    return Err(reg(format!(
                        "stream '{s}' declared by more than one module"
                    )));
                }
            }
            for c in m.emits.iter().chain(&m.subscribes) {
                if data.event_class(c).is_none() {
                    return Err(reg(format!(
                        "module '{}': event class '{c}' is not in the data registry",
                        m.id
                    )));
                }
            }
            for e in &m.escalations {
                if !m.publishes.contains(&e.view) {
                    return Err(reg(format!(
                        "module '{}': escalation reads '{}' which it does not publish",
                        m.id, e.view
                    )));
                }
            }
        }
        // Reads must resolve to the kernel view or a published view.
        for m in &manifests {
            for r in &m.reads {
                if r != KERNEL_STATUS_VIEW && !producers.contains_key(r) {
                    return Err(reg(format!(
                        "module '{}' reads unknown view '{r}' (no registered producer)",
                        m.id
                    )));
                }
            }
        }

        // --- Deterministic topological order --------------------------------
        let n = manifests.len();
        let mut deps: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n]; // i depends on j
        for (i, m) in manifests.iter().enumerate() {
            for r in &m.reads {
                if let Some(&p) = producers.get(r)
                    && p != i
                {
                    deps[i].insert(p);
                }
            }
        }
        let mut order: Vec<usize> = Vec::with_capacity(n);
        let mut placed = vec![false; n];
        while order.len() < n {
            // Ready candidates, selected by (phase, id) — total and stable.
            let next = (0..n)
                .filter(|&i| !placed[i] && deps[i].iter().all(|&j| placed[j]))
                .min_by_key(|&i| (manifests[i].phase, manifests[i].id.clone()));
            let Some(next) = next else {
                let cyclic: Vec<&str> = (0..n)
                    .filter(|&i| !placed[i])
                    .map(|i| manifests[i].id.as_str())
                    .collect();
                return Err(reg(format!(
                    "circular view dependency among modules: {}",
                    cyclic.join(", ")
                )));
            };
            placed[next] = true;
            order.push(next);
        }

        // Re-arrange into execution order.
        let mut indexed: Vec<(usize, Box<dyn SimModule>)> =
            modules.into_iter().enumerate().collect();
        indexed.sort_by_key(|(orig, _)| order.iter().position(|&o| o == *orig).expect("placed"));
        let ordered_modules: Vec<Box<dyn SimModule>> =
            indexed.into_iter().map(|(_, m)| m).collect();
        let ordered_manifests: Vec<ModuleManifest> =
            ordered_modules.iter().map(|m| m.manifest()).collect();
        let mut view_producer = BTreeMap::new();
        for (i, m) in ordered_manifests.iter().enumerate() {
            for v in &m.publishes {
                view_producer.insert(v.clone(), i);
            }
        }

        Ok(Registry {
            modules: ordered_modules,
            manifests: ordered_manifests,
            view_producer,
        })
    }

    /// All views available in this run (kernel + published) — the watch-template
    /// availability set computed at run creation (A1 remediation).
    pub fn available_views(&self) -> Vec<ViewId> {
        let mut v: Vec<ViewId> = vec![KERNEL_STATUS_VIEW.to_string()];
        v.extend(self.view_producer.keys().cloned());
        v
    }
}

fn reg(reason: String) -> CoreError {
    CoreError::RegistrationFailed { reason }
}
