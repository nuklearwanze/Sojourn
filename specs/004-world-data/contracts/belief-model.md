# Contract: Truth / Belief Separation + Refinement Model (FR-WORLD-301..304, SC-002/003)

The game's honesty contract, made structural and deterministic. Ground truth is engine-private;
belief is per-faction; observation is the only bridge, and it is an honest estimator that converges
to documented floors — never to truth.

## Separation (structural — R4)

- **Ground Truth Store**: a private field of the world slice. **Fixed** where reality fixes it
  (orbits, radii, documented compositions); **seeded** where the design calls for per-game variation
  (grades, hazard details, astrobiology) from **sourced plausibility distributions** via the
  `world-seed` stream at world creation.
- **No truth in any snapshot**: `WorldSnapshot` (the only thing query functions see) has no truth
  fields, so no pure query can return truth *by construction*. Truth is read only by the module's
  own `step`/`on_command` resolution and a `#[cfg(any(test, feature="privileged"))]` accessor.
- **Standing audit (SC-002)**: `tests/audit.rs` enumerates the entire public query surface and
  asserts no unsurveyed seeded truth is reachable; permanent CI regression guard.

## Belief state (per faction)

- **Scalar property** → Gaussian `(mean, variance)` in a transformed space (log-space for positive
  quantities, so beliefs can't imply negatives).
- **Ordinal/categorical** (hazard level, composition, PP category) → probability vector.
- **Init** from documented priors (`priors.ron`): tight for well-known majors, wide for unsurveyed
  sites/small bodies. A default wide prior MUST exist for any `(target, property)` (no error, no
  truth leak — edge case "belief before any prior").

## Observation classes & floors (data, `priors.ron`)

`remote-sensing | in-situ | sample-grade`, each with a documented uncertainty **floor** `floor(c)`
and a measurement-noise model `σ(c, quality)` with `σ ≥ floor`. Floors order
`remote ≥ in-situ ≥ sample-grade` — better classes can get tighter, none reaches zero.

## Refinement update (R3) — Gaussian precision addition

For a measurement `m = truth + ε`, `ε ~ N(0, σ²)`:

```
post_var  = 1 / (1/prior_var + 1/σ²)
post_mean = post_var * (prior_mean/prior_var + m/σ²)
post_var  = max(post_var, floor_var(class))
```

Categorical: Bayesian update by a per-class confusion-matrix likelihood.

### Guarantees this yields (SC-003)

| Property | Why it holds |
|---|---|
| **Information never decreases** | precision adds ⇒ `post_var ≤ prior_var` always; a worse (larger-σ) later observation only adds a little precision, never widens belief |
| **Estimate moves toward truth** | the mean is a precision-weighted blend pulled toward `m` (truth + zero-mean noise) |
| **Converges to the class floor, not truth** | the `max(.., floor_var)` clamp ⇒ repeated observations approach `floor(c)`, never 0 ("you can't remote-sense your way to ground truth") |
| **Stable at convergence** | once at the floor, further same-class observations leave variance at the floor (no oscillation, no underflow — edge case) |
| **Order-independent tightness** | precision sums commute; only the seeded noise *path* (deterministic) affects the mean |
| **Deterministic** | `ε` is drawn from the `obs-noise` stream keyed by `(faction,target,property,seq)`; double-run/replay bit-identical |

## Astrobiology (FR-WORLD-306, this slice = data + seeding)

Per candidate world, a seeded presence/absence (+ tier) from sourced plausibility distributions —
**mostly negative**, rarely >1–2 positives/game — in the private truth store, with belief fields at
honest 2026 priors. Leaks through no query. The staged evidence process and any
contamination-consequence mechanics are deferred to the mission/politics slices.
