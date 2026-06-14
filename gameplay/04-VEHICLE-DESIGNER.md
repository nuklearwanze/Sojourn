# GP-03 — Vehicle Designer (FA-14)

**Spec dir:** `specs/015-vehicle-designer` · **Depends on:** GP-02 (researched maturity), GP-01 (cost/production funds) · **Speckit:** `speckit/gameplay/gp-03-vehicle-designer.md`

Turns matured technology into hardware. The player composes spacecraft, launchers, landers and tugs from researched components and gets honest, fully-traced performance — the tyranny of mass and Δv felt at design time — then registers production. The designs built here are what GP-04 launches and flies.

## Goal & player-facing capability

Open a maturity-gated component palette; compose a design from stages + redundancy blocks + mission requirements; see the core compute mass, Δv (rocket equation), thrust, T/W, power, thermal/radiator balance, reliability and cost — every number traceable to its inputs; see realism **red flags** (negative margins, impossible T/W, radiator shortfall, low-thrust-only acceleration); edit, derive variants, compare designs; register a production run (drawing funds + facility capacity).

## Orchestration intents → command fan-out

In `sojourn-game`:
- `Intent::ComposeDesign { class, stages, redundancy, mission }` → `VehicleCommand::ComposeDesign`. The palette only offers components whose tech maturity (from GP-02) clears their gate; composing is reversible until registered, so it is a `Direct`/preview-as-you-type flow (the derived report is core-computed live).
- `Intent::EditDesign` / `Intent::DeriveDesign` → the matching `VehicleCommand`.
- `Intent::RegisterProduction { design, units }` → `VehicleCommand::RegisterProduction`, **gated** (preview = unit cost P50 with learning-curve note, total funds Δ, facility-capacity Δ, schedule). This is the irreversible step.

The performance report and realism flags are the vehicle slice's existing derivation (rocket equation, power/thermal, reliability block diagram, learning curve) — the UI displays, never recomputes.

## Cross-system causality & state touched

Designs consume researched maturity (GP-02); production draws funds + facility capacity (GP-01); a produced + launched (GP-01 launch) design becomes a craft in astro (GP-04) with its computed dry mass, propellant, engine, power. Reliability here feeds mission risk and (via GP-08) loss-of-crew consequences. State: vehicle slice (journalled). No new module.

## ESA data

Reuses `data/vehicle/*` (component catalogue, propulsion family models, reliability/cost/life-support/EDL/solar-distance params, class templates). Confirm component gates reference the GP-02 maturity surface and carry sources.

## UI/UX — S4 Vehicle Designer (now interactive)

Overview: the design list (ESA's saved designs) + "New design".

Subscreens:
- **Component catalogue** — palette grouped by subsystem (propulsion, power, structure, avionics, payload), each component showing its tech gate; ungated components are visible-but-locked with "needs <tech> TRL n" (a pointer back to S3).
- **Stage composer** — build stages, set propellant loads, add **redundancy blocks**, set **mission requirements** (target Δv, payload, environment); drag/duplicate; live mass roll-up.
- **Derived performance & trace** — the headline figures with the dotted "ⓘ" trace on each (e.g. Δv = Isp·g₀·ln(m₀/m_f)); power demand vs generation; thermal/radiator margin; reliability estimate; cost P50.
- **Realism flags** — the core's flags as tags (T/W, power shortfall, radiator deficit, propellant impossibility) with one-line plain-language explanations.
- **Compare** — two or more designs side by side on the same figures.
- **Production queue** — register units; queue with cost/schedule and learning-curve unit-cost decline.

Inspector: selected component or stage with its sourced parameters.

Plan→preview→commit: Register production (Build kind). Compose/edit are live-preview, reversible. Empty state: no designs → "Compose ESA's first vehicle — start from a class template."

View-model: extend the vehicle report into catalogue + composer + performance + compare + production builders; unit-test the gating (locked components), the trace pass-through, and the flag surfacing. Renderer wires the production gate.

## Testability

Harness `vehicle_play.ron`: boot ESA with electric-propulsion matured (from GP-02 script) → compose the Castor-class electric tug (4× Hall, 40 kW array, Xe tank, payload) → assert derived Δv ≈ 5.62 km/s (rocket equation), T/W flag = low-thrust-only, power within bounds, reliability present → register 1 unit → assert funds debited + facility capacity used + production queued. Try an ungated NTP component → assert it is locked. Determinism + round-trip. View-model tests. Human: build the tug, hover Δv to see the rocket equation, see the T/W flag, register production.

## Acceptance criteria

Palette is maturity-gated; all performance figures are core-derived and traceable; realism flags surface; compose/edit/derive are reversible-with-live-preview; register-production is gated and draws funds + facility capacity; numbers sourced; renderer holds no derivation.

## Out of scope

Launching/spawning the craft (GP-04). Crew sizing of the design (GP-07 consumes the vehicle's life-support params). EDL evaluation of landers (GP-07). Selling designs / IP licensing (GP-08 economy depth).
