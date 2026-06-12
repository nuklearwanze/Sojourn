# Contract: Body Catalogue Interface (FR-ASTRO-103/107)

Defined and consumed by `sojourn-astro`; **implemented by FA-03** (world data) in production.
A small sourced test catalogue ships in this slice (`data/astro/test-catalog.ron`).

## What the catalogue supplies (per body)

| Field | Meaning | Notes |
|---|---|---|
| `id`, `name_id` | stable identity | name is an identifier, not display text |
| `mu` | gravitational parameter (m³/s²) | sourced |
| `radius` | mean radius (m) | impact detection |
| `j2` | oblateness coefficient | optional; dominant-body perturbation |
| `rotation_period_s` | sidereal rotation | optional (frames; later EDL/sites) |
| `atmosphere` | `{interface_alt_m, rho0, scale_height_m}` | optional exponential model; drag + aero passes + entry handoff boundary |
| `gravitating` | far-field gravity flag | clarified rule: flagged ⇒ pulls all craft; unflagged ⇒ only within own SOI |
| `divertible` | small-body divert eligibility | never set on planets/major moons |
| `parent`, `elements` | rail definition | Keplerian elements about the parent, epoch-tagged |
| `srp_reference` | solar-flux reference | SRP model input |
| `source` | provenance | mandatory (Principle I; CI-enforced) |

## Behavioural contract

- **Rails**: `state_at(body, t) → (r, v)` is a pure function of time derived from `elements`
  (Kepler solution). FA-03 MAY supply richer ephemeris representations later (piecewise
  elements, interpolants) behind the same pure `state_at` semantics — purity and exact
  reproducibility per data version are the contract, not the representation.
- **Hierarchy**: `parent` links form a tree rooted at the star; SOI radii derive from
  `mu`/`parent.mu` and the rail geometry (standard Laplace-sphere formula, computed not stored).
- **Divertibility** (FR-ASTRO-107): only bodies with `divertible: true` may leave their rail;
  the motion-state lifecycle (Railed → Diverted → ReRailed) lives in the astro slice, not the
  catalogue — the catalogue stays immutable per data version, and saves pin it (FA-01 rules).
- **Mass for diversion physics**: divertible bodies need a mass estimate; FA-03 supplies it
  (`mu`-derived); deflection delta-v from an applied impulse scales by it honestly.
- **No display data**: human-readable naming, imagery, science text are FA-03/FA-10 concerns.

## Test catalogue (this slice's fixtures)

An idealised, fully-sourced system exercising every validation case: a star; an Earth-like
planet (μ, J2, exponential atmosphere from IERS/US Standard Atmosphere-class citations); a
Moon-like satellite; a Mars-like outer planet (porkchop/synodic cases); a small divertible
asteroid (gravitating only in-SOI; diversion cases). Parameters are textbook values with
`source` fields — idealisations are explicitly labelled as such in the source strings.
