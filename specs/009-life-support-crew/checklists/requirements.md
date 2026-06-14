# Specification Quality Checklist: Life Support & Crew (FA-08)

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-06-14
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

- Three high-impact decisions are surfaced as confirmation questions Q1–Q3 (per-individual crew tracking;
  loss-of-crew physical/event here with political fallout deferred to Slice 9; composed-value coupling
  with the dynamic time-evolution owned here). The spec is internally consistent under the recommended
  defaults and records them in the Clarifications section.
- Functional requirements use the `FR-LSC-###` scheme banded by user story (100s consumables/closure,
  200s radiation, 300s deconditioning, 400s psychology, 500s ECLSS failure, 600s EDL risk, 700s
  exposure/loss-of-crew, 800s cross-cutting).
- This is the first slice with **dynamic, time-stepped per-entity state on seeded streams** (SPE storms,
  ECLSS failures, EDL rolls, anomalies) — unlike the static derivations of FA-04/FA-07.
- `/speckit-clarify` (2026-06-14) resolved three composition models: seeded event probabilities use a
  **multiplicative hazard** (FR-LSC-808); the radiation limit is **REID-based, age/sex-adjusted** at the
  3% threshold (FR-LSC-203/204 — adds age/sex as a Slice 5 composed input + a sourced dose→risk curve);
  crew capability is the **multiplicative product** of per-state factors (FR-LSC-303). Recorded in the
  Clarifications section.
- Constitution principles in scope: I (sourced data), VII (tyranny of mass/Δv — crewed materially harder
  than robotic), VIII (educational honesty — real physiology/radiation, no misinformation), plus
  cross-cutting II (physics-authoritative), III (determinism + seeded streams), IV (headless), V
  (data-driven), IX (no combat — crew loss is a safety consequence).
- `/speckit-analyze` (2026-06-14) remediations applied: **A1** ECLSS closure recycles **air/water only**,
  food is open-loop (FR-LSC-102 + make-up gate); **I1** the roster age/sex is stub-fed (real FA-05 bridge
  deferred); **U1** consumables exhaustion is a named loss-of-crew trigger; **L1** the EDL command-time vs
  query-time suitability is intentional. Zero CRITICAL/HIGH findings.
