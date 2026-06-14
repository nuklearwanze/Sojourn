//! The read-only world/political-state query surface (R14, `contracts/polity-queries.md`).
//! Pure functions over the stored `PolitySlice` + captured composed values. **No query
//! exposes the astrobiology ground truth.** REID-free; mood/consensus/score are derived
//! here, no mutation, no hidden truth.

use crate::astrobiology;
use crate::faction::Resolution;
use crate::goals::{self, GoalInputs, Verdict};
use crate::ids::{CandidateId, FactionId, GoalKind, MilestoneId};
use crate::module::{PolityModule, PolitySlice};
use crate::mood;
use crate::params::Params;
use crate::score::{self, SoftFail};
use serde::{Deserialize, Serialize};
use sojourn_core::{CoreError, SimCore};

const DAYS_PER_YEAR: f64 = 365.25;
const TICKS_PER_DAY: f64 = 86_400.0;

/// A milestone's claim view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilestoneView {
    /// Id.
    pub id: u32,
    /// Era.
    pub era: String,
    /// Description.
    pub description: String,
    /// Significance weight.
    pub weight: f64,
    /// World-first claimant (faction id) + tick, if claimed.
    pub world_first: Option<(u32, u64)>,
    /// Faction-first claimants.
    pub faction_firsts: Vec<u32>,
}

/// Mood-derived modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Modifiers {
    /// Appropriation factor (agencies).
    pub appropriation_factor: f64,
    /// Valuation factor (companies).
    pub valuation_factor: f64,
    /// Approval latency (days).
    pub approval_latency_days: f64,
    /// Crewed-flight approval frozen (loss-of-crew)?
    pub crewed_frozen: bool,
}

/// An approval verdict.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Approval {
    /// Granted.
    Granted,
    /// Delayed by N days.
    Delayed(u64),
    /// Denied.
    Denied,
}

/// A planetary-protection view.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProtectionView {
    /// COSPAR category I–V.
    pub category: u8,
    /// Special Region?
    pub special_region: bool,
    /// Bioburden limit.
    pub bioburden_limit: f64,
    /// Pristine astrobiology value ∈ [0,1].
    pub pristine_value: f64,
}

/// A Grand-Goal view.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GoalView {
    /// Goal kind.
    pub kind: GoalKind,
    /// Progress ∈ [0,1].
    pub progress: f64,
    /// Pass/fail/pending verdict.
    pub verdict: Verdict,
}

/// A final-score view.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreView {
    /// The Grand-Goal verdict (primary result).
    pub goal_verdict: Verdict,
    /// The secondary composite score.
    pub composite: f64,
    /// Soft-fail flag, if any.
    pub soft_fail: Option<SoftFail>,
}

/// A truth-free, query-time snapshot of the polity slice.
pub struct WorldSnapshot {
    /// Tick.
    pub tick: u64,
    slice: PolitySlice,
    params: Params,
}

impl WorldSnapshot {
    /// Extract from a running kernel (the slice holds the captured composed values).
    pub fn from_core(core: &SimCore, module: &PolityModule) -> Result<WorldSnapshot, CoreError> {
        let tick = core.status().tick;
        core.with_slice("polity", |slice| {
            let s = slice
                .as_any()
                .downcast_ref::<PolitySlice>()
                .expect("polity slice type");
            WorldSnapshot {
                tick,
                slice: s.clone(),
                params: module.params.clone(),
            }
        })
    }

    /// Build directly from parts (tests).
    pub fn from_parts(module: &PolityModule, slice: PolitySlice) -> WorldSnapshot {
        WorldSnapshot {
            tick: slice.last_tick,
            slice,
            params: module.params.clone(),
        }
    }

    // ---- Milestones & prestige (US1) ----

    fn milestone_view(&self, id: MilestoneId) -> Option<MilestoneView> {
        let def = self
            .params
            .milestones
            .milestones
            .iter()
            .find(|m| m.id == id.0)?;
        let claim = self.slice.ledger.get(&id);
        Some(MilestoneView {
            id: def.id,
            era: def.era.clone(),
            description: def.description.clone(),
            weight: def.weight,
            world_first: claim.and_then(|c| c.world_first).map(|(f, t)| (f.0, t)),
            faction_firsts: claim
                .map(|c| c.faction_firsts.keys().map(|f| f.0).collect())
                .unwrap_or_default(),
        })
    }

    /// A milestone's claim view.
    pub fn milestone(&self, id: MilestoneId) -> Option<MilestoneView> {
        self.milestone_view(id)
    }

    /// The full catalogue with claim state.
    pub fn ledger(&self) -> Vec<MilestoneView> {
        self.params
            .milestones
            .milestones
            .iter()
            .filter_map(|m| self.milestone_view(MilestoneId(m.id)))
            .collect()
    }

    /// Milestones with no world-first yet (the race board).
    pub fn unclaimed_world_firsts(&self) -> Vec<MilestoneId> {
        self.params
            .milestones
            .milestones
            .iter()
            .map(|m| MilestoneId(m.id))
            .filter(|mid| {
                self.slice
                    .ledger
                    .get(mid)
                    .and_then(|c| c.world_first)
                    .is_none()
            })
            .collect()
    }

    /// A faction's accrued prestige.
    pub fn prestige(&self, faction: FactionId) -> Option<f64> {
        self.slice.factions.get(&faction).map(|f| f.prestige)
    }

    // ---- Politics & mood (US2) ----

    /// A faction's current mood.
    pub fn mood(&self, faction: FactionId) -> Option<f64> {
        self.slice.factions.get(&faction).map(|f| f.mood)
    }

    /// The mood-derived budget/valuation/approval modifiers.
    pub fn modifiers(&self, faction: FactionId) -> Option<Modifiers> {
        let f = self.slice.factions.get(&faction)?;
        Some(Modifiers {
            appropriation_factor: mood::appropriation_factor(f.mood, &self.params.mood),
            valuation_factor: mood::valuation_factor(f.mood, &self.params.mood),
            approval_latency_days: mood::approval_latency_days(f.mood, &self.params.mood),
            crewed_frozen: self.tick < f.loc_freeze_until_tick,
        })
    }

    /// Resolve a major-program approval from mood + policy.
    pub fn approval(&self, faction: FactionId, program: &str) -> Option<Approval> {
        let f = self.slice.factions.get(&faction)?;
        if program == "crewed" && self.tick < f.loc_freeze_until_tick {
            return Some(Approval::Denied);
        }
        // policy-gated programs (e.g. "nuclear-launch").
        if let Some(lever) = self.params.policy.levers.iter().find(|l| l.id == program) {
            let level = self
                .slice
                .policy
                .get(&crate::ids::PolicyId(program.into()))
                .copied()
                .unwrap_or(lever.default);
            if level < lever.grant_threshold {
                return Some(Approval::Denied);
            }
        }
        let latency = mood::approval_latency_days(f.mood, &self.params.mood);
        if latency <= 1.0 {
            Some(Approval::Granted)
        } else {
            Some(Approval::Delayed(latency as u64))
        }
    }

    // ---- Astrobiology (US3) — never the ground truth ----

    /// A faction's posterior probability of life on a candidate (false-hint capped).
    pub fn posterior(&self, faction: FactionId, candidate: CandidateId) -> Option<f64> {
        let c = self.slice.candidates.get(&candidate)?;
        let lo = c.posteriors.get(&faction).copied().unwrap_or(0.0);
        Some(astrobiology::faction_prob(
            lo,
            c.ground_truth,
            c.has_sample_return,
            &self.params.astro,
        ))
    }

    /// The prestige-weighted community consensus.
    pub fn consensus(&self, candidate: CandidateId) -> Option<f64> {
        self.slice.candidates.get(&candidate).map(|c| c.consensus)
    }

    /// The conclusive resolution, if the question is conclusively resolved.
    pub fn conclusive(&self, candidate: CandidateId) -> Option<Resolution> {
        self.slice
            .candidates
            .get(&candidate)
            .and_then(|c| c.conclusive)
    }

    /// Whether the question is conclusively resolved at all.
    pub fn is_conclusive(&self, candidate: CandidateId) -> bool {
        self.conclusive(candidate).is_some()
    }

    /// True iff the factions publicly disagree on a candidate.
    pub fn disagreement(&self, candidate: CandidateId) -> bool {
        let Some(c) = self.slice.candidates.get(&candidate) else {
            return false;
        };
        let probs: Vec<f64> = c
            .posteriors
            .values()
            .map(|lo| {
                astrobiology::faction_prob(
                    *lo,
                    c.ground_truth,
                    c.has_sample_return,
                    &self.params.astro,
                )
            })
            .collect();
        astrobiology::disagreement(&probs, self.params.astro.disagreement_width)
    }

    /// The achievable evidence value (degraded by contamination) ∈ [0,1].
    pub fn evidence_value(&self, candidate: CandidateId) -> Option<f64> {
        self.slice
            .candidates
            .get(&candidate)
            .map(|c| c.pristine_value)
    }

    // ---- Policy & planetary protection (US5/US6) ----

    /// A policy lever's current level.
    pub fn policy(&self, id: &str) -> Option<f64> {
        self.slice
            .policy
            .get(&crate::ids::PolicyId(id.into()))
            .copied()
    }

    /// Whether a required approval is granted at the current level of `id`.
    pub fn policy_granted(&self, id: &str) -> Option<bool> {
        let lever = self.params.policy.levers.iter().find(|l| l.id == id)?;
        let level = self.policy(id).unwrap_or(lever.default);
        Some(level >= lever.grant_threshold)
    }

    /// A body's planetary-protection view.
    pub fn protection(&self, body: u32) -> Option<ProtectionView> {
        self.slice.protection.get(&body).map(|p| ProtectionView {
            category: p.category,
            special_region: p.special_region,
            bioburden_limit: p.bioburden_limit,
            pristine_value: p.pristine_value,
        })
    }

    /// The number of forward/back contamination records for a body.
    pub fn contamination_count(&self, body: u32) -> usize {
        self.slice
            .contamination
            .iter()
            .filter(|r| r.body == body)
            .count()
    }

    // ---- Events (US4) ----

    /// The number of recorded events (the feed length).
    pub fn event_count(&self) -> usize {
        self.slice.events.len()
    }

    /// The number of recorded events of a given class.
    pub fn event_count_of(&self, class: &str) -> usize {
        self.slice
            .events
            .iter()
            .filter(|e| e.class == class)
            .count()
    }

    // ---- AI world (US7) ----

    /// The global science tide level.
    pub fn science_tide(&self) -> f64 {
        self.slice.science_tide.global_level
    }

    /// An AI faction's capability (0 if not an AI faction).
    pub fn ai_capability(&self, faction: FactionId) -> f64 {
        self.slice
            .ai
            .get(&faction)
            .map(|a| a.capability)
            .unwrap_or(0.0)
    }

    // ---- Grand Goals & scoring (US8) ----

    fn conclusive_worlds(&self) -> u32 {
        self.slice
            .candidates
            .values()
            .filter(|c| c.conclusive.is_some())
            .count() as u32
    }

    fn goal_inputs(&self, faction: FactionId) -> GoalInputs {
        let firsts = self
            .slice
            .ledger
            .values()
            .filter(|c| {
                c.world_first.map(|(f, _)| f == faction).unwrap_or(false)
                    || c.faction_firsts.contains_key(&faction)
            })
            .count() as u32;
        GoalInputs {
            firsts,
            embargo_index: self.slice.homestead.get(&faction).copied().unwrap_or(0.0),
            tonnage: self
                .slice
                .econ
                .get(&faction)
                .map(|e| e.off_earth_tonnage_profit)
                .unwrap_or(0.0),
            conclusive_worlds: self.conclusive_worlds(),
        }
    }

    fn at_horizon(&self) -> bool {
        let horizon_ticks = self.params.cfg.horizon_years as f64 * DAYS_PER_YEAR * TICKS_PER_DAY;
        self.tick as f64 >= horizon_ticks
    }

    /// A faction's Grand-Goal progress + verdict.
    pub fn grand_goal(&self, faction: FactionId) -> Option<GoalView> {
        let kind = self.slice.factions.get(&faction)?.grand_goal?;
        let inp = self.goal_inputs(faction);
        Some(GoalView {
            kind,
            progress: goals::progress(kind, &inp, &self.params.goals),
            verdict: goals::verdict(kind, &inp, &self.params.goals, self.at_horizon()),
        })
    }

    /// A faction's final score: the Grand-Goal verdict (primary) + the secondary
    /// composite score.
    pub fn score(&self, faction: FactionId) -> Option<ScoreView> {
        let f = self.slice.factions.get(&faction)?;
        let inp = self.goal_inputs(faction);
        let milestone_total: f64 = self
            .slice
            .ledger
            .iter()
            .filter_map(|(mid, claim)| {
                let def = self
                    .params
                    .milestones
                    .milestones
                    .iter()
                    .find(|m| m.id == mid.0)?;
                if claim
                    .world_first
                    .map(|(wf, _)| wf == faction)
                    .unwrap_or(false)
                {
                    Some(def.weight)
                } else if claim.faction_firsts.contains_key(&faction) {
                    Some(def.weight * def.faction_first_fraction)
                } else {
                    None
                }
            })
            .sum();
        let goal_verdict = f
            .grand_goal
            .map(|k| goals::verdict(k, &inp, &self.params.goals, self.at_horizon()))
            .unwrap_or(Verdict::Pending);
        let goal_progress = f
            .grand_goal
            .map(|k| goals::progress(k, &inp, &self.params.goals))
            .unwrap_or(0.0);
        let mut composite = score::composite(
            f.prestige,
            milestone_total,
            goal_progress,
            &self.params.goals,
            self.params.cfg.score_normalisation,
        );
        composite = score::apply_change_penalty(composite, f.goal_changes, &self.params.goals);
        Some(ScoreView {
            goal_verdict,
            composite,
            soft_fail: None,
        })
    }
}
