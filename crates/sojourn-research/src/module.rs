//! The `SimModule` implementation (FR-RESP-802): the research slice, the command
//! surface (via `ModulePayload`), the **time-stepped `step()` orchestration** (the
//! deterministic subsystem order that wins warp-invariance — G1), view publication,
//! and serde with research-data version pinning. Immutable tree/domain/params/trait
//! data is module data; everything dynamic lives in the slice.

use crate::breakthrough::{self, InsightMap};
use crate::domains::{self, UlMap, WorldUlMap};
use crate::ids::{DomainId, FactionId, PersonId, ProgramId, TechId};
use crate::personnel::{self, Person, Role, Roster};
use crate::programs::{self, Program};
use crate::reliability::{Heritage, HeritageMap};
use crate::rp_de::{AllocMap, Allocation};
use crate::seeding::{self, DeadEnds, Thresholds};
use crate::tree::{PrereqKind, ResearchData};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sojourn_core::{
    Command, CommandOutcome, CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;
use std::path::Path;

/// Base tick is 1 s; the research module steps daily.
const TICKS_PER_DAY: u64 = 86_400;

/// A journalled research command (routed via `Command::ModulePayload`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResearchCommand {
    /// Set a faction's RP/DE portfolio split + available facilities.
    SetAllocation {
        /// Faction.
        faction: u32,
        /// Science split: (domain, weight).
        domain_splits: Vec<(DomainId, f64)>,
        /// Engineering split: (program id, weight).
        program_splits: Vec<(u32, f64)>,
        /// Available facility capabilities (caller-asserted).
        facilities: Vec<String>,
    },
    /// Open an engineering program for a technology (gated on UL floors + prereqs).
    StartProgram {
        /// Faction.
        faction: u32,
        /// Target tech.
        tech: TechId,
        /// Optional lead person.
        lead: Option<u32>,
    },
    /// Publish or hold a domain.
    SetPublishPolicy {
        /// Faction.
        faction: u32,
        /// Domain.
        domain: DomainId,
        /// Publish (true) or hold (false).
        publish: bool,
    },
    /// Hire a person onto the roster.
    Hire {
        /// Faction.
        faction: u32,
        /// Role.
        role: Role,
        /// Discipline domain.
        discipline: DomainId,
        /// Skill rating.
        skill: f64,
        /// Trait ids.
        traits: Vec<String>,
    },
    /// Poach a person (a hire that signals a relations cost — FA-09 later).
    Poach {
        /// Faction.
        faction: u32,
        /// Role.
        role: Role,
        /// Discipline domain.
        discipline: DomainId,
        /// Skill rating.
        skill: f64,
        /// Trait ids.
        traits: Vec<String>,
    },
    /// Begin training a person (astronaut pipeline → Training).
    Train {
        /// Person id.
        person: u32,
    },
    /// Retire a person.
    Retire {
        /// Person id.
        person: u32,
    },
    /// Inject understanding into domains (a mission, FR-RESP-104).
    InjectUnderstanding {
        /// Faction.
        faction: u32,
        /// (domain, amount) injections.
        injections: Vec<(DomainId, f64)>,
    },
    /// Register operational-use heritage for a technology (FA-04+ drive it).
    RegisterHeritage {
        /// Faction.
        faction: u32,
        /// Tech.
        tech: TechId,
        /// Successful units.
        units: u32,
    },
    /// FA-08 in-mission crew feedback (dose/health/psych deltas).
    CrewFeedback {
        /// Faction.
        faction: u32,
        /// Astronaut person id.
        astronaut: u32,
        /// Dose delta.
        dose_delta: f64,
        /// Health delta.
        health_delta: f64,
        /// Psych delta.
        psych_delta: f64,
    },
}

/// Encode a [`ResearchCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn research_payload(cmd: &ResearchCommand) -> Command {
    Command::ModulePayload {
        module: "research".into(),
        kind: kind_of(cmd).into(),
        payload: postcard::to_allocvec(cmd).expect("research command encodes"),
    }
}

fn kind_of(cmd: &ResearchCommand) -> &'static str {
    use ResearchCommand::*;
    match cmd {
        SetAllocation { .. } => "set-allocation",
        StartProgram { .. } => "start-program",
        SetPublishPolicy { .. } => "set-publish-policy",
        Hire { .. } => "hire",
        Poach { .. } => "poach",
        Train { .. } => "train",
        Retire { .. } => "retire",
        InjectUnderstanding { .. } => "inject-understanding",
        RegisterHeritage { .. } => "register-heritage",
        CrewFeedback { .. } => "crew-feedback",
    }
}

/// The research slice — all dynamic state, serialized and gate-covered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchSlice {
    /// Ground Understanding Levels.
    pub(crate) ul: UlMap,
    /// World tide.
    pub(crate) world_ul: WorldUlMap,
    /// Publish/hold policy.
    pub(crate) publish: BTreeMap<(FactionId, DomainId), bool>,
    /// Engineering programs.
    pub(crate) programs: BTreeMap<ProgramId, Program>,
    /// Flight heritage.
    pub(crate) heritage: HeritageMap,
    /// Seeded dead ends.
    pub(crate) dead_ends: DeadEnds,
    /// Seeded breakthrough thresholds.
    pub(crate) thresholds: Thresholds,
    /// Breakthrough insight pressure.
    pub(crate) insight: InsightMap,
    /// Personnel roster.
    pub(crate) roster: Roster,
    /// Per-faction allocations.
    pub(crate) alloc: AllocMap,
    /// Next program id.
    pub(crate) next_program: u32,
    /// Next person id.
    pub(crate) next_person: u32,
    /// Last tick stepped to.
    pub(crate) last_tick: u64,
    /// Research-data content hash (pin).
    pub(crate) research_hash: [u8; 32],
}

/// The research module.
pub struct ResearchModule {
    /// Immutable research data.
    pub data: ResearchData,
}

impl ResearchModule {
    /// Load and validate research data from `data/research` + `data/tech` (given the
    /// parent data dir).
    pub fn load(data_dir: &Path) -> Result<ResearchModule, String> {
        let data = ResearchData::load(&data_dir.join("research"), &data_dir.join("tech"))?;
        Ok(ResearchModule { data })
    }

    fn slice_mut<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut ResearchSlice {
        slice
            .as_any_mut()
            .downcast_mut::<ResearchSlice>()
            .expect("research slice type")
    }

    /// Effective-UL tacit penalty closure for a roster snapshot.
    fn tacit<'a>(&'a self, roster: &'a Roster) -> impl Fn(FactionId, &DomainId) -> f64 + 'a {
        move |f, d| personnel::tacit_penalty(roster, &self.data.params, f, d)
    }
}

type Ev = (String, BTreeMap<String, ViewValue>);

impl SimModule for ResearchModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "research".into(),
            owned_slice: "research/slice".into(),
            publishes: vec!["research/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "breakthrough".into(),
                "dead-end-confirmed".into(),
                "test-failure".into(),
                "program-milestone".into(),
                "trl-advance".into(),
                "publish".into(),
            ],
            subscribes: vec![],
            phase: 1,
            streams: vec![
                "research/seed".into(),
                "research/overrun".into(),
                "research/test".into(),
                "research/breakthrough".into(),
            ],
            cadence_ticks: TICKS_PER_DAY,
            escalations: vec![],
        }
    }

    fn init(&self, ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        let (dead_ends, thresholds) = {
            let rng = ctx.rng("research/seed");
            seeding::seed(&self.data, rng)
        };
        Box::new(ResearchSlice {
            ul: BTreeMap::new(),
            world_ul: BTreeMap::new(),
            publish: BTreeMap::new(),
            programs: BTreeMap::new(),
            heritage: BTreeMap::new(),
            dead_ends,
            thresholds,
            insight: BTreeMap::new(),
            roster: BTreeMap::new(),
            alloc: BTreeMap::new(),
            next_program: 0,
            next_person: 0,
            last_tick: 0,
            research_hash: self.data.content_hash,
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

        let mut events: Vec<Ev> = Vec::new();
        {
            let rng = ctx.rng("research/test");
            let mut uniform = || (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
            step_world(self, s, dt_days, now, &mut uniform, &mut events);
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
        let cmd: ResearchCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed research payload (kind '{kind}'): {e}"
                ));
            }
        };
        let s = self.slice_mut(slice);
        apply_command(self, s, cmd, ctx)
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<ResearchSlice>()
            .expect("research slice type");
        let matured = s.programs.values().filter(|p| p.trl >= 6).count() as u64;
        vec![ViewSnapshot {
            id: "research/status".into(),
            fields: BTreeMap::from([
                (
                    "programs".to_string(),
                    ViewValue::U64(s.programs.len() as u64),
                ),
                ("matured".to_string(), ViewValue::U64(matured)),
                ("people".to_string(), ViewValue::U64(s.roster.len() as u64)),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<ResearchSlice>()
            .expect("research slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: ResearchSlice =
            postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
                reason: e.to_string(),
            })?;
        if s.research_hash != self.data.content_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.research_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's research data differs from the loaded data/research+data/tech \
                       content; install the matching research data version"
                    .into(),
            });
        }
        Ok(Box::new(s))
    }
}

// ----------------------------------------------------------------- stepping (G1)

/// The deterministic subsystem order: domains → RP/DE → programs → campaigns → tide
/// → insight → personnel/aging. Warp-invariant (driven by elapsed days).
fn step_world(
    m: &ResearchModule,
    s: &mut ResearchSlice,
    dt_days: f64,
    now: u64,
    uniform: &mut impl FnMut() -> f64,
    events: &mut Vec<Ev>,
) {
    let data = &m.data;
    let params = &data.params;
    let factions: Vec<FactionId> = s.alloc.keys().copied().collect();

    for f in &factions {
        let alloc = s.alloc.get(f).cloned().unwrap_or_default();
        let rp_total = personnel::faction_rp(&s.roster, params, *f) * dt_days;
        let de_total = personnel::faction_de(&s.roster, params, *f) * dt_days;

        // --- Track A: domains + insight -----------------------------------
        let dom_keys: Vec<DomainId> = alloc.domain_splits.iter().map(|(d, _)| d.clone()).collect();
        for dom in &dom_keys {
            let rp = rp_total * alloc.domain_fraction(dom);
            if rp <= 0.0 {
                continue;
            }
            let world = domains::world_ul(&s.world_ul, dom);
            domains::grow_domain(&mut s.ul, data, params, *f, dom, rp, world);
            breakthrough::accrue(&mut s.insight, data, params, *f, dom, rp, 1.0);
            if breakthrough::check_trigger(&mut s.insight, &s.thresholds, *f, dom) {
                let src = data
                    .domains
                    .get(dom)
                    .map(|d| d.source.clone())
                    .unwrap_or_default();
                events.push((
                    "breakthrough".into(),
                    BTreeMap::from([
                        ("faction".to_string(), ViewValue::U64(u64::from(f.0))),
                        ("domain".to_string(), ViewValue::Str(dom.0.clone())),
                        ("reference".to_string(), ViewValue::Str(src)),
                    ]),
                ));
            }
        }

        // --- Track B: programs + test campaigns ---------------------------
        let prog_ids: Vec<ProgramId> = s
            .programs
            .values()
            .filter(|p| p.faction == *f)
            .map(|p| p.id)
            .collect();
        for pid in prog_ids {
            let de = de_total * alloc.program_fraction(pid);
            if de <= 0.0 {
                continue;
            }
            let (tech, trl, lead) = {
                let p = &s.programs[&pid];
                (p.tech.clone(), p.trl, p.lead)
            };
            let Some(node) = data.nodes.get(&tech) else {
                continue;
            };
            let is_dead_end = seeding::is_dead_end(&s.dead_ends, &tech, trl + 1);
            let lead_p = lead.and_then(|id| s.roster.get(&PersonId(id.0)));
            let low_mult = personnel::trait_mult(data, lead_p, |t| t.low_trl_mult);
            let qual_mult = personnel::trait_mult(data, lead_p, |t| t.qual_mult);
            let overrun_mult = personnel::trait_mult(data, lead_p, |t| t.overrun_mult);
            let ul_margin = ul_margin(s, data, *f, node, &m.tacit(&s.roster));

            let p = s.programs.get_mut(&pid).expect("program");
            let out = programs::advance(
                p,
                node,
                de,
                dt_days,
                ul_margin,
                is_dead_end,
                &alloc,
                params,
                low_mult,
                qual_mult,
                overrun_mult,
                uniform,
            );
            if let Some(trl) = out.trl_advanced {
                events.push(("trl-advance".into(), prog_payload(pid, *f, Some(trl))));
            }
            if let Some(spectacular) = out.test_failed {
                let mut pl = prog_payload(pid, *f, None);
                pl.insert("spectacular".to_string(), ViewValue::Bool(spectacular));
                events.push(("test-failure".into(), pl));
            }
            if out.dead_end_confirmed {
                events.push(("dead-end-confirmed".into(), prog_payload(pid, *f, None)));
            }
            if out.ul_inject > 0.0
                && let Some((dom, _)) = node.ul_floors.first()
            {
                domains::inject(&mut s.ul, *f, dom, out.ul_inject);
            }
        }
    }

    // --- Tide ------------------------------------------------------------
    let publish = s.publish.clone();
    let publishing =
        |f: FactionId, d: &DomainId| publish.get(&(f, d.clone())).copied().unwrap_or(false);
    crate::tide::advance(
        &mut s.world_ul,
        &s.ul,
        data,
        params,
        &factions,
        &publishing,
        dt_days,
    );

    // --- Personnel aging / training -------------------------------------
    personnel::step_people(&mut s.roster, dt_days);
    let _ = now;
}

/// UL margin = how far the faction's effective UL exceeds the tech's floors (mean).
fn ul_margin(
    s: &ResearchSlice,
    _data: &ResearchData,
    f: FactionId,
    node: &crate::tree::TechNode,
    tacit: &impl Fn(FactionId, &DomainId) -> f64,
) -> f64 {
    if node.ul_floors.is_empty() {
        return 50.0;
    }
    let sum: f64 = node
        .ul_floors
        .iter()
        .map(|(d, fl)| (domains::effective_ul(&s.ul, f, d, tacit(f, d)) - fl).max(0.0))
        .sum();
    sum / node.ul_floors.len() as f64
}

fn prog_payload(pid: ProgramId, f: FactionId, trl: Option<u8>) -> BTreeMap<String, ViewValue> {
    let mut m = BTreeMap::from([
        ("program".to_string(), ViewValue::U64(u64::from(pid.0))),
        ("faction".to_string(), ViewValue::U64(u64::from(f.0))),
    ]);
    if let Some(t) = trl {
        m.insert("trl".to_string(), ViewValue::U64(u64::from(t)));
    }
    m
}

// ----------------------------------------------------------------- commands

fn apply_command(
    m: &ResearchModule,
    s: &mut ResearchSlice,
    cmd: ResearchCommand,
    ctx: &mut StepCtx<'_>,
) -> CommandOutcome {
    use ResearchCommand::*;
    let data = &m.data;
    let reject = |r: String| CommandOutcome::Rejected(r);
    match cmd {
        SetAllocation {
            faction,
            domain_splits,
            program_splits,
            facilities,
        } => {
            for (d, _) in &domain_splits {
                if !data.domains.contains_key(d) {
                    return reject(format!("unknown domain '{}'", d.0));
                }
            }
            s.alloc.insert(
                FactionId(faction),
                Allocation {
                    domain_splits,
                    program_splits: program_splits
                        .into_iter()
                        .map(|(p, w)| (ProgramId(p), w))
                        .collect(),
                    facilities,
                },
            );
            CommandOutcome::Applied
        }
        StartProgram {
            faction,
            tech,
            lead,
        } => {
            let f = FactionId(faction);
            let Some(node) = data.nodes.get(&tech) else {
                return reject(format!("unknown tech '{}'", tech.0));
            };
            // UL floors (effective).
            let tacit = m.tacit(&s.roster);
            if let Some((dom, fl)) = domains::first_unmet_floor(&s.ul, f, node, &tacit) {
                return reject(format!("UL floor unmet: domain '{}' needs {fl}", dom.0));
            }
            // Tech prereqs: Product requires a matured (TRL≥6) program; UlSatisfiable
            // is a leapfrog seam (satisfied via UL above — no product needed).
            for p in &node.tech_prereqs {
                if p.kind == PrereqKind::Product {
                    let have = s
                        .programs
                        .values()
                        .any(|pr| pr.faction == f && pr.tech == p.tech && pr.trl >= 6);
                    if !have {
                        return reject(format!("product prereq '{}' not matured", p.tech.0));
                    }
                }
            }
            // Derivative discount: start partway up the ladder per parent heritage.
            let start = node
                .derivative_of
                .as_ref()
                .map(|parent| {
                    let h = s
                        .heritage
                        .get(&(f, parent.clone()))
                        .map(|h| h.flight_units)
                        .unwrap_or(0);
                    crate::reliability::derivative_start_trl(node.start_trl, h)
                })
                .unwrap_or(node.start_trl);
            let id = ProgramId(s.next_program);
            s.next_program += 1;
            let mut prog = Program {
                id,
                faction: f,
                tech: tech.clone(),
                trl: start,
                step_progress: 0.0,
                days_on_step: 0.0,
                est_p50_days: 0.0,
                est_p80_days: 0.0,
                actual_days: 0.0,
                lead: lead.map(PersonId),
                risk_index: 0.0,
                fail_streak: 0,
                parallel_tag: Some(node.capability_category.clone()),
            };
            prog.estimate(node, &data.params);
            s.programs.insert(id, prog);
            CommandOutcome::Applied
        }
        SetPublishPolicy {
            faction,
            domain,
            publish,
        } => {
            if !data.domains.contains_key(&domain) {
                return reject(format!("unknown domain '{}'", domain.0));
            }
            let f = FactionId(faction);
            s.publish.insert((f, domain.clone()), publish);
            if publish {
                ctx.emit(
                    "publish",
                    BTreeMap::from([
                        ("faction".to_string(), ViewValue::U64(u64::from(faction))),
                        ("domain".to_string(), ViewValue::Str(domain.0)),
                    ]),
                );
            }
            CommandOutcome::Applied
        }
        Hire {
            faction,
            role,
            discipline,
            skill,
            traits,
        }
        | Poach {
            faction,
            role,
            discipline,
            skill,
            traits,
        } => {
            if !data.domains.contains_key(&discipline) {
                return reject(format!("unknown discipline '{}'", discipline.0));
            }
            let id = PersonId(s.next_person);
            s.next_person += 1;
            let astronaut = (role == Role::Astronaut).then(crate::astronaut::AstroState::default);
            s.roster.insert(
                id,
                Person {
                    id,
                    faction: FactionId(faction),
                    role,
                    discipline,
                    skill: skill.max(0.0),
                    traits,
                    age: 30.0,
                    morale: 1.0,
                    training_days_left: 0.0,
                    astronaut,
                },
            );
            CommandOutcome::Applied
        }
        Train { person } => {
            let Some(p) = s.roster.get_mut(&PersonId(person)) else {
                return reject(format!("unknown person {person}"));
            };
            p.training_days_left = data.params.astronaut_train_days;
            if let Some(a) = &mut p.astronaut {
                a.stage = crate::astronaut::Stage::Training;
            }
            CommandOutcome::Applied
        }
        Retire { person } => {
            if s.roster.remove(&PersonId(person)).is_none() {
                return reject(format!("unknown person {person}"));
            }
            CommandOutcome::Applied
        }
        InjectUnderstanding {
            faction,
            injections,
        } => {
            let f = FactionId(faction);
            for (dom, _) in &injections {
                if !data.domains.contains_key(dom) {
                    return reject(format!("unknown domain '{}'", dom.0));
                }
            }
            for (dom, amount) in injections {
                domains::inject(&mut s.ul, f, &dom, amount.max(0.0));
            }
            CommandOutcome::Applied
        }
        RegisterHeritage {
            faction,
            tech,
            units,
        } => {
            if !data.nodes.contains_key(&tech) {
                return reject(format!("unknown tech '{}'", tech.0));
            }
            let e = s.heritage.entry((FactionId(faction), tech)).or_default();
            *e = Heritage {
                flight_units: e.flight_units + units,
            };
            CommandOutcome::Applied
        }
        CrewFeedback {
            faction,
            astronaut,
            dose_delta,
            health_delta,
            psych_delta,
        } => {
            let _ = faction;
            let Some(p) = s.roster.get_mut(&PersonId(astronaut)) else {
                return reject(format!("unknown person {astronaut}"));
            };
            let Some(a) = &mut p.astronaut else {
                return reject(format!("person {astronaut} is not an astronaut"));
            };
            a.apply_feedback(
                dose_delta,
                health_delta,
                psych_delta,
                data.params.career_dose_limit,
            );
            CommandOutcome::Applied
        }
    }
}
