//! Nondeterminism injections (FR-CORE-705, SC-009): ten distinct ways simulation
//! code can go wrong, each of which the determinism gate MUST catch. This module
//! deliberately uses every construct the workspace lints forbid — that is its
//! entire purpose — so the lints are allowed here and nowhere else.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher, RandomState};
use std::sync::atomic::{AtomicI64, Ordering};

/// The injection catalogue (≥ 10 distinct types per SC-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum Mutation {
    /// Unseeded draw via a fresh `RandomState` hasher.
    RandomStateDraw,
    /// Wall-clock read: `SystemTime::now()` nanoseconds.
    SystemTimeNanos,
    /// Wall-clock read: `Instant::now()` monotonic sub-microsecond bits.
    InstantElapsed,
    /// Iteration over a hash-ordered `HashMap`.
    HashMapIter,
    /// Iteration over a hash-ordered `HashSet`.
    HashSetIter,
    /// Heap-address entropy: allocator-dependent `Box` pointer value.
    BoxAddress,
    /// OS thread-id entropy from a spawned thread.
    SpawnedThreadId,
    /// Hidden global state shared between runs in one process.
    GlobalStatic,
    /// Hidden thread-local state shared between runs on one thread.
    ThreadLocalCounter,
    /// Timing-dependent loop count (spin until a wall-clock deadline).
    TimingLoop,
}

impl Mutation {
    /// Every injection type, for `mutate --all`.
    pub fn all() -> Vec<Mutation> {
        use Mutation::*;
        vec![
            RandomStateDraw,
            SystemTimeNanos,
            InstantElapsed,
            HashMapIter,
            HashSetIter,
            BoxAddress,
            SpawnedThreadId,
            GlobalStatic,
            ThreadLocalCounter,
            TimingLoop,
        ]
    }
}

static GLOBAL_POISON: AtomicI64 = AtomicI64::new(0);
thread_local! {
    static LOCAL_POISON: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
}

/// Produce a value that *should* be deterministic but is not — the synthetic
/// module folds it into its state, and the double-run gate must see the
/// divergence (two runs in one process observe different values).
pub fn poison(m: Mutation) -> i64 {
    match m {
        Mutation::RandomStateDraw => {
            let mut h = RandomState::new().build_hasher();
            h.write_u64(0x50_4A_52_4E); // "SJRN"
            h.finish() as i64
        }
        Mutation::SystemTimeNanos => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as i64)
            .unwrap_or(0),
        Mutation::InstantElapsed => {
            let t = std::time::Instant::now();
            std::hint::black_box(&t);
            t.elapsed().as_nanos() as i64 ^ (format!("{t:?}").len() as i64)
        }
        Mutation::HashMapIter => {
            let map: HashMap<u64, i64> = (0..64u64).map(|i| (i, i as i64)).collect();
            // First key in hash order — varies per RandomState instance.
            map.iter().next().map(|(k, _)| *k as i64).unwrap_or(0)
        }
        Mutation::HashSetIter => {
            let set: HashSet<u64> = (0..64u64).collect();
            set.iter().next().map(|k| *k as i64).unwrap_or(0)
        }
        Mutation::BoxAddress => {
            let b = Box::new(0u8);
            std::ptr::from_ref(&*b) as i64
        }
        Mutation::SpawnedThreadId => {
            let handle = std::thread::spawn(|| format!("{:?}", std::thread::current().id()));
            let s = handle.join().unwrap_or_default();
            s.bytes()
                .fold(0i64, |a, b| a.wrapping_mul(31).wrapping_add(i64::from(b)))
        }
        Mutation::GlobalStatic => GLOBAL_POISON.fetch_add(1, Ordering::Relaxed),
        Mutation::ThreadLocalCounter => LOCAL_POISON.with(|c| {
            let v = c.get();
            c.set(v + 1);
            v
        }),
        Mutation::TimingLoop => {
            let start = std::time::Instant::now();
            let mut spins: i64 = 0;
            while start.elapsed().as_micros() < 5 {
                spins = spins.wrapping_add(1);
                std::hint::black_box(spins);
            }
            spins
        }
    }
}
