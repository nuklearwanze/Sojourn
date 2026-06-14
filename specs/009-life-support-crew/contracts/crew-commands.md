# Contract: Crew Commands & Events (FA-08)

Commands routed through the kernel as `Command::ModulePayload { module: "crew", kind, payload }` (the
FA-03…07 pattern), applied in `CrewModule::on_command`. The **daily seeded step** advances the dynamic
state; cross-slice physics arrives **inside command payloads / the query inputs** as composed values.
Events are emitted via the data registry. Implements R4/R9/R10/R12/R14.

## Commands (`CrewCommand`)

| Kind | Payload (abridged) | Effect |
|---|---|---|
| `OccupyAsset` | `{ faction, asset, crew: Vec<AstronautId>, mission, consumables_kg }` | create a crewed asset, seat the crew, start consumption/dose/decon/psych accrual |
| `AssignCrew` / `Vacate` | `{ asset, astronaut }` / `{ asset }` | add/remove a crew member; vacate ends occupancy |
| `Maintain` | `{ asset, crew_hr, spares_kg }` | apply ECLSS maintenance (lowers the failure hazard, FR-LSC-502) |
| `Shelter` | `{ asset, sheltering: bool }` | crew shelters during an SPE (mitigates acute dose, FR-LSC-202) |
| `Resupply` | `{ asset, kg }` | replenish consumables |
| `EvaluateEdl` | `{ asset, edl_suitability, body }` | roll the EDL crew-risk on `crew/edl-risk`; failure ⇒ loss-of-crew (FR-LSC-601) |

- **Validation**: ids resolve; amounts ≥ 0. A command that cannot complete returns a `CommandOutcome`
  carrying the violated constraint (no silent partial apply).
- **Determinism**: every stochastic effect draws from a **named seeded stream** (`crew/spe-storm`,
  `crew/eclss-failure`, `crew/edl-risk`, `crew/anomaly`) via `ctx.rng(path)`; no wall-clock.

## Step cadence (R4/R14)

- `cadence_ticks = 86_400` (daily). Each `step`, per crewed asset: deplete consumables; accrue per-member
  GCR dose (× shield attenuation); roll an SPE storm (`crew/spe-storm`, shelter-mitigated); accrue
  deconditioning (artificial-g strongest) + psych load; degrade ECLSS + roll failure (`crew/eclss-failure`,
  multiplicative hazard); roll anomalies (`crew/anomaly`); recompute REID + capability; check viability /
  loss-of-crew thresholds and emit events. EDL is command-driven (`EvaluateEdl`).

## Events (data registry — `data/kernel/event-classes.ron`)

| Event | Class | When |
|---|---|---|
| `spe-storm` | Interrupt | a solar-particle-event storm hits a crewed asset |
| `eclss-failure` | Interrupt | an ECLSS unit fails (critical beyond abort ⇒ loss-of-crew risk) |
| `crew-anomaly` | LogOnly | a psych/ops-driven crew anomaly |
| `astronaut-grounded` | Interrupt | an astronaut reaches the 3% REID limit |
| `loss-of-crew` | Interrupt | crew member(s) lost (consumables/ECLSS/EDL/dose) — consumed by FA-09 |

Interrupt-class events feed the FA-01 interrupt-and-pause loop ("stop on something that matters").
**No kernel change** — events are data registry entries. The political/prestige/flight-freeze fallout of
`loss-of-crew` is **FA-09's** (this slice emits the event + physical consequence only).
