# Quickstart — Politics, Events, Milestones & Astrobiology (FA-09)

Verifies the FA-09 vertical slice end-to-end against the spec's success criteria. All steps are headless
(`cargo test` + the harness); no UI. Run from the repo root with the workspace toolchain on `PATH`.

## Build & unit/integration tests

```pwsh
cargo test -p sojourn-polity
```

Covers (one test file per user story):
- **milestones.rs** (US1, SC-001/002): a world-first awards to exactly the first global claimant; later
  achievers get faction-first; same-tick tie → highest prestige then lowest id; prestige accrues by weight;
  ledger bit-identical across two runs.
- **mood.rs** (US2, SC-003): a world-first lifts mood + budget modifier with decay; a loss-of-crew drops
  mood deeper and longer than a routine failure and freezes crewed-flight approval for the recovery window.
- **astrobiology.rs** (US3, SC-004/005): ground truth drawn from FA-03 priors and faithful across many
  seeds; never query-exposed; staged evidence moves the per-faction posteriors + the weighted consensus;
  factions can publicly disagree; conclusive needs band-cross **and** a SampleReturn item; **never**
  conclusive-positive on a negative-ground-truth world.
- **events.rs** (US4, SC-006): a low-TRL/over-subscribed craft has a strictly higher realised event rate
  than a mature one; interrupt vs log classes behave; event stream identical across stepping patterns.
- **policy.rs** (US5, SC-007): a mission lacking nuclear-launch approval is gated/penalised; lobbying +
  drift stay in bounds; PP-stringency lever feeds the PP regime.
- **protection.rs** (US6, SC-008): a non-compliant lander in a Special Region degrades pristine value
  **graded by overage × crash/soft**; a compliant lander degrades nothing; back-contamination gated
  without a containment chain.
- **ai.rs** (US7, SC-009): AI factions claim firsts + advance the tide; capability never exceeds the
  plausibility envelope; difficulty raises funding/competence only.
- **goals.rs** (US8, SC-010): each Grand Goal computes a deterministic pass/fail from composed inputs;
  changing goal applies the penalty; the composite score is reproducible.

## Data validation + analytic gates (SC-011)

```pwsh
cargo run -q -p sojourn-harness -- validate-data data/polity
```

Prints the source-presence + analytic-gate results (prior fidelity, consensus band, contamination
monotonicity, event-hazard monotonicity, tiebreak/score determinism, mood bounds).

## Determinism + module conformance (SC-001/006/011)

```pwsh
cargo run -q -p sojourn-harness -- conformance --module polity
cargo run -q -p sojourn-harness -- verify     scenarios/politics_world.ron
cargo run -q -p sojourn-harness -- roundtrip  scenarios/politics_world.ron --save-at-ticks <t1>,<t2>,<t3>
cargo run -q -p sojourn-harness -- mutate --all --scenario scenarios/politics_world.ron
```

`politics_world.ron` drives a full mini-game: `InitWorld` (factions + FA-03 priors + PP) → achievements
racing the AI world for firsts → mood swings (incl. a loss-of-crew) → policy set/lobby → staged
astrobiology evidence on two candidates → a Special-Region contamination → seeded events → Grand-Goal
selection and horizon scoring. `verify` proves double-run + cross-stepping identity; `roundtrip` proves
bit-identical save/load; `mutate` proves the determinism gate has teeth on the seeded streams.

## Whole-workspace gates (SC-011)

```pwsh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace          # FA-01…08 stay green; no new cross-gameplay-crate edge
```

## Success-criteria map

| SC | Verified by |
|---|---|
| SC-001 world-/faction-first ledger determinism | `milestones.rs`, `verify` |
| SC-002 exactly-one world-first + tie rule | `milestones.rs` |
| SC-003 loss-of-crew deeper/longer + approval freeze | `mood.rs` |
| SC-004 ground-truth prior fidelity + never exposed | `astrobiology.rs`, `validate-data` |
| SC-005 no conclusive-positive on negative truth | `astrobiology.rs`, `validate-data` |
| SC-006 riskier craft → higher event rate; reproducible | `events.rs`, `verify` |
| SC-007 policy gating + stringency | `policy.rs` |
| SC-008 graded forward contamination | `protection.rs`, `validate-data` |
| SC-009 AI plausibility envelope + difficulty | `ai.rs` |
| SC-010 Grand-Goal pass/fail + composite score | `goals.rs` |
| SC-011 data/source validation + save/load + suites green | `validate-data`, `roundtrip`, `cargo test --workspace` |
| SC-012 performance at warp | bench (deferred, consistent with FA-03…08) |
