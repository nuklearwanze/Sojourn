# Contract: Research Commands + Events + Streams (FR-RESP-802)

All research mutations are **journaled commands** routed through FA-02's `Command::ModulePayload
{module: "research", kind, payload}` → `SimModule::on_command` — **no kernel change**. Payloads are
postcard DTOs. Effects are deterministic and seeded; replay reproduces them bit-identically. Funding,
facilities and money are opaque caller inputs (R13).

## Commands (`ResearchCommand`)

| Command | Effect (validation = structural; trust-the-caller for funding/entitlement) |
|---|---|
| `SetAllocation { faction, rp_budget, de_budget, splits:[(program/domain, weight)], available_facilities:[capability] }` | sets the portfolio split + the caller-asserted funded budget and facility-capability set for the next steps (FA-06 binds real budgets/facilities later) |
| `StartProgram { faction, tech, lead? }` | opens an Engineering Program if the tech's UL floors + tech prereqs are met (else `Rejected` reporting the gate); leapfrog allowed via `UlSatisfiable` prereqs |
| `SetProgramPriority { faction, program, priority }` | reorders DE within the portfolio |
| `SetPublishPolicy { faction, domain, Publish\|Hold }` | publish raises World UL + emits `publish`; hold retains the lead |
| `Hire / Poach / Train / Retire { faction, … }` | deterministic roster transitions; poach emits a relations-cost-eligible signal; train is multi-year + facility-gated |
| `SelectAstronaut / TrainAstronaut { faction, person }` | advances the astronaut pipeline (R12) |
| `CrewFeedback { faction, astronaut, dose_delta, health_delta, psych_delta }` | the FA-08-facing career update (see crew-interface.md) |
| `InjectUnderstanding { faction, domains:[(DomainId, amount)], class, quality }` | mission UL injection (FA-04+ drive it; scenarios drive synthetic) — rejects unknown domains |
| `RegisterHeritage { faction, tech, units }` | operational-use heritage (FA-04+ drive it; scenarios synthetic) — raises reliability + derivative discount |
| `License / Partner / BuyIn { faction, counterparty, tech/domain }` | grant the documented TRL/IP/UL credit; **no money moves** (FA-06 settles) |

Malformed payloads and structurally-invalid commands are deterministic `Rejected`, never panics.

## Events (data-registry additions to `data/kernel/event-classes.ron`)

| Class | Policy | Emitted when |
|---|---|---|
| `breakthrough` | `Interrupt` | a seeded+earned breakthrough fires (carries a sourced Sojournal reference) |
| `dead-end-confirmed` | `LogOnly` | an approach is confirmed a dead end in its TRL band |
| `test-failure` | `LogOnly` (spectacular flight-test failures carry a PR-eligible payload for FA-09) | a test campaign fails |
| `program-milestone` | `LogOnly` | a program reaches a notable point |
| `trl-advance` | `LogOnly` | a program advances a TRL |
| `publish` | `LogOnly` | a faction publishes (prestige-eligible payload for FA-09) |

All via the existing data registry — no kernel code.

## Streams (declared in the manifest)

| Stream | Used by | Keying |
|---|---|---|
| `research/seed` | world creation — dead-end seeding (constructive) + breakthrough thresholds | per `(tech, TRL band)` / `(faction, domain)` |
| `research/overrun` | per-step cost/schedule realisation (P50/P80) | per `(program, step)` |
| `research/test` | test-campaign success/failure | per `(program, step, attempt)` |
| `research/breakthrough` | insight-pressure threshold resolution | per `(faction, domain)` |

Keying is order-independent and replay-stable (BLAKE3-derived sub-streams, FA-01 RNG pattern).
