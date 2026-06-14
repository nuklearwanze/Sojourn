# GP-06 — Bases & Construction (FA-17)

**Spec dir:** `specs/018-bases-construction` · **Depends on:** GP-05 (surveyed site), GP-01 (logistics/funds), GP-04 (delivery missions) · **Speckit:** `speckit/gameplay/gp-06-bases-construction.md`

Settlement — the game's destination. The player founds a base at a surveyed site, composes it from modules, runs a build queue fed by delivered (or locally-fabricated) mass, and watches emergent properties decide whether it lives: power margin, ECLSS closure, radiation shielding, self-sufficiency. Building far from Earth is a real logistics-and-survival problem.

## Goal & player-facing capability

Found an orbital station or surface base at a sufficiently-surveyed site; add modules (habitat, power, ECLSS, ISRU, science, storage, shielding) from a catalogue; open construction and route delivered mass + crew-time through logistics, or build locally from regolith/ISRU; see emergent properties recompute live (power margin, ECLSS closure %, sustainability index, shielding); evaluate embargo survival ("if resupply stopped for N years, does it survive?"). Respect planetary-protection categories when siting.

## Orchestration intents → command fan-out

In `sojourn-game`:
- `Intent::FoundBase { site, class }` → `BaseCommand::FoundBase` — **gated** (preview: site must be surveyed ≥ threshold (GP-05 belief), PP category consequences, initial mass/crew-time, funds). Validation reads the belief-state.
- `Intent::AddModule { base, module_type }` → `BaseCommand::AddModule` then `OpenConstruction`.
- `Intent::DeliverModule { base, module, … }` → composes a **delivery mission** (GP-04 logistics: an economy `DispatchShipment` priced in Δv + funds, plus the mission thread) then `BaseCommand::DeliverToBase` on arrival; gated at dispatch.
- `Intent::BuildLocal { base, module, local_mass_kg }` → `BaseCommand::BuildLocal` (uses ISRU/regolith output to substitute imported mass).
- `Intent::EvaluateEmbargo { base, years }` → `BaseCommand::EvaluateEmbargo` (analysis, `Direct`).
- `Intent::DecommissionBase` → gated.

Emergent properties (power balance, mass-attenuation shielding, limiting-factor self-sufficiency, embargo survival) are the base slice's existing derivations — displayed, not recomputed.

## Cross-system causality & state touched

Founding needs a surveyed site (GP-05) and consumes logistics + funds (GP-01) via delivery missions (GP-04). Local build needs ISRU output (economy/world). A crewed base adds the life-support layer (GP-07). A first permanent settlement / self-sufficiency milestone feeds prestige and the Homestead Grand Goal (GP-08). State: base slice (journalled).

## ESA data

Reuses `data/base/*` (module catalogue with class params, shielding mass-attenuation lengths, closure-loop defs + dose limits, regolith-construction/build params, base-class templates, validation analytic cases). Confirm sources.

## UI/UX — S7 Bases & Construction (now interactive)

Overview: ESA's bases (none at start) + "Found base".

Subscreens:
- **Sites** — surveyed sites eligible to found on (gated by GP-05 belief threshold), with PP category and key properties; insufficiently-surveyed sites are visible-but-locked ("survey further to found").
- **Found base** — choose class; gate shows PP consequences + initial cost.
- **Module catalogue & layout** — the base **schematic** widget; add modules from the catalogue; each module shows its contribution (power ±, mass, closure).
- **Build queue** — modules awaiting delivery/local-build, with the delivery mission (Δv + funds + ETA) or local-build (ISRU mass) path; reorder.
- **Emergent properties** — the gauges: power margin, ECLSS closure %, sustainability index, shielding (mass-attenuation), population capacity — each traced to the contributing modules.
- **Self-sufficiency / embargo** — the limiting-factor breakdown and the embargo-survival evaluator (set N years → survives? what fails first?).

Inspector: selected base or module with sourced params.

Plan→preview→commit verbs: Found base (Build), Deliver module / Dispatch (Build), Decommission (Cancel). Add-module + queue ordering are reversible until dispatched. Empty state: no surveyed sites yet → "Survey a site (S5/S1) before founding."

View-model: `BasesVM`/`BaseSchematic` extended with catalogue, build queue, emergent-properties (traced), and embargo builders; unit-test the gauge derivation pass-through, the survey gate, and the limiting-factor display. Renderer wires found/deliver/decommission gates.

## Testability

Harness `bases_play.ron`: boot ESA → survey a lunar polar site (GP-05) → found a Shackleton outpost (assert gate requires survey threshold + applies PP) → add fission-power + hab + ECLSS + water-ISRU modules → dispatch deliveries (assert logistics shipment priced in Δv + funds, mission created) → on arrival assert modules installed and emergent power margin / ECLSS closure / sustainability recompute → run embargo eval at 1 year (assert survives/fails with limiting factor). Determinism + round-trip. View-model gauge tests. Human: survey → found → add modules → watch power margin and closure update → run the embargo check.

## Acceptance criteria

Bases can be founded only at sufficiently-surveyed sites (belief-gated) with PP consequences; modules deliver via logistics (Δv+funds) or build locally from ISRU; emergent properties recompute and trace to modules; embargo survival evaluates with a limiting factor; numbers sourced; renderer holds no base derivation.

## Out of scope

Crewing the base (GP-07). The Homestead Grand-Goal scoring (GP-08). Detailed in-space-manufacturing markets (GP-08 economy depth).
