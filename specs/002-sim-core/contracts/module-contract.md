# Contract: `SimModule` — what every game-system slice implements (FA-02…FA-09)

The kernel orchestrates modules; modules own all game-domain state and logic. This contract is
the deliverable later slices build against (FR-CORE-501…505, SC-010). It is satisfiable from
this document alone — the conformance suite, not kernel internals, is the arbiter.

## The trait (shape)

```rust
pub trait SimModule {
    /// Static declaration — everything the kernel needs to order, isolate, seed & schedule.
    fn manifest(&self) -> ModuleManifest;

    /// Create the module's owned state slice for a new run (seeded via ctx streams only).
    fn init(&self, ctx: &mut InitCtx) -> Box<dyn StateSlice>;

    /// Advance the owned slice for one scheduled step (kernel-managed cadence/escalation).
    /// May: mutate OWN slice; read declared views; draw declared streams; emit declared
    /// event classes; schedule future events; read the clock/calendar.
    /// May NOT: touch other slices, do I/O, read wall-clock, spawn threads, allocate per-tick
    /// in steady state (perf rule), or use non-ordered iteration over its containers.
    fn step(&self, slice: &mut dyn StateSlice, ctx: &mut StepCtx);

    /// Publish the read-only view(s) of the slice at a tick boundary (query/watch surface).
    fn publish(&self, slice: &dyn StateSlice) -> Vec<ViewSnapshot>;

    /// React to subscribed events (delivered in deterministic order within the tick).
    fn on_event(&self, slice: &mut dyn StateSlice, ev: &EventRecord, ctx: &mut StepCtx);

    /// Apply a typed module command (`Command::ModulePayload` routed here at
    /// command-application time; FA-02 amendment). The outcome is journaled like
    /// any command; malformed payloads MUST be deterministic `Rejected` outcomes,
    /// never panics. Default: rejects all kinds (modules opt in).
    fn on_command(&self, slice: &mut dyn StateSlice, kind: &str, payload: &[u8],
                  ctx: &mut StepCtx) -> CommandOutcome { /* default-rejecting */ }
}

pub trait StateSlice: erased Serialize/Deserialize {
    // serde round-trip + canonical encoding (fingerprint coverage is automatic via R3/R4)
}
```

## ModuleManifest — the six declarations (FR-CORE-501)

| # | Declaration | Kernel enforcement |
|---|---|---|
| a | `owned_slice` — exclusive state ownership | Registration rejects overlap; step gets `&mut` to OWN slice only (single-writer is structural, FR-CORE-502) |
| b | `reads` — consumed published views | Only declared views resolvable from `StepCtx`; undeclared access fails loudly |
| c | `emits` / `subscribes` — event classes | Emitting an undeclared class is a defect; delivery order deterministic |
| d | `phase` — update-phase placement | Final order = documented topological sort over (phase, declared reads, module id); identical every run; adding a module never silently reorders others (FR-CORE-503) |
| e | `streams` — randomness sub-stream paths | Streams resolved by name from the registry (stable identity, FR-CORE-202); drawing an undeclared stream is a defect |
| f | `cadence` + `escalations` — base tick multiple + state conditions forcing fine timestep | Kernel-managed scheduling (clarified 2026-06-12); escalation predicates evaluate over the module's own published view |

## StepCtx capabilities (the *whole* world a module sees)

```rust
ctx.tick() / ctx.sim_time() / ctx.calendar()       // clock & calendar services
ctx.view(ViewId) -> &ViewSnapshot                  // declared reads only (tick-boundary data)
ctx.rng(StreamPath) -> &mut RngStream              // declared streams only
ctx.emit(EventClassId, EventPayload)               // declared emissions; queued deterministically
ctx.schedule(at: Tick, PendingEvent)               // future known-time occurrence
ctx.defer_command(...)                             // module-originated command (journaled like any)
```

Nothing else. No filesystem, no network, no wall-clock, no global state — by construction.

## Determinism obligations (inherited by every module)

1. All randomness via `ctx.rng` declared streams.
2. Ordered/indexed iteration only (`Vec`, `slotmap`, `BTreeMap`); clippy config applies workspace-wide to sim crates.
3. Floats: follow the float policy (research R7) — `libm` for transcendentals, no fast-math, integers for time/counters.
4. Slice fully serde-serializable with deterministic encoding; no state outside the slice.
5. `step` must be pure w.r.t. (slice, ctx): same inputs ⇒ same outputs, no hidden statics.
6. Entity references are stable handles (slotmap keys / ids), never held Rust references.

## Registration & lifecycle

```text
SimCore::create(config, data, modules: Vec<Box<dyn SimModule>>)
  → validate manifests (ownership disjoint, views known, no cycles, streams unique)
  → derive deterministic order → init slices (seeded) → run loop:
      per scheduled step: module.step(own slice, ctx) in order
      → event delivery (on_event, deterministic order)
      → command application at tick boundary → watch evaluation → publish views
```

Registration failures are loud errors at startup, never silent reordering or merging.

## Conformance suite (kernel-provided; gate for every future module crate)

| Check | Verifies |
|---|---|
| Manifest validation | declarations complete, disjoint, acyclic |
| Double-run with module installed | module introduces no nondeterminism (state hash + event log identity, varied stepping) |
| Foreign-write attempt | structural rejection (compile-time where possible, loud runtime defect otherwise) |
| Stream isolation | changing module A's draw count leaves module B's values unchanged |
| Serde round-trip of slice | save/fingerprint coverage |
| Cadence/escalation behaviour | module steps exactly per declaration; escalation conditions honoured |
| Reference toy module | a documented example implementing this contract passes everything (SC-010) |

## What this contract deliberately does NOT cover

- Module-internal architecture (a slice may use any deterministic layout internally, including
  archetype/ECS storage, provided the obligations hold).
- Inter-module *gameplay* semantics (what an "anomaly" means) — defined by the owning slices.
- Parallel module execution — single-threaded in v1 slice; the declared-ownership graph is the
  designed escape hatch if a future slice proves the need (research R9), gated by the same
  double-run checks.
