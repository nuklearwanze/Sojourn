//! Canonical world-state encoding (FR-CORE-504, research R3/R4): one body shared
//! by saves and fingerprints, so hash coverage automatically equals save coverage —
//! anything that round-trips is fingerprinted; anything new in state is both.

use super::KernelState;
use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// The complete world state as one serializable unit: kernel state, the spilled
/// event-tier bytes (saves are self-contained — they never depend on the old run
/// directory), and every module slice (serialized by its owner), in execution
/// order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveBody {
    /// Kernel-owned state.
    pub kernel: KernelState,
    /// Spilled event-history tier content (length/hash tracked in `kernel.events`).
    pub tier: Vec<u8>,
    /// `(module id, slice bytes)` in registry execution order.
    pub slices: Vec<(String, Vec<u8>)>,
}

impl SaveBody {
    /// Canonical postcard encoding (deterministic: all containers are ordered).
    pub fn encode(&self) -> Result<Vec<u8>, CoreError> {
        postcard::to_allocvec(self).map_err(CoreError::ser)
    }

    /// Decode a canonical encoding.
    pub fn decode(bytes: &[u8]) -> Result<SaveBody, CoreError> {
        postcard::from_bytes(bytes).map_err(CoreError::ser)
    }

    /// Named sections for structural divergence diagnosis (hash/diff.rs): each
    /// kernel field separately, then each module slice. Adding a `KernelState`
    /// field without listing it here is caught by the coverage test below.
    pub fn sections(&self) -> Result<Vec<(String, Vec<u8>)>, CoreError> {
        let k = &self.kernel;
        let mut out: Vec<(String, Vec<u8>)> = Vec::new();
        macro_rules! sec {
            ($name:literal, $field:expr) => {
                out.push((
                    $name.to_string(),
                    postcard::to_allocvec($field).map_err(CoreError::ser)?,
                ));
            };
        }
        sec!("kernel/run_id", &k.run_id);
        sec!("kernel/config", &k.config);
        sec!("kernel/lifecycle", &k.lifecycle);
        sec!("kernel/clock", &k.clock);
        sec!("kernel/rng", &k.rng);
        sec!("kernel/next_event_id", &k.next_event_id);
        sec!("kernel/next_command_id", &k.next_command_id);
        sec!("kernel/next_watch_id", &k.next_watch_id);
        sec!("kernel/next_interrupt_id", &k.next_interrupt_id);
        sec!("kernel/commands_pending", &k.commands_pending);
        sec!("kernel/scheduled", &k.scheduled);
        sec!("kernel/events", &k.events);
        sec!("kernel/watches", &k.watches);
        sec!("kernel/interrupts", &k.interrupts);
        sec!("kernel/pause_policies", &k.pause_policies);
        sec!("kernel/views", &k.views);
        sec!("kernel/horizon_signalled", &k.horizon_signalled);
        out.push(("tier".to_string(), self.tier.clone()));
        for (id, bytes) in &self.slices {
            out.push((format!("slice/{id}"), bytes.clone()));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    /// Guards section coverage: if `KernelState` grows a field, `sections()` must
    /// list it. Postcard encodes structs as the concatenation of their fields, so
    /// the concatenated section bytes (kernel part) must reconstruct the whole
    /// kernel encoding exactly.
    #[test]
    fn sections_cover_every_kernel_field() {
        use super::*;
        use crate::clock::SimClock;
        use crate::rng::RngRegistry;
        use crate::state::{RunConfig, RunLifecycle, RunMode};
        use std::collections::BTreeMap;

        let kernel = KernelState {
            run_id: 7,
            config: RunConfig {
                master_seed: 1,
                horizon_years: 100,
                run_mode: RunMode::SaveAnywhere,
                difficulty_inputs: BTreeMap::new(),
            },
            lifecycle: RunLifecycle::Created,
            clock: SimClock::new(1_000_000_000, 100),
            rng: RngRegistry::new(1),
            next_event_id: 0,
            next_command_id: 0,
            next_watch_id: 0,
            next_interrupt_id: 0,
            commands_pending: Vec::new(),
            scheduled: BTreeMap::new(),
            events: crate::event::EventStoreState::new(),
            watches: BTreeMap::new(),
            interrupts: BTreeMap::new(),
            pause_policies: BTreeMap::new(),
            views: BTreeMap::new(),
            horizon_signalled: false,
        };
        let body = SaveBody {
            kernel,
            tier: vec![9, 9],
            slices: vec![("m".into(), vec![1, 2, 3])],
        };
        let whole_kernel = postcard::to_allocvec(&body.kernel).unwrap();
        let concat: Vec<u8> = body
            .sections()
            .unwrap()
            .iter()
            .filter(|(n, _)| n.starts_with("kernel/"))
            .flat_map(|(_, b)| b.clone())
            .collect();
        assert_eq!(
            concat, whole_kernel,
            "sections() is missing a KernelState field"
        );
    }
}
