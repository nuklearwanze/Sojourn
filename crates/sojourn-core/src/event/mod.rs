//! Event spine (FR-CORE-401/402): records, the deterministic per-tick queue
//! discipline, scheduled future events, and the full-history store with an
//! in-memory recent window and tiered older history (clarified 2026-06-12).

pub mod interrupt;
pub mod order;
pub mod policy;
pub mod store;

use crate::error::CoreError;
use crate::state::ViewValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use store::TierSink;

/// Sequential event id; total order is assignment order within the documented
/// per-tick execution order (FR-CORE-205, see [`order`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

/// Who raised an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSource {
    /// The kernel itself (horizon, diagnostics, command outcomes).
    Kernel,
    /// A registered module, by id.
    Module(String),
    /// A fired watch condition.
    Watch(u64),
}

/// One occurrence in the run's history. Payloads carry identifiers and SI
/// quantities, never display text (FA-10 renders).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    /// Sequential id.
    pub id: EventId,
    /// Tick at which it occurred.
    pub tick: u64,
    /// Event class (from the data registry).
    pub class: String,
    /// Origin.
    pub source: EventSource,
    /// Deterministically ordered payload.
    pub payload: BTreeMap<String, ViewValue>,
}

/// A not-yet-occurred event: scheduled for a future tick, or queued within a tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingEvent {
    /// Event class.
    pub class: String,
    /// Origin.
    pub source: EventSource,
    /// Payload.
    pub payload: BTreeMap<String, ViewValue>,
}

/// Query filter over the full run history (FR-CORE-401).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventFilter {
    /// Restrict to these classes (None = all).
    pub classes: Option<Vec<String>>,
    /// Inclusive lower tick bound.
    pub from_tick: Option<u64>,
    /// Inclusive upper tick bound.
    pub to_tick: Option<u64>,
    /// Skip this many matches (paging).
    pub offset: u64,
    /// Return at most this many (0 = kernel default of 1024).
    pub limit: u64,
}

/// One page of history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPage {
    /// Matching events in id order.
    pub events: Vec<EventRecord>,
    /// Total events in the run so far (all classes).
    pub total: u64,
}

/// Serializable bookkeeping of the history store. Identical regardless of where
/// the tier bytes live (file vs memory), so sink choice can never influence
/// state, fingerprints or replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventStoreState {
    /// Recent window, in id order.
    pub recent: Vec<EventRecord>,
    /// Number of events spilled to the tier.
    pub spilled_count: u64,
    /// Chained BLAKE3 over spilled events (identity of the tier content).
    pub spilled_hash: [u8; 32],
    /// Byte length of the tier (consistency check against the sink).
    pub spilled_len: u64,
}

impl EventStoreState {
    /// Empty store.
    pub fn new() -> Self {
        EventStoreState {
            recent: Vec::new(),
            spilled_count: 0,
            spilled_hash: [0; 32],
            spilled_len: 0,
        }
    }

    /// Total events recorded.
    pub fn total(&self) -> u64 {
        self.spilled_count + self.recent.len() as u64
    }

    /// Append a record; spill the oldest beyond `window` to the tier. Spilling
    /// behaviour is sink-independent by design.
    pub fn push(
        &mut self,
        ev: EventRecord,
        window: usize,
        tier: &mut TierSink,
    ) -> Result<(), CoreError> {
        self.recent.push(ev);
        while self.recent.len() > window.max(1) {
            let oldest = self.recent.remove(0);
            let body = postcard::to_allocvec(&oldest).map_err(CoreError::ser)?;
            let len = u32::try_from(body.len()).map_err(CoreError::ser)?;
            let mut frame = Vec::with_capacity(4 + body.len());
            frame.extend_from_slice(&len.to_le_bytes());
            frame.extend_from_slice(&body);
            tier.append(&frame)?;
            let mut h = blake3::Hasher::new();
            h.update(&self.spilled_hash);
            h.update(&body);
            self.spilled_hash = *h.finalize().as_bytes();
            self.spilled_count += 1;
            self.spilled_len += frame.len() as u64;
        }
        Ok(())
    }

    /// Query the full history (tier + recent window), id order, paged.
    pub fn query(&self, filter: &EventFilter, tier: &TierSink) -> Result<EventPage, CoreError> {
        let limit = if filter.limit == 0 {
            1024
        } else {
            filter.limit
        };
        let mut matched: Vec<EventRecord> = Vec::new();
        let mut skipped = 0u64;

        let consider = |ev: EventRecord, matched: &mut Vec<EventRecord>, skipped: &mut u64| {
            if !Self::matches(&ev, filter) {
                return true;
            }
            if *skipped < filter.offset {
                *skipped += 1;
                return true;
            }
            if (matched.len() as u64) < limit {
                matched.push(ev);
            }
            (matched.len() as u64) < limit
        };

        if self.spilled_count > 0 {
            let bytes = tier.read_all()?;
            if bytes.len() as u64 != self.spilled_len {
                return Err(CoreError::IntegrityFailure {
                    what: format!(
                        "event tier length {} does not match recorded {}",
                        bytes.len(),
                        self.spilled_len
                    ),
                });
            }
            let mut pos = 0usize;
            let mut full = true;
            while pos < bytes.len() {
                let len =
                    u32::from_le_bytes(bytes[pos..pos + 4].try_into().expect("4 bytes")) as usize;
                let ev: EventRecord =
                    postcard::from_bytes(&bytes[pos + 4..pos + 4 + len]).map_err(CoreError::ser)?;
                pos += 4 + len;
                if !consider(ev, &mut matched, &mut skipped) {
                    full = false;
                    break;
                }
            }
            if !full {
                return Ok(EventPage {
                    events: matched,
                    total: self.total(),
                });
            }
        }
        for ev in &self.recent {
            if !consider(ev.clone(), &mut matched, &mut skipped) {
                break;
            }
        }
        Ok(EventPage {
            events: matched,
            total: self.total(),
        })
    }

    fn matches(ev: &EventRecord, f: &EventFilter) -> bool {
        if let Some(classes) = &f.classes
            && !classes.iter().any(|c| c == &ev.class)
        {
            return false;
        }
        if let Some(from) = f.from_tick
            && ev.tick < from
        {
            return false;
        }
        if let Some(to) = f.to_tick
            && ev.tick > to
        {
            return false;
        }
        true
    }
}

impl Default for EventStoreState {
    fn default() -> Self {
        Self::new()
    }
}
