# GP-02 — Research & Personnel (FA-13)

**Spec dir:** `specs/014-research-personnel` · **Depends on:** GP-00, GP-01 (funds the portfolio) · **Speckit:** `speckit/gameplay/gp-02-research-personnel.md`

The strategic engine. The player sets the research portfolio, starts engineering programmes, advances them through TRL gates with test campaigns, and manages the people who do the work. This is where Understanding rises, technologies mature, dead-ends bite, and rare breakthroughs land — the depth that makes the tech tree feel earned.

## Goal & player-facing capability

Allocate the research portfolio across knowledge domains and active programmes (sliders summing to 1) plus facility assignment; start an engineering programme toward a tech node; watch domain ULs climb against the world tide; advance a programme through TRL 1–9 via test campaigns gated by domain UL; set publish-vs-patent policy; hire/poach/train/retire scientists, engineers, PMs and astronauts; read overrun risk, dead-end risk, and breakthrough surfacing. The maturity and reliability that come out here feed the vehicle designer.

## Orchestration intents → command fan-out

In `sojourn-game`:
- `Intent::SetResearchAllocation { domain_splits, program_splits, facilities }` → `ResearchCommand::SetAllocation` (reversible/`Direct`; the splits normalise to 1, drawing funds booked in economy).
- `Intent::StartProgram { tech, lead }` → `ResearchCommand::StartProgram` (gated: preview shows expected cost P50/P80, schedule, the domain UL floor required, dead-end probability band).
- `Intent::SetPublishPolicy { domain, publish }` → `ResearchCommand::SetPublishPolicy` (reversible).
- `Intent::Hire/Poach/Train/Retire { … }` → the matching `ResearchCommand` (Hire/Poach gated by funds; Train/Retire `Direct`).

Test campaigns advance automatically as the programme is funded and the domain UL clears the gate; failures inject understanding (the slice already models this). The preview for `StartProgram` is core-computed from the research slice.

## Cross-system causality & state touched

Research output is the **maturity()/heritage()/understanding()** surface the vehicle slice (GP-03) consumes for capability and reliability; ULs feed the science tide and (via GP-08) prestige. Funding for the portfolio is drawn from economy (GP-01). Personnel here are the same pool astronauts are drawn from for crew (GP-07). State: research slice (journalled). No new module.

## ESA data

Reuses `data/research/*` and `data/tech/*` (domains A1–A17, programmes, RP/DE generation, overrun/breakthrough/tide/reliability params, traits, dead-end seeding). ESA's starting ULs and roster come from `data/scenario/esa_2026.ron` (GP-00). Confirm sources present.

## UI/UX — S3 Research & Development + S8 Personnel (now interactive)

S3 overview: domain UL bars (with world-tide ghost) + active programme summary.

S3 subscreens:
- **Science portfolio** — the allocation widget: domain split sliders + programme split sliders + facility assignment, with a live **remainder** indicator (must sum to 1) and projected UL slope per domain. Committing allocation is `Direct`.
- **Engineering programmes** — programme board: each row a TRL ladder (1–9) with current rung, the active test campaign, the assigned lead, P50/P80 cost, schedule, overrun and dead-end risk tags. "Start programme" opens the gate.
- **Programme detail** — gate requirements (domain UL floor), campaign progress, parallel-approach options to mitigate dead-ends, breakthrough indicator.
- **Domain detail** — UL curve over time, the world tide and catch-up discount, synergy with neighbouring domains, publish/patent toggle.
- **Tech-tree graph** — the web-shaped sourced node graph: capability categories always reachable, specific nodes seeded; gates and cross-branch dependencies; click a node to start a programme toward it.

S8 Personnel subscreens: **Roster** (by role, with traits) · **Recruit** (hire/poach, gated by funds, with skill/trait preview) · **Training** (improve skill; tacit-knowledge-loss warning on retire) · **Assignments** (who leads which programme). Astronaut careers (dose/health) arrive in GP-07.

Inspector: selected domain / programme / person, with trace on every figure.

View-model: `ResearchVM` extended into portfolio + programme-board + domain/programme detail builders; a `PersonnelVM` for roster/recruit/train. Unit-tested headlessly (allocation normalisation, TRL-ladder shaping, gate-preview pass-through). Renderer wires Start-programme and Hire/Poach gates.

## Testability

Harness `research_play.ron`: boot ESA → allocate portfolio (assert splits normalise, funds draw) → start an electric-propulsion programme (assert gate preview = cost/schedule/UL-floor) → advance years → assert domain UL rises toward the tide, programme advances TRL, a test campaign runs, reliability emerges; force a seeded dead-end and assert understanding still injected; hire then train a scientist and assert skill change. Determinism + round-trip. View-model tests. Human: move sliders, start a programme, watch ULs climb across a few years of warp.

## Acceptance criteria

Portfolio allocation normalises and draws funds; programmes start behind a core-computed gate and advance through TRL via campaigns gated by UL; dead-ends/overruns/breakthroughs surface honestly; publish policy and personnel verbs work; maturity/heritage/understanding are queryable for GP-03; numbers sourced; no logic in renderer.

## Out of scope

Turning maturity into a vehicle (GP-03). Astronaut radiation/health careers (GP-07). Prestige from publications (GP-08). The Sojournal entries for each tech (GP-09 deep-links to existing data).
