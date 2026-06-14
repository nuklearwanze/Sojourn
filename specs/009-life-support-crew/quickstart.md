# Quickstart: Life Support & Crew (FA-08)

A headless walk through the slice that proves the spec's success criteria. Everything runs under the
deterministic harness; no UI. Commands are `Command::ModulePayload { module: "crew", … }`; queries are
pure functions over a `CrewSnapshot`. Cross-slice physics is fed as composed values
(`contracts/integration-seams.md`).

## 0. Build & validate data

```text
cargo test -p sojourn-crew
cargo run -p sojourn-harness -- validate-data data/crew        # schema + sources + analytic gates
cargo run -p sojourn-harness -- conformance --module crew      # manifest, double-run, serde round-trip, cadence
```

## 1. Consumables vs ECLSS closure (US1 → SC-001/007)

1. `OccupyAsset` with N crew + a sizing of closure C; advance days → consumables deplete at the sourced rate.
2. Query `consumables(asset)` → make-up rate = gross × (1 − C); raise C in the sizing → make-up falls.
3. Query `viability(asset, D)` → a mission whose stock + resupply can't cover D is non-viable.
4. Occupy a **robotic** asset (`crewed=false`) → no consumption, no constraint (crewed-difficulty premise).

## 2. Radiation dose & REID limit (US2 → SC-002)

1. Advance a crewed asset under a GCR environment → each member's career dose rises × shield attenuation.
2. Trigger a seeded SPE storm; `Shelter` the crew → the acute dose is reduced versus not sheltering.
3. Query `reid(astronaut)` → REID from the sourced dose→risk curve + age/sex; at 3% the astronaut is
   `Grounded` (`astronaut-grounded` event). Career dose carries across two missions.

## 3. Deconditioning & artificial gravity (US3 → SC-003)

1. Advance a long micro-g mission → bone/muscle/cardio/vision indices rise; `capability` falls.
2. Run the same mission with `spin_gravity = true` → materially less deconditioning, higher capability.

## 4. Psychology & anomalies (US4 → SC-004)

1. Advance a long, high-comms-lag, cramped mission → psych load rises faster than a short, near-Earth, roomy one.
2. Higher psych load → a higher seeded anomaly probability (`crew-anomaly`); reproducible per seed.

## 5. ECLSS failure (US5 → SC-005)

1. Run two assets differing only in ECLSS maturity → the lower-maturity one fails more often (seeded).
2. `Maintain` one with crew-time + spares → its failure probability falls.
3. A critical failure with `abort_reach=false` → a loss-of-crew risk surfaced as an interrupt, never absorbed.

## 6. EDL crew risk (US6 → SC-006)

1. `EvaluateEdl` on Mars vs an airless body for the same vehicle suitability → Mars carries a materially
   higher crew-loss probability (the Mars EDL gap).
2. An EDL failure → loss-of-vehicle / loss-of-crew (seeded, deterministic).

## 7. Exposure & loss-of-crew (US7 → SC-010)

1. Query `member`/`viability`/`loss_of_crew` → full per-member + per-asset state for politics/UI.
2. A loss-of-crew event physically marks the crew lost + emits `loss-of-crew` for FA-09 (political fallout is FA-09's).

## 8. Determinism, save/load, perf (cross-cutting → SC-008/009/011)

```text
cargo run -p sojourn-harness -- verify    scenarios/crew_mission.ron     # double-run bit-identical (SC-008)
cargo run -p sojourn-harness -- roundtrip scenarios/crew_mission.ron --save-at-ticks <t1>,<t2>
cargo run -p sojourn-harness -- mutate --all                            # the seeded gate has teeth
```

- SC-009: every `data/crew/*` entry has a `source`; CI `validate-data crew` enforces it.
- SC-011: dozens of crewed assets × hundreds of crew sustain ≥1 sim-year/min at high warp.

## Success-criteria coverage map

| SC | Proven by |
|---|---|
| SC-001 | §1 (make-up identity) |
| SC-002 | §2 (dose → REID, grounding) |
| SC-003 | §3 (artificial gravity vs micro-g) |
| SC-004 | §4 (psych → anomaly) |
| SC-005 | §5 (ECLSS maturity/maintenance failure) |
| SC-006 | §6 (Mars EDL gap) |
| SC-007 | §1.4 (robotic vs crewed) |
| SC-008 | §8 `verify` + `mutate` |
| SC-009 | §0 `validate-data crew` |
| SC-010 | §7 (exposure + loss-of-crew event) |
| SC-011 | §8 perf envelope |
