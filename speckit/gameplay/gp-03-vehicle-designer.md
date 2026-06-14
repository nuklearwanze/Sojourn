# GP-03 — Vehicle Designer · `/speckit` set (FA-14)

**Branch:** `015-vehicle-designer` · **Design:** `gameplay/04-VEHICLE-DESIGNER.md` · **Depends:** GP-02, GP-01

## /speckit.specify

```
/speckit.specify Make the Vehicle Designer (S4) a fully interactive screen that turns matured technology into hardware with honest, fully-traced performance. Authoritative design: gameplay/04-VEHICLE-DESIGNER.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles II, VII, VIII, V) — read them.

WHY: the designs built here are what GP-04 launches and flies. The tyranny of mass and Δv must be felt at design time, with every figure traceable.

Give the player a component palette gated by researched tech maturity (from GP-02); a stage composer (stages + propellant loads + redundancy blocks + mission requirements) with a live mass roll-up; a derived-performance panel where the core computes mass, Δv (rocket equation), thrust, T/W, power demand vs generation, thermal/radiator margin, reliability and cost — every number carrying a trace affordance down to its sourced inputs; realism red-flags (negative margins, impossible T/W, radiator shortfall, low-thrust-only acceleration) shown as plain-language tags; edit and derive-variant flows; a compare view; and a register-production step behind a plan→preview→commit gate (preview: unit cost P50 with the learning-curve note, total funds Δ, facility-capacity Δ, schedule). Compose/edit/derive are reversible with live core-computed preview; only register-production is irreversible.

All performance derivation stays in the vehicle slice — the UI displays, never recomputes. Intent→command expansion and the production preview live in the stateless orchestration crate.

Acceptance: the palette is maturity-gated (ungated components visible-but-locked with a pointer to the needed tech); all performance figures are core-derived and traceable; realism flags surface; compose/edit/derive are reversible-with-live-preview; register-production is gated and draws funds + facility capacity; numbers sourced; renderer holds no derivation. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- How component gating reads the GP-02 maturity surface (`maturity()`/`heritage()`/`understanding()`), and the locked-component messaging.
- Whether the live derived report on compose is a non-committing preview path or a transient design evaluation, and how it stays core-computed.
- Mission-requirements shape (target Δv, payload, environment) and how it drives flags.
- Production economics: learning-curve unit-cost decline, facility-capacity model.

## /speckit.plan — guidance

- `sojourn-game` intents `ComposeDesign`/`EditDesign`/`DeriveDesign` (reversible, live core-computed report) and `RegisterProduction` (gated) expanding to the real `VehicleCommand` variants.
- View-model: extend the vehicle report into catalogue + composer + performance(+trace) + compare + production builders; reuse the trace render and Δv-related widgets. Renderer: S4 subscreens (Component catalogue, Stage composer, Derived performance & trace, Realism flags, Compare, Production queue) wired to the production gate.
- Tests: harness `vehicle_play.ron` (compose the Castor electric tug → derived Δv ≈ 5.62 km/s by rocket equation, low-thrust-only flag, power within bounds, reliability present; register 1 unit → funds + facility capacity drawn; an ungated NTP component is locked); determinism + round-trip; view-model tests for gating, trace pass-through, flag surfacing.

## /speckit.tasks & /speckit.analyze — notes

Separate intents/preview, view-model builders, S4 renderer, tests. `/speckit.analyze` must confirm: physics-authoritative display only (Principle II — no UI recomputation), mass/Δv realism flags present (Principle VII), traceable figures (Principle VIII), sourced component/propulsion params (Principle V), thin renderer (Principle IV), core audit green.
