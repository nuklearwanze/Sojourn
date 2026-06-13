# Contract: Designer Engines → FA-02 (the propulsion binding, FR-VD-302)

How a designer-built vehicle's propulsion reaches the FA-02 propagator, settling the mechanism the
FA-02 propulsion-interface contract deferred to this slice. Reflects the clarified split (Q1:A):
**FA-04 owns design-time state; FA-02 owns flight-time craft state.**

## The binding: inline engine parameters at spawn (additive astro change)

`sojourn-astro`'s `AstroCommand::SpawnCraft` is extended (additively) to accept the engine's
**parameters inline** (the `PropulsionEndpoint`/`EngineDef` shape) as an alternative to a
catalogue-id reference:

- **Today (FA-02 fixture path)**: `engine: String` resolves against the data/astro `Engines`
  catalogue. **Unchanged** — all FA-02 gates stay green.
- **FA-04 path**: the spawn carries the designer-derived `EngineDef` (exhaust velocity, max thrust,
  throttle, rated power, `power_limited`, masses, boil-off) **inline**; FA-02 stores it on the
  `Craft` and flies it exactly as a fixture engine. Empty/unused ⇒ identical behaviour to today.

The host (or a later mission slice) spawns a craft from an FA-04 design by:
1. querying FA-04 `engine_defs(faction, design)` + dry mass + propellant capacity (the design-query
   surface);
2. issuing FA-02 `SpawnCraft` with those parameters inline.

## Why inline (not a runtime engine-catalogue view)

The kernel view system is scalar-only (FA-03 R5), so a published `Vec<EngineDef>` view can't carry
the rich shape; carrying the params inline at spawn keeps the engine snapshot **with the craft that
flies it**, preserves single-writer (FA-02 owns the craft + its engine), and needs no cross-module
view. Boil-off and time-dependent propellant losses are applied by FA-02 during propagation from the
inline rate; the propagator reads the resulting `propellant()` (the FA-02 contract).

## Obligations preserved

- The FA-02 physical couplings are unchanged: mass flow `T·dt/v_e`, power-limited EP, throttle
  clamping, the rocket equation — all per the propulsion-interface contract.
- All designer engine parameters are SI and sourced (Principle I); the analytic gates
  (`component-data.md`) cover the rocket-equation/T-W/power-limited identities.
- `consume` remains FA-02's craft mutation; FA-04 never mutates live craft mass.
