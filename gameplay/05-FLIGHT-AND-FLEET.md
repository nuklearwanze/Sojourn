# GP-04 — Flight & Fleet (FA-15)

**Spec dir:** `specs/016-flight-fleet` · **Depends on:** GP-01 (launch), GP-03 (a produced design) · **Speckit:** `speckit/gameplay/gp-04-flight-fleet.md`

The increment that makes things move — and the one that completes a **minimal end-to-end playable game** (fund → research → design → launch → fly). A booked launch of a produced design becomes a real craft; the player plans transfers with porkchop plots and manoeuvre nodes, commits burns behind a gate, and the core interrupts at each node. It introduces the durable **`sojourn-mission`** module that threads launch → craft → plan → arrival as one trackable Mission.

## Goal & player-facing capability

Launch a produced design (consuming the GP-01 booking) so a craft appears at its start location; open the Trajectory Planner; solve a transfer window (porkchop), lay manoeuvre nodes, read the Δv ladder vs the craft's available Δv, and **commit the plan** (gated, auto-pause at each node); plan low-thrust spiral arcs for electric craft; watch the live fleet on S5 and the craft on the System Map; get **interrupted at the node** to confirm execution; arrive.

## Orchestration intents → command fan-out

In `sojourn-game`:
- `Intent::LaunchDesign { design, orbit_class }` → consumes the booked launch (validate against GP-01 capacity) then `AstroCommand::SpawnCraft` with the design's dry mass / propellant / engine / power; **creates a Mission** via the new `sojourn-mission` module. Gated (preview = which booking is consumed, the resulting craft, start state).
- `Intent::PlanTransfer { craft, departure, arrival }` → `AstroCommand::CreateNode`(s) from the porkchop solution; reversible drafting.
- `Intent::CommitBurn { craft, nodes, aim }` → `AstroCommand::CommitPlan`; **gated** (preview = Δv consumed vs available, propellant after, TOF, arrival state, what becomes unrecoverable — the burn). Auto-registers a watch so the core interrupts at the node.
- `Intent::SetThrottle/SetGuidanceArc` for low-thrust → matching `AstroCommand`.

Porkchop/window/Δv come from the astro planners; the commit preview is core-computed.

## New durable module: `sojourn-mission`

A `SimModule` registered in the core. Owns **Mission** records (id, faction, vehicle/craft, current stage: queued-launch → ascent → transfer → operations → return → done, the linked plan, the science/delivery goal once GP-05/06 attach one) and **standing orders**. Advances missions deterministically with the clock, emits mission events (stage transitions, arrivals) into the event store, and is saved/migrated by the kernel. Depends only on `sojourn-core`; consumes craft/economy facts as composed values (the same decoupling pattern as the slices). This keeps determinism/replay/round-trip intact for the loop's spine.

## Cross-system causality & state touched

A launch booked in economy (GP-01) of a design produced in vehicle (GP-03) becomes an astro craft tracked by a mission. Arrival at a target enables survey (GP-05) and delivery (GP-06); crewed missions add the crew layer (GP-07); a successful first-of-its-kind arrival is an achievement (GP-08). State: astro slice (craft/nodes) + new mission module (threads), both journalled.

## ESA data

Reuses `data/astro/*` (engines, config: step tiers/thresholds/diversion budget/error model, validation analytic cases) and `data/world` ephemerides. New: `data/mission/*` for stage definitions/params (sourced where they carry numbers; mostly structural).

## UI/UX — S2 Trajectory Planner + S1 System Map + S5 Operations (now live)

S2 subscreens:
- **Porkchop / window solver** — the porkchop field widget; click to pick departure/arrival; shows Δv/C3/TOF at the cursor and the optimum.
- **Manoeuvre-node editor** — list/edit nodes (epoch + Δv in PRN), reconciled against the authoritative propagation; preview the resulting trajectory.
- **Low-thrust arc planner** — throttle/guidance for electric craft; shows the spiral and thrusting duration.
- **Δv budget** — the Δv ladder: required (per leg) vs available (selected craft), with margin.
- Commit-burn gate panel.

S1 System Map goes **live**: real craft positions, the planned trajectory overlay, transport-graph and (GP-05) resource layers; the reticle inspector shows the selected craft's state vector.

S5 Operations: **Asset register** (virtualised fleet table: asset, class, location, task, propellant, health, status); **Craft inspector** (state vector, Δv remaining, propellant, comms lag, mission link); **Missions** subscreen (mission threads with their stage timeline); **Standing orders**.

Plan→preview→commit verbs: Launch design (Launch kind), Commit burn (Burn kind). Drafting nodes is reversible. Empty state: no fleet → "No craft in flight — launch a produced design from Operations or Economy."

View-model: `PlannerVM` (porkchop + ladder) extended with the node editor and low-thrust arc; `OperationsVM`/`CraftRow` extended with mission link; a `MissionVM` for the threads. Unit-test the ladder math display, node reconciliation shaping, and mission-stage shaping. Renderer wires the launch and burn gates and the auto-pause-at-node.

## Testability

Harness `flight_play.ron`: boot ESA → (GP-01/03 prereqs) produce + book a launch → launch the electric tug (assert booking consumed, craft spawned in LEO, mission created at stage=transfer-pending) → solve a transfer, lay a node, commit the burn (assert Δv consumed ≤ available, watch armed) → advance → assert interrupt fires at the node, executes, craft proceeds, mission stage advances → arrival event. Validate the propagator still matches the analytic cases (Hohmann Δv, two-body period). Determinism double-run + save round-trip (including the mission module). Human: launch, plan a transfer on the porkchop, commit, warp to the node interrupt, watch it execute on the map.

## Acceptance criteria

A booked+produced design launches into a tracked craft; transfers are planned via porkchop+nodes and committed behind a core-computed gate; the core interrupts at nodes; low-thrust arcs work; the fleet and map are live; `sojourn-mission` is a journalled, saved, deterministic module; analytic astro validation still passes; renderer holds no astrodynamics.

## Out of scope

Why you fly there (survey GP-05 / delivery GP-06). Crew aboard (GP-07). Claiming the first (GP-08). Aerocapture/EDL execution detail (GP-07 EDL eval; planning surfaces here).
