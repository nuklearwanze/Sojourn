# Contract: Persistence Formats — Save Container & Journal (FA-01)

Internal programme contract (not a public modding format in v1.0). Governs FR-CORE-301…305,
FR-CORE-601…604; SC-002/003/004. Encoding: `postcard` (canonical, R4); hashes/checksums: BLAKE3.

## 1. Save container

```text
┌──────────────────────────────────────────────────────────┐
│ MAGIC "SJRN"                u32                          │
│ save_format_version         u32   (structure version)   │
│ build_id                    str   (toolchain+crates)     │
│ data_version                [32]  (pinned DataVersionId) │
│ run_id                      u64                          │
│ tick                        u64                          │
│ payload_len                 u64                          │
│ payload_checksum            [32]  (BLAKE3 of payload)    │
├──────────────────────────────────────────────────────────┤
│ payload = postcard(WorldState)  — complete: all module   │
│ slices, RNG stream states, event queue, pending          │
│ interrupts & commands, watch states, pause policies,     │
│ event-store index (history tiers referenced by run dir)  │
└──────────────────────────────────────────────────────────┘
```

**Rules**
- **Completeness** (FR-CORE-601): loading yields bit-identical `WorldState`; `fingerprint(loaded)
  == fingerprint(saved)`; subsequent evolution identical to an unbroken run (SC-002), including
  saves taken mid-fine-timestep and with pending interrupts.
- **Atomicity** (FR-CORE-604): write temp file → fsync → rename; the previous save survives any
  crash during writing.
- **Pinning** (FR-CORE-602 + umbrella clarification): `data_version` identifies the exact content
  DataSet; load resolves it via `DataResolver`; absence ⇒ `DataVersionUnavailable` with hint —
  never silent substitution.
- **Migration**: keyed by `save_format_version`; migrations transform structure only, run in
  sequence (v1→v2→…), are pure and tested with golden fixture saves per released version.
  Migration never touches pinned content values.
- **Integrity** (FR-CORE-305): checksum verified on load. Ironman: verification failure ⇒ refuse
  with explanation. Save-anywhere: refuse by default (corrupted ≠ loadable).

## 2. Journal (append-only, the replay determinant)

```text
file = HeaderFrame, Frame*          one journal per run, in the run directory
Frame = │ len u32 │ kind u8 │ seq u64 │ tick u64 │ body … │ checksum [32] │

kinds:
  0 HEADER     { run_id, master_seed, run_config, data_version, build_id, journal_format_version }
  1 COMMAND    { CommandEnvelope }                  ← replay input
  2 OUTCOME    { CommandId, CommandOutcome }        ← verification material
  3 EVENT      { EventRecord }                      ← verification material
  4 CHECKPOINT { tick, save_file_ref, fingerprint } ← recovery acceleration
  5 HASHMARK   { tick, fingerprint }                ← divergence diagnosis anchors
```

**Rules**
- **Replay determinant** = HEADER + ordered COMMAND frames (+ pinned data + same build): replays
  to bit-identical state at any tick (SC-003). EVENT/OUTCOME/HASHMARK frames are *verification*
  material: `replay --verify` compares them and reports the first divergence (SC-009).
- **Not journaled, by design**: warp-rate selections and host pacing (FR-CORE-204 — warp is pure
  playback speed); queries (read-only); derived events are *re-generated* by replay and checked
  against recorded frames, not consumed as inputs.
- **Durability** (FR-CORE-303): group-fsync on every COMMAND, every interrupt-raising EVENT, and
  a background cadence bounding loss to ≤ 5 s wall-clock (SC-004). The persistence layer's
  wall-clock timer is the single documented exception to the no-wall-clock rule and sits outside
  sim logic.
- **Tail corruption**: frames self-validate (len + checksum); recovery = newest CHECKPOINT save +
  replay of subsequent intact COMMAND frames; the truncated remainder is reported precisely
  (frame seq + tick range lost) — never a silent partial load.
- **Ironman** (FR-CORE-603/305): journal + rolling autosave are the only persistence; frames are
  additionally chained (each checksum covers the previous frame's checksum) so editing or
  truncating *interior* history is detectable, refusing load with an explanation.
- **Append-only**: frames are never rewritten; `seq` is strictly monotonic; a gap or reorder is
  corruption by definition.

## 3. Autosave & checkpoint policy

- Autosave (full save container) at: every interrupt-and-pause, horizon, configurable sim-time
  cadence, and clean exit. CHECKPOINT frames reference autosaves to bound recovery replay time.
- Cadences are kernel config DATA (R13), not code constants (Principle V).
- Retention: ironman keeps the rolling latest (+1 backup generation for media-failure safety,
  not player choice); save-anywhere keeps whatever the player/host names. Recovery uses the
  newest *valid* checkpoint in either mode.

## 4. Scenario scripts (harness interchange, RON)

Human-readable RON mirroring `ScenarioScript` (data-model §10): seed, config, tick-stamped
commands, checkpoints, optional golden fingerprints, optional synthetic load profile. Scripts are
test fixtures and bug-report vehicles; they are NOT a save format and carry no world state.

## 5. Compatibility promises (v1.x)

| Artifact | Promise |
|---|---|
| Save container | forward-migratable across all v1.x `save_format_version`s (structure only) |
| Journal | replayable on the **originating build** (+ pinned data); later builds may refuse gracefully with `BuildMismatch` detail — never wrong-answer replay (clarified 2026-06-12) |
| Scenario scripts | best-effort stable; harness versions them independently |
| Fingerprints | comparable only within (platform, build) — per umbrella clarification |
