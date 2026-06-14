# Contract — Data formats, sourcing & analytic gates (`data/polity/`)

All files RON, every quantitative entry carries `source` (Principle I/V); `validate-data polity` fails CI
on any missing source or a failed analytic gate. Content is blake3-hashed and pinned in saves. The
astrobiology **priors** are NOT here — they stay in FA-03's `data/world/astrobiology.ron`; the site PP
**categories** are bridged from FA-03's `data/world/sites.ron`. `data/polity/` holds the *mechanics*
parameters only.

## Files

- **milestones.ron** — `[{ id, era, description, weight, faction_first_fraction, conditions: [Condition],
  source }]`. ~120 firsts across foothold/cislunar/frontier/endgame eras; a **representative sourced
  subset** at this slice (structured for data-only completion). Endgame/breakthrough firsts flagged as
  Breakthrough-gated (FA-05).
- **mood.ron** — outcome deltas, `decay_per_day`, bounds, `loc_recovery_days`, mood→{appropriation,
  valuation, approval} curves. Sourced from agency appropriation volatility + post-accident crewed pauses.
- **events.ron** — `[{ class, base_rate, factor_refs, interrupt, source }]` for every event source.
- **policy.ron** — `[{ id, min, max, default, drift_per_period, lobby_step, gates, source }]`; PP-stringency
  lever required.
- **protection.ron** — per-body COSPAR `{ category, special_region, bioburden_limit, sterilisation_ref }`
  + `contamination { overage_curve, crash_factor, soft_factor, backcontam_penalty }`. Sourced from COSPAR
  policy.
- **astrobiology.ron** — `stage_lr` (OrbitalHint<InSitu<Microscopy<SampleReturn), `abiotic_competitor`,
  `consensus_weight: PrestigeWeighted`, `band_positive: 0.9`, `band_negative: 0.1`,
  `sample_return_required: true`, `false_hint_cap`. Sourced from biosignature-ambiguity / sample-return
  literature.
- **ai.ron** — heuristic weights, plausibility-envelope caps, difficulty hooks, tide-advance rate.
- **goals.ron** — per-goal thresholds (incl. `seeker_worlds: 3`), `change_penalty`, composite-score
  weights.
- **params.ron** — horizon, global bounds, score normalisation.
- **validation.ron** — analytic cases + tolerances (below).

## Analytic gates (`validate-data polity`)

1. **Prior fidelity** — over N seeds, realised positive ground-truth fraction ≈ each candidate's prior
   within tolerance.
2. **Consensus band** — a candidate flips to conclusive **only** when the prestige-weighted consensus
   crosses ≥ `band_positive` / ≤ `band_negative` **and** a SampleReturn item exists; and **never**
   conclusive-positive when ground truth is negative (`false_hint_cap < band_positive` enforced).
3. **Contamination monotonicity** — pristine-value degradation strictly increases with bioburden overage;
   `crash_factor ≥ soft_factor ≥ 1`; a compliant lander degrades nothing.
4. **Event hazard** — `clamp(base × Π factors, 0, 1)` monotone in each factor, clamped to [0,1].
5. **Tiebreak determinism** — same-tick world-first → highest prestige then lowest id, reproducible.
6. **Score determinism** — composite score is a pure function of composed inputs.
7. **Mood** — saturates within bounds; `|loss_of_crew delta| > |routine_failure delta|` and
   `loc_recovery_days` ≥ the sourced multi-year floor.
8. **Source presence** — every quantitative entry has a non-empty `source`.
