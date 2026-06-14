# Contract: Crew Data Formats, Sourcing & Analytic Gates (FA-08)

All physiology/ECLSS/radiation/EDL parameters are sourced, schema-validated `data/crew/*.ron` (Principle
I/V; FR-LSC-801). `validate-data crew` fails CI on any missing/empty `source`, unresolved reference, or a
failed analytic gate. CRLF-normalized content hashing pins the crew-data version in saves (FR-LSC-803).

## Files (all entries carry `source`)

| File | Holds | Key validations |
|---|---|---|
| `consumables.ron` | per-crew-day O₂/water/food/N₂ rates + closure-tier params | rates ≥ 0 |
| `radiation.ron` | GCR rates per environment, SPE storm arrival/magnitude, shelter attenuation, the dose→REID curve (age/sex) | rates ≥ 0; REID threshold = 3%; REID monotonic in dose |
| `physiology.ron` | deconditioning rates (bone/muscle/cardio/vision) + countermeasure/artificial-gravity effectiveness + capability curves | rates ≥ 0; artificial-g effectiveness ≥ exercise/pharma |
| `psychology.ron` | psych-load accrual + comms-lag/confinement sensitivities + anomaly hazard | sensitivities ≥ 0 |
| `eclss.ron` | ECLSS failure base rate + maturity/maintenance/heritage multipliers + degradation/spares params | rates/multipliers ≥ 0 |
| `edl.ron` | per-body EDL difficulty (incl. the Mars gap) + suitability multipliers | difficulties ≥ 0; **Mars > any airless body** |
| `params.ron` | hazard base rates, viability thresholds, the 3% REID threshold | thresholds in range |
| `validation.ron` | analytic validation cases + tolerances | see gates below |

## Analytic validation gates (Principle II / constitution testing mandate)

`validate-data crew` (and the test suite) enforce, each to a stated tolerance:

1. **Consumables make-up identity** — make-up rate = **air/water gross × (1 − closure) + food gross**
   (ECLSS closure recycles air/water only; food is open-loop, A1); a robotic asset has zero consumption
   (SC-001, R5).
2. **REID monotonicity** — REID is non-decreasing in accumulated dose and respects the 3% grounding
   threshold (SC-002, R6).
3. **Multiplicative-hazard monotonicity** — for ECLSS failure / anomaly / EDL, raising any single factor
   monotonically raises the probability, and the result is clamped to [0,1] (SC-005/006, R12/Q1).
4. **Mars EDL gap** — `edl.body_difficulty["mars"]` yields a materially higher crew-loss probability than
   an airless body for the same vehicle suitability (SC-006, R10).
5. **Capability product** — overall capability = ∏ per-state factors ∈ [0,1]; artificial gravity yields a
   materially higher capability than micro-g for the same mission (SC-003, R7/R11).

## Sourcing examples (illustrative, real values land in data)

- Consumables rates: ISS ECLSS consumables budgets (~5 kg/crew-day open-loop).
- GCR/SPE + REID: NASA/NCRP radiation-exposure & REID models; deep-space GCR dose rates (~0.5–1 Sv/yr).
- Deconditioning: ISS long-duration bone/muscle-loss rates; artificial-gravity countermeasure studies.
- ECLSS reliability: ISS ECLSS failure/maintenance data; closure-fraction studies (F2–F5).
- EDL difficulty: the Mars EDL gap literature (landing >1–2 t is a multi-tech grand challenge).
