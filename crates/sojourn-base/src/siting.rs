//! Siting & planetary-protection guards (R9, FR-BC-301…303). Query-time red-flags
//! composing `SiteFacts`: a Special Region without containment, a solar base in
//! permanent shadow, a shielding shortfall, an unbuildable slope, no comms. The
//! guards never silently permit a violation (the FA-04 realism-guard pattern).

use crate::base::Base;
use crate::catalogue::{Catalogue, ModuleParams};
use crate::inputs::{PpCategory, SiteFacts};
use crate::power::PowerBalance;
use crate::shielding::Shielding;
use serde::{Deserialize, Serialize};

/// Flag severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// A hard constraint violation (physically/operationally infeasible or illegal).
    Hard,
    /// A soft warning (buildable but flagged).
    Soft,
}

/// A siting/realism red-flag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedFlag {
    /// The violated constraint.
    pub constraint: String,
    /// Severity.
    pub severity: Severity,
    /// Detail (the offending value / rule).
    pub detail: String,
}

/// The forward-contamination consequence of a PP violation (representable).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ForwardContamination {
    /// Fraction of the site's astrobiology/science value lost.
    pub pp_value_loss: f64,
}

const MAX_BUILDABLE_SLOPE_DEG: f64 = 25.0;

/// True iff the base carries a sterilisation/containment kit (an EDL/shielding
/// module flagged for containment — proxied here by carrying any shielding, a
/// stand-in until FA-09 models the PP containment chain).
fn has_containment(base: &Base, cat: &Catalogue) -> bool {
    base.modules
        .values()
        .filter(|m| m.commissioned)
        .filter_map(|m| cat.module(&m.type_id))
        .any(|t| matches!(t.params, ModuleParams::Shielding { .. }))
}

/// True iff the base draws power only from solar modules.
fn solar_only(base: &Base, cat: &Catalogue) -> bool {
    let mut has_power = false;
    let mut all_solar = true;
    for m in base.modules.values().filter(|m| m.commissioned) {
        if let Some(t) = cat.module(&m.type_id)
            && let ModuleParams::Power { solar, .. } = &t.params
        {
            has_power = true;
            if !solar {
                all_solar = false;
            }
        }
    }
    has_power && all_solar
}

/// Compute the siting red-flags for a base at its site.
pub fn flags(
    base: &Base,
    cat: &Catalogue,
    site: &SiteFacts,
    power: &PowerBalance,
    shielding: &Shielding,
) -> Vec<RedFlag> {
    let mut out = Vec::new();

    if site.pp_category == PpCategory::SpecialRegion && !has_containment(base, cat) {
        out.push(RedFlag {
            constraint: "planetary-protection".into(),
            severity: Severity::Hard,
            detail: "Special Region sited without sterilisation/containment".into(),
        });
    }
    if power.margin_w < 0.0 {
        out.push(RedFlag {
            constraint: "power-margin".into(),
            severity: Severity::Hard,
            detail: format!("negative power margin {:.0} W", power.margin_w),
        });
    }
    if solar_only(base, cat) && site.illumination <= 0.01 {
        out.push(RedFlag {
            constraint: "illumination".into(),
            severity: Severity::Hard,
            detail: "solar-only base in permanent shadow".into(),
        });
    }
    if shielding.shortfall {
        out.push(RedFlag {
            constraint: "shielding".into(),
            severity: Severity::Hard,
            detail: format!(
                "transmitted dose {:.2} Sv/yr exceeds the crew limit",
                shielding.transmitted_dose_sv_yr
            ),
        });
    }
    if site.slope_deg > MAX_BUILDABLE_SLOPE_DEG {
        out.push(RedFlag {
            constraint: "slope".into(),
            severity: Severity::Hard,
            detail: format!("slope {:.0}° exceeds the buildable limit", site.slope_deg),
        });
    }
    if !site.comms_visible {
        out.push(RedFlag {
            constraint: "comms".into(),
            severity: Severity::Soft,
            detail: "no comms visibility to a relay/DSN".into(),
        });
    }
    out
}

/// The forward-contamination consequence if the base violates PP at its site.
pub fn forward_contamination(
    base: &Base,
    cat: &Catalogue,
    site: &SiteFacts,
) -> ForwardContamination {
    let violates = site.pp_category == PpCategory::SpecialRegion && !has_containment(base, cat);
    ForwardContamination {
        pp_value_loss: if violates { 1.0 } else { 0.0 },
    }
}
