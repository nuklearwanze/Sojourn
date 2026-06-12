# SOJOURN — Part 4: Spaceflight Simulation (Astrodynamics, Propulsion, Vehicles, Life Support, EDL)

> This is the physics core. Everything here obeys conservation laws. No top speed, no
> reactionless drives, no hand-waved "warp factor." The rocket equation is the law you live under.

---

## 1. Astrodynamics model

### 1.1 Dynamics
- **Default integrator:** patched-conics for fast planning + a true **n-body propagator**
  (for high-fidelity ops, Lagrange-point dynamics, low-energy transfers, perturbations). Two
  fidelity tiers: planning (analytic/patched-conic, instant) and simulation (numerical, the
  authoritative state). The game reconciles plans against the real propagation.
- **Perturbations** that matter are modelled: third-body, J2/oblateness, solar radiation
  pressure (for sails/large structures), atmospheric drag (LEO/VLEO/aerobraking), and
  low-thrust continuous acceleration.
- **Reference frames & SOIs**: heliocentric, planetocentric, rotating (for L-points/CR3BP).
  The map lets the player switch frames; trajectories are shown in the chosen frame.

### 1.2 Manoeuvre planning (the player's core spaceflight verb)
- **Porkchop plots** for ballistic transfers: pick departure/arrival windows, see C3/Δv/TOF
  contours; the Mars window every ~26 months is the campaign heartbeat.
- **Manoeuvre nodes** on the timeline: plan burns (prograde/normal/radial components), see the
  resulting conic/trajectory, chain nodes for multi-burn plans.
- **Gravity assists / flybys**: plan slingshots (Earth/Venus/Jupiter assists) with a flyby
  geometry tool; the game's solver helps find assist sequences (VEEGA-style) for the patient.
- **Low-thrust trajectories**: for EP/NEP, spiral transfers and continuous-thrust arcs are
  planned as guidance laws (e.g., tangential/optimal steering), with months-long spirals out of
  gravity wells — the characteristic NEP trade (great Isp, terrible patience).
- **Low-energy transfers**: weak-stability-boundary/ballistic-capture routes (cheaper Δv, longer
  TOF) once Astrodynamics UL is high enough to "see" them — a research-gated planning unlock.
- **Aerocapture/aerobraking** as planned manoeuvres at bodies with atmospheres (Δv savings vs
  thermal/structural risk).
- **Station-keeping & maintenance**: L-point halo upkeep, drag make-up, RTG/PV degradation over
  time — ongoing propellant/power sinks.

### 1.3 What the player sees
A plan is never free of consequences: every node debits propellant from a specific tank on a
specific vehicle; insufficient Δv is flagged; light-time delay to comms is shown; arrival
conditions feed EDL (§5). The sim then *flies* the plan and reality (perturbations, execution
error, engine reliability) may require correction burns (TCMs) — modelled, not ignored.

## 2. The rocket equation, as a felt constraint

Every vehicle's capability is `Δv = Isp·g₀·ln(m0/mf)`. The designer (§4) surfaces the mass
fractions live. Consequences the game makes visceral:
- Staging vs single-stage trade-offs; payload-fraction cliffs.
- The brutal cost of high Δv with chemical (why the outer system needs NTP/NEP or patience).
- Why ISRU/refuelling/depots are economically transformative (reset m0 at a new node).
- Boil-off eating your Δv on long coasts (cryo management as gameplay).

## 3. Propulsion physics (model, not list)

Each propulsion Technology (02-TECH-TREE.md §B) exposes a physical model with: **Isp** (function
of propellant, chamber/exhaust conditions, mode), **thrust**, **thrust-to-weight**, **input
power** (for EP/NEP), **propellant type & density**, **throttle range**, **restart/duty limits**,
**mass model** (engine + feed + power + radiators for nuclear/electric), and a **reliability
curve** (TRL + heritage + duty).

Key honest couplings the sim enforces:
- **EP is power-limited**: thrust ∝ power/Isp; high Isp ⇒ low thrust ⇒ long burns; you must carry
  (and cool!) the power source. NEP drags reactor + radiators everywhere — modelled as mass.
- **NTP**: high thrust *and* high Isp (~900 s solid-core) but heavy reactor, hydrogen storage
  (boil-off!), and a real test/operational radiation/heritage cost.
- **Thermal limits**: radiators (C6/B4.4) are a first-class mass on any nuclear/high-power craft;
  you can't ignore waste heat. This is a deliberate, frequently-binding constraint.
- **Two-burn reality**: gravity losses, finite-burn losses, and steering losses are computed —
  impulsive-approximation plans get a correction when flown.

## 4. The Vehicle Designer (Aurora-style, physics-checked)

A spreadsheet-grade designer where the player composes vehicles from **researched component
Technologies** and the game computes emergent performance and cost.

### 4.1 Inputs (components)
Structure/tanks (D1–D2), propulsion (B), power (C1–C5), thermal/radiators (C6), avionics/GNC
(E), comms (E6), life support & crew accommodation (F) for crewed craft, ISRU/science/cargo
payloads (G/J/I), landing gear/heat shield/EDL kit (I, A10), docking/berthing, RCS.

### 4.2 Computed outputs
Dry mass & wet mass, Δv (per stage/mode), thrust & T/W (at each gravity field of interest),
power & thermal balance (margin must be ≥0), life-support closure & endurance, payload capacity,
**reliability** (composed from component TRLs/heritage), unit cost & build time (with learning
curve), and **suitability checks** (can it actually land here? survive this radiation dose? close
its power budget at Jupiter?).

### 4.3 Vehicle classes (templates, all designer-built)
Launch vehicles (H) · crew capsules · cargo spacecraft · tugs (chem/EP/nuclear) · landers
(lunar/Mars/cargo/crew) · ascent vehicles & drop shuttles · transit habitats / cyclers / spin-
habs · rovers (teleop/crewed) · surface mobility (hoppers, movers) · station modules · base
modules · ISRU plants · relay/comm sats · science probes & body-specific explorers (I12).

### 4.4 Realism guards
The designer refuses or red-flags the impossible: negative power margin, radiators too small for
heat load, Δv short of the planned mission, structure that can't take the thrust, a lander whose
T/W < local g, crewed endurance < mission duration, radiation dose > crew limit. You can *fly*
marginal designs, but the reliability/risk numbers tell the truth. (No physics cheats; only
informed gambles.)

## 5. Entry, Descent & Landing (EDL) and aerocapture

EDL is its own simulated phase because it's where missions die.
- **Bodies with atmosphere** (Earth, Mars, Venus, Titan): entry heating (A10 materials),
  ballistic coefficient, parachutes/​supersonic retropropulsion, the **Mars EDL gap** (landing
  >~1–2 t is genuinely hard → multi-tech challenge, I6). Aerocapture/aerobraking available as Δv-
  saving but risky.
- **Airless bodies** (Moon, asteroids, most moons): pure propulsive descent; precision &
  hazard-relative landing (E2) determines where you can safely set down (matters for polar-ice
  sites with rough terrain).
- **Microgravity bodies** (small asteroids, Phobos/Deimos): "landing" is rendezvous + anchoring;
  touch-and-go sampling.
- EDL outcome depends on vehicle suitability (heat shield, T/W, throttle, guidance), site
  hazards (slope, boulders, illumination), and reliability roll; failures are dramatic and
  costly (loss of vehicle/crew → 05-WORLD.md §5 consequences).

## 6. Life support & the crewed-mission multiplier

Crewed missions are *much* harder than robotic — by design (Pillar P2/P5). For any crewed
vehicle/base the sim tracks, per crew member over time:
- **Consumables**: O₂, water, food, N₂ buffer, CO₂ removal capacity — supplied open-loop (mass!)
  or closed via ECLSS closure fraction (F2–F5). Closure % directly sets resupply mass and thus
  feasibility of distant missions.
- **Radiation dose**: accumulated career + mission dose vs limits; GCR + SPE events (solar storm
  → need storm shelter F7); bounds deep-space mission duration.
- **Health/physiology**: bone/muscle/cardio deconditioning vs countermeasures (exercise, pharma,
  artificial gravity F6); long micro-g missions degrade crew capability and post-mission health.
- **Psychology**: isolation/confinement load over time and with distance/comms-lag (Mars crews
  can't call home in real time); affects error rates, anomaly risk, morale.
- **Spares & maintenance**: closed life support breaks; crew-time + spares keep it alive; a
  failed ECLSS far from Earth is an existential event.

The result: a Mars crewed mission is an integrated puzzle of Δv (transfer + EDL + ascent via
ISRU), closure (survive ~2.5 yr round trip incl. surface stay), radiation (stay under dose),
power, and abort options — exactly the real architecture trade space.

## 7. Operations & autonomy under light-lag

Comms light-time is real (1.3 s Moon, 3–22 min Mars one-way, hours to the outer system). Beyond
a threshold, real-time teleop is impossible → onboard autonomy (E5) and pre-planned sequences
matter; ops capacity (03-ECONOMY.md §6) limits how many craft you can babysit. Anomalies must
sometimes be handled autonomously; insufficient autonomy ⇒ lost craft.

## 8. Implementer notes

- Authoritative state is the **numerical propagator** on the fixed timestep; planning tools are
  fast approximations reconciled against it. Determinism required (constitution) → use a fixed-
  step symplectic/high-order integrator with deterministic step control; no wall-clock-dependent
  substepping.
- Burns, EDL and docking force fine-timestep resolution regardless of time-warp.
- All propulsion/vehicle/EDL constants live in `data/` with source tags; the **physics engine
  itself contains no per-tech magic numbers** — it reads them, so balance/realism is auditable.
- Provide a **headless deterministic mode** for testing trajectories against known analytic cases
  (Hohmann Δv, two-body periods, simple flyby) as part of the test suite (constitution §Testing).
