//! Personnel (FR-RESP-501/502/503, R11): the roster, RP/DE generation, trait
//! modifiers, deterministic transitions and the tacit-knowledge effective-UL
//! penalty (a recompute over roster state — never a mutation of the ground UL).

use crate::astronaut::{AstroState, Stage};
use crate::ids::{DomainId, FactionId, PersonId};
use crate::tree::{Params, ResearchData};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Personnel role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Generates RP.
    Scientist,
    /// Generates DE.
    Engineer,
    /// Schedule/cost control.
    ProgramManager,
    /// Mission controller.
    Controller,
    /// Lobbyist/diplomat.
    Diplomat,
    /// Crew (career pipeline).
    Astronaut,
}

/// A person on the roster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    /// Identity.
    pub id: PersonId,
    /// Employer.
    pub faction: FactionId,
    /// Role.
    pub role: Role,
    /// Specialisation domain.
    pub discipline: DomainId,
    /// Skill rating.
    pub skill: f64,
    /// Trait ids (resolved against `traits.ron`).
    pub traits: Vec<String>,
    /// Age (years).
    pub age: f64,
    /// Morale (0..1).
    pub morale: f64,
    /// Remaining training days (>0 while training).
    pub training_days_left: f64,
    /// Astronaut career state (Some for Role::Astronaut).
    pub astronaut: Option<AstroState>,
}

/// The roster.
pub type Roster = BTreeMap<PersonId, Person>;

/// Total RP a faction generates per day (sum of scientist skill × rate × morale).
pub fn faction_rp(roster: &Roster, params: &Params, f: FactionId) -> f64 {
    roster
        .values()
        .filter(|p| p.faction == f && p.role == Role::Scientist && p.training_days_left <= 0.0)
        .map(|p| p.skill * params.rp_per_scientist_day * p.morale)
        .sum()
}

/// Total DE a faction generates per day (sum of engineer skill × rate × morale).
pub fn faction_de(roster: &Roster, params: &Params, f: FactionId) -> f64 {
    roster
        .values()
        .filter(|p| p.faction == f && p.role == Role::Engineer && p.training_days_left <= 0.0)
        .map(|p| p.skill * params.de_per_engineer_day * p.morale)
        .sum()
}

/// Aggregate a trait modifier across a lead's traits (product of multipliers).
pub fn trait_mult(data: &ResearchData, person: Option<&Person>, pick: impl Fn(&crate::tree::TraitDef) -> f64) -> f64 {
    let Some(p) = person else { return 1.0 };
    let mut m = 1.0;
    for t in &p.traits {
        if let Some(def) = data.traits.get(t) {
            m *= pick(def);
        }
    }
    m
}

/// Additive trait bonus (e.g. reliability) across a lead's traits.
pub fn trait_bonus(data: &ResearchData, person: Option<&Person>, pick: impl Fn(&crate::tree::TraitDef) -> f64) -> f64 {
    let Some(p) = person else { return 0.0 };
    p.traits.iter().filter_map(|t| data.traits.get(t)).map(pick).sum()
}

/// Tacit-knowledge penalty (FR-RESP-503): a domain with **no** active scientist of
/// that discipline loses effective UL. A recompute over roster state — restored by
/// re-hiring; the ground UL is never touched.
pub fn tacit_penalty(roster: &Roster, params: &Params, f: FactionId, domain: &DomainId) -> f64 {
    let has_specialist = roster.values().any(|p| {
        p.faction == f
            && p.role == Role::Scientist
            && p.training_days_left <= 0.0
            && &p.discipline == domain
    });
    if has_specialist { 0.0 } else { params.tacit_per_head }
}

/// Advance training and aging for one step (days). Astronauts in training become
/// Ready when their clock expires.
pub fn step_people(roster: &mut Roster, days: f64) {
    for p in roster.values_mut() {
        p.age += days / 365.0;
        if p.training_days_left > 0.0 {
            p.training_days_left = (p.training_days_left - days).max(0.0);
            if p.training_days_left <= 0.0
                && let Some(a) = &mut p.astronaut
                && a.stage == Stage::Training
            {
                a.stage = Stage::Ready;
            }
        }
    }
}
