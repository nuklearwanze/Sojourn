# SOJOURN — Part 2: The Full Tech Tree

> Format. The tree has two layers (per 01-RESEARCH.md): **Knowledge Domains** (Science, measured
> in Understanding Level 0–100) and **Engineering Branches** (Technologies advanced through TRL).
> Each Technology node lists: **[start TRL @ 2026]** · prerequisites (Domain UL floors + tech
> prereqs) · one-line plausibility/source tag. Numbers are design-intent; final values live in
> `data/` with full citations. Nothing here requires new physics; "future" items are funded
> programs or peer-reviewed concepts.
>
> Legend: `UL:` domain understanding floor · `→` unlocks/leads to · `‡` requires a Breakthrough
> or heavy leapfrog to reach early · TRL in brackets is the *real-world 2026 starting maturity*.

---

## A. KNOWLEDGE DOMAINS (Science track)

Grouped; each is a continuous 0–100 track with synergy links (⇄).

**Physical sciences**
- A1 Materials Science (alloys, composites, refractories, ceramics) ⇄ Cryogenics, Nuclear
- A2 Combustion & Chemical Propulsion Science ⇄ Materials
- A3 Plasma & Electromagnetics ⇄ Plasma Propulsion, Power
- A4 Nuclear Physics (fission, later fusion) ⇄ Materials, Power, Plasma
- A5 Thermal Physics (heat transfer, radiators, cryo) ⇄ Materials, Power
- A6 Power-Systems Science (PV, conversion cycles, storage) ⇄ Thermal, Nuclear
- A7 Cryogenics & Propellant Science (boil-off, densification, slush) ⇄ Thermal, Materials

**Flight sciences**
- A8 Astrodynamics & Trajectory Theory (n-body, low-thrust, low-energy transfers)
- A9 GNC & Autonomy (navigation, control, automation, AI ops) ⇄ Astrodynamics
- A10 Aerothermodynamics & EDL Science (entry, aerocapture, descent) ⇄ Materials, Thermal

**Life & Earth/space sciences**
- A11 Radiation Physics & Biology ⇄ Materials, Human Factors
- A12 Closed-Ecology / Bioregenerative Science (food, air, water loops) ⇄ Human Factors
- A13 Human Factors & Space Medicine (micro-g physiology, psych) ⇄ Radiation Bio
- A14 In-Situ Resource Chemistry (extraction, beneficiation, refining) ⇄ Materials, Geo
- A15 Geosciences — per-body sub-tracks (Moon, Mars, NEAs, Ceres, Europa, Enceladus, Titan,
      Venus, Mercury, comets…). Advanced primarily by *missions*, not labs.
- A16 Astrobiology (biosignatures, habitability, contamination science) ⇄ Geo, In-Situ Chem
- A17 Manufacturing & Construction Science (additive, regolith processing, ISAM) ⇄ Materials

> Missions inject UL into A8–A16 directly; labs dominate A1–A7, A11–A12, A17. This is why a
> data-rich flyby can be worth a decade of armchair theory in the relevant Geo/Astrobio track.

---

## B. ENGINEERING BRANCH — PROPULSION

The deepest branch. Categories follow real propulsion families. Each node feeds the Vehicle
Designer (04-SPACEFLIGHT.md §4) with Isp, thrust, thrust-to-weight, power demand, propellant
type, throttle range, restart count, reliability curve, and mass model.

### B1 Chemical
- B1.1 Pressure-fed storable (MMH/NTO, etc.) **[9]** — RCS, landers; baseline.
- B1.2 Gas-generator kerolox **[9]** UL:A2≥40 — workhorse first stages.
- B1.3 Staged-combustion kerolox (ox-rich) **[8]** UL:A2≥55, A1≥50 — high-perf reusable boosters.
- B1.4 Gas-generator/expander hydrolox **[9]** — high-Isp upper stages; needs A7 for long coast.
- B1.5 Full-flow staged combustion methalox **[6]** UL:A2≥60 → reusable, deep-throttle, ISRU-
  friendly (CH₄+O₂ makeable on Mars). Source: flying-class FFSC engines.
- B1.6 Deep-throttle / restartable landing engines **[7]** ⇄ B1.5 → precision EDL.
- B1.7 Long-duration cryo upper stage (low boil-off) **[5]** UL:A7≥55 → depot-compatible stages.
- B1.8 Tripropellant / densified-propellant optimisation **[3]** UL:A7≥65 ‡ — niche perf gains.

### B2 Electric (low-thrust, high-Isp)
- B2.1 Gridded ion (xenon) **[9]** UL:A3≥40, A6≥40 — proven deep-space cargo.
- B2.2 Hall-effect thruster (xenon/krypton) **[9]** — high-TRL cargo/stationkeeping.
- B2.3 High-power Hall (≥50 kW class) **[5]** UL:A3≥60, A6≥60 → NEP tug building block.
- B2.4 Magnetoplasmadynamic (MPD) **[4]** UL:A3≥70 ‡ → high-thrust EP at MWe scale.
- B2.5 VASIMR / variable-Isp RF plasma **[4]** UL:A3≥70, A4 link ‡ — variable thrust/Isp.
- B2.6 Electrospray / colloid (small sats) **[7]** — cubesat mobility.
- B2.7 Air/atmosphere-breathing EP (VLEO, or Mars-air) **[3]** UL:A3≥65 ‡ — drag make-up.

### B3 Nuclear-Thermal (NTP)
- B3.1 Solid-core NTR (NERVA-class, LEU) **[5]** UL:A4≥55, A1≥60, A7≥55 — ~900 s Isp, high thrust.
  Source: DRACO/NTP programs, LEU NERVA derivatives.
- B3.2 CERMET-fuel high-temp NTR **[4]** UL:A1≥70, A4≥60 → higher Isp/thrust, reusable cores.
- B3.3 LOX-augmented NTR (LANTR) **[3]** UL:A2 link ‡ — thrust boost in LEO/landing modes.
- B3.4 Liquid/centrifugal-core NTR **[2]** UL:A4≥80, A1≥80 ‡‡ — ~1300–1600 s; deep research.
- B3.5 Gas-core / open-cycle NTR **[1]** UL:A4≥90 ‡‡‡ — extreme Isp; may be a seeded dead end.
- B3.6 Bimodal NTR (thrust + onboard power) **[3]** UL:A6 link → tug that powers itself.

### B4 Nuclear-Electric (NEP)
- B4.1 Space fission reactor 10–100 kWe (Kilopower/KRUSTY-derived) **[5]** UL:A4≥55, A6≥55.
- B4.2 100 kWe–1 MWe reactor + power conversion (Brayton/Stirling) **[3]** UL:A4≥65, A5≥60, A6≥65.
- B4.3 Multi-MWe NEP reactor **[2]** UL:A4≥80, A5≥75 ‡ — outer-system cargo backbone.
- B4.4 High-temp radiators (droplet/heat-pipe/deployable) **[4]** UL:A5≥65, A17 link — the real
  NEP bottleneck (reject MW of waste heat). Multiple competing approaches → dead-end seeding.
- B4.5 NEP power-management & distribution (kV-class) **[4]** UL:A6≥65.
- B4.6 NEP integrated tug (reactor+PMAD+EP+radiators) **[2]** — system program; needs B2.3/4, B4.2/4.

### B5 Frontier / late-game (gated hard; many seeded as dead ends or breakthrough-only)
- B5.1 Fission-fragment rocket **[1]** UL:A4≥90 ‡‡‡ — very high Isp, low thrust.
- B5.2 Nuclear salt-water rocket **[1]** UL:A4≥90 ‡‡‡ — high thrust+Isp; extreme engineering.
- B5.3 Inertial/​magneto-inertial **fusion** propulsion **[1]** UL:A4(fusion)≥85, A3≥85 ‡‡‡ —
  only after a Fusion breakthrough; never guaranteed in a given game. Source: published fusion-
  propulsion concepts (e.g., direct-fusion-drive, magneto-inertial). Endgame, optional.
- B5.4 Solar/laser thermal & solar sails **[4]** UL:A6/A3 — niche low-cost inner-system cargo.
- B5.5 Nuclear-pulse (Orion-type) **[2]** — *intentionally locked out in v1* (treaty + no-weapons
  pillar); present only as a Sojournal historical entry.

### B6 Propellant logistics tech (enables everything above)
- B6.1 On-orbit propellant transfer (cryo) **[5]** UL:A7≥55 → depots, refuelling.
- B6.2 Zero-boil-off / active cryocooling **[5]** UL:A5≥60, A7≥60.
- B6.3 Propellant depot (storage + transfer + station-keeping) **[4]** — system tech.
- B6.4 In-space propellant production tie-in (from B/ISRU) — see G.

---

## C. ENGINEERING BRANCH — POWER & THERMAL

- C1 Photovoltaics: rigid → flexible/roll-out (ROSA-class) → concentrator → thin-film **[9→6]**
  UL:A6. Inner-system workhorse; useless at Jupiter without huge area.
- C2 Radioisotope power (RTG/MMRTG, Stirling RPS) **[9→6]** UL:A6, A4 — small deep-space/night.
  Constrained by Pu-238/Am-241 supply (an economy resource, 03-ECONOMY.md §3).
- C3 Surface fission (see B4.1) — base power through lunar night / dust storms.
- C4 Energy storage: Li-ion → solid-state → regenerative fuel cell (O₂/H₂) → flywheel **[9→4]**
  UL:A6. Fuel cells double as life-support tie-in.
- C5 Power conversion cycles: thermoelectric → Stirling → Brayton (closed) **[9→5]** UL:A5,A6.
- C6 Thermal management: heat pipes, loop heat pipes, deployable radiators, cryocoolers **[9→5]**
  UL:A5. Shared bottleneck with NEP (B4.4).
- C7 Wireless/beamed power (surface, then orbit→surface laser/microwave) **[4]** UL:A3,A6 ‡.

## D. ENGINEERING BRANCH — STRUCTURES, MATERIALS & MANUFACTURING

- D1 Lightweight primary structures (Al-Li, composites, isogrid) **[9→7]** UL:A1.
- D2 Cryotanks: metallic → composite (linerless) → conformal **[9→5]** UL:A1,A7.
- D3 Inflatable/expandable habitats (BEAM-class → large) **[7→5]** UL:A1,A13.
- D4 Radiation shielding: passive (regolith, water, polyethylene) → active (magnetic/plasma ‡)
  **[9→3]** UL:A11,A1.
- D5 In-space additive manufacturing (polymer → metal → recycling) **[6→4]** UL:A17.
- D6 Regolith construction: sintering, 3D-print habitats, sulfur/geopolymer concrete **[4→3]**
  UL:A17,A14,A15 — enables radiation-safe surface structures from local material.
- D7 Large deployables/booms/antennas, on-orbit assembly (ISAM) **[6→4]** UL:A17.
- D8 Self-replicating/automated construction fabric **[2]** ‡‡ — late, partial automation only.

## E. ENGINEERING BRANCH — GNC, AUTONOMY & COMMS

- E1 Navigation: ground (DSN) → optical → pulsar/X-ray → onboard autonomous nav **[9→5]** UL:A8,A9.
- E2 Precision & hazard-relative landing (terrain-relative nav, hazard avoidance) **[7→6]** UL:A9,A10.
  Source: SLIM, lunar HDL programs.
- E3 Autonomous rendezvous, proximity ops & docking **[8→6]** UL:A9.
- E4 Robotic manipulation & teleoperation (arms, sample handling, light-lag autonomy) **[8→5]** UL:A9.
- E5 Mission autonomy / onboard planning (less ground-in-the-loop as light-lag grows) **[6→4]** UL:A9.
- E6 Comms: Ka-band → optical/laser comms → relay constellations (lunar, Mars, deep) **[7→5]** UL:A3.
  DSN/relay capacity is a *shared operational resource* (03-ECONOMY.md §6).

## F. ENGINEERING BRANCH — LIFE SUPPORT & CREW

The crewed-mission difficulty multiplier. Closure fraction is the key metric.

- F1 Open-loop ECLSS (carry consumables) **[9]** — ISS-class, mass-prohibitive far from Earth.
- F2 Physico-chemical recycling: water (≥90%), O₂ (Sabatier/​electrolysis), CO₂ scrub **[8→7]**
  UL:A12. ISS-derived; the realistic near-term backbone.
- F3 High-closure ECLSS (≥98% water/air) **[5]** UL:A12≥60 — for long Mars stays.
- F4 Bioregenerative loops: salad-machine → partial → high-fraction food production **[4→3]**
  UL:A12≥70 — slow, fragile, mass/power heavy; *food autonomy* is a settlement gate.
- F5 Closed-ecology system integration (MELiSSA-class, multi-compartment) **[3]** UL:A12≥80 ‡ —
  required for the *Homestead* embargo-survival goal.
- F6 Crew health countermeasures: exercise → pharmacological → **artificial gravity** (tethered
  spin → rotating hab) **[7→4]** UL:A13. Spin-g is the big crewed-comfort unlock.
- F7 Radiation protection for crew (storm shelters, dosimetry, pharma, habitat shielding) **[6→4]**
  UL:A11,D4 — bounds crewed deep-space mission duration.
- F8 EVA systems: suits (current → mechanical-counterpressure ‡), surface mobility suits **[7→5]** UL:A13.
- F9 Medical autonomy (telemedicine → autonomous surgery support) **[5→3]** UL:A13.

## G. ENGINEERING BRANCH — ISRU & RESOURCE PROCESSING

The economic engine off-Earth (economics in 03-ECONOMY.md §3–4).

- G1 Prospecting instruments: neutron/spectral, GPR, drills, sample assay **[7→5]** UL:A14,A15.
- G2 Lunar polar **water-ice** extraction (thermal mining, cold-trap capture) **[3]** UL:A14≥55,
  A15:Moon≥40 → water, then LOX/LH₂. Source: lunar ISRU studies, cold-trap thermal mining papers.
- G3 Regolith oxygen extraction (molten regolith electrolysis, H₂ reduction, carbothermal) **[4→3]**
  UL:A14≥60 — O₂ from anywhere with regolith; multiple competing processes (dead-end seeding).
- G4 Mars atmosphere ISRU: CO₂ acquisition → **MOXIE-class O₂** → Sabatier **CH₄+O₂** propellant
  **[6→4]** UL:A14≥55, A15:Mars. Source: MOXIE flight demo. *Makes Mars ascent affordable.*
- G5 Water from hydrated minerals / Mars ground ice / C-type asteroids **[3]** UL:A14≥60,A16 link.
- G6 Metals & silicon refining (regolith → structural metals, solar-grade Si) **[2]** UL:A14≥70,A17 ‡.
- G7 Volatiles from C-type/comet bodies (water, CO₂, organics, ammonia) **[2]** UL:A14≥60,A15:NEA.
- G8 Helium-3 / rare-isotope extraction **[1]** ‡‡ — only meaningful if fusion (B5.3) arrives;
  modelled honestly as currently uneconomic.
- G9 ISRU plant scale-up & autonomy (pilot → production, teleop → autonomous) **[3→2]** UL:A14,A9.

## H. ENGINEERING BRANCH — LAUNCH & EARTH-TO-ORBIT VEHICLES

(These are *Technologies* feeding the Vehicle Designer's launch-vehicle mode.)

- H1 Expendable medium/heavy lift **[9]** — baseline access, high $/kg.
- H2 Partially reusable (boostback/​droneship landing) **[8]** UL:A8,A9,B1 → big $/kg drop.
- H3 Fully reusable two-stage (rapid reuse) **[5]** UL:A9≥60,B1.5,D2 → order-of-magnitude $/kg
  drop; the economy-transforming program. Source: flying full-reuse development vehicles.
- H4 Reusable upper stage / on-orbit refuel architecture **[4]** UL:B6.1 → enables high-energy
  departures from LEO via tanker flights ("distributed launch").
- H5 Small/responsive launch **[9]** — niche, fast cadence, smallsats.
- H6 Air-launch / horizontal **[8]** — niche.
- H7 Alternative assist: rotating-launch/​mass-driver/​skyhook **[2→1]** ‡‡ — research-tier, often
  seeded as not-worth-it; present for plausibility completeness.

## I. ENGINEERING BRANCH — IN-SPACE VEHICLES & SURFACE SYSTEMS

System-integration techs the designer composes into named vehicle classes.

- I1 Crew capsule (LEO → cislunar → interplanetary-rated) **[9→6]** UL:A10,F.
- I2 Cargo/logistics spacecraft (resupply, automated) **[9→7]**.
- I3 Reusable space tug (chemical → electric → nuclear) **[8→4]** UL:propulsion+B6.
- I4 Crewed deep-space transit habitat (Gateway-class → cycler/​spin-hab) **[5→3]** UL:D3,F,F6.
- I5 Lunar lander (cargo → crewed → reusable) **[7→5]** UL:B1.6,E2.
- I6 Mars lander/ascent (EDL the hard way: supersonic retroprop, aerocapture) **[4→2]** UL:A10,E2 —
  the "Mars EDL gap"; landing tens of tonnes is a multi-tech grand challenge.
- I7 Drop shuttles / sample-return ascent vehicles **[7→5]** UL:E2,A10.
- I8 Pressurised & unpressurised rovers (teleop → crewed → autonomous) **[8→5]** UL:E4,F8.
- I9 Surface mobility & logistics (hoppers, cranes, regolith movers) **[5→3]** UL:E4,D6.
- I10 Orbital station modules (LEO commercial → cislunar → other-planet orbit) **[8→5]** UL:D3,F.
- I11 Surface base modules (habitat, ISRU, power, greenhouse, workshop, medical) **[5→3]** UL:D3,D6,F,G.
- I12 Aerial/ocean explorers: Mars helicopter, Venus aerobot, Titan rotorcraft, Europa cryobot
  **[6→2]** UL:A10,E,A15 — body-specific exploration platforms. Source: Ingenuity, Dragonfly,
  Venus aerobot & Europa cryobot mission studies.

## J. ENGINEERING BRANCH — SCIENCE INSTRUMENTS & ASTROBIOLOGY

- J1 Remote sensing (imaging, spectrometers, radar, magnetometers, GPR) **[9→6]** UL:A15.
- J2 In-situ geochem & mineralogy (XRD, LIBS, mass spec) **[8→6]** UL:A14,A15.
- J3 Life-detection suite (organics, chirality, metabolism, microscopy) **[5→3]** UL:A16 —
  staged: biosignature hints → in-situ tests → sample return → (seeded) resolution.
- J4 Subsurface access: deep drills, melt-probes/cryobots, caves/lava-tube exploration **[4→2]**
  UL:A15,E4 — required to reach ocean-world habitats and Mars subsurface.
- J5 Sample-return chain & receiving (containment, BSL, back-contamination) **[6→4]** UL:A16,
  planetary-protection (05-WORLD.md §3).
- J6 Long-baseline & deep observation assets (space telescopes as agency projects) **[8→6]**.

## K. ENGINEERING BRANCH — OPERATIONS, AUTOMATION & DATA

- K1 Mission ops automation & fleet management (more craft per controller) **[6→4]** UL:A9.
- K2 Logistics & supply-chain planning tools (manifesting, depot routing) **[5→3]**.
- K3 Robotic construction & maintenance autonomy **[4→2]** UL:A9,D7.
- K4 Digital-twin/simulation for test-cost reduction **[5→3]** — discounts later TRL steps (you
  validate more in sim), but only as far as underlying Domain UL supports (no free lunch).

---

## L. Tech-tree topology notes (for implementers)

1. **Capability categories are guaranteed reachable; specific nodes are not.** Each "grand
   capability" (cheap launch, lunar propellant, Mars ascent, MW-class deep-space tug, closed
   life support, ocean-world access) has ≥2 candidate tech paths so per-game dead-end seeding
   never bricks a strategy (01-RESEARCH.md §4.1, §6).
2. **Cross-branch gates are common and deliberate.** NEP needs Power+Thermal+EP+Reactor; Mars
   settlement needs EDL+ISRU+ECLSS+Power+Construction. The tree is a web, not parallel ladders.
3. **Heritage discounts** (01-RESEARCH.md §2) let real 2026 leaders start advanced sub-branches
   higher: NASA/Helion on reuse & methalox; Roscosmos on NTP/NEP; JAXA on precision landing &
   sample return; ESA on aerocapture & deep-space science; Astrolith/Caravel on ISRU/depots.
4. **Every node carries a `source` field.** A node without a citable basis does not ship. If a
   plausible concept lacks sources, it lives behind a Breakthrough as an "if-discovered" entry.
5. **Fusion and all B5/G8 nodes are optional endgame**: a game can be completed fully without
   them; they exist so a long, science-heavy game can plausibly reach further — never assumed.
