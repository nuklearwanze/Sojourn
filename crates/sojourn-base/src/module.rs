//! The `SimModule` implementation: the base slice, the command surface via
//! `ModulePayload`, the daily construction step (delivery-driven commissioning,
//! R8), events, and serde with base-data pinning. Depends only on `sojourn-core`;
//! cross-slice physics arrives inside command payloads as composed values.

use crate::base::{Base, ModuleInstance};
use crate::catalogue::Catalogue;
use crate::construction::{self, ConstructionProject};
use crate::ids::{BaseId, FactionId, ModuleId, ModuleTypeId, ProjectId, SiteId};
use serde::{Deserialize, Serialize};
use sojourn_core::{
    Command, CommandOutcome, CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const TICKS_PER_DAY: u64 = 86_400;

/// A journalled base command (routed via `Command::ModulePayload`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BaseCommand {
    /// Found a base at a Site / dynamical location.
    FoundBase {
        /// Owner.
        faction: u32,
        /// Site id.
        site: String,
        /// Class-template id.
        class: String,
    },
    /// Add a planned module (gated by composed maturity at query time).
    AddModule {
        /// Base id.
        base: u32,
        /// Catalogue module-type id.
        module_type: String,
    },
    /// Open the construction project (derive per-module demands).
    OpenConstruction {
        /// Base id.
        base: u32,
    },
    /// Accrue a delivery (host bridges FA-06 delivery); commission when met.
    DeliverToBase {
        /// Base id.
        base: u32,
        /// Module id.
        module: u32,
        /// Mass delivered (kg).
        mass_kg: f64,
        /// Crew-time applied (hours).
        crew_time_hr: f64,
    },
    /// Satisfy a module's mass with on-site regolith construction (no import).
    BuildLocal {
        /// Base id.
        base: u32,
        /// Module id.
        module: u32,
        /// Local mass built (kg).
        local_mass_kg: f64,
    },
    /// Evaluate an embargo stress test (records result + milestone).
    EvaluateEmbargo {
        /// Base id.
        base: u32,
        /// Embargo span (years).
        years: f64,
        /// Whether the base survives (host computes from the snapshot).
        survives: bool,
    },
    /// Retire a base.
    DecommissionBase {
        /// Base id.
        base: u32,
    },
}

/// Encode a [`BaseCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn base_payload(cmd: &BaseCommand) -> Command {
    let kind = match cmd {
        BaseCommand::FoundBase { .. } => "found",
        BaseCommand::AddModule { .. } => "add-module",
        BaseCommand::OpenConstruction { .. } => "open-construction",
        BaseCommand::DeliverToBase { .. } => "deliver",
        BaseCommand::BuildLocal { .. } => "build-local",
        BaseCommand::EvaluateEmbargo { .. } => "evaluate-embargo",
        BaseCommand::DecommissionBase { .. } => "decommission",
    };
    Command::ModulePayload {
        module: "base".into(),
        kind: kind.into(),
        payload: postcard::to_allocvec(cmd).expect("base command encodes"),
    }
}

/// The base slice — all owned state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseSlice {
    /// Bases by id.
    pub bases: BTreeMap<BaseId, Base>,
    /// Construction projects by id.
    pub projects: BTreeMap<ProjectId, ConstructionProject>,
    /// Next base id.
    pub next_base: u32,
    /// Next project id.
    pub next_project: u32,
    /// Settlement milestones reached.
    pub milestones: BTreeSet<String>,
    /// The query-snapshot tick (set at save; informational).
    pub tick: u64,
    /// Base-data content hash (pin).
    pub data_hash: [u8; 32],
}

/// The base module (immutable loaded data).
pub struct BaseModule {
    /// Module catalogue + params + classes.
    pub catalogue: Catalogue,
}

impl BaseModule {
    /// Load and validate the catalogue from a data directory.
    pub fn load(dir: &Path) -> Result<BaseModule, String> {
        Ok(BaseModule {
            catalogue: Catalogue::load_dir(dir)?,
        })
    }

    fn slice_mut<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut BaseSlice {
        slice
            .as_any_mut()
            .downcast_mut::<BaseSlice>()
            .expect("base slice type")
    }

    /// Check + commission a module if its delivery thresholds are met; returns
    /// `true` if it transitioned to commissioned.
    fn try_commission(
        &self,
        base: &mut Base,
        project: &ConstructionProject,
        mid: ModuleId,
    ) -> bool {
        let Some(demand) = project.demands.get(&mid).copied() else {
            return false;
        };
        let Some(m) = base.modules.get_mut(&mid) else {
            return false;
        };
        if !m.commissioned
            && m.delivered_mass_kg + 1e-9 >= demand.required_mass_kg
            && m.crew_time_hr + 1e-9 >= demand.required_crew_time_hr
        {
            m.commissioned = true;
            return true;
        }
        false
    }
}

impl SimModule for BaseModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "base".into(),
            owned_slice: "base/slice".into(),
            publishes: vec!["base/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "module-commissioned".into(),
                "base-operational".into(),
                "pp-violation".into(),
                "embargo-result".into(),
                "settlement-milestone".into(),
            ],
            subscribes: vec![],
            phase: 1,
            streams: vec![],
            cadence_ticks: TICKS_PER_DAY,
            escalations: vec![],
        }
    }

    fn init(&self, _ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        Box::new(BaseSlice {
            bases: BTreeMap::new(),
            projects: BTreeMap::new(),
            next_base: 0,
            next_project: 0,
            milestones: BTreeSet::new(),
            tick: 0,
            data_hash: self.catalogue.content_hash,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        // Construction is delivery-driven (commands); the step only stamps the tick
        // so the query snapshot reflects current time.
        let now = ctx.tick().0;
        let s = self.slice_mut(slice);
        s.tick = now;
    }

    fn on_command(
        &self,
        slice: &mut dyn StateSlice,
        kind: &str,
        payload: &[u8],
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let cmd: BaseCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed base payload (kind '{kind}'): {e}"
                ));
            }
        };
        let s = self.slice_mut(slice);
        match cmd {
            BaseCommand::FoundBase {
                faction,
                site,
                class,
            } => {
                if !self.catalogue.classes.contains_key(&class) {
                    return CommandOutcome::Rejected(format!("unknown class '{class}'"));
                }
                let id = BaseId(s.next_base);
                s.next_base += 1;
                s.bases.insert(
                    id,
                    Base {
                        id,
                        faction: FactionId(faction),
                        site: SiteId(site),
                        class,
                        modules: BTreeMap::new(),
                        next_module: 0,
                        project: None,
                    },
                );
                CommandOutcome::Applied
            }
            BaseCommand::AddModule { base, module_type } => {
                let tid = ModuleTypeId(module_type);
                if self.catalogue.module(&tid).is_none() {
                    return CommandOutcome::Rejected(format!("unknown module type '{}'", tid.0));
                }
                let Some(b) = s.bases.get_mut(&BaseId(base)) else {
                    return CommandOutcome::Rejected(format!("unknown base {base}"));
                };
                let mid = ModuleId(b.next_module);
                b.next_module += 1;
                b.modules.insert(
                    mid,
                    ModuleInstance {
                        id: mid,
                        type_id: tid,
                        commissioned: false,
                        delivered_mass_kg: 0.0,
                        crew_time_hr: 0.0,
                        built_local: false,
                    },
                );
                CommandOutcome::Applied
            }
            BaseCommand::OpenConstruction { base } => {
                let Some(b) = s.bases.get(&BaseId(base)).cloned() else {
                    return CommandOutcome::Rejected(format!("unknown base {base}"));
                };
                let mut demands = BTreeMap::new();
                for (mid, m) in &b.modules {
                    if let Some(t) = self.catalogue.module(&m.type_id) {
                        demands.insert(*mid, construction::demand_for(t, &self.catalogue));
                    }
                }
                let pid = ProjectId(s.next_project);
                s.next_project += 1;
                s.projects.insert(
                    pid,
                    ConstructionProject {
                        id: pid,
                        base: BaseId(base),
                        demands,
                        started_tick: ctx.tick().0,
                    },
                );
                s.bases.get_mut(&BaseId(base)).unwrap().project = Some(pid);
                CommandOutcome::Applied
            }
            BaseCommand::DeliverToBase {
                base,
                module,
                mass_kg,
                crew_time_hr,
            } => self.accrue(s, base, module, mass_kg, crew_time_hr, false, ctx),
            BaseCommand::BuildLocal {
                base,
                module,
                local_mass_kg,
            } => self.accrue(s, base, module, local_mass_kg, 0.0, true, ctx),
            BaseCommand::EvaluateEmbargo {
                base,
                years,
                survives,
            } => {
                if !s.bases.contains_key(&BaseId(base)) {
                    return CommandOutcome::Rejected(format!("unknown base {base}"));
                }
                ctx.emit(
                    "embargo-result",
                    BTreeMap::from([
                        ("base".to_string(), ViewValue::U64(u64::from(base))),
                        ("years".to_string(), ViewValue::F64(years)),
                        ("survives".to_string(), ViewValue::Bool(survives)),
                    ]),
                );
                if survives {
                    let key = format!("embargo-survivor:{base}");
                    if s.milestones.insert(key) {
                        ctx.emit(
                            "settlement-milestone",
                            BTreeMap::from([
                                ("base".to_string(), ViewValue::U64(u64::from(base))),
                                (
                                    "milestone".to_string(),
                                    ViewValue::Str("embargo-survivor".into()),
                                ),
                            ]),
                        );
                    }
                }
                CommandOutcome::Applied
            }
            BaseCommand::DecommissionBase { base } => {
                if s.bases.remove(&BaseId(base)).is_none() {
                    return CommandOutcome::Rejected(format!("unknown base {base}"));
                }
                CommandOutcome::Applied
            }
        }
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<BaseSlice>()
            .expect("base slice type");
        vec![ViewSnapshot {
            id: "base/status".into(),
            fields: BTreeMap::from([("bases".to_string(), ViewValue::U64(s.bases.len() as u64))]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<BaseSlice>()
            .expect("base slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: BaseSlice = postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })?;
        if s.data_hash != self.catalogue.content_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.data_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's base module data differs from the loaded data/base content; \
                       install the matching version"
                    .into(),
            });
        }
        Ok(Box::new(s))
    }
}

impl BaseModule {
    #[allow(clippy::too_many_arguments)]
    fn accrue(
        &self,
        s: &mut BaseSlice,
        base: u32,
        module: u32,
        mass_kg: f64,
        crew_time_hr: f64,
        local: bool,
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let bid = BaseId(base);
        let mid = ModuleId(module);
        let Some(pid) = s.bases.get(&bid).and_then(|b| b.project) else {
            return CommandOutcome::Rejected(format!(
                "base {base} has no open construction project"
            ));
        };
        let Some(project) = s.projects.get(&pid).cloned() else {
            return CommandOutcome::Rejected("project missing".into());
        };
        let Some(b) = s.bases.get_mut(&bid) else {
            return CommandOutcome::Rejected(format!("unknown base {base}"));
        };
        {
            let Some(m) = b.modules.get_mut(&mid) else {
                return CommandOutcome::Rejected(format!("unknown module {module} in base {base}"));
            };
            m.delivered_mass_kg += mass_kg.max(0.0);
            m.crew_time_hr += crew_time_hr.max(0.0);
            if local {
                m.built_local = true;
            }
        }
        let commissioned = self.try_commission(b, &project, mid);
        if commissioned {
            ctx.emit(
                "module-commissioned",
                BTreeMap::from([
                    ("base".to_string(), ViewValue::U64(u64::from(base))),
                    ("module".to_string(), ViewValue::U64(u64::from(module))),
                ]),
            );
            if b.modules.values().all(|m| m.commissioned) {
                ctx.emit(
                    "base-operational",
                    BTreeMap::from([("base".to_string(), ViewValue::U64(u64::from(base)))]),
                );
            }
        }
        CommandOutcome::Applied
    }
}
