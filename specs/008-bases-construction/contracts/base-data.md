# Contract: Base Data Formats, Sourcing & Analytic Gates (FA-07)

All module/construction parameters are sourced, schema-validated `data/base/*.ron` (Principle I/V;
FR-BC-701). `validate-data base` fails CI on any missing/empty `source`, unresolved reference, or a
failed analytic gate. CRLF-normalized content hashing pins the base-data version in saves (FR-BC-703).

## Files (all entries carry `source`)

| File | Holds | Key validations |
|---|---|---|
| `modules.ron` | the module catalogue (`Habitat`/`Power`/`Eclss`/`IsruHost`/`Science`/`Storage`/`Manufacturing`/`Shielding`) | `closure_fraction` ∈ [0,1]; `dry_mass`/`power_demand` ≥ 0; `Shielding.material` resolves to a `params.ron` attenuation length; no combat module; unique ids |
| `params.ron` | shielding attenuation lengths λ per material, dose limit, closure-loop defs, regolith-construction/build rates, PV reference distance | λ > 0; dose limit > 0; ratios ≥ 0; the five closure loops present |
| `classes.ron` | base-class templates (orbital station, surface base, settlement) | flags (orbital/crewed/buildable) consistent; sourced |
| `validation.ron` | analytic validation cases + tolerances | see gates below |

## Analytic validation gates (Principle II / constitution testing mandate)

`validate-data base` (and the test suite) enforce, each to a stated tolerance:

1. **Power-margin additivity** — adding a sourced power module raises a base's margin by exactly that
   module's (solar-scaled) generation; demand is additive (SC-001, R5).
2. **Shielding exp-attenuation** — `exp(−Σᵢ ρxᵢ/λᵢ)` (per-material sum in the exponent) is correct and
   monotone: for a single material, doubling areal density squares the attenuation factor; two materials
   compose as the product of their per-material attenuations (SC-002/004, R7).
3. **Limiting-factor index** — the self-sufficiency index equals the minimum loop ratio; improving the
   binding loop raises it (SC-006, R11).
4. **Embargo rate+buffer** — a base with production ≥ demand (or buffer ≥ deficit×span) on every loop
   survives; one with a loop short of both fails (SC-005, R12).

## Sourcing examples (illustrative, real values land in data)

- Module masses/power: ISS module masses, Gateway HALO/I-HAB, commercial-LEO-station studies (D3, I10/I11).
- Shielding attenuation lengths: radiation mass-attenuation data for regolith/water/polyethylene (D4).
- ECLSS closure: ISS ECLSS water/air recovery fractions; MELiSSA closed-ecology studies (F2–F5).
- Regolith-construction rates: sintering / 3D-print-habitat / geopolymer-concrete studies (D6).
- Dose limits: career/annual crew dose limits (radiation-protection standards).
