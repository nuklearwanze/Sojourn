# GP-04 — Flight & Fleet · `/speckit` set (FA-15)

**Branch:** `016-flight-fleet` · **Design:** `gameplay/05-FLIGHT-AND-FLEET.md` · **Depends:** GP-01, GP-03

## /speckit.specify

```
/speckit.specify Make things move and complete a minimal end-to-end playable game: launch a produced design into a real craft, plan transfers, commit burns behind a gate, and track it as a Mission. Make the Trajectory Planner (S2) and Operations/Fleet (S5) interactive and the System Map (S1) live. Authoritative design: gameplay/05-FLIGHT-AND-FLEET.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles II, III, IV) — read them.

WHY: after this increment the player can fund → research → design → launch → fly, the first complete loop. It also introduces the durable Mission thread that ties the systems together.

Let the player launch a produced design (consuming the GP-01 launch booking) so a craft appears at its real start state, creating a Mission; solve a transfer window on a porkchop plot; lay manoeuvre nodes and read a Δv ladder of required-per-leg versus the craft's available Δv; COMMIT THE BURN behind a plan→preview→commit gate (preview: Δv consumed vs available, propellant after, time-of-flight, arrival state, and that the burn is irreversible) which arms a watch so the core interrupts and pauses at the node; plan low-thrust spiral arcs for electric craft (throttle + guidance); watch the live fleet (virtualised asset register + per-craft inspector with state vector, propellant, comms lag, mission link) and the craft on the live System Map with its planned-trajectory overlay; and get interrupted at each node to confirm execution, then arrive.

Introduce a new durable simulation module (a journalled SimModule registered in the core like a slice) that owns Mission records (vehicle/craft, stage: queued-launch → ascent → transfer → operations → return → done, the linked plan, and a science/delivery goal once later increments attach one) and standing orders; it advances missions deterministically, emits mission events, and is saved/migrated by the kernel. It depends only on the core and consumes other slices' outputs as composed values (the same decoupling pattern as the existing slices). The stateless orchestration crate composes launch and burn intents; this module stores the threads.

The astrodynamics derivation stays in the astro slice — the UI displays and previews, never recomputes — and the analytic validation cases (Hohmann Δv, two-body period) must still pass.

Acceptance: a booked+produced design launches into a tracked craft; transfers are planned via porkchop+nodes and committed behind a core-computed gate; the core interrupts at nodes; low-thrust arcs work; the fleet and map are live; the new mission module is journalled, saved and deterministic; analytic astro validation still passes; renderer holds no astrodynamics. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- New module name and boundary (proposed `sojourn-mission`): exact Mission record shape, stage machine, and what is journalled vs derived.
- How a booked launch (GP-01 economy capacity) is matched/consumed at launch, and how the design's mass/propellant/engine/power flow into `AstroCommand::SpawnCraft`.
- Node-editor reconciliation against the authoritative propagation, and how the porkchop solution seeds nodes.
- Auto-pause-at-node via the kernel watch/interrupt mechanism.

## /speckit.plan — guidance

- Create `sojourn-mission` (lib, a `SimModule`) and register it in `build_modules`/`new_game_esa`. `sojourn-game` intents `LaunchDesign` (consume booking → `AstroCommand::SpawnCraft` → create Mission; gated), `PlanTransfer` (`CreateNode`; reversible), `CommitBurn` (`CommitPlan`; gated; arms watch), `SetThrottle`/`SetGuidanceArc`.
- View-model: extend `PlannerVM` (porkchop + ladder + node editor + low-thrust arc), `OperationsVM`/`CraftRow` (mission link), add `MissionVM`; make the map view-model live. Renderer: S2 subscreens (Porkchop solver, Node editor, Low-thrust arc, Δv budget), S5 subscreens (Asset register, Craft inspector, Missions, Standing orders), live S1; wire the launch and burn gates and auto-pause.
- Tests: harness `flight_play.ron` (produce+book → launch consumes booking + spawns craft + creates mission; plan+commit burn ≤ available Δv arms watch; advance → interrupt fires at node, executes, mission stage advances; arrival event); re-assert Hohmann/two-body analytic cases; determinism double-run + save round-trip including the mission module; view-model tests.

## /speckit.tasks & /speckit.analyze — notes

Separate the new module, intents/previews, view-model builders, S2/S5/S1 renderer, tests. `/speckit.analyze` must confirm: the new module depends only on `sojourn-core` and the `sojourn-core` audit stays green (Principle IV); all durable mission state is journalled and round-trips (Principle III); astro analytic validation still passes (Principle II); renderer holds no astrodynamics (Principle IV).
