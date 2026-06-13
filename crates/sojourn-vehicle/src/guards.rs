//! Realism guards (FR-VD-601, R13): red-flag the physically impossible while
//! leaving marginal-but-possible designs buildable. Each flag carries the specific
//! violated constraint and the offending value. No guard is bypassable by a
//! non-sourced value (the inputs are all derived from sourced data).

use crate::catalogue::ClassTemplate;
use crate::design::MissionReqs;
use serde::{Deserialize, Serialize};

/// Guard severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Physically impossible — refused.
    Hard,
    /// Marginal/risky — buildable, but the numbers tell the truth.
    Soft,
}

/// A realism red-flag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedFlag {
    /// The violated constraint.
    pub constraint: String,
    /// The offending value.
    pub value: f64,
    /// Severity.
    pub severity: Severity,
}

/// Everything the guards check (computed by the query layer).
pub struct GuardInputs<'a> {
    /// Class template (lander/crewed flags).
    pub class: Option<&'a ClassTemplate>,
    /// Power margin (W).
    pub power_margin_w: f64,
    /// Thermal margin (W).
    pub thermal_margin_w: f64,
    /// Total Δv (m/s).
    pub total_dv: f64,
    /// T/W at the target body (for landers).
    pub lander_tw: f64,
    /// Structure thrust rating (N).
    pub structure_limit_n: f64,
    /// First-stage thrust (N).
    pub max_thrust_n: f64,
    /// Crew capacity.
    pub crew_capacity: u32,
    /// Components below TRL 6.
    pub sub_trl6_count: usize,
    /// Stated mission requirements.
    pub mission: MissionReqs,
}

/// Run the guards.
pub fn check(g: &GuardInputs) -> Vec<RedFlag> {
    let mut flags = Vec::new();
    let hard = |c: &str, v: f64| RedFlag {
        constraint: c.to_string(),
        value: v,
        severity: Severity::Hard,
    };
    let soft = |c: &str, v: f64| RedFlag {
        constraint: c.to_string(),
        value: v,
        severity: Severity::Soft,
    };

    if g.power_margin_w < 0.0 {
        flags.push(hard("negative power margin", g.power_margin_w));
    }
    if g.thermal_margin_w < 0.0 {
        flags.push(hard(
            "radiator shortfall (heat > rejection)",
            g.thermal_margin_w,
        ));
    }
    if g.structure_limit_n > 0.0 && g.max_thrust_n > g.structure_limit_n {
        flags.push(hard(
            "structure thrust limit exceeded",
            g.max_thrust_n - g.structure_limit_n,
        ));
    }
    if g.class.is_some_and(|c| c.lander) && g.lander_tw < 1.0 {
        flags.push(hard("lander T/W below local gravity", g.lander_tw));
    }
    if let Some(req) = g.mission.dv
        && g.total_dv < req
    {
        flags.push(soft("Δv short of mission requirement", req - g.total_dv));
    }
    if let Some(crew) = g.mission.crew
        && g.crew_capacity < crew
    {
        flags.push(hard(
            "crew capacity below mission requirement",
            f64::from(crew - g.crew_capacity),
        ));
    }
    if g.sub_trl6_count > 0 {
        flags.push(soft(
            "immature (sub-TRL-6) component",
            g.sub_trl6_count as f64,
        ));
    }
    flags
}
