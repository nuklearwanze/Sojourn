//! Milestone award logic (R2, FR-PEA-101…107). The world-/faction-first ledger and
//! the highest-prestige tiebreak live in `module.rs` (they mutate the slice); this
//! module owns the pure **condition evaluation** over composed `AchievementFacts`.

use crate::inputs::AchievementFacts;
use crate::params::{Condition, MilestoneDef};

/// True iff a single condition holds for the facts.
fn condition_met(c: &Condition, f: &AchievementFacts) -> bool {
    match c {
        Condition::Reusable => f.reusable,
        Condition::Crewed => f.crewed,
        Condition::Body(id) => f.body == Some(*id),
        Condition::PropellantSold(min) => f.propellant_sold_kg >= *min,
        Condition::IsruOrigin => f.isru_origin,
        Condition::Tag(t) => f.tags.iter().any(|x| x == t),
    }
}

/// True iff **all** of a milestone's award conditions hold (FR-PEA-103).
pub fn satisfies(f: &AchievementFacts, def: &MilestoneDef) -> bool {
    def.conditions.iter().all(|c| condition_met(c, f))
}
