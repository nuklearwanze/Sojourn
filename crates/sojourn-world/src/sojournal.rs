//! The Sojournal (FR-WORLD-601/602, Principle VIII): cited encyclopedia data for
//! bodies, classes, location/site types and concepts. Entries are pure data and
//! describe the real world and the game's honest mechanics only — they cannot
//! bind to a per-game seeded truth. CI enforces citation presence, link
//! resolution and major-body coverage.

use crate::locations::Locations;
use crate::sites::Sites;
use serde::{Deserialize, Serialize};
use sojourn_astro::{BodyId, Catalog};
use std::collections::BTreeMap;
use std::path::Path;

/// What an entry is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    /// A specific body.
    Body,
    /// A class of bodies.
    BodyClass,
    /// A type of dynamical location.
    LocationType,
    /// A class of site.
    SiteClass,
    /// A general concept.
    Concept,
}

/// A reference to another world object or entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ref {
    /// A catalogued body.
    Body(BodyId),
    /// A site (by data key).
    Site(String),
    /// A location (by data key).
    Location(String),
    /// Another Sojournal entry (by id).
    Entry(String),
}

/// A cited source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    /// Provenance string.
    pub source: String,
}

/// A Sojournal entry (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SojournalEntry {
    /// Stable id.
    pub id: String,
    /// What kind of entry.
    pub kind: EntryKind,
    /// The primary subject, if any.
    pub subject_ref: Option<Ref>,
    /// Title identifier (display text is FA-10's).
    pub title_id: String,
    /// Body text (identifier-keyed prose; real-world / mechanics only).
    pub body_text: String,
    /// Citations (≥1 enforced).
    pub citations: Vec<Citation>,
    /// Resolvable links.
    pub links: Vec<Ref>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SojournalFile {
    entries: Vec<SojournalEntry>,
}

/// The loaded Sojournal (DATA, `data/world/sojournal/*.ron`).
#[derive(Debug, Clone, Default)]
pub struct Sojournal {
    entries: Vec<SojournalEntry>,
    index_of: BTreeMap<String, usize>,
    body_entry: BTreeMap<BodyId, usize>,
    /// Canonical concatenated source text (for the world content hash).
    pub canonical: String,
}

impl Sojournal {
    /// Load all `*.ron` files in `dir/sojournal/` (sorted), if the directory
    /// exists; otherwise an empty Sojournal.
    pub fn load_dir(dir: &Path) -> Result<Sojournal, String> {
        let sj = dir.join("sojournal");
        if !sj.is_dir() {
            return Ok(Sojournal::default());
        }
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&sj)
            .map_err(|e| format!("reading {}: {e}", sj.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "ron"))
            .collect();
        files.sort();
        let mut canonical = String::new();
        let mut entries = Vec::new();
        for f in &files {
            let text = std::fs::read_to_string(f).map_err(|e| format!("reading {}: {e}", f.display()))?;
            let file: SojournalFile = ron::from_str(&text).map_err(|e| format!("{}: {e}", f.display()))?;
            canonical.push_str(&text.replace("\r\n", "\n"));
            canonical.push('\n');
            entries.extend(file.entries);
        }
        let mut index_of = BTreeMap::new();
        let mut body_entry = BTreeMap::new();
        for (i, e) in entries.iter().enumerate() {
            if index_of.insert(e.id.clone(), i).is_some() {
                return Err(format!("sojournal: duplicate entry id '{}'", e.id));
            }
            if let Some(Ref::Body(b)) = &e.subject_ref {
                body_entry.insert(*b, i);
            }
        }
        Ok(Sojournal {
            entries,
            index_of,
            body_entry,
            canonical,
        })
    }

    /// An entry by id.
    pub fn entry(&self, id: &str) -> Option<&SojournalEntry> {
        self.index_of.get(id).map(|&i| &self.entries[i])
    }

    /// Whether a body has an entry.
    pub fn has_body_entry(&self, id: BodyId) -> bool {
        self.body_entry.contains_key(&id)
    }

    /// All entries.
    pub fn entries(&self) -> &[SojournalEntry] {
        &self.entries
    }

    /// Validate citations and link resolution (FR-WORLD-601). `major_bodies` are
    /// the ids that MUST each have an entry.
    pub fn validate(
        &self,
        catalog: &Catalog,
        sites: &Sites,
        locations: &Locations,
        major_bodies: &[BodyId],
    ) -> Result<(), String> {
        for e in &self.entries {
            if e.citations.is_empty() {
                return Err(format!("sojournal entry '{}' has no citation", e.id));
            }
            for c in &e.citations {
                if c.source.trim().is_empty() {
                    return Err(format!("sojournal entry '{}' has an empty citation", e.id));
                }
            }
            for r in e.subject_ref.iter().chain(&e.links) {
                self.resolve_ref(r, catalog, sites, locations)
                    .map_err(|m| format!("sojournal entry '{}': {m}", e.id))?;
            }
        }
        for b in major_bodies {
            if !self.has_body_entry(*b) {
                return Err(format!("sojournal: major body {} has no entry", b.0));
            }
        }
        Ok(())
    }

    fn resolve_ref(
        &self,
        r: &Ref,
        catalog: &Catalog,
        sites: &Sites,
        locations: &Locations,
    ) -> Result<(), String> {
        match r {
            Ref::Body(b) => catalog
                .body(*b)
                .map(|_| ())
                .ok_or_else(|| format!("link to unknown body {}", b.0)),
            Ref::Site(k) => sites
                .resolve(k)
                .map(|_| ())
                .ok_or_else(|| format!("link to unknown site '{k}'")),
            Ref::Location(k) => locations
                .def(k)
                .map(|_| ())
                .ok_or_else(|| format!("link to unknown location '{k}'")),
            Ref::Entry(k) => self
                .index_of
                .contains_key(k)
                .then_some(())
                .ok_or_else(|| format!("link to unknown entry '{k}'")),
        }
    }
}
