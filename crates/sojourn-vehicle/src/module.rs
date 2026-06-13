//! The `SimModule` implementation (FR-VD-802): the design-library slice, the
//! command surface (compose/edit/save/derive/register-production) via
//! `ModulePayload`, view publication, and serde with component-data pinning.
//! Flight-time craft state stays in FA-02 (clarified Q1:A); zero random streams
//! (derivations are deterministic pure computations).

use crate::catalogue::Catalogue;
use crate::design::{MissionReqs, RedundancyBlock, Stage, VehicleDesign};
use crate::ids::{DesignId, FactionId};
use serde::{Deserialize, Serialize};
use sojourn_core::{
    Command, CommandOutcome, CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;
use std::path::Path;

/// A journalled vehicle command (routed via `Command::ModulePayload`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VehicleCommand {
    /// Compose a new design.
    ComposeDesign {
        /// Owner.
        faction: u32,
        /// Class-template id.
        class: String,
        /// Stages.
        stages: Vec<Stage>,
        /// Redundancy blocks.
        redundancy: Vec<RedundancyBlock>,
        /// Stated mission requirements.
        mission: MissionReqs,
    },
    /// Edit a design in place (a new revision; derivatives are independent copies).
    EditDesign {
        /// Owner.
        faction: u32,
        /// Design id.
        design: u32,
        /// New stages.
        stages: Vec<Stage>,
        /// New redundancy blocks.
        redundancy: Vec<RedundancyBlock>,
        /// New mission requirements.
        mission: MissionReqs,
    },
    /// Create a derivative (independent copy of a parent's composition; heritage
    /// discounts apply via the parent's component techs in FA-05).
    DeriveDesign {
        /// Owner.
        faction: u32,
        /// Parent design id.
        parent: u32,
    },
    /// Register cumulative production (learning curve) + emit `vehicle-produced`.
    RegisterProduction {
        /// Owner.
        faction: u32,
        /// Design id.
        design: u32,
        /// Units produced.
        units: u32,
    },
}

/// Encode a [`VehicleCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn vehicle_payload(cmd: &VehicleCommand) -> Command {
    Command::ModulePayload {
        module: "vehicle".into(),
        kind: match cmd {
            VehicleCommand::ComposeDesign { .. } => "compose",
            VehicleCommand::EditDesign { .. } => "edit",
            VehicleCommand::DeriveDesign { .. } => "derive",
            VehicleCommand::RegisterProduction { .. } => "register-production",
        }
        .into(),
        payload: postcard::to_allocvec(cmd).expect("vehicle command encodes"),
    }
}

/// The vehicle slice — the design library (the only design-time state).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleSlice {
    /// Designs by id.
    pub(crate) designs: BTreeMap<DesignId, VehicleDesign>,
    /// Next design id.
    pub(crate) next_design: u32,
    /// Component-data content hash (pin).
    pub(crate) data_hash: [u8; 32],
}

/// The vehicle module.
pub struct VehicleModule {
    /// Immutable component catalogue + propulsion + params + classes.
    pub catalogue: Catalogue,
}

impl VehicleModule {
    /// Load and validate the component catalogue from a data directory.
    pub fn load(dir: &Path) -> Result<VehicleModule, String> {
        Ok(VehicleModule {
            catalogue: Catalogue::load_dir(dir)?,
        })
    }

    fn slice_mut<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut VehicleSlice {
        slice
            .as_any_mut()
            .downcast_mut::<VehicleSlice>()
            .expect("vehicle slice type")
    }

    fn valid_components(&self, stages: &[Stage]) -> Result<(), String> {
        for s in stages {
            for id in &s.components {
                if self.catalogue.component(id).is_none() {
                    return Err(format!("unknown component '{}'", id.0));
                }
            }
            if let Some(e) = &s.engine
                && self.catalogue.component(e).is_none()
            {
                return Err(format!("unknown engine '{}'", e.0));
            }
        }
        Ok(())
    }
}

impl SimModule for VehicleModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "vehicle".into(),
            owned_slice: "vehicle/slice".into(),
            publishes: vec!["vehicle/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec!["vehicle-produced".into()],
            subscribes: vec![],
            phase: 1,
            streams: vec![],
            cadence_ticks: 86_400,
            escalations: vec![],
        }
    }

    fn init(&self, _ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        Box::new(VehicleSlice {
            designs: BTreeMap::new(),
            next_design: 0,
            data_hash: self.catalogue.content_hash,
        })
    }

    fn step(&self, _slice: &mut dyn StateSlice, _ctx: &mut StepCtx<'_>) {
        // Designs do not evolve with time; everything is command-driven.
    }

    fn on_command(
        &self,
        slice: &mut dyn StateSlice,
        kind: &str,
        payload: &[u8],
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let cmd: VehicleCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed vehicle payload (kind '{kind}'): {e}"
                ));
            }
        };
        let s = self.slice_mut(slice);
        match cmd {
            VehicleCommand::ComposeDesign {
                faction,
                class,
                stages,
                redundancy,
                mission,
            } => {
                if !self.catalogue.classes.contains_key(&class) {
                    return CommandOutcome::Rejected(format!("unknown class '{class}'"));
                }
                if let Err(e) = self.valid_components(&stages) {
                    return CommandOutcome::Rejected(e);
                }
                let id = DesignId(s.next_design);
                s.next_design += 1;
                s.designs.insert(
                    id,
                    VehicleDesign {
                        id,
                        faction: FactionId(faction),
                        class,
                        stages,
                        redundancy,
                        mission,
                        parent: None,
                        production_count: 0,
                    },
                );
                CommandOutcome::Applied
            }
            VehicleCommand::EditDesign {
                faction,
                design,
                stages,
                redundancy,
                mission,
            } => {
                if let Err(e) = self.valid_components(&stages) {
                    return CommandOutcome::Rejected(e);
                }
                let Some(d) = s.designs.get_mut(&DesignId(design)) else {
                    return CommandOutcome::Rejected(format!("unknown design {design}"));
                };
                if d.faction != FactionId(faction) {
                    return CommandOutcome::Rejected("design belongs to another faction".into());
                }
                d.stages = stages;
                d.redundancy = redundancy;
                d.mission = mission;
                CommandOutcome::Applied
            }
            VehicleCommand::DeriveDesign { faction, parent } => {
                let Some(p) = s.designs.get(&DesignId(parent)).cloned() else {
                    return CommandOutcome::Rejected(format!("unknown parent design {parent}"));
                };
                let id = DesignId(s.next_design);
                s.next_design += 1;
                s.designs.insert(
                    id,
                    VehicleDesign {
                        id,
                        faction: FactionId(faction),
                        parent: Some(DesignId(parent)),
                        production_count: 0,
                        ..p
                    },
                );
                CommandOutcome::Applied
            }
            VehicleCommand::RegisterProduction {
                faction,
                design,
                units,
            } => {
                let Some(d) = s.designs.get_mut(&DesignId(design)) else {
                    return CommandOutcome::Rejected(format!("unknown design {design}"));
                };
                d.production_count = d.production_count.saturating_add(units);
                ctx.emit(
                    "vehicle-produced",
                    BTreeMap::from([
                        ("design".to_string(), ViewValue::U64(u64::from(design))),
                        ("faction".to_string(), ViewValue::U64(u64::from(faction))),
                        ("units".to_string(), ViewValue::U64(u64::from(units))),
                    ]),
                );
                CommandOutcome::Applied
            }
        }
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<VehicleSlice>()
            .expect("vehicle slice type");
        vec![ViewSnapshot {
            id: "vehicle/status".into(),
            fields: BTreeMap::from([(
                "designs".to_string(),
                ViewValue::U64(s.designs.len() as u64),
            )]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<VehicleSlice>()
            .expect("vehicle slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: VehicleSlice =
            postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
                reason: e.to_string(),
            })?;
        if s.data_hash != self.catalogue.content_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.data_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's vehicle component data differs from the loaded data/vehicle \
                       content; install the matching version"
                    .into(),
            });
        }
        Ok(Box::new(s))
    }
}
