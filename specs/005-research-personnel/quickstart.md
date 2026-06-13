# Quickstart: Research & Personnel (FA-05)

Headless scenarios that exercise the slice end-to-end and double as the independent tests for each user
story. All run through `sojourn-harness` with the `research` flag (installs the research module over
`data/research` + `data/tech`). No UI anywhere.

## Build & gate

```pwsh
cargo test -p sojourn-research                          # unit + integration
cargo run -p sojourn-harness -- validate-data data/research   # schema + sources + reachability sweep
cargo run -p sojourn-harness -- conformance --module research
cargo run -p sojourn-harness -- verify    scenarios/research_program.ron   # double-run bit-identity
cargo run -p sojourn-harness -- roundtrip scenarios/research_program.ron --save-at-ticks <t1>,<t2>
cargo bench -p sojourn-harness --bench research                            # multi-faction tick + query latency
```

## US1 — Understanding before capability
1. Fund a domain; assert UL rises with diminishing returns + synergy from sourced params.
2. A program below its domain-UL floor is `available_programs`-absent; crossing the floor makes it available (the gating domain/threshold reported).
3. An `InjectUnderstanding` command raises the named domains beyond lab-only growth; an unknown domain is rejected.
4. `verify` double-run: all ULs bit-identical.

## US2 — Maturing a technology through TRL
1. `research_program.ron`: start a program, allocate DE, advance; assert each TRL step respects its cost, min-duration floor, facility + UL gate; schedule compression raises overrun risk, never sub-floor time.
2. Estimates carry P50/P80 and realise with seeded overrun variance; reliability tracks TRL + flight-units + UL; flying below TRL 6 is refused.
3. `RegisterHeritage` raises reliability toward the ceiling and a `derivative_of` program starts partway up the ladder.

## US3 — The tree is alive
1. Across many seeds: some approaches are dead ends within TRL bands (risk index rises, error bars stall before `dead-end-confirmed`) while a parallel approach to the same category stays viable.
2. A failing test campaign costs money/schedule **and** injects UL; repeated failure without UL growth is the dead-end signal.
3. Sustained basic-science investment accrues insight; a `breakthrough` fires only at a seeded threshold at the documented rare cadence, with a sourced reference; applied-only rarely triggers.
4. Leapfrogging reaches a higher tier via UL-satisfiable prereqs.
5. **Reachability sweep** (`validate-data research` + a ≥100-seed test): every capability category keeps ≥1 viable path in every seed.

## US4 — The global tide
1. `research_tide.ron` (multi-faction): World UL advances from aggregate + baseline; private = world + lead/lag.
2. `publish` accelerates World UL + emits a prestige-eligible event; hold retains the lead.
3. A trailing faction researches cheaper than the frontier; catch-up cannot exceed World UL without frontier investment.

## US5 — People make it happen
1. Traits shift the documented outcomes (Visionary low-TRL, Closer 6→9 qual, Maverick breakthrough/overrun).
2. Hire/poach/train/age transitions are deterministic; poach emits a relations-cost signal; training takes the documented years.
3. Efficiency multipliers respond to under-staffing, domain mismatch and facility bottlenecks.
4. Disbanding a team reduces effective niche UL (recompute), without corrupting the stored ground UL.

## US6 — The astronaut pipeline
1. A candidate advances select→train→ready under the documented facility/time gate.
2. `CrewFeedback` dose/health deltas accumulate deterministically; crossing a limit removes an astronaut from the ready pool.

## US7 — Maturity/heritage/understanding on tap
1. Query maturity/heritage/understanding/program-status/personnel across factions; verify faction privacy (no cross-faction private state) and that flyability refuses sub-TRL-6.
2. Calling any query between ticks leaves the state fingerprint unchanged (pure).

## Determinism & performance
- `verify` + `roundtrip` + `conformance --module research` pass; saves pin and verify the research-data version.
- Multi-faction full-roster + node-subset holds the ≥1 sim-year/min envelope; queries < 50 ms — checked by the `research` bench.
