# Phase 0 Research: Simulation Core & Time (FA-01)

All Technical Context unknowns resolved. Each decision lists rationale and alternatives.
Licence constraint applied throughout: MIT/Apache-2.0 or compatible (CC0, Zlib acceptable).

## R1 — Language & toolchain

- **Decision**: Rust, pinned stable toolchain via `rust-toolchain.toml` (latest stable at adoption, e.g. 1.88+), edition 2024, no nightly features. Default codegen flags only — explicitly no `-ffast-math` analogues (Rust has none by default; we additionally forbid `target-feature` overrides that alter FP behaviour in CI).
- **Rationale**: User-directed. Rust gives the determinism-friendly properties the spec leans on: no GC pauses, explicit data layout, ownership preventing aliasing bugs, `Send`-free single-threaded sim trivially expressible, first-class WASM/FFI options later. Pinning the toolchain operationalises the "determinism per platform+build" clarification — the build *is* part of the guarantee.
- **Alternatives considered**: C++ (more FP-determinism folklore but worse safety/build ergonomics); C# (GC + JIT complicate same-binary reproducibility claims); Zig (immature ecosystem for serde-class tooling).

## R2 — PRNG: seeded, splittable, named sub-streams, serializable

- **Decision**: `rand_chacha::ChaCha12Rng` (via `rand_core` traits) as the stream engine. Sub-stream derivation: child seed = BLAKE3 keyed-hash of (master seed, canonical stream path string e.g. `"research/breakthrough/A4"`), giving stable *named identity* rather than positional splitting. RNG states serialize with the world state (serde support in `rand_chacha`).
- **Rationale**: ChaCha is portable and bit-stable across platforms/architectures (pure integer ops), well-audited, serializable, and fast enough for game use. Name-derived seeding satisfies FR-CORE-202's "stable identity, not positional": adding a new consumer can never shift an existing stream because each stream's seed depends only on its name + master seed. 12 rounds balances margin vs speed (8 would suffice statistically; 12 is the conservative default).
- **Alternatives considered**: `rand_pcg` (smaller state, but PCG stream-selection is positional and less natural for named hierarchies); `rand_xoshiro` SplitMix (fast, but ad-hoc splitting invites correlation if misused); bare SplitMix64 hand-rolled (less scrutiny, no win).

## R3 — State fingerprint (canonical hash)

- **Decision**: BLAKE3 over the canonical `postcard` encoding of the full world state (the same encoding path saves use), computed at tick boundaries on demand / at configured cadence. 256-bit digest; harness also implements a structural diff mode (walk two serialized state trees, report first divergent module/field) for divergence diagnosis (FR-CORE-203, SC-009 diagnostics).
- **Rationale**: Hashing the serialization bytes guarantees fingerprint coverage automatically equals save coverage — anything that round-trips is fingerprinted, anything new added to state is automatically included. BLAKE3 is fast (multi-GB/s), incremental, and CC0/Apache. Collision probability at 256 bits is negligible for CI equality evidence.
- **Alternatives considered**: bespoke `StableHash` trait per type (faster, but coverage drifts from serialization and every new field is a hash bug waiting); xxh3 (faster still but non-cryptographic — fine for equality, yet BLAKE3's speed is already sufficient and doubles as the integrity/keyed-derivation primitive, one dependency instead of two).

## R4 — Serialization & save format

- **Decision**: `serde` + `postcard` as the canonical binary codec for state payloads, journal frames and fingerprint input. Save container: explicit header (magic, save-format version, build identifier, content-data version id, BLAKE3 integrity checksum) + postcard payload. Save-format migration = versioned Rust migration functions applied on load (structure only — pinned content values never substituted, per umbrella clarification). Determinism rule: state types use only deterministic-order containers (`Vec`, `slotmap`, `BTreeMap`) so the canonical encoding is unique.
- **Rationale**: postcard is compact, fast, `no_std`-grade simple, schema-explicit through Rust types, and MIT/Apache. A hand-rolled header gives exact control over versioning, pinning and integrity (FR-CORE-601/602/305) without a heavyweight schema system.
- **Alternatives considered**: `bincode` v2 (fine, but postcard's varint encoding is more compact and its spec is frozen/documented); MessagePack/CBOR (self-describing — wasted bytes for a closed format we fully control; slower); JSON saves (human-readable but huge, slow, float-roundtrip hazards); `rkyv` (zero-copy speed, but mmap'd archives complicate migration + integrity and invite undefined-behaviour foot-guns; revisit only if save sizes become a problem).

## R5 — Entity storage & world-state organisation

- **Decision**: No ECS framework in this slice. Module-owned typed stores built on `slotmap` (`SlotMap`/`SecondaryMap` with generational keys re-exported as stable 64-bit handles), composed under a `StateSlice` contract (serialize, hash-input, snapshot for views). Cross-entity references are handles, never Rust references held across steps. Iteration in sim logic is over `slotmap` (deterministic given identical operation history — which determinism guarantees) or explicitly ordered indices; `BTreeMap` for ordered key→value lookups.
- **Rationale**: The spec's single-writer module-ownership contract (FR-CORE-501/502) maps 1:1 onto module-owned stores; a shared ECS world would blur exactly the ownership boundary this slice exists to define. slotmap gives the stable-ID/generational safety the user asked for with zero framework lock-in, trivially serde-serializable, and deterministic iteration. The kernel in this slice owns few entities itself (events, conditions, registrations); heavy entity volume arrives with content slices, which can still adopt archetype layouts *inside* their own slice if profiling demands it.
- **Alternatives considered**: `bevy_ecs` (drags in scheduling/parallelism defaults that are determinism hazards; heavy); `hecs` (lean, but iteration order is archetype-internal and easier to misuse than explicit stores; still a framework where a contract is the deliverable); `specs`/`legion` (unmaintained); hand-rolled generational arena (≈ slotmap with less testing).

## R6 — Time, clock & calendar

- **Decision**: Simulation time = `u64` tick index plus exact integer nanosecond offset from epoch 2026-01-01T00:00:00 game-UTC (`i128` ns where absolute time is needed; century ≈ 3.16×10¹⁸ ns fits i64 but i128 leaves headroom for arithmetic). Base tick duration is config data (default 1 s; fine-step escalation runs at 1-tick resolution, coarse cadences are integer tick multiples). Civil calendar (dates, leap years, fiscal-year/cycle boundaries through 2126) implemented as a small pure deterministic module (proleptic Gregorian, ~100 lines, exhaustively tested) — no external date crate, no timezone concept beyond game-UTC.
- **Rationale**: Integer time satisfies FR-CORE-101/103 and SC-011 (zero drift by construction: time = ticks × exact integer dt). Hand-rolling the calendar removes a dependency whose timezone/locale machinery we must *not* use anyway, and makes leap-year correctness a 20-case unit test instead of an upstream trust exercise.
- **Alternatives considered**: `time` crate (good, deterministic, but 95% unused surface); `chrono` (local-time APIs are exactly the wall-clock trap the spec forbids); `hifitime` (astronomy-grade time scales — attractive for FA-02's TDB/UTC needs later, but premature here; FA-02 can layer it as a *conversion* concern without changing kernel ticks).

## R7 — Float determinism policy

- **Decision**: Scope: same-binary reproducibility (per umbrella/spec clarification). Policy shipped as a documented kernel rule for all slices: (a) default codegen only — CI forbids fast-math-like `target-feature`/`RUSTFLAGS` overrides; (b) any transcendental/special function in simulation logic uses `libm` (pure-Rust, bit-stable) — never `std::f64` intrinsics that route to platform libm; (c) no float accumulation for time or counters (integers only); (d) no FMA-vs-non-FMA codegen divergence concerns inside one pinned build (the guarantee unit). Kernel itself is near-float-free; the policy exists chiefly as the contract FA-02+ inherit. Enforcement: clippy `disallowed-methods` for `f64::sin` & co. in workspace sim crates + CI flag audit.
- **Rationale**: Pinning build + platform already yields bit-stability for compiled float ops; libm closes the one hole (platform math libraries) cheaply and buys *likely* cross-platform identity as a free bonus without promising it.
- **Alternatives considered**: software floats (`rug`/MPFR — absurd overhead for a game); fixed-point everywhere (kills FA-02 ergonomics; unnecessary given per-build scope); "just std" (breaks reproducibility across OS math libraries for the *same* source, muddying even per-platform claims when toolchains update libm).

## R8 — Deterministic collections & lint enforcement

- **Decision**: Sim logic uses `Vec`, `slotmap`, `BTreeMap`/`BTreeSet` only. `HashMap`/`HashSet` are banned from `sojourn-core` via `clippy.toml` `disallowed-types` (allowed in harness CLI plumbing where order can't leak into results). `std::time::{SystemTime,Instant}`, `rand::thread_rng`, `std::env` reads are `disallowed-methods`/`disallowed-types` in core. CI runs `cargo clippy -- -D warnings` with this config; plus a dependency audit (`cargo tree`) asserting no Tauri/webview/render crates and licence check (`cargo deny`) for MIT/Apache compatibility.
- **Rationale**: Turns the spec's "forbidden constructs" (FR-CORE-202/205, plan constraints) from review discipline into mechanical gates — the same philosophy as the determinism tests, applied at compile time.
- **Alternatives considered**: custom lint crate (overkill); `IndexMap` as a hash-map compromise (insertion-ordered and fine in principle, but BTreeMap's canonical ordering also canonicalises serialization — fewer ways to hold it wrong).

## R9 — Parallelism stance

- **Decision**: The simulation step is single-threaded in this slice. The module contract documents the only future parallelism escape hatch: kernel-orchestrated parallel execution of modules proven independent by their declared ownership/views, with deterministic join order — to be considered only when a content slice demonstrably needs it and only behind the same double-run gates. No `rayon`/threads in sim logic now.
- **Rationale**: SC-006's synthetic load is comfortably single-thread territory; determinism risk from premature parallelism vastly outweighs unneeded speed. The declared-ownership contract was designed so independence is *provable* later rather than guessed.
- **Alternatives considered**: rayon-parallel module execution now (no need, real hazard); sharded worlds (a different game's problem).

## R10 — Journal: framing, durability, recovery

- **Decision**: Append-only journal file per run: length-prefixed frames (header frame: seed, config, content-data version, build id; then command/event/checkpoint-marker frames), each frame BLAKE3-checksummed; group-flush with `fsync` on (a) every command, (b) every interrupt-raising event, (c) a wall-clock-bounded background cadence ≤ 5 s (wall-clock used *only* in the persistence layer, outside sim logic — documented exception). Recovery: load newest valid checkpoint/autosave, replay journal tail to the last intact frame, report truncated remainder precisely (FR-CORE-303, SC-004). Atomic save writes: write-temp + checksum + rename; previous save never destroyed until the new one is durable (FR-CORE-604).
- **Rationale**: Standard WAL discipline sized for a game: command-rate is human-scale so per-command fsync is cheap; the 5 s bound matches SC-004. Frame checksums make torn tails detectable (kill-test gate).
- **Alternatives considered**: SQLite as journal store (robust but a heavyweight dependency for a linear log; harder to hand-inspect/ship in bug reports); journaling every tick (pointless I/O — ticks are derived from seed+commands; only *inputs* and checkpoints need durability).

## R11 — Harness, scenario scripts & CI

- **Decision**: `sojourn-harness` = clap CLI + library. Scenario scripts in RON (serde): seed, run config, command script (tick-stamped commands incl. watch registrations and pause-policy changes), checkpoint ticks, expected hashes (optional golden values). Subcommands: `run`, `verify` (double-run with deliberately varied stepping patterns per FR-CORE-204), `roundtrip`, `replay`, `killtest` (spawns child process, SIGKILL/TerminateProcess at random points, verifies recovery), `bench`, `validate-data`, `mutate` (builds with `cfg`-gated injected nondeterminism — unseeded draw, wall-clock read, hash-order iteration — asserting the gate FAILS, per FR-CORE-705/SC-009). CI: GitHub Actions matrix (Windows + Linux), jobs: fmt → clippy(determinism config) → cargo-deny (licences/deps) → unit+integration tests → determinism suite → benches with budget assertions.
- **Rationale**: RON reads naturally for Rust enums (commands) and diff-friendly review; the mutation subcommand proves the harness has teeth rather than asserting it; per-OS CI legs verify the per-platform guarantee on each platform independently (never comparing across them).
- **Alternatives considered**: JSON scenarios (fine, kept as export option; RON wins on enum ergonomics); `cargo-mutants` for mutation testing (general-purpose, slower; targeted nondeterminism injections express FR-CORE-705 exactly); proptest for fuzzing command sequences (worthwhile — adopted as a stretch test, seeds fixed for reproducibility).

## R12 — Public API boundary shape (Tauri/IPC seam)

- **Decision**: A single `SimCore` handle with a command/query surface using only owned, `serde`-serializable DTOs (no lifetimes, no callbacks crossing the boundary; interrupt/event delivery via polled queries — fits the "queries between step calls" clarification). Boundary operations defined once in `contracts/core-api.md`; the same types derive `Serialize`/`Deserialize` so an IPC bridge is a transport detail, not a redesign. No async in the core (the host owns pacing/threads; FR-CORE-105).
- **Rationale**: Polling + owned DTOs is the only shape that is simultaneously ergonomic in-process, trivially IPC-able, deterministic (no reentrancy), and consistent with tick-boundary query semantics.
- **Alternatives considered**: trait-object observer callbacks (reentrancy + ordering hazards, hostile to IPC); async streams (drags an executor into a deliberately synchronous kernel); gRPC/protobuf contract now (premature — serde DTOs preserve the option).

## R13 — Kernel data files & validation hooks

- **Decision**: Kernel-owned data files (RON): event-class registry (ids, default pause policies), watch-condition template catalogue (template id, parameter schema, referenced view fields), kernel config (base tick, cadence defaults, autosave/flush cadences, hash cadence). Loaded through a `DataSet` abstraction stamped with a `DataVersionId` (BLAKE3 of canonical content) — the id saves pin (FR-CORE-505/601). Validation: strict serde (`deny_unknown_fields`) + semantic checks + `sojourn-harness validate-data` in CI. `source` field present on any plausibility-bearing value (none expected in kernel data; field supported so content slices inherit the pattern and CI check).
- **Rationale**: Establishes the Principle V pipeline at minimal scope: content-version pinning needs real identity now; later slices add their own data domains under the same `DataSet`/validation machinery.
- **Alternatives considered**: JSON Schema validation (heavier; strict serde + checks cover a closed internal format); embedding kernel config in code (violates Principle V and FR-CORE-505).
