# Contract: Tech-Tree & Research Data + Validation (FR-RESP-701/702/901)

How the web-shaped tech tree and the research-tuning numbers enter the game as sourced, schema-validated
data, and how the capability-reachability invariant is guaranteed and verified.

## Data format

- `data/research/domains.ron` — the **full A1–A17** Knowledge-Domain set with synergy links and
  diminishing-returns params (R3). Every domain sourced.
- `data/tech/tech-tree.ron` — a **representative sourced subset** of engineering Technology nodes
  (clarified Q3:A) exercising every mechanic: cross-branch gates, ≥2 paths per capability category,
  leapfrog seams (`UlSatisfiable` prereqs), seeded dead ends, breakthroughs, heritage discounts. Each
  node carries `start_trl`, `ul_floors`, `tech_prereqs`, `trl_steps` (cost / min-duration floor /
  facility / S-curve), `reliability_curve`, `capability_category`, optional `derivative_of`, and a
  **mandatory `source`**. The full node population is a documented data expansion behind this schema.
- `data/tech/capability-categories.ron` — `category → [candidate TechId paths]`; the domain of the
  reachability invariant (each category ≥2 paths).
- `data/research/params.ron`, `data/research/traits.ron` — sourced tuning + trait modifiers.

Content is hashed and **pinned in saves** (extends FA-02's catalogue-hash; R15). No combat/weapons
nodes; Orion-type pulse (design B5.5) is intentionally absent except as a Sojournal historical entry
(Principle IX).

## Constructive reachability (R6, clarified Q2:A)

Dead-end seeding processes capability categories and **never closes a category's last viable path** —
so every category retains ≥1 viable path in every seed *by construction*. This is an algorithm
invariant, not a post-hoc rejection.

## Validation (`validate-data research`, CI)

- Schema + non-empty `source` for every domain, node, param and trait.
- Synergy links, `ul_floors`, `tech_prereqs` (incl. cross-branch) and `derivative_of` resolve.
- Every node's `capability_category` is in the category map; every category has ≥2 candidate paths.
- Reliability/overrun/breakthrough/tide parameters present and sourced.
- A **reachability sweep** over a sampled seed set asserts the FR-RESP-301 constructive guarantee
  (every category keeps ≥1 viable path), catching any seeding-algorithm regression.
- No node carries a combat/weapons capability category.
