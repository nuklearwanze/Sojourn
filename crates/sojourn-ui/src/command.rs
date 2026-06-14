//! Plan → preview → commit (R4, FR-UI-301…305). An irreversible action is **gated**:
//! it MUST present a **core-computed** consequence preview and an explicit confirm
//! before any command is submitted; a reversible action is `Direct` (no gate). The
//! `Preview` consequences are the core's, never computed here. A rejected commit
//! surfaces its reason.

use crate::trace::TraceRender;

/// The class of an action the player initiates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    /// Commit a burn / manoeuvre.
    Burn,
    /// Launch a vehicle.
    Launch,
    /// Cancel a program / scrap a design.
    Cancel,
    /// Found/build/commission (irreversible delivery).
    Build,
    /// Change a Grand Goal (takes a penalty).
    ChangeGoal,
    /// A reversible UI/config action (layer toggle, sort, allocation tweak, lobby).
    Reversible,
}

impl ActionKind {
    /// True iff the action is irreversible/destructive and MUST be gated (FR-UI-301).
    pub fn is_irreversible(self) -> bool {
        !matches!(self, ActionKind::Reversible)
    }
}

/// A UI-only draft of an action, pending preview + commit.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftPlan {
    /// The action class.
    pub kind: ActionKind,
    /// A human label.
    pub label: String,
    /// The state generation this draft was built against (for stale detection).
    pub built_at_tick: u64,
}

/// A single previewed consequence (a delta the player will incur), traceable.
#[derive(Debug, Clone, PartialEq)]
pub struct TracedDelta {
    /// What changes.
    pub label: String,
    /// The formatted magnitude.
    pub formatted: String,
    /// The sourced derivation, when the core provides one.
    pub tree: Option<TraceRender>,
}

/// The **core-computed** consequence preview of a draft (never UI-computed).
#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    /// The consequences.
    pub deltas: Vec<TracedDelta>,
    /// What becomes unrecoverable if committed.
    pub becomes_unrecoverable: Vec<String>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// The tick the preview was computed against (stale if the core has advanced).
    pub computed_at_tick: u64,
}

impl Preview {
    /// True iff this preview is stale relative to `current_tick` and must be
    /// re-computed before commit (FR-UI-304).
    pub fn is_stale(&self, current_tick: u64) -> bool {
        self.computed_at_tick != current_tick
    }
}

/// The result of the plan→preview→commit gate.
#[derive(Debug, Clone, PartialEq)]
pub enum Gate {
    /// Irreversible: show the preview + require confirm before submitting.
    Gated(Preview),
    /// Reversible: submit directly, no confirmation.
    Direct,
}

/// Classify a draft into the gate (FR-UI-301/303). The `preview` is obtained from the
/// host (core-computed) for irreversible actions only.
pub fn gate(draft: &DraftPlan, preview: impl FnOnce() -> Preview) -> Gate {
    if draft.kind.is_irreversible() {
        Gate::Gated(preview())
    } else {
        Gate::Direct
    }
}

/// The outcome of a commit, surfaced from the core's `CommandOutcome`.
#[derive(Debug, Clone, PartialEq)]
pub enum CommitOutcome {
    /// The core applied the command.
    Applied,
    /// The core rejected it; the reason is surfaced, never hidden (FR-UI-305).
    Rejected(String),
}
