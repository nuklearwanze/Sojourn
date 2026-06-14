# Contract: Base Commands & Events (FA-07)

Commands routed through the kernel as `Command::ModulePayload { module: "base", kind, payload }` (the
FA-03…06 pattern), applied in `BaseModule::on_command`. Trust-the-caller for structural validity;
cross-slice physics arrives **inside the payload** as composed values (the plan→preview→commit pattern).
Events are emitted via the data registry. Implements R8/R9/R10/R14.

## Commands (`BaseCommand`)

| Kind | Payload (abridged) | Effect |
|---|---|---|
| `FoundBase` | `{ faction, site, class }` | create a base at a Site / dynamical location |
| `AddModule` | `{ base, module_type }` | add a planned module (gated by composed `TechMaturity` at query time) |
| `OpenConstruction` | `{ base }` | open the construction project; derive per-module mass + crew-time demands |
| `DeliverToBase` | `{ base, module, mass_kg, crew_time_hr }` | accrue delivery (the host bridges FA-06 delivery); **commission** the module when its demand is met |
| `BuildLocal` | `{ base, module, local_mass_kg }` | satisfy a module's mass with on-site regolith construction (marks `built_local`; reduces import demand) |
| `RecordIsruHost` | `{ base, module, process }` | register that the base hosts an FA-06 ISRU process (output composed in) |
| `EvaluateEmbargo` | `{ base, years }` | record an embargo-test result + emit `embargo-result` (and a `settlement-milestone` on first survival) |
| `DecommissionBase` | `{ base }` | retire a base |

- **Validation**: ids resolve (`site`/`class`/`module_type`); amounts ≥ 0. A module commissions only
  when `delivered_mass ≥ required_mass` **and** `crew_time ≥ required_crew_time` (R8). A command that
  cannot complete returns a `CommandOutcome` carrying the violated constraint (no silent partial apply).
- **Determinism**: no stochastic effects (zero streams, R14); commissioning is delivery-driven; no
  wall-clock.

## Step cadence (R14)

- `cadence_ticks = 86_400` (daily). Each `step`: advance time-based construction bookkeeping (elapsed
  schedule), finalize any module whose accrued delivery+crew-time crossed its threshold, and emit
  `module-commissioned` / `base-operational` as bases complete. Commissioning *inputs* arrive via
  `DeliverToBase`/`BuildLocal` commands.

## Events (data registry — `data/kernel/event-classes.ron`)

| Event | Class | When |
|---|---|---|
| `module-commissioned` | LogOnly | a module becomes operational |
| `base-operational` | LogOnly | all of a base's modules are commissioned |
| `pp-violation` | Interrupt | siting/operating violates a planetary-protection rule |
| `embargo-result` | LogOnly | an embargo stress test is evaluated |
| `settlement-milestone` | Interrupt | a base first reaches a settlement milestone (e.g., embargo-survivor) |

Interrupt-class events feed the FA-01 interrupt-and-pause loop ("stop on something that matters").
**No kernel change** — events are data registry entries.
