# Contract: Astro Commands & Events (+ the kernel ModulePayload amendment)

## Kernel amendment (additive, domain-agnostic — updates FA-01 contracts)

`sojourn-core` gains one `Command` variant and one trait hook:

```rust
// command/mod.rs (kernel)
Command::ModulePayload {
    module: String,     // target module id (validated at submit: module must exist)
    kind: String,       // module-defined command kind (diagnostics/filtering)
    payload: Vec<u8>,   // module-defined, postcard-encoded command struct
}

// module/mod.rs (kernel trait, default-rejecting)
fn on_command(&self, slice: &mut dyn StateSlice, kind: &str, payload: &[u8],
              ctx: &mut StepCtx<'_>) -> CommandOutcome {
    CommandOutcome::Rejected(format!("module accepts no commands (kind '{kind}')"))
}
```

Routing: applied in step 1 of the kernel tick order like every command; the outcome is
journaled (OUTCOME frame); a malformed payload is a deterministic `Rejected`, never a crash.
The kernel never decodes `payload` — FR-CORE-505 (no domain logic in kernel) holds. The
existing `ModuleCommand{key,value}` stays for simple cases (synthetic module keeps using it).
FA-01 contract docs (`core-api.md`, `module-contract.md`) are updated by this slice's tasks.

## Astro command set (postcard-encoded `AstroCommand` in the payload)

| Kind | Payload | State validity (Rejected when…) |
|---|---|---|
| `create-node` | craft, epoch_tick, dv_prn | craft unknown/not propagating; epoch in past |
| `edit-node` / `delete-node` | node id (+ new fields) | node unknown |
| `commit-plan` | node ids, aim point | any node infeasible/invalidated |
| `set-throttle` | craft, throttle | out of endpoint throttle range |
| `set-guidance-arc` / `clear-guidance-arc` | craft, law, end condition | unknown law; craft not propagating |
| `schedule-station-keeping` / `cancel` | craft, cadence, budget | craft unknown |
| `divert-body` | body, applied impulse | not divertible; budget exceeded (config, default 16) |
| `re-rail-body` | body | body not in Diverted state |
| `set-research-gate` | gate, open | — (config input; FA-05 will own this) |
| `spawn-craft` / `despawn-craft` | definition / craft | fixtures & scenarios (and FA-04 later) |

All structurally-invalid payloads (decode failures, NaN/∞ components, negative magnitudes)
are deterministic rejections with reasons.

## Event classes added (data/kernel/event-classes.ron — data-only change)

| Class | Default pause | Raised when |
|---|---|---|
| `soi-crossing` | LogOnly | a craft/diverted body rebases to a new dominant body |
| `impact` | Interrupt | a trajectory intersects a body surface (terminal here) |
| `atmosphere-entry` | Interrupt | interface-altitude crossing outside a planned aero pass (EDL handoff) |
| `plan-invalidated` | Interrupt | committed plan infeasible after state change |
| `propellant-exhausted` | Interrupt | tank empties mid-burn/arc |
| `aero-violation` | Interrupt | flown pass exceeds corridor depth/load limit |

(`maneuver-node` already exists in the kernel registry.) Payloads carry identifiers and SI
quantities only. All entries carry `source` fields referencing the design/spec sections.

## Published views (flat scalars, kernel rules)

`astro/status`: craft_count, active_burns, diverted_count, next_node_tick, fine_needed
(the escalation binding). Per-craft detail beyond flat scalars flows through the
planning-query surface, not views. (As-built note: divergence is surfaced via
kernel-diagnostic events and the `reconcile_report` query rather than a view scalar.)

KNOWN FOLLOW-UP: watch conditions bind views only, so per-craft thresholds (e.g.
"propellant below X") are not watchable in this slice; adding bounded per-craft views +
watch templates is the designated extension point when FA-10/ops need it.
