# GP-02 — Research & Personnel · `/speckit` set (FA-13)

**Branch:** `014-research-personnel` · **Design:** `gameplay/03-RESEARCH-PERSONNEL.md` · **Depends:** GP-00, GP-01

## /speckit.specify

```
/speckit.specify Make Sojourn's research a playable strategic engine and turn the R&D (S3) and Personnel (S8) screens interactive. Authoritative design: gameplay/03-RESEARCH-PERSONNEL.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles VI, V, VIII) — read them.

WHY: research is the modelled process that matures the technology the vehicle designer (GP-03) consumes. Today the ULs sit at their seeded values and nothing advances them.

Let the player allocate the research portfolio across knowledge domains and active programmes plus facility assignment (splits that normalise to 1, drawing funds from the economy), watching projected Understanding-Level slopes; start an engineering programme toward a tech node behind a plan→preview→commit gate whose core-computed preview shows expected cost (P50/P80), schedule, the domain-UL floor required, and the dead-end probability band; advance programmes through TRL 1–9 via test campaigns gated by domain UL, with overruns, seeded dead-ends (which still inject understanding) and rare breakthroughs surfaced honestly; set publish-versus-patent policy per domain; and hire, poach, train and retire scientists, engineers, project managers and astronauts. The maturity, heritage and understanding produced here must be queryable by GP-03.

All intent→command expansion and preview composition live in the stateless orchestration crate; the renderer is thin.

Acceptance: portfolio allocation normalises and draws funds; programmes start behind a core-computed gate and advance through TRL via UL-gated campaigns; dead-ends/overruns/breakthroughs surface honestly; publish policy and personnel verbs work; maturity/heritage/understanding are queryable for GP-03; numbers sourced; renderer holds no research logic. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- Allocation normalisation UX: live remainder while dragging, commit on release; confirm splits-sum-to-1 enforcement is display-side with the slice as source of truth.
- How a programme's tech-node target is chosen (from the tech-tree graph) and how gates reference domain UL floors.
- Personnel recruitment economics (hire vs poach cost, the funds draw) and tacit-knowledge-loss on retire.
- Which 2026 ULs/roster come from the GP-00 bootstrap versus set here.

## /speckit.plan — guidance

- `sojourn-game` intents `SetResearchAllocation`, `StartProgram`, `SetPublishPolicy`, `Hire/Poach/Train/Retire` expanding to the real `ResearchCommand` variants. `StartProgram` and `Hire/Poach` are gated (core-computed preview); allocation and publish policy are Direct.
- View-model: extend `ResearchVM` into portfolio + programme-board + domain-detail + programme-detail builders; add `PersonnelVM` (roster/recruit/train/assignments). Renderer: S3 subscreens (Science portfolio, Engineering programmes, Programme detail, Domain detail, Tech-tree graph) + S8 subscreens (Roster, Recruit, Training, Assignments); reuse the TRL-ladder and Understanding-bars widgets (with the world-tide ghost).
- Tests: harness `research_play.ron` (allocate normalises + draws funds; start a programme with the gate preview; advance years → UL rises toward tide, TRL advances, campaign runs, reliability emerges; forced dead-end still injects understanding; hire+train changes skill); determinism + round-trip; view-model tests for allocation normalisation and ladder shaping.

## /speckit.tasks & /speckit.analyze — notes

Separate intents/previews, view-model builders, S3 renderer, S8 renderer, tests. `/speckit.analyze` must confirm: research modelled-process integrity preserved (Principle VI — no tech-point purchase shortcuts), dead-end/breakthrough honesty (Principle VIII), sourced params (Principle V), thin renderer (Principle IV), core audit green.
