//! The `SimModule` implementation: the polity slice, the command surface via
//! `ModulePayload`, and the **daily seeded step** (events, mood decay, policy drift,
//! the AI world, the astrobiology consensus recompute + conclusive resolution).
//! Depends only on `sojourn-core`; composed values are captured into the slice at
//! command time (the FA-06/FA-08 pattern). The astrobiology **ground truth is drawn
//! at `InitWorld` and never query-exposed**.

use crate::faction::{
    AiState, CandidateState, ContaminationRecord, EventRecord, FactionState, MilestoneClaim,
    ProtectionState, Resolution,
};
use crate::ids::{CandidateId, FactionId, GoalKind, MilestoneId, PolicyId};
use crate::inputs::{
    Achievement, CandidatePrior, CrewFacts, Difficulty, EconomyFacts, EvidenceInput, FactionInit,
    MissionFacts, ScienceTide, SiteProtection,
};
use crate::params::Params;
use crate::{ai, astrobiology, events, milestones, mood, policy, protection};
use serde::{Deserialize, Serialize};
use sojourn_core::{
    Command, CommandOutcome, CoreError, InitCtx, ModuleManifest, SimModule, StateSlice, StepCtx,
    ViewSnapshot, ViewValue,
};
use std::collections::BTreeMap;
use std::path::Path;

const TICKS_PER_DAY: u64 = 86_400;

/// A journalled polity command (routed via `Command::ModulePayload`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolityCommand {
    /// Initialise the world (captures roster + FA-03 priors/PP; draws ground truth).
    InitWorld {
        /// The faction roster.
        factions: Vec<FactionInit>,
        /// Candidate-world priors (FA-03).
        candidates: Vec<CandidatePrior>,
        /// Site planetary-protection facts (FA-03).
        protections: Vec<SiteProtection>,
        /// Difficulty.
        difficulty: Difficulty,
    },
    /// Refresh the composed science tide + difficulty.
    UpdateWorld {
        /// Science tide (FA-05).
        science_tide: ScienceTide,
        /// Difficulty.
        difficulty: Difficulty,
    },
    /// Record an achievement (may claim a world-/faction-first).
    RecordAchievement(Achievement),
    /// Feed the mood model + refresh composed economy bases.
    RecordOutcome {
        /// Outcome facts (FA-08 loss-of-crew among them).
        crew: CrewFacts,
        /// Economy facts (FA-06) the mood modifiers scale.
        economy: EconomyFacts,
    },
    /// Collect a staged astrobiology evidence item (updates a faction's posterior).
    CollectEvidence(EvidenceInput),
    /// Set a policy lever level directly.
    SetPolicy {
        /// Lever id.
        id: String,
        /// New level.
        level: f64,
    },
    /// Lobby a policy lever (seeded nudge; +1 tighten / -1 relax).
    Lobby {
        /// Lobbying faction.
        faction: FactionId,
        /// Lever id.
        id: String,
        /// Direction (+ tighten / − relax).
        direction: f64,
    },
    /// Evaluate a mission for forward/back contamination.
    EvaluateContamination(MissionFacts),
    /// Select the Grand Goal at start.
    SelectGrandGoal {
        /// Faction.
        faction: FactionId,
        /// Goal kind.
        kind: GoalKind,
    },
    /// Change the Grand Goal mid-game (applies the penalty).
    ChangeGrandGoal {
        /// Faction.
        faction: FactionId,
        /// New goal kind.
        kind: GoalKind,
    },
    /// Record composed Homestead embargo-survival index (for the Homestead goal).
    RecordHomestead {
        /// Faction.
        faction: FactionId,
        /// Embargo-survival index ∈ [0,1].
        index: f64,
    },
}

/// Encode a [`PolityCommand`] into the kernel's `Command::ModulePayload` envelope.
pub fn polity_payload(cmd: &PolityCommand) -> Command {
    let kind = match cmd {
        PolityCommand::InitWorld { .. } => "init-world",
        PolityCommand::UpdateWorld { .. } => "update-world",
        PolityCommand::RecordAchievement(_) => "achievement",
        PolityCommand::RecordOutcome { .. } => "outcome",
        PolityCommand::CollectEvidence(_) => "evidence",
        PolityCommand::SetPolicy { .. } => "set-policy",
        PolityCommand::Lobby { .. } => "lobby",
        PolityCommand::EvaluateContamination(_) => "contamination",
        PolityCommand::SelectGrandGoal { .. } => "select-goal",
        PolityCommand::ChangeGrandGoal { .. } => "change-goal",
        PolityCommand::RecordHomestead { .. } => "homestead",
    };
    Command::ModulePayload {
        module: "polity".into(),
        kind: kind.into(),
        payload: postcard::to_allocvec(cmd).expect("polity command encodes"),
    }
}

/// The polity slice — all owned dynamic state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolitySlice {
    /// Faction political state.
    pub factions: BTreeMap<FactionId, FactionState>,
    /// The world-/faction-first ledger.
    pub ledger: BTreeMap<MilestoneId, MilestoneClaim>,
    /// Candidate-world astrobiology state (incl. the **hidden** ground truth).
    pub candidates: BTreeMap<CandidateId, CandidateState>,
    /// Policy lever levels.
    pub policy: BTreeMap<PolicyId, f64>,
    /// Per-body planetary-protection state.
    pub protection: BTreeMap<u32, ProtectionState>,
    /// Contamination records.
    pub contamination: Vec<ContaminationRecord>,
    /// AI faction behaviour state.
    pub ai: BTreeMap<FactionId, AiState>,
    /// Emitted-event feed.
    pub events: Vec<EventRecord>,
    /// Captured composed economy facts (for mood modifiers).
    pub econ: BTreeMap<FactionId, EconomyFacts>,
    /// Captured composed Homestead indices.
    pub homestead: BTreeMap<FactionId, f64>,
    /// Captured science tide.
    pub science_tide: ScienceTide,
    /// Captured difficulty.
    pub difficulty: Difficulty,
    /// Whether the world has been initialised.
    pub initialised: bool,
    /// Last stepped tick.
    pub last_tick: u64,
    /// Polity-data content hash (pin).
    pub data_hash: [u8; 32],
}

/// The polity module (immutable loaded parameters).
pub struct PolityModule {
    /// The sourced parameter set.
    pub params: Params,
}

impl PolityModule {
    /// Load and validate the parameter set from a data directory (`data/polity`).
    pub fn load(dir: &Path) -> Result<PolityModule, String> {
        Ok(PolityModule {
            params: Params::load_dir(dir)?,
        })
    }

    fn slice_mut<'a>(&self, slice: &'a mut dyn StateSlice) -> &'a mut PolitySlice {
        slice
            .as_any_mut()
            .downcast_mut::<PolitySlice>()
            .expect("polity slice type")
    }
}

/// A uniform `f64 ∈ [0,1)` from a kernel rng.
fn uniform(rng: &mut dyn rand_core::RngCore) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

impl SimModule for PolityModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            id: "polity".into(),
            owned_slice: "polity/slice".into(),
            publishes: vec!["polity/status".into()],
            reads: vec!["kernel/status".into()],
            emits: vec![
                "milestone-claimed".into(),
                "rival-milestone".into(),
                "mood-shift".into(),
                "approval-frozen".into(),
                "anomaly".into(),
                "launch-failure".into(),
                "solar-storm".into(),
                "funding-crisis".into(),
                "funding-boom".into(),
                "political-shakeup".into(),
                "supply-shock".into(),
                "personnel-event".into(),
                "discovery".into(),
                "astrobiology-evidence".into(),
                "astrobiology-conclusive".into(),
                "contamination".into(),
                "grand-goal-met".into(),
                "soft-fail".into(),
            ],
            subscribes: vec![],
            phase: 1,
            streams: vec![
                "polity/ground-truth".into(),
                "polity/events".into(),
                "polity/policy-drift".into(),
                "polity/lobby".into(),
                "polity/ai".into(),
                "polity/contamination".into(),
            ],
            cadence_ticks: TICKS_PER_DAY,
            escalations: vec![],
        }
    }

    fn init(&self, _ctx: &mut InitCtx<'_>) -> Box<dyn StateSlice> {
        Box::new(PolitySlice {
            factions: BTreeMap::new(),
            ledger: BTreeMap::new(),
            candidates: BTreeMap::new(),
            policy: BTreeMap::new(),
            protection: BTreeMap::new(),
            contamination: Vec::new(),
            ai: BTreeMap::new(),
            events: Vec::new(),
            econ: BTreeMap::new(),
            homestead: BTreeMap::new(),
            science_tide: ScienceTide::default(),
            difficulty: Difficulty::default(),
            initialised: false,
            last_tick: 0,
            data_hash: self.params.content_hash,
        })
    }

    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx<'_>) {
        let now = ctx.tick().0;
        let s = self.slice_mut(slice);
        if now <= s.last_tick || !s.initialised {
            s.last_tick = now.max(s.last_tick);
            return;
        }
        let dt_days = (now - s.last_tick) as f64 / TICKS_PER_DAY as f64;
        s.last_tick = now;

        let faction_ids: Vec<FactionId> = s.factions.keys().copied().collect();
        let event_defs = self.params.events.classes.clone();
        let lever_ids: Vec<PolicyId> = s.policy.keys().cloned().collect();
        let ai_ids: Vec<FactionId> = s
            .factions
            .iter()
            .filter(|(_, f)| f.ai)
            .map(|(id, _)| *id)
            .collect();

        // --- Pass 1: draw all seeded uniforms (one handle per stream) ---
        let ev_rolls: Vec<f64> = {
            let rng = ctx.rng("polity/events");
            (0..faction_ids.len() * event_defs.len())
                .map(|_| uniform(rng))
                .collect()
        };
        let drift_rolls: Vec<f64> = {
            let rng = ctx.rng("polity/policy-drift");
            (0..lever_ids.len()).map(|_| uniform(rng)).collect()
        };
        let ai_rolls: Vec<f64> = {
            let rng = ctx.rng("polity/ai");
            (0..ai_ids.len()).map(|_| uniform(rng)).collect()
        };

        let mut emitted: Vec<(String, BTreeMap<String, ViewValue>)> = Vec::new();

        // --- Events: per faction × per class, daily Bernoulli multiplicative hazard ---
        for (fi, fid) in faction_ids.iter().enumerate() {
            let tide = s
                .science_tide
                .per_faction_level
                .get(fid)
                .copied()
                .unwrap_or(s.science_tide.global_level);
            let risk = events::risk_factor(tide);
            for (ci, def) in event_defs.iter().enumerate() {
                let p = events::event_prob(def, risk, s.difficulty.event_rate_mult);
                let roll = ev_rolls[fi * event_defs.len() + ci];
                if roll < (p * dt_days).min(1.0) {
                    s.events.push(EventRecord {
                        class: def.class.clone(),
                        tick: now,
                        faction: Some(*fid),
                        interrupt: def.interrupt,
                    });
                    let mut payload = BTreeMap::new();
                    payload.insert("faction".to_string(), ViewValue::U64(u64::from(fid.0)));
                    emitted.push((def.class.clone(), payload));
                }
            }
        }

        // --- Mood decay toward baseline; expire loss-of-crew freeze ---
        for fid in &faction_ids {
            if let Some(f) = s.factions.get_mut(fid) {
                f.mood = mood::clamp(
                    mood::decay(f.mood, f.mood_baseline, &self.params.mood, dt_days),
                    &self.params.mood,
                );
            }
        }

        // --- Policy drift (seeded, bounded) ---
        for (li, lid) in lever_ids.iter().enumerate() {
            if let (Some(lever), Some(level)) = (
                self.params.policy.levers.iter().find(|l| l.id == lid.0),
                s.policy.get(lid).copied(),
            ) {
                let new = policy::drift(level, lever, drift_rolls[li]);
                s.policy.insert(lid.clone(), new);
            }
        }

        // --- AI world: advance the tide, pursue/claim milestones (rival-milestone) ---
        for (ai_i, fid) in ai_ids.iter().enumerate() {
            let tide = s
                .science_tide
                .per_faction_level
                .get(fid)
                .copied()
                .unwrap_or(s.science_tide.global_level);
            let cap = ai::capability(tide, s.difficulty.ai_competence_mult, &self.params.ai);
            if let Some(a) = s.ai.get_mut(fid) {
                a.capability = cap;
            }
            // advance the global tide (clamped envelope already in capability).
            s.science_tide.global_level +=
                ai::tide_advance(s.difficulty.ai_funding_mult, &self.params.ai, dt_days);
            if ai::pursues(cap, &self.params.ai) && ai_rolls[ai_i] < (0.002 * dt_days).min(1.0) {
                // claim the lowest-id unclaimed world-first (abstracted, no condition check).
                if let Some(mid) = self.next_unclaimed_world_first(s) {
                    self.award_milestone(s, *fid, mid, now);
                    let mut payload = BTreeMap::new();
                    payload.insert("faction".to_string(), ViewValue::U64(u64::from(fid.0)));
                    payload.insert("milestone".to_string(), ViewValue::U64(u64::from(mid.0)));
                    emitted.push(("rival-milestone".into(), payload));
                }
            }
        }

        // --- Astrobiology: recompute consensus; resolve conclusively ---
        self.recompute_astrobiology(s, now, &mut emitted);

        // --- Grand-goal pass emission (first time a faction's goal passes) ---
        // (scoring is read at the horizon via the query surface; we emit the milestone moment)

        for (class, payload) in emitted {
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
        let cmd: PolityCommand = match postcard::from_bytes(payload) {
            Ok(c) => c,
            Err(e) => {
                return CommandOutcome::Rejected(format!(
                    "malformed polity payload (kind '{kind}'): {e}"
                ));
            }
        };
        let now = ctx.tick().0;
        self.apply_command(slice, cmd, now, ctx)
    }

    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot> {
        let s = slice
            .as_any()
            .downcast_ref::<PolitySlice>()
            .expect("polity slice type");
        let conclusive = s
            .candidates
            .values()
            .filter(|c| c.conclusive.is_some())
            .count();
        vec![ViewSnapshot {
            id: "polity/status".into(),
            fields: BTreeMap::from([
                (
                    "factions".to_string(),
                    ViewValue::U64(s.factions.len() as u64),
                ),
                (
                    "claimed_world_firsts".to_string(),
                    ViewValue::U64(
                        s.ledger
                            .values()
                            .filter(|c| c.world_first.is_some())
                            .count() as u64,
                    ),
                ),
                (
                    "conclusive_worlds".to_string(),
                    ViewValue::U64(conclusive as u64),
                ),
            ]),
        }]
    }

    fn save_slice(&self, slice: &dyn StateSlice) -> Result<Vec<u8>, CoreError> {
        let s = slice
            .as_any()
            .downcast_ref::<PolitySlice>()
            .expect("polity slice type");
        postcard::to_allocvec(s).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })
    }

    fn load_slice(&self, bytes: &[u8]) -> Result<Box<dyn StateSlice>, CoreError> {
        let s: PolitySlice = postcard::from_bytes(bytes).map_err(|e| CoreError::Serialization {
            reason: e.to_string(),
        })?;
        if s.data_hash != self.params.content_hash {
            return Err(CoreError::DataVersionUnavailable {
                pinned: s.data_hash.iter().map(|b| format!("{b:02x}")).collect(),
                hint: "this save's polity data differs from the loaded data/polity content; install the matching version".into(),
            });
        }
        Ok(Box::new(s))
    }
}

impl PolityModule {
    /// The lowest-id milestone with no world-first yet (catalogue order).
    fn next_unclaimed_world_first(&self, s: &PolitySlice) -> Option<MilestoneId> {
        self.params
            .milestones
            .milestones
            .iter()
            .map(|m| MilestoneId(m.id))
            .find(|mid| s.ledger.get(mid).and_then(|c| c.world_first).is_none())
    }

    /// Award a milestone to a faction, resolving world-/faction-first with the
    /// **highest-prestige (then lowest-id)** same-tick tiebreak (FR-PEA-102…105).
    fn award_milestone(&self, s: &mut PolitySlice, faction: FactionId, mid: MilestoneId, now: u64) {
        let Some(def) = self
            .params
            .milestones
            .milestones
            .iter()
            .find(|m| m.id == mid.0)
            .cloned()
        else {
            return;
        };
        let claim = s.ledger.entry(mid).or_default();
        match claim.world_first {
            None => {
                claim.world_first = Some((faction, now));
                add_prestige(s, faction, def.weight);
            }
            Some((holder, t_held)) if t_held == now && holder != faction => {
                // Same-tick contest → highest **pre-award** prestige, then lowest id.
                let holder_base = prestige_of(s, holder) - def.weight;
                let claimant_base = prestige_of(s, faction);
                let claimant_wins = claimant_base > holder_base
                    || (claimant_base == holder_base && faction.0 < holder.0);
                if claimant_wins {
                    // demote previous holder to faction-first, promote claimant.
                    add_prestige(
                        s,
                        holder,
                        -def.weight + def.weight * def.faction_first_fraction,
                    );
                    s.ledger
                        .get_mut(&mid)
                        .unwrap()
                        .faction_firsts
                        .insert(holder, now);
                    s.ledger.get_mut(&mid).unwrap().world_first = Some((faction, now));
                    add_prestige(s, faction, def.weight);
                } else {
                    record_faction_first(s, &def, faction, mid, now);
                }
            }
            Some((holder, _)) if holder == faction => { /* already world-first holder */ }
            Some(_) => record_faction_first(s, &def, faction, mid, now),
        }
    }

    /// Recompute each candidate's prestige-weighted consensus and resolve conclusively.
    fn recompute_astrobiology(
        &self,
        s: &mut PolitySlice,
        now: u64,
        emitted: &mut Vec<(String, BTreeMap<String, ViewValue>)>,
    ) {
        let ap = &self.params.astro;
        let cand_ids: Vec<CandidateId> = s.candidates.keys().copied().collect();
        for cid in cand_ids {
            let (gt, has_sr) = {
                let c = &s.candidates[&cid];
                (c.ground_truth, c.has_sample_return)
            };
            // prestige-weighted consensus over per-faction probabilities.
            let items: Vec<(f64, f64)> = {
                let c = &s.candidates[&cid];
                c.posteriors
                    .iter()
                    .map(|(fid, lo)| {
                        let prob = astrobiology::faction_prob(*lo, gt, has_sr, ap);
                        let w = prestige_of(s, *fid) + 1.0;
                        (prob, w)
                    })
                    .collect()
            };
            let consensus = astrobiology::weighted_consensus(&items);
            let resolution = astrobiology::is_conclusive(consensus, has_sr, ap);
            let newly = {
                let c = s.candidates.get_mut(&cid).unwrap();
                c.consensus = consensus;
                if c.conclusive.is_none() && resolution.is_some() {
                    c.conclusive = resolution;
                    true
                } else {
                    false
                }
            };
            if newly {
                self.on_conclusive(s, cid, resolution.unwrap(), now, emitted);
            }
        }
    }

    /// React to a candidate's conclusive resolution: emit, award the top-tier
    /// milestone to the discoverer, **tighten PP stringency** (the T028 hook → real),
    /// and shift the discoverer's mood.
    fn on_conclusive(
        &self,
        s: &mut PolitySlice,
        cid: CandidateId,
        res: Resolution,
        now: u64,
        emitted: &mut Vec<(String, BTreeMap<String, ViewValue>)>,
    ) {
        // discoverer = faction with the highest posterior on this candidate.
        let discoverer = s.candidates[&cid]
            .posteriors
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(fid, _)| *fid);
        let mut payload = BTreeMap::new();
        payload.insert("body".to_string(), ViewValue::U64(u64::from((cid.0).0)));
        payload.insert(
            "positive".to_string(),
            ViewValue::Bool(res == Resolution::Positive),
        );
        emitted.push(("astrobiology-conclusive".into(), payload));

        if let Some(fid) = discoverer {
            // the "first conclusive astrobiology result" milestone (id 15).
            self.award_milestone(s, fid, MilestoneId(15), now);
            // mood lift for the discoverer (a spectacular first).
            if let Some(f) = s.factions.get_mut(&fid) {
                f.mood = mood::clamp(
                    f.mood + self.params.mood.world_first_delta,
                    &self.params.mood,
                );
            }
        }
        // tighten PP stringency (single source for US6) — the conclusive-resolution hook.
        let pp = PolicyId("pp-stringency".into());
        if let Some(lever) = self
            .params
            .policy
            .levers
            .iter()
            .find(|l| l.id == "pp-stringency")
        {
            let cur = s.policy.get(&pp).copied().unwrap_or(lever.default);
            s.policy.insert(pp, (cur + 0.1).min(lever.max));
        }
    }

    fn apply_command(
        &self,
        slice: &mut dyn StateSlice,
        cmd: PolityCommand,
        now: u64,
        ctx: &mut StepCtx<'_>,
    ) -> CommandOutcome {
        match cmd {
            PolityCommand::InitWorld {
                factions,
                candidates,
                protections,
                difficulty,
            } => {
                {
                    let s = self.slice_mut(slice);
                    if s.initialised {
                        return CommandOutcome::Rejected("world already initialised".into());
                    }
                }
                // draw the seeded ground truth (stream polity/ground-truth, candidate order).
                let mut sorted = candidates.clone();
                sorted.sort_by_key(|c| (c.candidate.0).0);
                let truths: Vec<bool> = {
                    let rng = ctx.rng("polity/ground-truth");
                    sorted
                        .iter()
                        .map(|c| uniform(rng) < c.presence_prob)
                        .collect()
                };
                let s = self.slice_mut(slice);
                for fi in &factions {
                    s.factions.insert(
                        fi.id,
                        FactionState {
                            funding_model: fi.funding_model,
                            relationships: BTreeMap::new(),
                            partnership: Default::default(),
                            prestige: 0.0,
                            mood: fi.baseline_mood,
                            mood_baseline: fi.baseline_mood,
                            loc_freeze_until_tick: 0,
                            ai: fi.ai,
                            grand_goal: None,
                            goal_changes: 0,
                        },
                    );
                    if fi.ai {
                        s.ai.insert(
                            fi.id,
                            AiState {
                                capability: self.params.ai.base_competence,
                                targets: Default::default(),
                            },
                        );
                    }
                }
                for (c, gt) in sorted.iter().zip(truths) {
                    let posteriors = factions.iter().map(|fi| (fi.id, 0.0)).collect();
                    s.candidates.insert(
                        c.candidate,
                        CandidateState {
                            tier: c.tier,
                            ground_truth: gt,
                            posteriors,
                            has_sample_return: false,
                            abiotic_strength: self.params.astro.abiotic_competitor_strength,
                            consensus: 0.5,
                            conclusive: None,
                            pristine_value: 1.0,
                        },
                    );
                }
                for sp in &protections {
                    s.protection.insert(
                        sp.body.0,
                        ProtectionState {
                            category: sp.category,
                            special_region: sp.special_region,
                            bioburden_limit: sp.bioburden_limit,
                            pristine_value: 1.0,
                        },
                    );
                }
                // seed default PP bodies from data not already provided.
                for b in &self.params.protection.bodies {
                    s.protection.entry(b.body).or_insert(ProtectionState {
                        category: b.category,
                        special_region: b.special_region,
                        bioburden_limit: b.bioburden_limit,
                        pristine_value: 1.0,
                    });
                }
                for lever in &self.params.policy.levers {
                    s.policy.insert(PolicyId(lever.id.clone()), lever.default);
                }
                s.difficulty = difficulty;
                s.initialised = true;
                CommandOutcome::Applied
            }
            PolityCommand::UpdateWorld {
                science_tide,
                difficulty,
            } => {
                let s = self.slice_mut(slice);
                s.science_tide = science_tide;
                s.difficulty = difficulty;
                CommandOutcome::Applied
            }
            PolityCommand::RecordAchievement(a) => {
                let Some(def) = self
                    .params
                    .milestones
                    .milestones
                    .iter()
                    .find(|m| m.id == a.milestone.0)
                    .cloned()
                else {
                    return CommandOutcome::Rejected(format!(
                        "unknown milestone {}",
                        a.milestone.0
                    ));
                };
                let s = self.slice_mut(slice);
                if !s.factions.contains_key(&a.faction) {
                    return CommandOutcome::Rejected(format!("unknown faction {}", a.faction.0));
                }
                if !milestones::satisfies(&a.facts, &def) {
                    return CommandOutcome::Rejected(format!(
                        "milestone {} conditions unmet",
                        a.milestone.0
                    ));
                }
                let world_first_now = s
                    .ledger
                    .get(&a.milestone)
                    .and_then(|c| c.world_first)
                    .is_none();
                self.award_milestone(s, a.faction, a.milestone, now);
                let mut payload = BTreeMap::new();
                payload.insert(
                    "faction".to_string(),
                    ViewValue::U64(u64::from(a.faction.0)),
                );
                payload.insert(
                    "milestone".to_string(),
                    ViewValue::U64(u64::from(a.milestone.0)),
                );
                payload.insert("world_first".to_string(), ViewValue::Bool(world_first_now));
                ctx.emit("milestone-claimed", payload);
                CommandOutcome::Applied
            }
            PolityCommand::RecordOutcome { crew, economy } => {
                let s = self.slice_mut(slice);
                let Some(f) = s.factions.get_mut(&crew.faction) else {
                    return CommandOutcome::Rejected(format!("unknown faction {}", crew.faction.0));
                };
                let d = mood::outcome_delta(&crew, &self.params.mood);
                f.mood = mood::clamp(f.mood + d, &self.params.mood);
                if crew.loss_of_crew {
                    f.loc_freeze_until_tick =
                        now + (self.params.mood.loc_recovery_days * TICKS_PER_DAY as f64) as u64;
                }
                s.econ.insert(economy.faction, economy);
                let mut payload = BTreeMap::new();
                payload.insert(
                    "faction".to_string(),
                    ViewValue::U64(u64::from(crew.faction.0)),
                );
                if crew.loss_of_crew {
                    ctx.emit("approval-frozen", payload);
                } else {
                    ctx.emit("mood-shift", payload);
                }
                CommandOutcome::Applied
            }
            PolityCommand::CollectEvidence(e) => {
                let s = self.slice_mut(slice);
                if !s.candidates.contains_key(&e.candidate) {
                    return CommandOutcome::Rejected(format!(
                        "unknown candidate {}",
                        (e.candidate.0).0
                    ));
                }
                if !s.factions.contains_key(&e.faction) {
                    return CommandOutcome::Rejected(format!("unknown faction {}", e.faction.0));
                }
                let gt = s.candidates[&e.candidate].ground_truth;
                let delta =
                    astrobiology::evidence_delta(e.stage, e.quality, gt, &self.params.astro);
                let c = s.candidates.get_mut(&e.candidate).unwrap();
                *c.posteriors.entry(e.faction).or_insert(0.0) += delta;
                if e.stage == crate::inputs::EvidenceStage::SampleReturn {
                    c.has_sample_return = true;
                }
                let mut payload = BTreeMap::new();
                payload.insert(
                    "body".to_string(),
                    ViewValue::U64(u64::from((e.candidate.0).0)),
                );
                payload.insert(
                    "faction".to_string(),
                    ViewValue::U64(u64::from(e.faction.0)),
                );
                ctx.emit("astrobiology-evidence", payload);
                CommandOutcome::Applied
            }
            PolityCommand::SetPolicy { id, level } => {
                let s = self.slice_mut(slice);
                let pid = PolicyId(id.clone());
                let Some(lever) = self.params.policy.levers.iter().find(|l| l.id == id) else {
                    return CommandOutcome::Rejected(format!("unknown policy lever '{id}'"));
                };
                s.policy.insert(pid, level.clamp(lever.min, lever.max));
                CommandOutcome::Applied
            }
            PolityCommand::Lobby {
                faction,
                id,
                direction,
            } => {
                let pid = PolicyId(id.clone());
                let Some(lever) = self
                    .params
                    .policy
                    .levers
                    .iter()
                    .find(|l| l.id == id)
                    .cloned()
                else {
                    return CommandOutcome::Rejected(format!("unknown policy lever '{id}'"));
                };
                let u = {
                    let rng = ctx.rng("polity/lobby");
                    uniform(rng)
                };
                let s = self.slice_mut(slice);
                if !s.factions.contains_key(&faction) {
                    return CommandOutcome::Rejected(format!("unknown faction {}", faction.0));
                }
                let cur = s.policy.get(&pid).copied().unwrap_or(lever.default);
                s.policy
                    .insert(pid, policy::lobby(cur, &lever, direction, u));
                CommandOutcome::Applied
            }
            PolityCommand::EvaluateContamination(m) => {
                let s = self.slice_mut(slice);
                let cp = &self.params.protection.contamination;
                let mut payload = BTreeMap::new();
                payload.insert("body".to_string(), ViewValue::U64(u64::from(m.body.0)));
                // forward contamination (graded), only in a Special Region.
                if m.special_region
                    && let Some(ps) = s.protection.get(&m.body.0)
                {
                    let sev = protection::forward_severity(
                        m.lander_bioburden,
                        ps.bioburden_limit,
                        m.crash,
                        cp,
                    );
                    if sev > 0.0 {
                        let new_pv = protection::degrade(ps.pristine_value, sev);
                        s.protection.get_mut(&m.body.0).unwrap().pristine_value = new_pv;
                        // confound the candidate's evidence value (US3 link).
                        if let Some(c) = s.candidates.get_mut(&CandidateId(m.body)) {
                            c.pristine_value = new_pv;
                        }
                        s.contamination.push(ContaminationRecord {
                            body: m.body.0,
                            forward: true,
                            severity: sev,
                            cause: if m.crash {
                                "crash".into()
                            } else {
                                "soft-landing".into()
                            },
                            tick: now,
                        });
                        payload.insert("forward_severity".to_string(), ViewValue::F64(sev));
                    }
                }
                // back contamination (sample return without a chain).
                if m.sample_return {
                    let pen = protection::back_penalty(m.containment_chain, cp);
                    if pen > 0.0 {
                        s.contamination.push(ContaminationRecord {
                            body: m.body.0,
                            forward: false,
                            severity: pen,
                            cause: "back-contamination".into(),
                            tick: now,
                        });
                        payload.insert("back_penalty".to_string(), ViewValue::F64(pen));
                    }
                }
                ctx.emit("contamination", payload);
                CommandOutcome::Applied
            }
            PolityCommand::SelectGrandGoal { faction, kind } => {
                let s = self.slice_mut(slice);
                let Some(f) = s.factions.get_mut(&faction) else {
                    return CommandOutcome::Rejected(format!("unknown faction {}", faction.0));
                };
                f.grand_goal = Some(kind);
                CommandOutcome::Applied
            }
            PolityCommand::ChangeGrandGoal { faction, kind } => {
                let s = self.slice_mut(slice);
                let Some(f) = s.factions.get_mut(&faction) else {
                    return CommandOutcome::Rejected(format!("unknown faction {}", faction.0));
                };
                f.grand_goal = Some(kind);
                f.goal_changes += 1;
                CommandOutcome::Applied
            }
            PolityCommand::RecordHomestead { faction, index } => {
                let s = self.slice_mut(slice);
                if !s.factions.contains_key(&faction) {
                    return CommandOutcome::Rejected(format!("unknown faction {}", faction.0));
                }
                s.homestead.insert(faction, index);
                CommandOutcome::Applied
            }
        }
    }
}

fn prestige_of(s: &PolitySlice, faction: FactionId) -> f64 {
    s.factions.get(&faction).map(|f| f.prestige).unwrap_or(0.0)
}

fn add_prestige(s: &mut PolitySlice, faction: FactionId, delta: f64) {
    if let Some(f) = s.factions.get_mut(&faction) {
        f.prestige += delta;
    }
}

fn record_faction_first(
    s: &mut PolitySlice,
    def: &crate::params::MilestoneDef,
    faction: FactionId,
    mid: MilestoneId,
    now: u64,
) {
    use std::collections::btree_map::Entry;
    let claim = s.ledger.entry(mid).or_default();
    let is_new = if let Entry::Vacant(slot) = claim.faction_firsts.entry(faction) {
        slot.insert(now);
        true
    } else {
        false
    };
    if is_new {
        add_prestige(s, faction, def.weight * def.faction_first_fraction);
    }
}
