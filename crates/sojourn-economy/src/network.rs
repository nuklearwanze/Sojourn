//! The directed transport graph (data-model §4, R4). Nodes are a **curated set**
//! referencing FA-03 world location ids; edges are transfers whose price
//! (Δv/TOF/window) is supplied as a composed `EdgePrice` (R2) — the economy does
//! not propagate. Launch edges consume mass-to-orbit; in-space edges consume Δv
//! (R5).

use crate::ids::{LocationId, OrbitClass};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What role a node plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    /// A staging node (orbit band, L-point, staging orbit).
    Staging,
    /// A surveyable Site a faction operates at.
    Site,
    /// A buffer depot.
    Depot,
}

/// How an edge is traversed (sets which budget it debits).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Surface→orbit launch: consumes mass-to-orbit capacity (R5).
    Launch {
        /// The orbit class priced by the launch market.
        orbit_class: OrbitClass,
    },
    /// In-space transfer: consumes propellant/Δv (R5).
    InSpace,
    /// Surface mobility (rover/hopper); consumes Δv-equivalent.
    Surface,
}

/// A graph node (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeDef {
    /// Stable id.
    pub id: String,
    /// The world location this node references.
    pub location: LocationId,
    /// Role.
    pub role: NodeRole,
    /// Provenance.
    pub source: String,
}

/// A graph edge (DATA).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeDef {
    /// Stable id.
    pub id: String,
    /// From-node id.
    pub from: String,
    /// To-node id.
    pub to: String,
    /// How it is traversed.
    pub kind: EdgeKind,
    /// Provenance.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct NetworkFile {
    nodes: Vec<NodeDef>,
    edges: Vec<EdgeDef>,
}

/// The loaded, validated transport graph (module data).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NetworkDefs {
    /// Nodes by id.
    pub nodes: BTreeMap<String, NodeDef>,
    /// Edges by id.
    pub edges: BTreeMap<String, EdgeDef>,
    /// Canonical source text (for the econ content hash).
    pub canonical: String,
}

impl NetworkDefs {
    /// Parse + validate `network.ron`.
    pub fn from_source(text: &str) -> Result<NetworkDefs, String> {
        let f: NetworkFile = ron::from_str(text).map_err(|e| format!("network.ron: {e}"))?;
        let mut nodes = BTreeMap::new();
        for n in f.nodes {
            if n.source.trim().is_empty() {
                return Err(format!("network node '{}' has empty source", n.id));
            }
            if nodes.insert(n.id.clone(), n.clone()).is_some() {
                return Err(format!("duplicate network node '{}'", n.id));
            }
        }
        let mut edges = BTreeMap::new();
        for e in f.edges {
            if e.source.trim().is_empty() {
                return Err(format!("network edge '{}' has empty source", e.id));
            }
            if !nodes.contains_key(&e.from) {
                return Err(format!("edge '{}' from unknown node '{}'", e.id, e.from));
            }
            if !nodes.contains_key(&e.to) {
                return Err(format!("edge '{}' to unknown node '{}'", e.id, e.to));
            }
            if edges.insert(e.id.clone(), e.clone()).is_some() {
                return Err(format!("duplicate network edge '{}'", e.id));
            }
        }
        Ok(NetworkDefs {
            nodes,
            edges,
            canonical: text.replace("\r\n", "\n"),
        })
    }

    /// An edge by id.
    pub fn edge(&self, id: &str) -> Option<&EdgeDef> {
        self.edges.get(id)
    }

    /// A node by id.
    pub fn node(&self, id: &str) -> Option<&NodeDef> {
        self.nodes.get(id)
    }

    /// The world location ids referenced by nodes (resolved against the world in CI).
    pub fn location_refs(&self) -> impl Iterator<Item = &str> {
        self.nodes.values().map(|n| n.location.0.as_str())
    }
}
