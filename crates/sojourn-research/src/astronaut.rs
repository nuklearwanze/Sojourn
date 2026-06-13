//! Astronaut career pipeline (FR-RESP-601/602, R12): select → train → ready →
//! retired, with running career dose/health/psych budgets. In-mission dynamics are
//! FA-08's, fed back through the `CrewFeedback` command (contracts/crew-interface.md).

use serde::{Deserialize, Serialize};

/// Pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Selected, not yet trained.
    Candidate,
    /// In multi-year training.
    Training,
    /// Mission-ready.
    Ready,
    /// Aged out / over a career limit.
    Retired,
}

/// An astronaut's career state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AstroState {
    /// Current pipeline stage.
    pub stage: Stage,
    /// Accumulated career radiation dose.
    pub career_dose: f64,
    /// Health budget (1.0 = full).
    pub health: f64,
    /// Psychological load.
    pub psych: f64,
}

impl Default for AstroState {
    fn default() -> Self {
        AstroState {
            stage: Stage::Candidate,
            career_dose: 0.0,
            health: 1.0,
            psych: 0.0,
        }
    }
}

impl AstroState {
    /// Apply an FA-08 in-mission feedback delta; crossing the dose limit retires.
    pub fn apply_feedback(&mut self, dose: f64, health: f64, psych: f64, dose_limit: f64) {
        self.career_dose += dose;
        self.health = (self.health + health).clamp(0.0, 1.0);
        self.psych = (self.psych + psych).max(0.0);
        if self.career_dose >= dose_limit || self.health <= 0.0 {
            self.stage = Stage::Retired;
        }
    }

    /// Is this astronaut in the ready pool?
    pub fn ready(&self) -> bool {
        self.stage == Stage::Ready
    }
}
