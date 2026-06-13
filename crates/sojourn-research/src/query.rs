//! The read-only maturity/heritage/understanding query surface (FR-RESP-801,
//! contracts/maturity-queries.md): pure functions over an immutable
//! [`ResearchSnapshot`], faction-scoped. Same seam as FA-02/FA-03 — `with_slice` +
//! pure fns, called between ticks. No query returns another faction's private state.

use crate::domains::{self, UlMap, WorldUlMap};
use crate::ids::{DomainId, FactionId, ProgramId, TechId};
use crate::module::ResearchSlice;
use crate::personnel::{self, Role, Roster};
use crate::programs::Program;
use crate::reliability::{self, HeritageMap};
use crate::tree::ResearchData;
use serde::{Deserialize, Serialize};
use sojourn_core::{CoreError, SimCore};
use std::collections::BTreeMap;

/// A technology's maturity for a faction (the FA-04 contract; R10).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Maturity {
    /// Current TRL (0 if no program).
    pub trl: u8,
    /// Scalar per-use success probability ∈ [0,1] (0 if unflyable).
    pub reliability: f64,
    /// Raw input: accumulated flight-units.
    pub flight_units: u32,
    /// Raw input: domain-UL margin above the tech's floors.
    pub ul_margin: f64,
    /// Flyable (TRL ≥ 6)?
    pub flyable: bool,
}

/// A faction's understanding of a domain.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Understanding {
    /// Private (ground) UL.
    pub private_ul: f64,
    /// World UL (the shared tide).
    pub world_ul: f64,
    /// Effective UL (tacit-knowledge adjusted) — what gates use.
    pub effective_ul: f64,
}

/// A program's status (the R&D-screen contract).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProgramStatus {
    /// Current TRL.
    pub trl: u8,
    /// P50 schedule estimate (days).
    pub p50_days: f64,
    /// P80 schedule estimate (days).
    pub p80_days: f64,
    /// Realised days.
    pub actual_days: f64,
    /// Risk index (dead-end hint when high).
    pub risk_index: f64,
    /// Dead-end hint (risk index above the hint threshold).
    pub dead_end_hint: bool,
}

/// A truth-free, faction-scoped snapshot for queries.
pub struct ResearchSnapshot {
    /// Tick.
    pub tick: u64,
    ul: UlMap,
    world_ul: WorldUlMap,
    programs: BTreeMap<ProgramId, Program>,
    heritage: HeritageMap,
    roster: Roster,
    data: ResearchData,
}

impl ResearchSnapshot {
    /// Extract a snapshot from a running kernel at the current tick boundary.
    pub fn from_core(core: &SimCore, module: &crate::module::ResearchModule) -> Result<ResearchSnapshot, CoreError> {
        let tick = core.status().tick;
        core.with_slice("research", |slice| {
            let s = slice
                .as_any()
                .downcast_ref::<ResearchSlice>()
                .expect("research slice type");
            ResearchSnapshot {
                tick,
                ul: s.ul.clone(),
                world_ul: s.world_ul.clone(),
                programs: s.programs.clone(),
                heritage: s.heritage.clone(),
                roster: s.roster.clone(),
                data: module.data.clone(),
            }
        })
    }

    fn tacit(&self) -> impl Fn(FactionId, &DomainId) -> f64 + '_ {
        move |f, d| personnel::tacit_penalty(&self.roster, &self.data.params, f, d)
    }

    fn best_program(&self, f: FactionId, tech: &TechId) -> Option<&Program> {
        self.programs
            .values()
            .filter(|p| p.faction == f && &p.tech == tech)
            .max_by_key(|p| p.trl)
    }

    /// A technology's maturity for a faction (R10). `trl` 0 ⇒ no program.
    pub fn maturity(&self, f: FactionId, tech: &TechId) -> Option<Maturity> {
        let node = self.data.nodes.get(tech)?;
        let prog = self.best_program(f, tech);
        let trl = prog.map(|p| p.trl).unwrap_or(0);
        let flight_units = self.heritage.get(&(f, tech.clone())).map(|h| h.flight_units).unwrap_or(0);
        let tacit = self.tacit();
        let ul_margin = if node.ul_floors.is_empty() {
            50.0
        } else {
            node.ul_floors
                .iter()
                .map(|(d, fl)| (domains::effective_ul(&self.ul, f, d, tacit(f, d)) - fl).max(0.0))
                .sum::<f64>()
                / node.ul_floors.len() as f64
        };
        let lead = prog.and_then(|p| p.lead).and_then(|id| self.roster.get(&id));
        let trait_bonus = personnel::trait_bonus(&self.data, lead, |t| t.reliability_bonus);
        let rel = reliability::reliability(&node.reliability, trl, flight_units, ul_margin, trait_bonus);
        Some(Maturity {
            trl,
            reliability: rel,
            flight_units,
            ul_margin,
            flyable: reliability::flyable(trl),
        })
    }

    /// A faction's understanding of a domain.
    pub fn understanding(&self, f: FactionId, dom: &DomainId) -> Option<Understanding> {
        self.data.domains.get(dom)?;
        let tacit = self.tacit();
        Some(Understanding {
            private_ul: domains::ground_ul(&self.ul, f, dom),
            world_ul: domains::world_ul(&self.world_ul, dom),
            effective_ul: domains::effective_ul(&self.ul, f, dom, tacit(f, dom)),
        })
    }

    /// A program's status.
    pub fn program_status(&self, f: FactionId, prog: ProgramId) -> Option<ProgramStatus> {
        let p = self.programs.get(&prog).filter(|p| p.faction == f)?;
        Some(ProgramStatus {
            trl: p.trl,
            p50_days: p.est_p50_days,
            p80_days: p.est_p80_days,
            actual_days: p.actual_days,
            risk_index: p.risk_index,
            dead_end_hint: p.risk_index >= 0.5,
        })
    }

    /// Technologies whose UL floors + product prereqs are currently met for `f`
    /// (UL-satisfiable prereqs are leapfrog seams — treated as met).
    pub fn available_programs(&self, f: FactionId) -> Vec<TechId> {
        let tacit = self.tacit();
        self.data
            .nodes
            .values()
            .filter(|n| domains::first_unmet_floor(&self.ul, f, n, &tacit).is_none())
            .filter(|n| {
                n.tech_prereqs.iter().all(|p| {
                    p.kind == crate::tree::PrereqKind::UlSatisfiable
                        || self.programs.values().any(|pr| pr.faction == f && pr.tech == p.tech && pr.trl >= 6)
                })
            })
            .map(|n| n.id.clone())
            .collect()
    }

    /// Personnel summary for a faction (counts by role; astronaut-ready count).
    pub fn personnel(&self, f: FactionId) -> BTreeMap<String, u64> {
        let mut m: BTreeMap<String, u64> = BTreeMap::new();
        let mut ready = 0u64;
        for p in self.roster.values().filter(|p| p.faction == f) {
            *m.entry(format!("{:?}", p.role)).or_default() += 1;
            if p.role == Role::Astronaut && p.astronaut.is_some_and(|a| a.ready()) {
                ready += 1;
            }
        }
        m.insert("AstronautReady".into(), ready);
        m
    }

    /// World UL for a domain (the shared channel).
    pub fn tide(&self, dom: &DomainId) -> Option<f64> {
        self.data.domains.get(dom)?;
        Some(domains::world_ul(&self.world_ul, dom))
    }
}
