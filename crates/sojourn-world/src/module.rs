//! The `SimModule` implementation (FR-WORLD-702): the world slice (engine-private
//! ground truth + per-faction belief + generated bodies + change log), the
//! command surface (observation, prospecting) routed through the kernel's
//! `ModulePayload`, view publication, and serde with world-data version pinning.
//! Immutable world data (catalogue, sites, locations, fields, priors, Sojournal,
//! astrobiology, resources) is module data; everything dynamic lives in the slice.

use crate::belief::{BeliefChange, FactionId, ObsClass, Priors, Property};
use crate::catalogue::{BodyMetaTable, Resources};
use crate::ids::GeneratedIds;
use crate::locations::Locations;
use crate::observe::{self, BeliefMap};
use crate::prospect::ProspectFields;
use crate::sites::Sites;
use crate::sojournal::Sojournal;
use crate::truth::{AstrobiologyCandidates, GroundTruth};
use serde::{Deserialize, Serialize};
use sojourn_astro::{BodyDef, BodyId, Catalog};
use sojourn_core::{
    Command, CommandOutcome, CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;
use std::path::Path;

/// A journalled world command (routed via `Command::ModulePayload`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorldCommand {
    /// Observe a site's property (or all sensed properties) for a faction.
    Observe {
        /// Acting faction.
        faction: u32,
        /// Site data key.
        site: String,
        /// Specific property, or `None` for all the class can sense.
        property: Option<Property>,
        /// Observation class.
        class: ObsClass,
        /// Instrument quality in `[0, 1]`.
        quality: f64,
    },
    /// Prospect a field for a faction, minting new small bodies.
    Prospect {
        /// Discovering faction.
        faction: u32,
        /// Field data key.
        field: String,
        /// Effort applied.
        effort: f64,
    },
}

/// Encode a [`WorldCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn world_payload(cmd: &WorldCommand) -> Command {
    Command::ModulePayload {
        module: "world".into(),
        kind: match cmd {
            WorldCommand::Observe { .. } => "observe",
            WorldCommand::Prospect { .. } => "prospect",
        }
        .into(),
        payload: postcard::to_allocvec(cmd).expect("world command encodes"),
    }
}

/// The world slice — engine-private truth, per-faction belief, generated bodies,
/// the change log and the generated-id counter. Serialized; covered by saves,
/// fingerprints and the determinism gates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSlice {
    /// Engine-private ground truth (never published; never in a snapshot).
    pub(crate) truth: GroundTruth,
    /// Per-faction belief.
    pub(crate) beliefs: BeliefMap,
    /// Tick-stamped belief revisions (for UI deltas).
    pub(crate) change_log: Vec<BeliefChange>,
    /// Prospecting-generated bodies (per-world facts), by id.
    pub(crate) generated: BTreeMap<BodyId, BodyDef>,
    /// Generated-id allocator.
    pub(crate) gen_ids: GeneratedIds,
    /// World-data content hash (saves must reload against the same data).
    pub(crate) world_hash: [u8; 32],
}

/// The world module.
pub struct WorldModule {
    /// Real catalogue over `data/world` (for queries, locations, prospecting).
    pub catalog: Catalog,
    /// Surveyable sites.
    pub sites: Sites,
    /// Belief priors / floors / refinement params.
    pub priors: Priors,
    /// Dynamical locations.
    pub locations: Locations,
    /// Prospecting fields.
    pub fields: ProspectFields,
    /// Sojournal data.
    pub sojournal: Sojournal,
    /// Astrobiology candidate worlds.
    pub astrobiology: AstrobiologyCandidates,
    /// Resource taxonomy.
    pub resources: Resources,
    /// Per-body metadata.
    pub body_meta: BodyMetaTable,
    /// Combined world-data content hash (pinned in saves).
    pub world_hash: [u8; 32],
    /// System root (the Sun) — parent of generated bodies.
    pub root: BodyId,
}

impl WorldModule {
    /// Load and validate all world data from a directory.
    pub fn load(dir: &Path) -> Result<WorldModule, String> {
        let catalog = Catalog::load_dir(dir)?;
        let sites = Sites::load_dir(dir)?;
        let priors = Priors::load_dir(dir)?;
        let locations = Locations::load_dir(dir)?;
        locations.validate_refs(&catalog)?;
        let fields = ProspectFields::load_dir(dir)?;
        let sojournal = Sojournal::load_dir(dir)?;
        let astrobiology = AstrobiologyCandidates::load_dir(dir)?;
        let resources = Resources::load_dir(dir)?;
        let body_meta = BodyMetaTable::load_dir(dir)?;

        // Cross-validate: site anchors and resources exist.
        for (_sid, d) in sites.all() {
            if catalog.body(d.body).is_none() {
                return Err(format!(
                    "site '{}' anchors to unknown body {}",
                    d.id, d.body.0
                ));
            }
            if !resources.has(&d.resource) {
                return Err(format!(
                    "site '{}' references unknown resource '{}'",
                    d.id, d.resource
                ));
            }
        }

        // World-data content hash (fixed order; catalogue is pinned separately by
        // the astro slice).
        let mut hasher = blake3::Hasher::new();
        for part in [
            &priors.canonical,
            &sites.canonical,
            &locations.canonical,
            &fields.canonical,
            &sojournal.canonical,
            &astrobiology.canonical,
            &resources.canonical,
            &body_meta.canonical,
        ] {
            hasher.update(part.as_bytes());
            hasher.update(b"\0");
        }
        let world_hash = *hasher.finalize().as_bytes();
        let root = catalog.root();

        Ok(WorldModule {
            catalog,
            sites,
            priors,
            locations,
            fields,
            sojournal,
            astrobiology,
            resources,
            body_meta,
            world_hash,
            root,
        })
    }

    fn slice_mut<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut WorldSlice {
        slice
            .as_any_mut()
            .downcast_mut::<WorldSlice>()
            .expect("world slice type")
    }
}

impl SimModule for WorldModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "world".into(),
            owned_slice: "world/slice".into(),
            publishes: vec!["world/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec!["body-catalogued".into(), "survey-milestone".into()],
            subscribes: vec![],
            phase: 1,
            streams: vec![
                "world/seed".into(),
                "world/obs-noise".into(),
                "world/prospect".into(),
            ],
            cadence_ticks: 86_400, // event-driven; nothing time-dependent to step
            escalations: vec![],
        }
    }

    fn init(&self, ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        let truth = {
            let rng = ctx.rng("world/seed");
            GroundTruth::seed(&self.sites, &self.astrobiology, rng)
        };
        Box::new(WorldSlice {
            truth,
            beliefs: BeliefMap::new(),
            change_log: Vec::new(),
            generated: BTreeMap::new(),
            gen_ids: GeneratedIds::default(),
            world_hash: self.world_hash,
        })
    }

    fn step(&self, _slice: &mut dyn StateSlice, _ctx: &mut StepCtx<'_>) {
        // No time-dependent dynamics: belief changes only on observation commands,
        // bodies are generated only on prospecting commands.
    }

    fn on_command(
        &self,
        slice: &mut dyn StateSlice,
        kind: &str,
        payload: &[u8],
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        let cmd: WorldCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed world payload (kind '{kind}'): {e}"
                ));
            }
        };
        let tick = ctx.tick().0;
        let s = self.slice_mut(slice);
        match cmd {
            WorldCommand::Observe {
                faction,
                site,
                property,
                class,
                quality,
            } => {
                let res = {
                    let rng = ctx.rng("world/obs-noise");
                    observe::apply_observe(
                        &mut s.beliefs,
                        &s.truth,
                        &mut s.change_log,
                        &self.priors,
                        &self.sites,
                        rng,
                        tick,
                        FactionId(faction),
                        &site,
                        property,
                        class,
                        quality,
                    )
                };
                for p in &res.milestones {
                    ctx.emit(
                        "survey-milestone",
                        BTreeMap::from([
                            ("site".to_string(), ViewValue::Str(site.clone())),
                            ("faction".to_string(), ViewValue::U64(u64::from(faction))),
                            ("property".to_string(), ViewValue::Str(format!("{p:?}"))),
                        ]),
                    );
                }
                res.outcome
            }
            WorldCommand::Prospect {
                faction,
                field,
                effort,
            } => {
                let Some(fld) = self.fields.get(&field) else {
                    return CommandOutcome::Rejected(format!(
                        "unknown prospecting field '{field}'"
                    ));
                };
                if !(effort.is_finite() && effort >= 0.0) {
                    return CommandOutcome::Rejected("effort must be finite and ≥ 0".into());
                }
                let new = {
                    let rng = ctx.rng("world/prospect");
                    crate::prospect::generate(
                        fld,
                        effort,
                        rng,
                        &mut s.gen_ids,
                        self.root,
                        tick as i64 * 1_000_000_000,
                    )
                };
                let mut new_ids = Vec::with_capacity(new.len());
                for b in new {
                    new_ids.push(b.id);
                    s.generated.insert(b.id, b);
                }
                for id in new_ids {
                    ctx.emit(
                        "body-catalogued",
                        BTreeMap::from([
                            ("body".to_string(), ViewValue::U64(u64::from(id.0))),
                            ("field".to_string(), ViewValue::Str(field.clone())),
                            ("faction".to_string(), ViewValue::U64(u64::from(faction))),
                        ]),
                    );
                }
                CommandOutcome::Applied
            }
        }
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<WorldSlice>()
            .expect("world slice type");
        vec![ViewSnapshot {
            id: "world/status".into(),
            fields: BTreeMap::from([
                (
                    "belief_count".to_string(),
                    ViewValue::U64(s.beliefs.len() as u64),
                ),
                (
                    "generated_count".to_string(),
                    ViewValue::U64(s.generated.len() as u64),
                ),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<WorldSlice>()
            .expect("world slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: WorldSlice = postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })?;
        if s.world_hash != self.world_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.world_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's world data differs from the loaded data/world content; \
                       install the matching world data version"
                    .into(),
            });
        }
        Ok(Box::new(s))
    }
}
