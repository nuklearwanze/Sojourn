//! Structural divergence diagnosis (FR-CORE-203, SC-009): when two fingerprints
//! disagree, report *which* state region differs first — module slice or kernel
//! field — so the first divergent tick found by the harness is actionable.

use crate::error::CoreError;
use crate::state::serde_support::SaveBody;
use serde::{Deserialize, Serialize};

/// Result of a structural comparison of two world states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffReport {
    /// True when the canonical encodings are identical.
    pub identical: bool,
    /// Sections present in only one side, or differing — in section order.
    pub divergent_sections: Vec<SectionDiff>,
}

/// One differing section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionDiff {
    /// Section name (`kernel/<field>` or `slice/<module id>`).
    pub section: String,
    /// Byte offset of the first difference within the section (None = only one
    /// side has this section).
    pub first_byte: Option<u64>,
    /// Section byte lengths (left, right).
    pub lens: (u64, u64),
}

/// Compare two canonical bodies section by section.
pub fn diff(a: &SaveBody, b: &SaveBody) -> Result<DiffReport, CoreError> {
    let sa = a.sections()?;
    let sb = b.sections()?;
    let names: Vec<String> = {
        let mut n: Vec<String> = sa.iter().map(|(n, _)| n.clone()).collect();
        for (name, _) in &sb {
            if !n.contains(name) {
                n.push(name.clone());
            }
        }
        n
    };
    let mut divergent = Vec::new();
    for name in names {
        let la = sa.iter().find(|(n, _)| n == &name).map(|(_, b)| b);
        let lb = sb.iter().find(|(n, _)| n == &name).map(|(_, b)| b);
        match (la, lb) {
            (Some(x), Some(y)) if x == y => {}
            (Some(x), Some(y)) => {
                let first = x.iter().zip(y.iter()).position(|(p, q)| p != q);
                divergent.push(SectionDiff {
                    section: name,
                    first_byte: Some(first.unwrap_or(x.len().min(y.len())) as u64),
                    lens: (x.len() as u64, y.len() as u64),
                });
            }
            (Some(x), None) => divergent.push(SectionDiff {
                section: name,
                first_byte: None,
                lens: (x.len() as u64, 0),
            }),
            (None, Some(y)) => divergent.push(SectionDiff {
                section: name,
                first_byte: None,
                lens: (0, y.len() as u64),
            }),
            (None, None) => unreachable!(),
        }
    }
    Ok(DiffReport {
        identical: divergent.is_empty(),
        divergent_sections: divergent,
    })
}
