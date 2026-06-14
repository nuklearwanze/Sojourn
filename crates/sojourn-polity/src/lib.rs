//! # sojourn-polity — Politics, Events, Milestones & Astrobiology (FA-09)
//!
//! **The non-combat competitive and narrative layer** — the system that makes a
//! weaponless game tense. It owns three sources of pressure and the machinery
//! around them: the **race for ~120 historic firsts** (world-first > faction-first,
//! a global ledger so the player races an AI world), the **politics of money and
//! approval** (public/political mood that swings on successes and failures —
//! loss-of-crew most of all — and drives budgets, valuations, approvals, policy),
//! and the **honest astrobiology question** (a per-game **seeded ground truth** on
//! candidate worlds resolved only through a staged, probabilistic, mission-driven
//! evidence process against abiotic competitors, with **per-faction beliefs** that
//! aggregate to a **prestige-weighted community consensus** and may publicly
//! disagree until it crosses a confidence band). Around these sit the **seeded,
//! state-driven event system** (a daily Bernoulli multiplicative-hazard scheduler
//! feeding the FA-01 interrupt loop), the **COSPAR planetary-protection regime**
//! (categories I–V, Special Regions, **graded** forward contamination + a
//! back-contamination chain), the **abstracted AI world** (heuristic, seeded rivals
//! that never cheat physics), and the **Grand Goals** (Pathfinder/Homestead/
//! Prospector/Seeker) that resolve a run to a **pass/fail verdict + a secondary
//! composite score**.
//!
//! Architecturally the **capstone slice**, but it follows the FA-04 C1 / FA-08
//! decoupling exactly: **depends only on `sojourn-core`**. Every upstream gameplay
//! fact (achievements, research-tech maturity + the science tide, budgets/
//! valuations/tonnage/supply, FA-03 candidate priors + site PP categories, FA-08
//! loss-of-crew) flows in as **composed values** the host assembles. All
//! stochastics (ground-truth draw, events, policy drift, lobbying, AI decisions,
//! contamination) derive from **named seeded streams** on the daily step. The
//! astrobiology **ground truth is stored but never query-exposed**. There is **no
//! combat and no aliens**: discovered life is a **science object, never an actor**.
//! Contracts: `specs/010-politics-events-astrobiology/contracts/`.

#![forbid(unsafe_code)]

pub mod ai;
pub mod astrobiology;
pub mod events;
pub mod faction;
pub mod goals;
pub mod hazard;
pub mod ids;
pub mod inputs;
pub mod milestones;
pub mod module;
pub mod mood;
pub mod params;
pub mod policy;
pub mod protection;
pub mod query;
pub mod score;
pub mod trace;

pub use ids::{BodyId, CandidateId, EventClassId, FactionId, GoalKind, MilestoneId, PolicyId};
pub use inputs::{
    Achievement, AchievementFacts, CandidatePrior, CrewFacts, Difficulty, EconomyFacts,
    EvidenceInput, EvidenceStage, FactionInit, FundingModel, HomesteadFacts, MissionFacts,
    ScienceTide, SiteProtection, Tier,
};
pub use module::{PolityCommand, PolityModule, polity_payload};
pub use query::WorldSnapshot;

/// Errors raised by the polity slice.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolityError {
    /// A referenced faction does not exist.
    #[error("unknown faction {0}")]
    UnknownFaction(u32),
    /// A referenced milestone does not exist.
    #[error("unknown milestone {0}")]
    UnknownMilestone(u32),
    /// A referenced candidate world does not exist.
    #[error("unknown candidate world {0}")]
    UnknownCandidate(u32),
    /// A referenced policy lever does not exist.
    #[error("unknown policy lever '{0}'")]
    UnknownPolicy(String),
    /// The world has already been initialised.
    #[error("world already initialised")]
    AlreadyInitialised,
}
