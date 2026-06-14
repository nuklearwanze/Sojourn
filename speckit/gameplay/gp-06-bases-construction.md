# GP-06 — Bases & Construction · `/speckit` set (FA-17)

**Branch:** `018-bases-construction` · **Design:** `gameplay/07-BASES-CONSTRUCTION.md` · **Depends:** GP-05, GP-01, GP-04

## /speckit.specify

```
/speckit.specify Let the player settle: found a base at a surveyed site, compose it from modules, run a build queue fed by delivered or locally-fabricated mass, and watch emergent properties decide whether it lives. Make the Bases & Construction screen (S7) interactive. Authoritative design: gameplay/07-BASES-CONSTRUCTION.md, gameplay/00-CORE-LOOP.md, gameplay/UI-UX-CONVENTIONS.md and .specify/memory/constitution.md (Principles I, VII, V, IX) — read them.

WHY: settlement is the game's destination, and building far from Earth is a real logistics-and-survival problem.

Let the player found an orbital station or surface base at a sufficiently-surveyed site (the founding gate validates the GP-05 belief threshold and applies planetary-protection consequences); add modules (habitat, power, ECLSS, ISRU, science, storage, shielding) from a catalogue, each showing its contribution; open construction and route delivered mass + crew-time through logistics (a delivery composes an economy shipment priced in Δv + funds plus a delivery mission, gated at dispatch) OR build locally from regolith/ISRU output; see emergent properties recompute live and traced to their contributing modules (power margin, ECLSS closure %, sustainability index, radiation shielding via mass-attenuation, population capacity); and evaluate embargo survival ("if resupply stopped for N years, does it survive, and what fails first?"). Respect planetary-protection categories when siting.

The emergent-property derivations stay in the base slice — the UI displays, never recomputes. Intent expansion and the delivery-mission composition live in the orchestration crate / mission module.

Acceptance: bases can be founded only at sufficiently-surveyed sites (belief-gated) with PP consequences; modules deliver via logistics (Δv+funds) or build locally from ISRU; emergent properties recompute and trace to modules; embargo survival evaluates with a limiting factor; numbers sourced; renderer holds no base derivation. Flag ambiguities for /speckit.clarify. Do not choose a tech stack — the workspace stack is fixed (Rust, egui); reuse it.
```

## /speckit.clarify — focus points

- The survey-belief threshold required to found, and the locked-site messaging for under-surveyed sites.
- How a module delivery composes into a logistics shipment (`EconomyCommand::DispatchShipment` priced in Δv + funds) plus a delivery mission, and the on-arrival `BaseCommand::DeliverToBase`.
- Local-build (`BuildLocal`) substitution: how ISRU/regolith output offsets imported mass.
- Planetary-protection consequences of founding per site category.

## /speckit.plan — guidance

- `sojourn-game` intents `FoundBase` (gated; validates GP-05 belief + PP), `AddModule`+`OpenConstruction`, `DeliverModule` (compose shipment + mission, gated at dispatch), `BuildLocal`, `EvaluateEmbargo` (Direct analysis), `DecommissionBase` (gated) expanding to the real `BaseCommand`/`EconomyCommand` variants.
- View-model: extend `BasesVM`/`BaseSchematic` with catalogue, build queue, emergent-properties (traced) and embargo builders; reuse the base-schematic and logistics-graph widgets. Renderer: S7 subscreens (Sites, Found base, Module catalogue & layout, Build queue, Emergent properties, Self-sufficiency/embargo) wired to the gates.
- Tests: harness `bases_play.ron` (survey a lunar polar site → found a Shackleton outpost with the survey+PP gate → add fission-power+hab+ECLSS+water-ISRU → dispatch deliveries priced in Δv+funds → on arrival modules install and emergent power margin/closure/sustainability recompute → embargo eval at 1 year reports survives/fails + limiting factor); determinism + round-trip; view-model gauge-derivation tests.

## /speckit.tasks & /speckit.analyze — notes

Separate intents/preview + delivery composition, view-model builders, S7 renderer, tests. `/speckit.analyze` must confirm: founding belief-gated (consumes GP-05, not ground truth), emergent properties displayed not recomputed (Principle II/IV), sourced module/shielding/closure params (Principle I/V), thin renderer (Principle IV), core audit green.
