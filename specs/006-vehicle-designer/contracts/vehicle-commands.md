# Contract: Vehicle Commands + Events (FR-VD-802)

Design-library mutations are **journaled commands** routed through FA-02's `Command::ModulePayload
{module: "vehicle", kind, payload}` → `SimModule::on_command` — **no kernel change**. Payloads are
postcard DTOs. Deterministic; replay reproduces them. The slice holds only design-time state
(R2/R3); flight-time craft state stays in FA-02.

## Commands (`VehicleCommand`)

| Command | Effect |
|---|---|
| `ComposeDesign { faction, class, stages, redundancy, mission_reqs }` | creates a versioned design. **Validation = trust-the-caller** (R4): structural only (components/techs exist, staging well-formed, redundancy references real components); component *availability/maturity* gating is the query surface's + host's job. |
| `EditDesign { faction, design, … }` | edits a design into a **new version**; existing derivatives are not mutated (FR-VEH-007 edge case). |
| `SaveDesign { faction, design, name }` | names/promotes a design as a saved class/template. |
| `DeriveDesign { faction, parent, … }` | creates a derivative referencing `parent`'s lineage (heritage discounts apply via the parent's techs in FA-05). |
| `RegisterProduction { faction, design, units }` | increments the design's cumulative production count (learning curve, R11) and emits `vehicle-produced` (the host turns operational use into FA-05 `RegisterHeritage`). |

Malformed payloads and structurally-invalid commands are deterministic `Rejected`, never panics.

## Events (data-registry addition to `data/kernel/event-classes.ron`)

| Class | Policy | Emitted when |
|---|---|---|
| `vehicle-produced` | `LogOnly` | a `RegisterProduction` applies (carries design + units; the host bridges to FA-05 heritage) |

No kernel code — a data-registry addition. Heritage itself lives in FA-05 (its `RegisterHeritage`);
FA-04 reads it back through `maturity()`.

## Streams

None — the designer's derivations are **deterministic pure computations** with no randomness; the
slice has no seeded behaviour. (This is the first game-system module to declare zero streams.)
