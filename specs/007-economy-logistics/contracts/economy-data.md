# Contract: Economy Data Formats, Sourcing & Analytic Gates (FA-06)

All economic constants are sourced, schema-validated `data/econ/*.ron` (Principle I/V; FR-EC-801).
`validate-data econ` fails CI on any missing/empty `source`, unresolved reference, or a failed
analytic gate. CRLF-normalized content hashing pins the econ-data version in saves (FR-EC-803, R16).

## Files (all entries carry `source`)

| File | Holds | Key validations |
|---|---|---|
| `commodities.ron` | the tradable-commodity taxonomy (`Raw`/`Processed`/`Manufactured`/`Consumable`/`Strategic`/`Service`) | `Raw.resource_ref` resolves to an FA-03 resource id; `Processed.from`/`Strategic.cap_ref` resolve; no combat commodity; unique ids |
| `funding.ron` | per-faction funding profiles (agency / private) | one variant per faction; baselines/floors ≥ 0 |
| `launch_market.ron` | $/kg by orbit class, world-capacity baseline, elasticity | prices ≥ 0; elasticity finite |
| `isru.ron` | process params (yield, plant mass, power, scale-up/reliability ramp) for ice/Sabatier/regolith/asteroid | `input` is `Raw`, `output` is `Processed`; yields/mass/power ≥ 0 |
| `cost.ron` | P50/P80 spread + overrun-shape params | spreads ≥ 0; produce `p50 < p80` |
| `facilities.ron` | facility/ground-segment templates (capex/opex/capacity/upgrade) | capex/opex/capacity ≥ 0 |
| `strategic.ron` | strategic-material supply caps (Pu-238/Am-241, HEU/LEU, rare-earth) | caps ≥ 0; policy gate label valid |
| `network.ron` | curated transport-graph node set (ref world location ids) + edge templates | node `location` ids resolve against the world; edge endpoints exist; launch edges carry an orbit class |
| `markets.ron` | RFP-generator params, partnership/trust params, tourism/ISM market sizes + price ceilings | sizes/ceilings ≥ 0 |
| `validation.ron` | analytic validation cases + tolerances | see gates below |

## Analytic validation gates (Principle II / constitution testing mandate)

`validate-data econ` (and the test suite) enforce, each to a stated tolerance:

1. **Conservation** — over a scripted set of modelled processes, Σ accounted inflow = Σ outflow and no
   stock goes negative (SC-003, R11).
2. **ISRU break-even sign** — for the sourced lunar-ice and Mars-Sabatier cases, net is **negative
   below** the break-even scale and **positive above** it (SC-004, R10).
3. **Learning monotonicity** — realised unit cost is non-increasing as cumulative production rises,
   along the (FA-04) learning exponent (SC-005, R9).
4. **P50 < P80** — every cost estimate's bands satisfy `p50 < p80` (SC-005, R9).
5. **Launch-price elasticity sign** — raising the world-capacity index lowers $/kg (SC-007, R12).

## Sourcing examples (illustrative, real values land in data)

- Launch $/kg by orbit class: current commercial launch price sheets (Falcon 9/Heavy, Ariane 6, etc.).
- ISRU yields/plant mass: lunar-ice ISRU & cislunar propellant-market studies; MOXIE / Mars Sabatier architectures.
- Budget baselines: NASA/ESA/Roscosmos/CSA/JAXA published appropriations.
- Strategic supply: Pu-238 production-rate literature; HEU/LEU policy sources.
- Learning exponents: Wright's-law / spaceflight cost-learning literature (reuses the FA-04 cost basis).
