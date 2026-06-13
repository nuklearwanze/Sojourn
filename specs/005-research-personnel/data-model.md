# Phase 1 Data Model: Research & Personnel (FA-05)

Entities, fields, relationships, validation and state transitions for `sojourn-research`. Ordered
stores are `BTreeMap` (determinism). All quantitative content is sourced data (Principles I/V/VI);
"slice state" is serialized in the module's slice and covered by save/roundtrip/replay gates. Every
faction-keyed entity is parametric in `FactionId` (R14).

---

## 1. Tech-tree & domain data (immutable module data; `data/tech/`, `data/research/`)

### Knowledge Domain (`domains.ron`)
| Field | Type | Rules |
|---|---|---|
| `id` | `DomainId` (e.g. `A1`) | unique; the A1–A17 set |
| `name_id` | string | identifier (display is FA-10's) |
| `synergy` | `[(DomainId, f64)]` | coupled domains + synergy weights; targets must exist |
| `dr_knee`, `dr_steepness` | f64 | diminishing-returns curve params (cost rises sharply above the knee) |
| `basic_science` | bool | basic vs applied (insight-pressure weighting, R7) |
| `mission_injectable` | bool | true for A8–A16 (missions inject UL beyond labs) |
| `source` | string | mandatory |

### Technology Node (`tech-tree.ron`)
| Field | Type | Rules |
|---|---|---|
| `id` | `TechId` | unique |
| `capability_category` | string | the reachability unit (e.g. `cheap-launch`, `lunar-propellant`) |
| `start_trl` | 1..=9 | real 2026 maturity |
| `ul_floors` | `[(DomainId, f64)]` | domain-UL availability gates |
| `tech_prereqs` | `[TechPrereq]` | each `{tech, kind: Product\|UlSatisfiable}` — `UlSatisfiable` marks a leapfrog seam (R8); cross-branch gates allowed |
| `trl_steps` | `[TrlStep]` | per-step cost, min_duration_days (floor), facility capability, S-curve params |
| `reliability_curve` | params | scalar-reliability curve over (TRL, flight-units, UL); ceiling |
| `derivative_of` | `Option<TechId>` | heritage discount source (starts partway up the ladder) |
| `source` | string | mandatory (CI-enforced) |

### Capability Category map (`capability-categories.ron`)
`category → [candidate TechId paths]` — the domain over which the **constructive reachability**
invariant holds (R6): seeding never closes a category's last viable path. Validation: every node's
`capability_category` appears here; every category has ≥2 candidate paths (design L244).

### Research params (`params.ron`) and Traits (`traits.ron`)
Sourced tuning: RP/DE generation rates, overrun (P50/P80) variance, breakthrough insight rates +
cadence, tide baseline + aggregate weights + catch-up discount, reliability-curve globals, trait
modifiers. All carry `source`; `validate-data` enforces presence.

---

## 2. Understanding & the tide (slice state, per faction)

### Understanding Level — *slice state*
`ul: BTreeMap<(FactionId, DomainId), f64>` in `[0,100]` — the **ground** private UL. Growth per step
(R3): `Δ = alloc_rp × efficiency × dr_factor(ul) × synergy(domain)`. **Effective UL** (used by gates
and queries) = ground UL adjusted by tacit-knowledge recompute (R11) — derived, never stored.

### World Tide — *slice state*
`world_ul: BTreeMap<DomainId, f64>` advancing from baseline + aggregate faction activity (R9).
`publish_policy: BTreeMap<(FactionId, DomainId), Publish|Hold>`. Catch-up discount derives from
`world_ul − ul` (bounded by world_ul).

---

## 3. Programs, campaigns & technologies (slice state, per faction)

### Engineering Program — *slice state*
| Field | Type | Rules |
|---|---|---|
| `id` | `ProgramId` | unique |
| `faction` | `FactionId` | owner |
| `tech` | `TechId` | target |
| `trl` | 1..=9 | current |
| `step_progress` | f64 | S-curve progress in the current step |
| `est_p50`, `est_p80` | cost/schedule | estimate at step entry |
| `actual_cost`, `actual_days` | f64 | realised (overrun = actual vs P50) |
| `lead` | `Option<PersonId>` | assigned lead (trait effects) |
| `risk_index` | f64 | rises on stalled progress (dead-end hint) |
| `parallel_tag` | `Option<String>` | groups parallel approaches to one capability |

**State transitions**: `Proposed → Active →` (per step) `StepTesting →` {`StepPassed` → next TRL |
`StepFailed` → UL injected, risk_index↑, retry} `→ … → Matured (TRL≥6 usable)`. Flyable only at TRL ≥ 6.

### Test Campaign — *transient per step*
Seeded success/failure (R5) from the `research/test` stream; failure injects UL and may emit
`test-failure`; repeated failure without UL growth raises `risk_index` toward the dead-end signal.

### Technology Maturity — *derived from programs + heritage*
Per `(faction, tech)`: current TRL (max matured program), **scalar reliability ∈ [0,1]** (R10) +
raw inputs, flyability (TRL≥6), heritage.

### Flight Heritage — *slice state*
`heritage: BTreeMap<(FactionId, TechId), {flight_units, ceiling}>` raised by `register-heritage`
events (FA-04+); raises reliability asymptotically + discounts `derivative_of` programs.

### Dead-end Seeding — *slice state (seeded at init)*
`dead_ends: BTreeMap<(TechId, TrlBand), bool>` fixed by the `research/seed` stream with the
**constructive** guarantee (R6). `breakthrough_thresholds: BTreeMap<(FactionId?,DomainId), f64>`
seeded.

### Breakthrough insight — *slice state*
`insight: BTreeMap<(FactionId, DomainId), f64>` accruing basic-science-weighted pressure (R7);
crossing a seeded threshold fires a `breakthrough` (cluster discount | early unlock | hidden-path
reveal) with a sourced reference.

---

## 4. Personnel & astronauts (slice state, per faction)

### Person — *slice state*
| Field | Type | Rules |
|---|---|---|
| `id` | `PersonId` | unique |
| `faction` | `FactionId` | employer |
| `role` | enum | Scientist \| Engineer \| ProgramManager \| Controller \| Diplomat \| Astronaut |
| `discipline` | `DomainId`/tag | specialisation |
| `skill` | f64 | rating |
| `traits` | `[TraitId]` | from `traits.ron`; shift low-TRL/qual/breakthrough/overrun/reliability |
| `age`, `morale` | f64 | aging + retention |
| `training` | `Option<{program, progress, facility}>` | multi-year training in progress |

**Transitions**: hire / poach (relations-cost signal) / train (multi-year, facility-gated) / assign /
retire/age-out — deterministic. RP/DE **efficiency multipliers** derive from staffing level vs program
need, domain mismatch and facility bottlenecks. **Tacit-knowledge**: effective UL in a niche domain is
recomputed from current roster (loss of key staff lowers it; re-hire restores) — never mutates ground UL.

### Astronaut career — *slice state (Person sub-state, R12)*
`{stage: Candidate|Training|Ready|Retired, career_dose, health, psych}` budgets; crossing a documented
limit → leaves the Ready pool. Updated by the **`crew-feedback`** command (FA-08 deltas).

---

## 5. Module slice & manifest (kernel contract)

### ResearchSlice — *the owned, serialized state*
`ul`, `world_ul`, `publish_policy`, `programs`, `heritage`, `dead_ends`, `breakthrough_thresholds`,
`insight`, `personnel`, `astronaut_state`, `next_*` id counters, `research_hash` (data-version pin).
(Immutable tree/domain/params/traits data is module data loaded at init, hashed + pinned — not
duplicated in the slice.)

### ResearchModule manifest
| Field | Value |
|---|---|
| `id` | `research` |
| `owned_slice` | `research/slice` |
| `publishes` | `research/status` (counts: programs, matured techs, factions) |
| `reads` | `kernel/status` |
| `streams` | `research/seed` (creation: dead ends, breakthrough thresholds), `research/overrun`, `research/test`, `research/breakthrough` |
| `emits` | `breakthrough`, `dead-end-confirmed`, `test-failure`, `program-milestone`, `trl-advance`, `publish` |
| `subscribes` | — (v1) |
| `cadence` | fixed (default daily; data-tunable); time-stepped, warp-invariant |

- **Pinning**: research-data content hash pinned in saves (extends FA-02 hash; R15).
- **Conformance/determinism**: ordered stores, libm-only, declared streams, no wall-clock → passes
  `conformance --module research` and the harness double-run/roundtrip/replay gates.

---

## Entity relationship summary

```text
Knowledge Domain ──synergy──▷ Domain        Capability Category ──has──▷ ≥2 candidate Tech paths
       │ gates (UL floor)                              ▲ reachability invariant (constructive)
       ▼                                               │
Technology Node ──trl_steps──▷ TRL 1..9 ──advanced by──▶ Engineering Program (faction)
       │ derivative_of                                   │ test campaign (seeded) → UL injection
       ▼                                                 ▼
Flight Heritage ──raises──▶ scalar Reliability ∈[0,1]    Dead-end seed / Breakthrough insight (seeded)
Personnel (roles, traits) ──generate──▶ RP/DE ──allocate──▶ Programs & Domain UL
       └─ Astronaut career (dose/health) ◀── crew-feedback ── FA-08
World Tide (World UL, publish/hold, catch-up) ◀──aggregate── all factions
Query surface (faction-scoped): maturity · heritage · understanding · program-status · personnel
```
