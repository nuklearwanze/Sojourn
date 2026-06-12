# Data Model: Simulation Core & Time (FA-01)

Conceptual model for the kernel. Types are expressed as Rust-shaped pseudo-declarations;
exact field visibility/derives are implementation detail. Everything in **WorldState** must be
serde-serializable with deterministic encoding (Vec/slotmap/BTreeMap only) — that closure is
what makes saves, fingerprints and replay coverage identical by construction (research R3/R4).

## 1. Run & configuration

```rust
RunId(u64)                          // unique per run, from master seed derivation
struct RunConfig {
    master_seed: u64,
    horizon: HorizonYears,          // 25 | 50 | 100 (default 100)
    run_mode: RunMode,              // SaveAnywhere | Ironman
    base_tick_ns: u64,              // exact tick duration (config data; default 1 s)
    difficulty_inputs: DifficultyInputs,  // opaque pass-through to modules (FR-XCU-010)
    kernel_tunables: KernelTunables,      // autosave cadence, flush cadence, hash cadence (from DataSet)
}
struct Run {
    id: RunId,
    config: RunConfig,
    data_version: DataVersionId,    // pinned content-data identity (FR-CORE-601)
    build_id: BuildId,              // toolchain+crate version stamp (replay binds to build)
    lifecycle: RunLifecycle,
}
enum RunLifecycle { Created, Active, HorizonReached, SandboxContinued }
// Transitions: Created→Active (first step); Active→HorizonReached (horizon tick, end-of-run
// event emitted exactly once); HorizonReached→SandboxContinued (explicit command; flagged unscored).
// Pause is NOT a lifecycle state: pausing is host pacing (warp invariance, FR-CORE-204).
```

## 2. Time

```rust
struct Tick(u64);                            // monotonic tick index, the only clock
struct SimTimeNs(i128);                      // = tick × base_tick_ns, exact (SC-011: zero drift)
struct CalendarDate { year: u16, month: u8, day: u8, /* derived */ }
// Calendar service (pure functions): tick↔date/time, leap years, fiscal-year & recurring-cycle
// boundaries through 2126. No timezone; game-UTC only. Epoch: 2026-01-01T00:00:00.
```

## 3. World state (the single authoritative state)

```rust
struct WorldState {
    tick: Tick,
    run: Run,
    rng: RngRegistry,               // all stream states (serialized!)
    slices: Vec<ModuleSliceBox>,    // module-owned state, in registration order
    events: EventStore,             // history (recent window in memory, older tiers on disk)
    queue: EventQueue,              // scheduled/pending occurrences
    watches: WatchStore,            // registered conditions + armed/fired status
    interrupts: InterruptStore,     // pending (un-acknowledged) interrupts — survives save/load
    commands_pending: Vec<CommandEnvelope>,  // accepted, not yet applied (apply at next tick)
    pause_policies: BTreeMap<EventClassId, PausePolicy>,
}
```

**Invariants**
- Single-writer: `slices[i]` is mutated only by its owning module during its step phase (kernel-enforced; violation = loud defect, FR-CORE-502).
- Anything reachable from `WorldState` is serializable + fingerprint-covered; nothing outside it influences outcomes (FR-CORE-101).
- All container iteration orders are deterministic (R8).

## 4. Randomness

```rust
struct StreamPath(String);          // canonical name, e.g. "kernel/anomaly", "research/breakthrough/A4"
struct RngRegistry {
    master_seed: u64,
    streams: BTreeMap<StreamPath, ChaChaState>,  // lazily created; state serialized
}
// Derivation: stream_seed = blake3_keyed(master_seed, path). Identity is name-based, never
// positional: new consumers can never shift existing streams (FR-CORE-202, edge case "stream
// exhaustion / new consumers").
```

## 5. Commands (decisions)

```rust
struct CommandId(u64);              // sequential per run
struct CommandEnvelope {
    id: CommandId,
    submitted_at: Tick,             // tick of acceptance (the paused/boundary tick)
    applies_at: Tick,               // earliest next tick boundary (== submitted_at when paused)
    payload: Command,               // closed enum; this slice ships kernel commands only
}
enum Command {                      // kernel-owned variants (modules add via registry later)
    RegisterWatch(WatchSpec), ModifyWatch(WatchId, WatchSpec), RemoveWatch(WatchId),
    SetPausePolicy(EventClassId, PausePolicy),
    AcknowledgeInterrupt(InterruptId),
    ContinueSandbox,                // HorizonReached → SandboxContinued
    Synthetic(SyntheticCommand),    // harness-only test commands (cfg-gated)
}
enum CommandOutcome { Applied, Rejected(RejectReason) }   // both journaled + emitted as events
// Lifecycle: Submitted(validated) → Pending → Applied | Rejected — deterministic, never partial
// (FR-CORE-304). NOTE: warp-rate selection is NOT a command and never enters the journal
// (FR-CORE-204). Pause-policy & watch commands are journaled (player intent record) though they
// do not perturb module state.
```

## 6. Events, classes & interrupts

```rust
struct EventClassId(&'static str / interned id);   // from data registry (R13)
struct EventClassDef {                              // DATA (event-class registry file)
    id: EventClassId,
    default_pause: PausePolicy,
    // v1 kernel registry ships: maneuver-node, mission-milestone, anomaly, program-review,
    // budget-vote, discovery, watch-fired, command-rejected, end-of-horizon, kernel-diagnostic.
    // Content slices register more through the same data mechanism (FR-CORE-403).
}
enum PausePolicy { Interrupt, LogOnly }

struct EventId(u64);                // sequential per run; total order = (tick, seq) (FR-CORE-205)
struct EventRecord {
    id: EventId, tick: Tick, class: EventClassId,
    source: EventSource,            // Kernel | Module(ModuleId) | Watch(WatchId)
    payload: EventPayload,          // serializable, identifier-based (no display text; FA-10 renders)
}
struct EventStore   // full-history queryable: in-memory recent window + disk-backed older tiers
                    // (clarified 2026-06-12); filter by class/time-range/source (FR-CORE-401)

enum QueueEntry {
    ScheduledAt(Tick, PendingEvent),        // known-time occurrences (nodes, reviews)
    // condition-triggered occurrences materialize via watch evaluation, not stored here
}
struct EventQueue   // ordered by (tick, deterministic tiebreak: source order, seq) (FR-CORE-205/404)

struct InterruptId(u64);
struct Interrupt {
    id: InterruptId, raised_by: EventId, raised_at: Tick,
    status: InterruptStatus,        // Pending → Acknowledged (command); Pending survives save/load
}
// Stepping refuses to advance past an un-acknowledged Pending interrupt's tick — that is the
// whole interrupt-and-pause mechanism: "pause" = kernel returns control + refuses further
// stepping until acknowledgement (FR-CORE-404/405; host pacing stays presentation-side).
```

## 7. Watch conditions

```rust
struct WatchTemplateId(String);     // from template catalogue (DATA, R13)
struct WatchTemplateDef {           // DATA: parameter schema + referenced published-view field
    id: WatchTemplateId,
    params: Vec<ParamSpec>,         // typed: quantity, threshold, entity handle, date…
    view_binding: ViewFieldRef,     // which published view/field it reads
}
struct WatchSpec {                  // player-supplied instance
    expr: WatchExpr,                // composition tree
}
enum WatchExpr {                    // catalogue + AND/OR composition ONLY (clarified 2026-06-12)
    Leaf { template: WatchTemplateId, args: Vec<ParamValue> },
    And(Vec<WatchExpr>), Or(Vec<WatchExpr>),
}
struct WatchId(u64);
struct WatchState {
    id: WatchId, spec: WatchSpec,
    armed: bool,                    // edge semantics: fires on first truth (incl. truth at
                                    // registration tick); re-arms on documented false→true edge
}
// Evaluation: at every tick boundary where any bound view changed (cheap dirty-tracking),
// deterministic order by WatchId; cost budget: 500 conditions within SC-007.
```

## 8. Modules (the contract entities)

```rust
struct ModuleId(String);            // e.g. "astro", "economy"; "kernel" reserved
struct ModuleManifest {
    id: ModuleId,
    owned_slice: SliceTypeId,                  // exclusive ownership (a)
    reads: Vec<ViewId>,                        // consumed published views (b)
    emits: Vec<EventClassId>, subscribes: Vec<EventClassId>,   // (c)
    phase: PhaseHint,                          // update-phase placement (d) — final order derived
    streams: Vec<StreamPath>,                  // declared randomness (e)
    cadence: CadenceDecl,                      // (f) base cadence (tick multiple) +
    escalations: Vec<EscalationDecl>,          //     state conditions forcing fine timestep
}
struct ModuleRegistry {
    modules: Vec<RegisteredModule>,            // in derived deterministic topological order
    // Registration rejects: ownership overlap, unknown views, circular ordering (FR-CORE-501).
}
struct PublishedView   // read-only, serializable snapshot surface of a slice at tick boundary;
                       // also the query surface (core-api) and watch binding target
```

## 9. Journal & persistence

```rust
struct JournalFrame {               // length-prefixed + BLAKE3 checksum per frame (R10)
    kind: FrameKind,                // Header | Command | Event | CheckpointMark | HashMark
    body: bytes,                    // postcard-encoded
}
struct JournalHeader { run_id, master_seed, config, data_version, build_id, format_version }
// Replay determinant: Header + ordered Command frames (Events/marks are verification material).
// Durability: group-fsync per command, per interrupt event, and ≤5 s background cadence —
// the persistence layer's wall-clock use is the documented exception outside sim logic (R10).

struct SaveFile {
    header: SaveHeader,             // magic, save_format_version, build_id, data_version,
                                    // run_id, tick, blake3 integrity checksum
    payload: bytes,                 // postcard(WorldState) — complete, incl. RNG states, queues,
                                    // pending interrupts/commands, watch states (FR-CORE-601)
}
// Migration: save_format_version-keyed structural migrations on load; pinned content values
// never substituted (FR-CORE-602). Ironman: rolling autosave+journal only; integrity-check
// failure ⇒ refuse load with explanation (FR-CORE-305/603).

struct StateFingerprint([u8; 32]);  // blake3(canonical postcard(WorldState)) at tick boundary
struct DataVersionId([u8; 32]);     // blake3 of canonical DataSet content (R13)
```

## 10. Harness artefacts (outside WorldState)

```rust
struct ScenarioScript {             // RON file
    name: String, seed: u64, config: RunConfig,
    commands: Vec<(Tick, Command)>,           // scripted decisions (FR-CORE-703 / FR-SIM-007)
    checkpoints: Vec<Tick>,                   // fingerprint comparison points
    expected: Option<Vec<(Tick, StateFingerprint)>>,  // golden values (optional)
    load: Option<SyntheticLoadProfile>,
}
struct SyntheticLoadProfile {       // FR-CORE-704: entity churn count, events/sim-year,
    entities: u32, events_per_year: u32, watch_conditions: u32, craft_stand_ins: u32 }
```

## 11. Relationship summary

- `Run` 1—1 `WorldState`; 1—1 `Journal`; 1—N `SaveFile`; 1—1 pinned `DataVersionId`.
- `WorldState` 1—N `ModuleSlice` (ownership exact partition); 1—N `EventRecord`; 1—N `WatchState`; 1—N pending `Interrupt`/`CommandEnvelope`; 1—1 `RngRegistry`.
- `EventRecord` N—1 `EventClassDef` (data); `Interrupt` 1—1 raising `EventRecord`.
- `WatchState` N—N `WatchTemplateDef` (data) via expression leaves; binds to `PublishedView` fields.
- `ModuleManifest` declares everything the kernel needs to order, isolate, seed and schedule a module — it is the contract artifact later slices implement (contracts/module-contract.md).
