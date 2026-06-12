# SOJOURN — Part 5: The World (Solar System, Sites, Astrobiology, Politics, Events, Milestones)

> The board the game is played on, and the living systems that make it react to the player.

---

## 1. Solar-system data model

- **Bodies**: Sun; 8 planets; Pluto + major dwarf planets (Ceres, Eris, Haumea, Makemake);
  ~150 significant moons; a curated catalogue of ~3,000 asteroids & comets with **real orbital
  elements** (sourced from JPL/MPC small-body data); the rest of the main belt and Kuiper region
  abstracted as **statistical prospecting fields** (you survey to convert statistics into known
  targets).
- **Per body**: orbit (real elements), mass/radius/gravity, rotation/day length, atmosphere
  (composition, pressure, scale height — drives EDL & aerocapture), thermal environment,
  radiation environment, known composition/geology (a *belief state* that missions refine),
  hazards (dust, radiation, terrain), and a **Geoscience UL** sub-track that grows as you explore
  it.
- **Dynamical locations** (non-body nodes): LEO/MEO/GEO/GTO bands, Earth-Moon & Sun-Earth
  Lagrange points (halo/NRHO orbits — Gateway lives at lunar NRHO), low orbits of each body,
  cyclers. These are first-class nodes in the logistics graph (03-ECONOMY.md §6).
- **Ephemeris**: positions propagated from real elements so windows, alignments, and assist
  opportunities are physically correct over the 2026–2126 span.

## 2. Sites & resources (surface play)

Bodies expose **Sites** — specific, characterised locations — rather than tiles. Examples seeded
from real targets:
- **Moon**: Shackleton/Cabeus & other PSRs (polar water ice — grade unknown until surveyed),
  mare basalt plains, highland regolith, lava tubes (radiation-safe habitats), peaks of near-
  eternal light (solar power), KREEP terrains.
- **Mars**: Jezero/delta deposits, hydrated-mineral terrains, mid-latitude ground ice, Valles
  Marineris & cave entrances, Tharsis volcanics, polar caps, recurring-slope-lineae sites.
- **NEAs / Belt**: C-type (volatiles/water), S-type (metals/silicates), M-type (metals); Ceres
  (water, possible brines), Vesta.
- **Outer system**: Europa/Enceladus (subsurface ocean access via ice; astrobiology), Titan
  (organics, lakes, dense atmosphere — aerial & surface ops), Mars moons Phobos/Deimos (low-Δv
  staging), Io (off-limits radiation), Ganymede/Callisto (ice, lower radiation than Europa).

**Site properties** (revealed progressively by survey, J1–J2): resource type & grade,
illumination/thermal profile, slope/roughness (EDL & mobility), comms visibility to relays,
science value, hazard level, and **planetary-protection category** (§3). Survey uncertainty is
modelled — early estimates have error bars that missions narrow. *You can build a mine on a bad
ice grade and lose money; surveying first is strategy.*

## 3. Planetary protection (a real, modelled constraint)

Sojourn implements a COSPAR-style planetary-protection regime as gameplay (Pillar P1/P6):
- **Body categories** (I–V) set sterilisation/containment requirements. Special Regions on Mars
  (potential liquid water) and ocean worlds (Europa/Enceladus) carry strict forward-contamination
  rules: bioburden limits, sterilisation costs/mass, restricted access.
- **Forward contamination**: crash a non-sterile lander into a Special Region → science penalty,
  political/reputation cost, possible loss of the body's "pristine" astrobiology value (you can
  ruin the experiment for everyone, including future you).
- **Back contamination**: sample return from a potentially-habitable world requires a
  containment/receiving chain (J5) — BSL facilities, restricted-Earth-return trajectories.
- Policy levers (§5): factions/world can tighten or relax categories; cutting corners is a real,
  tempting, consequential choice. The *Seeker* goal cares a lot about not poisoning your own
  evidence.

## 4. Astrobiology (no aliens — but maybe microbes; seeded ground truth)

Per Pillar P1 and the user's allowance: **life elsewhere in the Solar System is possible** and is
modelled honestly as an open scientific question with a **per-game seeded ground truth**.
- **Candidate habitats** carry a hidden truth flag set at game start within plausibility bounds:
  Mars subsurface, Europa/Enceladus oceans, Titan (exotic chemistry), Ceres brines, possibly
  Venus cloud layer. Most games: mostly negative with one or two positives; some games: all
  negative; rarely: more. The *space* is constrained to what science deems plausible; the *draw*
  is seeded → replayable "are we alone here?" each game.
- **Detection is a staged scientific process** (J3): orbital biosignature hints → in-situ
  chemistry (organics, chirality, disequilibria) → microscopy/metabolism → **sample return &
  independent confirmation**. Each stage gives probabilistic evidence, not a popup "LIFE FOUND."
  False positives/negatives are possible; abiotic explanations compete; consensus forms over
  time and missions. This is the most "research-process" system in the game.
- **Consequences**: confirming (or conclusively excluding) life is a top-tier milestone and
  prestige event; a positive result tightens planetary protection (and politics) hard. No
  gameplay "aliens" — discovered life is microbial/chemical and a *science object*, not an actor.

## 5. Politics, public mood & events

A lightweight geopolitical/PR layer that drives budgets, approvals and drama — never combat.

- **Per-faction relationships** (cooperation ↔ rivalry), partnership/consortium state, and a
  **prestige** score (firsts, science output, reliability). Prestige feeds budgets (agencies) and
  valuation/contracts (companies).
- **Public & political mood**: shifts with successes, failures (loss-of-crew is severe and long-
  lasting), spectacular firsts, accidents, and economic cycles. Drives appropriation modifiers,
  approval delays, and the chance of directed/cancelled programs.
- **Policy & treaties**: launch licensing & range access, nuclear-launch approval (NTP/RTG
  politics), planetary-protection stringency, export controls (Roscosmos avionics, ITAR-like
  effects on partnerships), debris/sustainability regulations. The world's policy can drift; some
  factions can lobby it.
- **Event system** (interrupt-and-pause hooks): launch successes/failures, anomalies (stuck
  valve, comms loss, software bug), solar storms (radiation events), funding crises/booms,
  political shake-ups, rival milestones, discoveries, supply shocks (Pu-238 shortage), personnel
  events (key hire/loss). Events are **seeded + state-driven**, not pure RNG: a low-TRL,
  under-tested, over-subscribed-ops craft *earns* its anomaly probability.

## 6. Milestones / "Firsts" (scoring spine)

~120 scored historic firsts; **world-first** > **faction-first**. Illustrative (full list in
`data/world/milestones.json`):
- Foothold era: first fully-reusable orbital flight; first commercial LEO station; first crewed
  lunar return of the new era; first lunar-polar ice ground-truth; first on-orbit cryo propellant
  transfer; first surface fission reactor operated through a lunar night.
- Cislunar/Mars era: first kg of lunar-derived propellant sold; first crewed Mars landing; first
  Mars ascent on locally-made propellant; first MW-class NEP transit; first Phobos/Deimos staging
  base; first closed-loop ECLSS to run a full Mars synodic cycle.
- Frontier era: first sample returned from an ocean-world plume (Enceladus); first cryobot
  through Europan ice; first conclusive astrobiology result (positive *or* negative) for a
  candidate world; first settlement to survive a 5-year resupply embargo (Homestead).
- Optional endgame (only if breakthroughs occur): first fission-fragment/fusion-driven transit;
  first off-Earth-built large structure.

Milestones grant prestige, sometimes funding/market effects, and feed the score & Grand Goals
(00-OVERVIEW.md §9). Many are tracked globally so the player races the AI world for the "world-
first" bonus — the natural competitive pressure that replaces combat.

## 7. The AI world (competitors & partners)

Non-player factions (incl. always-AI CNSA + minor agencies) run simplified versions of the same
systems: they research (advancing the global tide), build, fly, bid on contracts, partner, hit
milestones, and suffer accidents. They provide: a milestone race, a contract/partnership market,
licensing/buy options, and narrative texture. They obey the same physics and plausibility rules
(no AI cheating into impossible tech). Difficulty tunes their funding/competence, not their
physics.

## 8. The Sojournal (in-game encyclopedia — Pillar P6)

Every body, technology, resource process, manoeuvre type, mission archetype and discovered
result has an encyclopedia entry explaining the **real science** and citing sources. It updates
as the player's belief-state and the world advance (e.g., your Europa entry fills in as you
explore). This is the educational-honesty backbone and a soft tutorial layer.

## 9. Implementer notes

- Body/site/ephemeris data ships in `data/world/` from real JPL/MPC sources with provenance; the
  belief-state layer (what the player *knows*) is separate from ground truth.
- Astrobiology ground truth and tech-tree dead-end/breakthrough rolls share the per-game seed
  (01-RESEARCH.md §6) so a run is fully reproducible from seed + decisions (determinism).
- Politics/events are a state-machine + seeded scheduler feeding the interrupt-and-pause loop
  (00-OVERVIEW.md §5); no event invents physics-violating outcomes.
