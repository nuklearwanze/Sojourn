# Phase 0 Research: Life Support & Crew (FA-08)

Decisions resolving the Technical Context against Constitution v1.0.0 (esp. Principles I/II/III/VII/VIII),
the FA-01/04/05/06/07 contracts in the tree, and the spec's clarified scope. Format: **Decision /
Rationale / Alternatives rejected**. No `NEEDS CLARIFICATION` remain (spec clarified 2026-06-14: three
scope forks + three composition models — multiplicative hazard, REID dose limit, multiplicative capability).

---

## R1 — Crate topology: a dynamic crew slice above vehicle/research/economy/base, coupled only to core

**Decision.** A new crate `crates/sojourn-crew` takes a **hard dependency only on `sojourn-core`**.
Vehicle/base static sizing, the crew roster + age/sex + traits, ECLSS-tech maturity/heritage, and ops
capacity/light-time/abort-reach enter as **composed values / opaque caller inputs** assembled by the
host. Later slices (FA-09 politics) depend on `sojourn-crew` (loss-of-crew, crew state).

**Rationale.** The FA-04 C1 / FA-06 R1 finding: hard-linking every upstream crate over-couples the slice
and slows tests. The crew model needs *values* (a vehicle's closure capability, an astronaut's age/sex,
a tech's maturity, the ops load), not the upstream engines. Core-only keeps the dependency graph acyclic
with **no new crate edges** and unit-testable with stubs (confirmed Q3).

**Alternatives rejected.** (a) Depend on vehicle+research+economy+base crates — heavy coupling, repeats
the C1 mistake. (b) Put crew logic in the host — violates Principle IV.

---

## R2 — Composed-value integration seams (the five inputs)

**Decision.** Narrow input shapes the host composes and the crew model consumes:
- **AssetSizing** `{ closure_capability, shield_attenuation, population_capacity, spin_gravity: bool,
  habitat_volume_m3_per_crew, consumables_capacity_kg, crewed: bool }` — from FA-04 vehicle sizing /
  FA-07 base static state.
- **EnvFacts** `{ gcr_rate_sv_yr, body, comms_lag_s, abort_reach: bool }` — from FA-03 environment /
  FA-06 light-time + logistics abort-reach.
- **CrewRoster** `BTreeMap<AstronautId, AstronautFacts{ age, sex, traits, training }>` — from FA-05.
- **TechMaturity** `{ trl, reliability, flight_units }` — ECLSS tech, from FA-05.
- **OpsLoad** `{ oversubscription }` — from FA-06's ops pool.

Carried into **commands** at decision time (assign crew, occupy asset, evaluate EDL) and into the
**CrewSnapshot** at query time. Tests feed stubs; the harness feeds the real upstream queries.

**Rationale.** Narrow value types keep coupling explicit, serializable (IPC for the Tauri host), and
honest — the model acts on the **composed** sizing/roster, never the upstream authoritative state.
Age/sex feed the REID model (Q2); the host bridges FA-05's astronaut pipeline → the roster.

**Alternatives rejected.** (a) Pass whole upstream snapshots — leaks their surfaces. (b) Recompute
sizing/maturity in-crew — duplicates FA-04/05/07.

---

## R3 — Dynamic per-entity state is **stored** and evolved on the step (not pure derivation)

**Decision.** Unlike FA-04/07's pure query-time derivations, FA-08 **stores** the dynamic health state —
per `CrewedAsset` (consumables stock, ECLSS reliability/degradation/maintenance) and per `CrewMember`
(career dose, deconditioning indices, psych load, alive/grounded) — and **evolves it on the daily step**.
Derived figures (REID, capability, viability) remain pure functions over the stored state + composed
sizing.

**Rationale.** The slice's defining behaviour is accumulation over time (dose, deconditioning, psych,
consumables) and seeded daily events — this is genuinely stateful, like FA-05's research progress. The
stored state is small (scalars per asset/member) so saves/round-trips stay deterministic and cheap; the
*derived* layer stays pure for the query surface (R13).

**Alternatives rejected.** (a) Recompute health from a mission log each query — expensive, fragile,
can't carry seeded-event history. (b) Store derived REID/capability — stale vs roster/sizing; recompute
(R13).

---

## R4 — Daily seeded step (the FA-05 pattern)

**Decision.** `CrewModule` declares `cadence_ticks = 86_400` (daily). Each step, for every crewed asset:
deplete consumables; accrue per-member GCR dose (× shield attenuation); **roll an SPE storm** on the
`crew/spe-storm` stream (seeded arrival + magnitude, mitigated by sheltering); accrue deconditioning
(× (1 − countermeasure), artificial-gravity strongest) and psych load (f(duration, comms-lag,
confinement)); degrade ECLSS and **roll a failure** on `crew/eclss-failure`; recompute REID + capability;
check viability/loss-of-crew thresholds and **emit events**. EDL is **command-driven** (`EvaluateEdl`
rolls `crew/edl-risk`); anomalies roll `crew/anomaly`. Streams threaded via `ctx.rng(path)` (FA-05's
idiom).

**Rationale.** Resolves the cadence question; daily granularity matches dose/physiology accumulation and
SPE/failure arrival; named streams keep every stochastic outcome deterministic and `mutate`-catchable.

**Alternatives rejected.** (a) Per-tick (sub-day) — needless cost; nothing resolves faster than a day.
(b) Analytic per-mission accumulation — drops the seeded daily-event texture the design requires.

---

## R5 — Consumables vs ECLSS closure (US1)

**Decision.** A crewed asset's consumables stock depletes at a **sourced per-crew-day rate × crew**.
ECLSS **closure** (composed `AssetSizing.closure_capability`) recycles the **air/water loop only**
(O₂/water/N₂/CO₂ — what ECLSS does, design 04-SPACEFLIGHT §6); **food is open-loop** (carried as mass;
bioregenerative food production is FA-07's greenhouse loop, optionally composed in as a reduction). So
**make-up = air/water gross × (1 − closure) + food gross** (A1). **Viability**: stock + scheduled
resupply must cover the mission duration; otherwise non-viable (and exhaustion ⇒ loss-of-crew risk). A
**robotic** asset (`crewed = false`) consumes nothing.

**Rationale.** FR-LSC-101…105 + Principle VII — *crewed = mass*; closing the loop is the central trade.
The make-up identity is a clean analytic gate (SC-001).

**Alternatives rejected.** (a) Ignore closure (always full resupply) — removes the loop-closing trade.
(b) Apply consumables to robotic assets — breaks the crewed-difficulty premise.

---

## R6 — Radiation dose + REID limit (US2, clarified Q2)

**Decision.** Per crew member: accrue **GCR** dose (sourced rate per environment × `shield_attenuation`
× dt) plus seeded **SPE** storms (mitigated by a storm shelter when the crew shelters). Career dose →
**REID** via a **sourced dose→risk curve** parameterised by the astronaut's **age/sex** (from
`CrewRoster`); the astronaut is **grounded at the 3% REID threshold**, and a mission that would push REID
past 3% is flagged non-viable. SPE storms are seeded + state-driven (R4).

**Rationale.** FR-LSC-201…205 + Q2 — NASA's actual radiation-limit model; data-driven and honest
(Principle VIII). REID monotonic in dose is a clean gate (SC-002).

**Alternatives rejected.** (a) Single career sievert cap — the design wants career + mission limits;
REID supersedes both with the age/sex risk model (Q2-A/B rejected). (b) Unseeded SPE — breaks
determinism.

---

## R7 — Deconditioning + countermeasures (US3)

**Decision.** Per crew member, **bone/muscle/cardiovascular/vision** indices accrue at sourced micro-g
rates × `(1 − countermeasure_effectiveness)`; **artificial gravity** (spin-hab, composed
`AssetSizing.spin_gravity`) is the strongest mitigation (largest effectiveness). Each index maps through
a sourced curve to a **deconditioning capability factor** ∈ [0,1] (R11). Post-mission recovery is a
sourced relaxation back toward baseline.

**Rationale.** FR-LSC-301…303 — long micro-g is costly; spin-g is the real unlock. Spin-vs-micro-g is a
clean comparative gate (SC-003).

**Alternatives rejected.** (a) A single "health" scalar — loses the per-system honesty. (b) No
recovery — unrealistic and removes the crew-rotation strategy.

---

## R8 — Psychology (US4)

**Decision.** Per crew member, **psych load** accrues over time at a sourced rate scaled by **mission
duration, confinement** (`habitat_volume_m3_per_crew`) and **comms-lag** (`EnvFacts.comms_lag_s`). Load
maps to a capability factor (R11) and contributes an **anomaly hazard multiplier** (R9/R10 on the
`crew/anomaly` stream). High load ⇒ higher anomaly probability + morale loss.

**Rationale.** FR-LSC-401…403 — psychology is the human cost of distance/isolation and a seeded anomaly
driver; load-vs-anomaly monotonicity is a gate (SC-004).

**Alternatives rejected.** (a) Psychology as pure flavour — the design forbids; it must bind via
anomalies. (b) Deterministic anomalies — must be seeded + state-driven.

---

## R9 — ECLSS spares, maintenance & failure (US5, clarified Q1)

**Decision.** ECLSS **reliability** derives from composed `TechMaturity` (trl + flight_units) + heritage;
it **degrades** over time and is maintained by **crew-time + spares** (a maintenance deficit raises
risk). A daily **failure roll** on `crew/eclss-failure` uses the **multiplicative hazard**
(`base_rate × maturity_factor × maintenance_factor × age_factor`, clamped — R12/Q1). A **critical failure
beyond abort reach** (`EnvFacts.abort_reach == false`) is a **loss-of-crew risk surfaced as an
interrupt**; near Earth, abort is an option. Never silently absorbed.

**Rationale.** FR-LSC-501…504 + Q1 — a failed loop far from home is the existential crewed risk; the
multiplicative hazard makes maturity/maintenance bite, each factor sourced + monotone (gate).

**Alternatives rejected.** (a) Fixed failure probability — ignores maturity/maintenance. (b) Silently
absorb a critical failure — dishonest; the design forbids.

---

## R10 — EDL & aerocapture crew risk (US6, clarified Q1)

**Decision.** A command-driven `EvaluateEdl` rolls `crew/edl-risk` with the **multiplicative hazard**:
`crew_loss_prob = base_rate × suitability_factor(EdlSuitability) × body_difficulty(EnvFacts.body) ×
crew_state_factor(capability)`, clamped. **Per-body difficulty** is sourced (`edl.ron`) with the **Mars
case the highest** (the modelled "Mars EDL gap"). Failure ⇒ a **loss-of-vehicle / loss-of-crew**
consequence (R12).

**Rationale.** FR-LSC-601…603 + Q1 — EDL is where missions die; the Mars gap is data-driven (Mars
difficulty ≫ airless), a clean comparative gate (SC-006).

**Alternatives rejected.** (a) Deterministic EDL — drops the crew-risk roll. (b) Same difficulty for all
bodies — erases the Mars gap.

---

## R11 — Crew capability = multiplicative product of per-state factors (clarified Q3)

**Decision.** The composite **crew-capability** ∈ [0,1] = **∏** of per-state capability factors —
`deconditioning_factor × psych_factor × health_factor` — each a **sourced [0,1] curve** of its state.
Stressors compound (two moderate impairments ⇒ a larger loss). Capability feeds the EDL crew-state gate
(R10) and viability (R12); each factor is independently sourced + testable.

**Rationale.** FR-LSC-303 + Q3 — multiplicative compounding is physically honest for independent
stressors and consistent with the multiplicative-hazard choice (Q1).

**Alternatives rejected.** (a) Minimum/limiting-factor — moderate stressors don't compound (Q3-B). (b)
Weighted average — a strong factor masks a fatal weak one (Q3-C).

---

## R12 — Multiplicative-hazard primitive + loss-of-crew (US7, Q1)

**Decision.** A shared `hazard.rs` primitive composes every seeded event probability as
`clamp(base_rate × ∏ factor_multipliers, 0, 1)` (FR-LSC-808). **Loss-of-crew** is a real consequence:
the affected crew member(s) are marked **lost**, the mission **failed**, and a `loss-of-crew` event
(Interrupt) is emitted; the **political/prestige/flight-freeze fallout is FA-09** (consumed downstream).
Crewed **viability** is a pure check over the composite state (consumables / REID / ECLSS / capability
thresholds).

**Rationale.** FR-LSC-701…703 + Q1/Q2 — one hazard model backs failures/anomalies/EDL; loss-of-crew is
honest and interrupt-surfaced; viability composes the sub-systems.

**Alternatives rejected.** (a) Per-event bespoke probability math — inconsistent, un-auditable. (b)
Model political fallout here — FA-09's scope (confirmed Q2).

---

## R13 — Crew-state query surface + traceability

**Decision.** `CrewSnapshot::from_core(&core, &crew_module, inputs)` via kernel `with_slice` over the
crew slice, composing the R2 inputs. Pure functions answer: each crew member's dose/REID/deconditioning/
psych/**capability**/grounded state; each asset's **consumables/ECLSS/viability** and resupply make-up;
loss-of-crew; and traceability trees to sourced leaves. Faction-scoped. Read-only, between ticks,
IPC-serializable for FA-09/10.

**Rationale.** FR-LSC-701 + Principle VIII — the identical seam as FA-04…07; the trace is the honesty
contract; the surface is what FA-09 (politics) and FA-10 (UI) consume.

**Alternatives rejected.** (a) Mutable handles — break read-only/determinism. (b) Store derived
REID/capability/viability (R3).

---

## R14 — Determinism, data-version pin, events, cadence

**Decision.** Named seeded streams: `crew/spe-storm`, `crew/eclss-failure`, `crew/edl-risk`,
`crew/anomaly` (threaded via `ctx.rng(path)`). Ordered `BTreeMap`/`BTreeSet`; libm-only; no wall-clock.
Crew data is content-hashed and **pinned in saves** (extends the FA-02…07 pattern). New **event classes**
(data registry, `data/kernel/event-classes.ron`): `spe-storm` (Interrupt), `eclss-failure` (Interrupt),
`crew-anomaly` (LogOnly), `astronaut-grounded` (Interrupt), `loss-of-crew` (Interrupt). **Cadence**: daily
`step` (`cadence_ticks = 86_400`). **No kernel change.** This slice exercises the `mutate` gate hardest
(many seeded draws) — a strong determinism proof.

**Rationale.** Mirrors FA-05's seeded discipline; interrupt-class events (SPE storm, ECLSS failure,
grounding, loss-of-crew) feed the FA-01 interrupt-and-pause loop ("stop on something that matters").

**Alternatives rejected.** (a) Global RNG — non-deterministic. (b) Unpinned data — silent realism drift.
(c) New kernel event plumbing — events are data registry entries.

---

## Summary of decisions feeding Phase 1

| # | Decision | Primary artifacts |
|---|---|---|
| R1 | `sojourn-crew` above all, coupled only to core | plan structure |
| R2 | Composed-value seams (5 inputs incl. age/sex) | contracts/integration-seams |
| R3 | Dynamic state stored + evolved; derived stays pure | data-model, contracts/crew-queries |
| R4 | Daily seeded step (FA-05 pattern) | contracts/crew-commands, module |
| R5 | Consumables vs ECLSS closure; robotic exempt | data-model, contracts/crew-data |
| R6 | GCR + seeded SPE; REID dose limit (age/sex) | data-model, contracts/crew-data |
| R7 | Deconditioning + countermeasures (artificial g) | data-model |
| R8 | Psychology → anomaly hazard | data-model |
| R9 | ECLSS multiplicative-hazard failure; abort reach | data-model, contracts/crew-commands |
| R10 | EDL multiplicative-hazard crew risk; Mars gap | data-model, contracts/crew-data |
| R11 | Capability = multiplicative product of factors | data-model |
| R12 | Shared hazard primitive + loss-of-crew | data-model, contracts/crew-queries |
| R13 | Composed crew-query surface + traceability | contracts/crew-queries |
| R14 | Determinism; seeded streams; data-version pin; events; daily cadence | contracts/crew-commands, contracts/crew-data |
