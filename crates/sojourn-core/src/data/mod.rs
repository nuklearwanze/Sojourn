//! Kernel data files, validation hooks and content-data version identity
//! (FR-CORE-505, research R13). This is the Principle V pipeline in miniature:
//! content lives in versioned, strictly-validated data files; saves pin the exact
//! `DataVersionId` they started with (FR-CORE-601/602).

use crate::error::CoreError;
use crate::event::policy::PausePolicy;
use crate::watch::{CmpOp, ParamKind};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// BLAKE3 digest of the canonical data-set content. What saves pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DataVersionId(pub [u8; 32]);

impl DataVersionId {
    /// Hex form for headers, errors and the status query.
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// One event class definition (data file `event-classes.ron`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventClassDef {
    /// Canonical class id, e.g. `"anomaly"`.
    pub id: String,
    /// Out-of-the-box pause policy; players adjust per class at runtime.
    pub default_pause: PausePolicy,
    /// Provenance of the entry (Principle I pattern; kernel entries cite design docs).
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventClassFile {
    classes: Vec<EventClassDef>,
}

/// One watch-condition template (data file `watch-templates.ron`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WatchTemplateDef {
    /// Template id players select, e.g. `"date-reached"`.
    pub id: String,
    /// Published view the template reads.
    pub view: String,
    /// Field within that view.
    pub field: String,
    /// Comparison applied between the field value and the player-supplied parameter.
    pub op: CmpOp,
    /// Type the parameter must have.
    pub param: ParamKind,
    /// Provenance of the entry.
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WatchTemplateFile {
    templates: Vec<WatchTemplateDef>,
}

/// Kernel tunables (data file `config.ron`) — engineering values, never code constants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelConfig {
    /// Exact duration of one tick in nanoseconds.
    pub base_tick_ns: u64,
    /// Autosave cadence in ticks (in addition to every interrupt).
    pub autosave_interval_ticks: u64,
    /// Journal background flush bound in wall-clock milliseconds (≤ 5000 per SC-004).
    pub journal_flush_wall_ms: u64,
    /// HASHMARK frame cadence in ticks.
    pub hash_mark_interval_ticks: u64,
    /// Events kept in memory before spilling to disk tiers.
    pub event_memory_window: usize,
    /// Provenance of the values.
    pub source: String,
}

/// The loaded, validated kernel data set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSet {
    /// Event class registry.
    pub event_classes: Vec<EventClassDef>,
    /// Watch template catalogue.
    pub watch_templates: Vec<WatchTemplateDef>,
    /// Kernel tunables.
    pub config: KernelConfig,
    /// Identity of this exact content.
    version: DataVersionId,
}

impl DataSet {
    /// Load and validate the three kernel data files from a directory.
    pub fn load_dir(dir: &Path) -> Result<DataSet, CoreError> {
        let ec_bytes = read(dir, "event-classes.ron")?;
        let wt_bytes = read(dir, "watch-templates.ron")?;
        let cf_bytes = read(dir, "config.ron")?;
        Self::from_sources(&ec_bytes, &wt_bytes, &cf_bytes)
    }

    /// Build from raw file contents (used by tests and by in-memory resolvers).
    pub fn from_sources(
        event_classes_ron: &str,
        watch_templates_ron: &str,
        config_ron: &str,
    ) -> Result<DataSet, CoreError> {
        let ec: EventClassFile = parse("event-classes.ron", event_classes_ron)?;
        let wt: WatchTemplateFile = parse("watch-templates.ron", watch_templates_ron)?;
        let config: KernelConfig = parse("config.ron", config_ron)?;

        // Canonical identity: file-name-tagged content, in fixed order, LF-normalized
        // (the repo enforces LF via .gitattributes; normalize again for safety).
        let mut hasher = blake3::Hasher::new();
        for (name, body) in [
            ("event-classes.ron", event_classes_ron),
            ("watch-templates.ron", watch_templates_ron),
            ("config.ron", config_ron),
        ] {
            hasher.update(name.as_bytes());
            hasher.update(&[0]);
            hasher.update(body.replace("\r\n", "\n").as_bytes());
            hasher.update(&[0]);
        }
        let version = DataVersionId(*hasher.finalize().as_bytes());

        let ds = DataSet {
            event_classes: ec.classes,
            watch_templates: wt.templates,
            config,
            version,
        };
        ds.validate()?;
        Ok(ds)
    }

    /// Semantic validation beyond strict parsing.
    pub fn validate(&self) -> Result<(), CoreError> {
        let mut seen = std::collections::BTreeSet::new();
        for c in &self.event_classes {
            if c.source.trim().is_empty() {
                return Err(CoreError::DataInvalid {
                    reason: format!("event class '{}' has an empty source field", c.id),
                });
            }
            if !seen.insert(&c.id) {
                return Err(CoreError::DataInvalid {
                    reason: format!("duplicate event class id '{}'", c.id),
                });
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for t in &self.watch_templates {
            if t.source.trim().is_empty() {
                return Err(CoreError::DataInvalid {
                    reason: format!("watch template '{}' has an empty source field", t.id),
                });
            }
            if !seen.insert(&t.id) {
                return Err(CoreError::DataInvalid {
                    reason: format!("duplicate watch template id '{}'", t.id),
                });
            }
        }
        if self.config.base_tick_ns == 0 {
            return Err(CoreError::DataInvalid {
                reason: "base_tick_ns must be > 0".into(),
            });
        }
        if self.config.journal_flush_wall_ms > 5000 {
            return Err(CoreError::DataInvalid {
                reason: "journal_flush_wall_ms must be ≤ 5000 (SC-004 durability bound)".into(),
            });
        }
        Ok(())
    }

    /// Identity of this exact content (what saves pin).
    pub fn version(&self) -> DataVersionId {
        self.version
    }

    /// Look up an event class definition.
    pub fn event_class(&self, id: &str) -> Option<&EventClassDef> {
        self.event_classes.iter().find(|c| c.id == id)
    }

    /// Look up a watch template.
    pub fn watch_template(&self, id: &str) -> Option<&WatchTemplateDef> {
        self.watch_templates.iter().find(|t| t.id == id)
    }
}

fn read(dir: &Path, name: &str) -> Result<String, CoreError> {
    std::fs::read_to_string(dir.join(name))
        .map_err(|e| CoreError::io(&format!("reading {}", dir.join(name).display()), e))
}

fn parse<T: serde::de::DeserializeOwned>(name: &str, body: &str) -> Result<T, CoreError> {
    ron::from_str(body).map_err(|e| CoreError::DataInvalid {
        reason: format!("{name}: {e}"),
    })
}

/// Resolves a pinned `DataVersionId` back to its `DataSet` on load/recovery
/// (umbrella clarification: saves pin content data; never silent substitution).
pub trait DataResolver {
    /// Return the data set with exactly this identity, if available.
    fn resolve(&self, id: DataVersionId) -> Option<DataSet>;
}

/// Resolver over candidate directories (installed data versions side by side).
pub struct DirResolver {
    /// Directories to probe, in order.
    pub dirs: Vec<std::path::PathBuf>,
}

impl DataResolver for DirResolver {
    fn resolve(&self, id: DataVersionId) -> Option<DataSet> {
        for d in &self.dirs {
            if let Ok(ds) = DataSet::load_dir(d)
                && ds.version() == id
            {
                return Some(ds);
            }
        }
        None
    }
}

/// Resolver over already-loaded data sets (tests, embedded data).
pub struct MemResolver {
    /// Candidate data sets.
    pub sets: Vec<DataSet>,
}

impl DataResolver for MemResolver {
    fn resolve(&self, id: DataVersionId) -> Option<DataSet> {
        self.sets.iter().find(|d| d.version() == id).cloned()
    }
}
