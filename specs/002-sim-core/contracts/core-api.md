# Contract: `sojourn-core` Public API (the simulation ↔ presentation seam)

This is the authoritative boundary (FR-CORE-701/702). Rules that define the contract:

1. **Owned, serializable types only.** Every parameter and return type is an owned DTO deriving
   `Serialize`/`Deserialize`. No lifetimes, no callbacks, no trait objects cross the boundary.
   The identical surface therefore works in-process (Tauri Rust host calling directly) and over
   serialized IPC (each operation = request/response message) with no redesign.
2. **Synchronous, host-paced.** No async, no threads inside the core. The host drives stepping;
   warp rates and pause UX are host concerns (FR-CORE-105; warp invariance FR-CORE-204).
3. **Queries at tick boundaries only.** All queries are answered between step calls and reflect
   a completed tick (clarified 2026-06-12). Calls never observe a half-applied tick.
4. **Polling, not callbacks.** Interrupts/events are discovered via step results and queries.
5. **No presentation logic.** Payloads carry identifiers and quantities (SI), never display text.

## Handle & lifecycle

```rust
pub struct SimCore { /* opaque */ }

impl SimCore {
    /// Create a new run with the module set that defines this game's systems.
    /// DataSet supplies kernel data files; its DataVersionId is pinned. Module manifests
    /// are validated here (ownership, views, ordering) — registration failures are loud.
    pub fn create(config: RunConfig, data: DataSet, modules: Vec<Box<dyn SimModule>>)
        -> Result<SimCore, CoreError>;

    /// Restore from a save. Fails with actionable error if pinned data version unavailable
    /// or integrity check fails (ironman: any verification failure refuses load).
    pub fn load(save: SaveBytes, data_resolver: &DataResolver) -> Result<SimCore, CoreError>;

    /// Recover after crash: newest valid checkpoint + journal-tail replay.
    /// Reports exactly what (if anything) was truncated.
    /// NOTE: recovery is deliberately file-anchored (host-side path), unlike load(SaveBytes):
    /// a crashed run is identified by its on-disk journal. An IPC host performs recovery
    /// host-side and ships the resulting state; this asymmetry is by design.
    pub fn recover(journal_path: &Path, data_resolver: &DataResolver)
        -> Result<(SimCore, RecoveryReport), CoreError>;
}
```

## Stepping

```rust
pub enum StepRequest {
    Ticks(u64),                 // advance up to N ticks
    UntilSimTime(SimTimeNs),    // advance until simulated time reached
    UntilInterrupt,             // advance until next interrupt (or horizon)
}

pub struct StepResult {
    pub advanced_to: Tick,
    pub stopped: StopReason,    // Completed | Interrupted(Vec<InterruptId>) | HorizonReached
    pub new_events: Vec<EventId>,   // events raised during this step call (ids; bodies via query)
}

impl SimCore {
    /// Advances the world. Returns early (deterministically) on the first tick carrying a
    /// pause-enabled occurrence; the world is AT that tick, consequences not yet applied
    /// (FR-CORE-404). Refuses to advance while any interrupt is un-acknowledged.
    pub fn step(&mut self, req: StepRequest) -> Result<StepResult, CoreError>;
}
```

## Commands (the only mutation pathway — FR-CORE-301)

```rust
impl SimCore {
    /// Validate + accept a command. Applies at the next tick boundary (immediately-visible
    /// if currently at a boundary/paused). Outcome (Applied/Rejected) is journaled and
    /// emitted as an event; rejection is deterministic, never partial (FR-CORE-304).
    pub fn submit(&mut self, cmd: Command) -> Result<CommandReceipt, CoreError>;

    pub fn acknowledge(&mut self, id: InterruptId) -> Result<(), CoreError>;  // sugar for submit
}
```

Kernel command set: `RegisterWatch`, `ModifyWatch`, `RemoveWatch`, `SetPausePolicy`,
`AcknowledgeInterrupt`, `ContinueSandbox`, `ModuleCommand { module, key, value }` (simple
key/value routing as a `module-command` event), and — added by the FA-02 amendment
(specs/003-astrodynamics, research R11) — `ModulePayload { module, kind, payload: Vec<u8> }`:
an opaque, module-defined encoded command routed to `SimModule::on_command` at application
time. The kernel never decodes the payload (FR-CORE-505); malformed payloads are deterministic
`Rejected` outcomes, journaled like any command.

Additionally (FA-02 amendment): `SimCore::with_slice(module, f)` grants **read-only** access
to a module's slice at the current tick boundary — the channel module-specific query surfaces
(e.g. astrodynamics planning snapshots) use. It obeys all query rules (between steps only,
never mutating).

## Queries (read-only, complete coverage — FR-CORE-701)

```rust
impl SimCore {
    pub fn status(&self) -> RunStatus;                 // tick, sim time, calendar date, lifecycle,
                                                       // pending interrupts, pinned data version
    pub fn view(&self, id: ViewId) -> Result<ViewSnapshot, CoreError>;   // any published view
    pub fn views(&self) -> Vec<ViewDescriptor>;        // discoverable view catalogue
    pub fn events(&self, filter: EventFilter) -> EventPage;   // FULL run history, filterable by
                                                       // class/time-range/source, paged
                                                       // (disk-backed tiers transparent)
    pub fn interrupts(&self) -> Vec<Interrupt>;        // pending (un-acknowledged)
    pub fn watches(&self) -> Vec<WatchState>;
    pub fn pause_policies(&self) -> BTreeMap<EventClassId, PausePolicy>;
    pub fn calendar(&self, t: SimTimeNs) -> CalendarInfo;   // date math service (FR-CORE-103)
    pub fn watch_templates(&self) -> Vec<WatchTemplateDef>;  // the data-driven catalogue
}
```

## Persistence, fingerprint, replay

```rust
impl SimCore {
    pub fn save(&self) -> Result<SaveBytes, CoreError>;          // complete state (FR-CORE-601)
    pub fn fingerprint(&self) -> StateFingerprint;               // canonical hash at boundary
    pub fn fingerprint_diff(&self, other: &SaveBytes) -> DiffReport;  // divergence diagnosis
    pub fn export_journal(&self) -> Result<JournalBytes, CoreError>;

    /// Reconstruct a run by replaying header+commands; verification mode compares
    /// recorded events/hashes and reports the first divergence (SC-003/SC-009 diagnostics).
    pub fn replay(journal: JournalBytes, data_resolver: &DataResolver, until: Option<Tick>,
                  verify: bool) -> Result<(SimCore, ReplayReport), CoreError>;
}
```

## Error taxonomy (`CoreError`)

`InvalidConfig`, `DataVersionUnavailable { pinned, hint }`, `IntegrityFailure { what }`,
`SaveFormatUnsupported { found, supported }`, `MigrationFailed`, `JournalCorrupt { valid_prefix_tick, lost }`,
`CommandInvalid { reason }` (distinct from a *journaled* deterministic rejection),
`InterruptPending { ids }` (step refused), `UnknownView/Watch/Template`, `Internal(bug)`.
All errors are serializable (IPC-safe) and actionable (FR-CORE-602 / edge cases).

## Compatibility & versioning

- This API is an **internal programme contract** (no semver promise to third parties in v1.0 —
  modding deferred). Within the programme: additive evolution preferred; breaking changes require
  updating this contract document + all dependent slices in the same change.
- `BuildId` accompanies saves/journals; replay binds to build (clarified 2026-06-12).

## Conformance tests (shipped with this slice)

- Architecture audit: `sojourn-core` dependency tree contains no UI/Tauri/webview/render crates.
- API completeness: every spec-defined player action is achievable via this surface headlessly
  (SC-008 scenario coverage).
- DTO round-trip: every boundary type serializes/deserializes losslessly (IPC readiness).
- Query boundary: no API call can observe mid-tick state (enforced by construction + test).
